//! SceneScript modules and their values stay inside one assignment-owned VM.
use anyhow::{ensure, Context as _, Result};
use rquickjs::{Context, Function, Module, Object, Persistent, Runtime, TypedArray};
use serde_json::Value as Json;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
#[cfg(test)]
thread_local! {static LIVE:std::cell::Cell<usize>=const {std::cell::Cell::new(0)};}
#[cfg(test)]
pub(super) fn counts() -> usize {
    LIVE.with(std::cell::Cell::get)
}
struct Binding {
    id: i64,
    property: String,
    module: Persistent<Object<'static>>,
    disabled: bool,
}
pub(super) struct Scripts {
    bindings: Vec<Binding>,
    context: Context,
    runtime: Runtime,
    deadline: Arc<Mutex<Instant>>,
    timers: super::script_timers::Shared,
    audio_buffers: Rc<RefCell<Vec<Persistent<Object<'static>>>>>,
}
fn diagnostic(ctx: &rquickjs::Ctx<'_>, error: rquickjs::Error) -> anyhow::Error {
    let exception = ctx.catch();
    anyhow::anyhow!("SceneScript: {error}; exception {exception:?}")
}
impl Scripts {
    pub fn new(objects: &[super::assets::Object], properties: &Json) -> Result<Self> {
        let runtime = Runtime::new()?;
        runtime.set_memory_limit(32 * 1024 * 1024);
        runtime.set_max_stack_size(512 * 1024);
        let deadline = Arc::new(Mutex::new(Instant::now() + Duration::from_millis(20)));
        let guard = deadline.clone();
        runtime.set_interrupt_handler(Some(Box::new(move || {
            guard.lock().map(|v| Instant::now() >= *v).unwrap_or(true)
        })));
        let context = Context::full(&runtime)?;
        let mut scripts = Self {
            bindings: Vec::new(),
            context,
            runtime,
            deadline,
            timers: Default::default(),
            audio_buffers: Rc::new(RefCell::new(Vec::new())),
        };
        #[cfg(test)]
        LIVE.with(|v| v.set(v.get() + 1));
        scripts.context.with(|ctx| -> Result<()> {
            let engine = Object::new(ctx.clone())?;
            engine.set("time", 0.0)?;
            engine.set("frametime", 0.0)?;
            engine.set(
                "userProperties",
                ctx.json_parse(serde_json::to_vec(properties)?)?,
            )?;
            super::script_timers::install(ctx.clone(), &engine, scripts.timers.clone())?;
            let audio_buffers = scripts.audio_buffers.clone();
            let register_audio = Function::new(ctx.clone(), move |ctx, size: f64| {
                if ![16.0, 32.0, 64.0].contains(&size) {
                    return Err(rquickjs::Exception::throw_range(
                        &ctx,
                        "Scene audio buffer size must be 16, 32 or 64",
                    ));
                }
                let size = size as usize;
                let mut registrations = audio_buffers.borrow_mut();
                if registrations.len() >= 128 {
                    return Err(rquickjs::Exception::throw_range(
                        &ctx,
                        "Scene audio buffer registration budget exceeded",
                    ));
                }
                let object = Object::new(ctx.clone())?;
                object.prop(
                    "left",
                    TypedArray::<f32>::new(ctx.clone(), vec![0.0; size])?,
                )?;
                object.prop(
                    "right",
                    TypedArray::<f32>::new(ctx.clone(), vec![0.0; size])?,
                )?;
                object.prop(
                    "average",
                    TypedArray::<f32>::new(ctx.clone(), vec![0.0; size])?,
                )?;
                registrations.push(Persistent::save(&ctx, object.clone()));
                Ok(object)
            })?;
            engine.set("registerAudioBuffers", register_audio)?;
            for size in [16, 32, 64] {
                engine.set(format!("AUDIO_RESOLUTION_{size}"), size)?;
            }
            ctx.globals().set("engine", engine)?;
            let layers = Object::new(ctx.clone())?;
            for object in objects {
                let mut layer = object.properties.clone();
                if let Some(values) = layer.as_object_mut() {
                    for value in values.values_mut() {
                        if value.is_object() && value.get("value").is_some() {
                            *value = value["value"].clone();
                        }
                    }
                }
                layers.set(
                    object.id.to_string(),
                    ctx.json_parse(serde_json::to_vec(&layer)?)?,
                )?;
            }
            ctx.globals().set("__sceneLayers", layers)?;
            // These small JS adapters expose objects already created by Rust.
            // No module/file/network/process loader is installed in the VM.
            ctx.eval::<(), _>(include_str!("script_helpers.js"))
                .map_err(|error| diagnostic(&ctx, error))?;
            Ok(())
        })?;
        for object in objects {
            let Some(properties) = object.properties.as_object() else {
                continue;
            };
            for (property, value) in properties {
                let Some(source) = value.get("script").and_then(Json::as_str) else {
                    continue;
                };
                ensure!(
                    scripts.bindings.len() < 4096 && source.len() <= 1024 * 1024,
                    "SceneScript binding/source budget exceeded"
                );
                *scripts.deadline.lock().unwrap() = Instant::now() + Duration::from_millis(20);
                let id = object.id;
                let property_json = serde_json::to_string(property)?;
                let seed = value
                    .get("scriptproperties")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({}));
                ensure!(
                    seed.as_object().is_some_and(|v| v.len() <= 256),
                    "Invalid SceneScript property seed"
                );
                let seed = serde_json::to_string(&seed)?;
                let wrapped=format!("const thisLayer=globalThis.__sceneLayers[{id}];\nfunction createScriptProperties(){{return globalThis.__createScriptProperties({seed});}}\n{source}\nexport function __sceneInit(){{if(typeof init==='function'){{const result=init(thisLayer[{property_json}]);if(result!==undefined)thisLayer[{property_json}]=result;}}}}\nexport function __sceneTick(){{if(typeof update==='function'){{const result=update(thisLayer[{property_json}]);if(result!==undefined)thisLayer[{property_json}]=result;}}}}");
                let module = scripts
                    .context
                    .with(|ctx| -> Result<_> {
                        let module =
                            Module::declare(ctx.clone(), format!("layer-{id}-{property}"), wrapped)
                                .map_err(|e| diagnostic(&ctx, e))?;
                        let (module, promise) = module.eval().map_err(|e| diagnostic(&ctx, e))?;
                        promise.finish::<()>().map_err(|e| diagnostic(&ctx, e))?;
                        let init: Function = module.get("__sceneInit")?;
                        init.call::<_, ()>(()).map_err(|e| diagnostic(&ctx, e))?;
                        Ok(Persistent::save(&ctx, module.namespace()?))
                    })
                    .with_context(|| {
                        format!("SceneScript layer '{}' property '{property}'", object.name)
                    })?;
                scripts.bindings.push(Binding {
                    id,
                    property: property.clone(),
                    module,
                    disabled: false,
                });
            }
        }
        Ok(scripts)
    }
    pub fn tick(&mut self, elapsed: f64, delta: f64) -> Result<Vec<(i64, Json)>> {
        ensure!(
            elapsed.is_finite()
                && elapsed >= 0.0
                && delta.is_finite()
                && (0.0..=0.25).contains(&delta),
            "Invalid SceneScript clock"
        );
        *self.deadline.lock().unwrap() = Instant::now() + Duration::from_millis(20);
        self.timers.borrow_mut().now = elapsed;
        self.context.with(|ctx| -> Result<_> {
            let engine: Object = ctx.globals().get("engine")?;
            engine.set("time", elapsed)?;
            engine.set("frametime", delta)?;
            let scene: Object = ctx.globals().get("thisScene")?;
            scene.set("time", elapsed)?;
            scene.set("currentTime", elapsed)?;
            scene.set("dt", delta)?;
            scene.set("fps", if delta > 0.0 { 1.0 / delta } else { 60.0 })?;
            for binding in &mut self.bindings {
                if binding.disabled {
                    continue;
                }
                let module = binding.module.clone().restore(&ctx)?;
                let update: Function = module.get("__sceneTick")?;
                if let Err(error) = update.call::<_, ()>(()) {
                    binding.disabled = true;
                    return Err(diagnostic(&ctx, error)).with_context(|| {
                        format!(
                            "SceneScript layer {} property '{}'",
                            binding.id, binding.property
                        )
                    });
                }
            }
            let due = self.timers.borrow().due();
            for id in due {
                let callback = self.timers.borrow_mut().take(id);
                if let Some(callback) = callback {
                    callback
                        .restore(&ctx)?
                        .call::<_, ()>(())
                        .map_err(|error| diagnostic(&ctx, error))?;
                }
            }
            Ok(())
        })?;
        for _ in 0..64 {
            match self.runtime.execute_pending_job() {
                Ok(false) => break,
                Ok(true) => {}
                Err(error) => anyhow::bail!("SceneScript pending job failed: {error:?}"),
            }
        }
        ensure!(
            !self.runtime.is_job_pending(),
            "SceneScript pending job budget exceeded"
        );
        self.context.with(|ctx| -> Result<_> {
            let json:String=ctx.eval("JSON.stringify(__sceneLayers,(key,value)=>{if(typeof value==='number'&&!Number.isFinite(value))throw new TypeError('Nonfinite SceneScript output');return value;})").map_err(|error|diagnostic(&ctx,error))?;
            ensure!(
                json.len() <= 4 * 1024 * 1024,
                "SceneScript output budget exceeded"
            );
            let values: Json = serde_json::from_str(&json)?;
            values
                .as_object()
                .context("SceneScript layers must be objects")?
                .iter()
                .map(|(id, value)| Ok((id.parse()?, value.clone())))
                .collect()
        })
    }
    pub fn set_properties(&self, properties: &Json) -> Result<()> {
        let encoded = serde_json::to_vec(properties)?;
        ensure!(
            encoded.len() <= 1024 * 1024,
            "SceneScript project property budget exceeded"
        );
        self.context.with(|ctx| -> Result<()> {
            let engine: Object = ctx.globals().get("engine")?;
            engine.set("userProperties", ctx.json_parse(encoded)?)?;
            Ok(())
        })
    }
    pub fn set_audio(&mut self, snapshot: &crate::audio::AudioSnapshot) -> Result<()> {
        self.context.with(|ctx| -> Result<()> {
            for persistent in self.audio_buffers.borrow().iter() {
                let object = persistent.clone().restore(&ctx)?;
                let left: TypedArray<f32> = object.get("left")?;
                let right: TypedArray<f32> = object.get("right")?;
                let average: TypedArray<f32> = object.get("average")?;
                let size = left.len();
                ensure!(
                    matches!(size, 16 | 32 | 64) && right.len() == size && average.len() == size,
                    "Scene audio buffer shape changed"
                );
                let bands = crate::audio::BANDS / size;
                for index in 0..size {
                    let start = index * bands;
                    let end = start + bands;
                    let mut left_value = 0.0f32;
                    let mut right_value = 0.0f32;
                    for band in start..end {
                        left_value += finite_audio(snapshot.spectrum[band]);
                        right_value += finite_audio(snapshot.spectrum[crate::audio::BANDS + band]);
                    }
                    left_value /= bands as f32;
                    right_value /= bands as f32;
                    let index = index as u32;
                    left.set(index, left_value)?;
                    right.set(index, right_value)?;
                    average.set(index, (left_value + right_value) * 0.5)?;
                }
            }
            Ok(())
        })
    }
}

fn finite_audio(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

impl Drop for Scripts {
    fn drop(&mut self) {
        self.audio_buffers.borrow_mut().clear();
        self.timers.borrow_mut().clear();
        self.bindings.clear();
        #[cfg(test)]
        LIVE.with(|v| v.set(v.get() - 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rust_modules_hooks_properties_assignment_isolation_and_error_cleanup() {
        use crate::scene::{assets::Assets, fixtures};
        let baseline = counts();
        let mut resources = fixtures::image_resources();
        let mut document: Json = serde_json::from_slice(&resources.0["scene.json"]).unwrap();
        document["objects"][0]["alpha"] = serde_json::json!({"value":1,"script":"let ticks=0; export function init(value){return 0.5;} export function update(value){ticks++;return engine.userProperties.opacity+ticks*0.01;}"});
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
        let assets = Assets::load("scene.json", &resources).unwrap();
        let mut a = Scripts::new(&assets.objects, &serde_json::json!({"opacity":0.2})).unwrap();
        let mut b = Scripts::new(&assets.objects, &serde_json::json!({"opacity":0.8})).unwrap();
        let values = a.tick(0.0, 0.0).unwrap();
        assert!((values[0].1["alpha"].as_f64().unwrap() - 0.21).abs() < 1e-10);
        let values = b.tick(0.0, 0.0).unwrap();
        assert!((values[0].1["alpha"].as_f64().unwrap() - 0.81).abs() < 1e-10);
        a.set_properties(&serde_json::json!({"opacity":0.3}))
            .unwrap();
        let values = a.tick(0.1, 0.1).unwrap();
        assert!((values[0].1["alpha"].as_f64().unwrap() - 0.32).abs() < 1e-10);
        drop(a);
        drop(b);
        assert_eq!(counts(), baseline);
        let mut objects = assets.objects.clone();
        objects[0].properties["alpha"]["script"] =
            serde_json::json!("throw new Error('bad init');");
        assert!(Scripts::new(&objects, &serde_json::json!({})).is_err());
        assert_eq!(counts(), baseline);
    }

    #[test]
    fn rust_audio_buffer_registration_is_typed_stable_and_bounded() {
        use crate::scene::{assets::Assets, fixtures};
        let assets = Assets::load("scene.json", &fixtures::image_resources()).unwrap();
        let mut objects = assets.objects.clone();
        objects[0].properties["alpha"] = serde_json::json!({
            "value": 1,
            "script": "const audio=engine.registerAudioBuffers(16); export function update(value){ return audio.average[0]; }"
        });
        let mut scripts = Scripts::new(&objects, &serde_json::json!({})).unwrap();
        let mut snapshot = crate::audio::AudioSnapshot::default();
        snapshot.spectrum[..4].fill(1.0);
        snapshot.spectrum[64..68].fill(0.5);
        scripts.set_audio(&snapshot).unwrap();
        assert_eq!(scripts.tick(0.0, 0.0).unwrap()[0].1["alpha"], 0.75);
        let mut other = Scripts::new(&objects, &serde_json::json!({})).unwrap();
        other
            .set_audio(&crate::audio::AudioSnapshot::default())
            .unwrap();
        assert_eq!(other.tick(0.0, 0.0).unwrap()[0].1["alpha"], 0.0);
        snapshot.spectrum[..4].fill(f32::NAN);
        snapshot.spectrum[64..68].fill(f32::INFINITY);
        scripts.set_audio(&snapshot).unwrap();
        assert_eq!(scripts.tick(0.1, 0.1).unwrap()[0].1["alpha"], 0.0);

        for size in [8, 128] {
            let mut objects = assets.objects.clone();
            objects[0].properties["alpha"] = serde_json::json!({
                "value": 1,
                "script": format!("engine.registerAudioBuffers({size});")
            });
            assert!(Scripts::new(&objects, &serde_json::json!({})).is_err());
        }

        let mut objects = assets.objects.clone();
        objects[0].properties["alpha"] = serde_json::json!({
            "value": 1,
            "script": (0..129)
                .map(|_| "engine.registerAudioBuffers(16);\n")
                .collect::<String>()
        });
        assert!(Scripts::new(&objects, &serde_json::json!({})).is_err());
        drop(scripts);
    }

    #[test]
    fn rust_script_execution_memory_job_and_io_limits_release_vm() {
        use crate::scene::{assets::Assets, fixtures};
        let baseline = counts();
        let assets = Assets::load("scene.json", &fixtures::image_resources()).unwrap();
        for source in [
            "while(true){}",
            "let values=[];while(true){values.push(new Uint8Array(1048576));}",
            "export function init(){return std.loadFile('/etc/passwd');}",
            "import * as fs from 'std';",
        ] {
            let mut objects = assets.objects.clone();
            objects[0].properties["alpha"] = serde_json::json!({"value":1,"script":source});
            let start = Instant::now();
            assert!(
                Scripts::new(&objects, &serde_json::json!({})).is_err(),
                "Script unexpectedly accepted: {source}"
            );
            assert!(start.elapsed() < Duration::from_secs(2));
            assert_eq!(counts(), baseline);
        }
        let mut objects = assets.objects.clone();
        objects[0].properties["alpha"] = serde_json::json!({"value":1,"script":"export function update(value){while(true){} return value;}"});
        let mut scripts = Scripts::new(&objects, &serde_json::json!({})).unwrap();
        assert!(scripts.tick(0.0, 0.0).is_err());
        assert!(
            scripts.tick(0.1, 0.1).is_ok(),
            "Failed binding must be disabled without poisoning subsequent frames"
        );
        drop(scripts);
        assert_eq!(counts(), baseline);
    }
    #[test]
    fn rust_timer_cancel_reentry_vectors_and_registration_budget() {
        use crate::scene::{assets::Assets, fixtures};
        let baseline = counts();
        let assets = Assets::load("scene.json", &fixtures::image_resources()).unwrap();
        let mut objects = assets.objects.clone();
        objects[0].properties["alpha"] = serde_json::json!({"value":1,"script":"const cancelled=engine.setTimeout(()=>{thisLayer.alpha=99;},0);cancelled();engine.setTimeout(()=>{thisLayer.alpha=0.5;engine.setTimeout(()=>{thisLayer.alpha=0.25;},0);},100);export function update(value){thisScene.getLayer('Right').origin=new Vec3(40,32,0);return value;}"});
        let mut scripts = Scripts::new(&objects, &serde_json::json!({})).unwrap();
        let values = scripts.tick(0.0, 0.0).unwrap();
        assert_eq!(values[0].1["alpha"], 1);
        assert_eq!(values[1].1["origin"], serde_json::json!([40, 32, 0]));
        assert_eq!(scripts.tick(0.1, 0.1).unwrap()[0].1["alpha"], 0.5);
        assert_eq!(scripts.tick(0.11, 0.01).unwrap()[0].1["alpha"], 0.25);
        drop(scripts);
        assert_eq!(counts(), baseline);
        objects[0].properties["alpha"]["script"] =
            serde_json::json!("for(let i=0;i<1025;i++)engine.setTimeout(()=>{},1000);");
        assert!(Scripts::new(&objects, &serde_json::json!({})).is_err());
        assert_eq!(counts(), baseline);
    }
    #[test]
    fn rust_script_property_builder_seeds_and_init_project_properties() {
        use crate::scene::{assets::Assets, fixtures};
        let assets = Assets::load("scene.json", &fixtures::image_resources()).unwrap();
        let mut objects = assets.objects.clone();
        objects[0].properties["alpha"] = serde_json::json!({"value":1,"scriptproperties":{"gain":0.25},"script":"export var scriptProperties=createScriptProperties().addSlider({name:'gain',value:0.5}).finish();export function init(value){thisLayer.alpha=engine.userProperties.opacity;} export function update(value){return value*scriptProperties.gain;}"});
        let mut scripts = Scripts::new(&objects, &serde_json::json!({"opacity":0.8})).unwrap();
        assert!(
            (scripts.tick(0.0, 0.0).unwrap()[0].1["alpha"]
                .as_f64()
                .unwrap()
                - 0.2)
                .abs()
                < 1e-10
        );
    }
}
