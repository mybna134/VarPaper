//! Typed, bounded CPU assets. GPU construction follows successful validation.
use super::{
    model::{vector, SceneDocument},
    texture::Texture,
    SceneResources,
};
use anyhow::{bail, ensure, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use wayvid_library::project::resource_name;

#[derive(Clone, Debug)]
pub enum ObjectKind {
    Image(String),
    Text(Value),
    Sound(Vec<String>),
    Particle(String),
    Group,
}
#[derive(Clone, Debug)]
pub struct Object {
    pub id: i64,
    pub name: String,
    pub dependencies: Vec<i64>,
    pub parent: Option<i64>,
    pub origin: [f32; 3],
    pub scale: [f32; 3],
    pub angles: [f32; 3],
    pub size: [f32; 2],
    pub kind: ObjectKind,
    /// Dynamic property/script bindings remain available to the assignment VM.
    pub properties: Value,
}
#[derive(Debug)]
pub struct ImageModel {
    pub material: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub properties: Value,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    Normal,
    Additive,
    Translucent,
}
#[derive(Debug)]
pub struct Pass {
    pub shader: String,
    pub textures: BTreeMap<u32, String>,
    pub combos: BTreeMap<String, i32>,
    pub blend: Blend,
    pub cull: bool,
    pub depth_test: bool,
    pub depth_write: bool,
    pub constants: Value,
    pub sources: Option<[super::shader::Source; 2]>,
}
#[derive(Debug)]
pub struct Material {
    pub passes: Vec<Pass>,
}
#[derive(Debug)]
pub enum EffectStep {
    Draw {
        material: String,
        pass: usize,
        source: Option<String>,
        target: Option<String>,
    },
    Copy {
        source: String,
        target: String,
    },
}
#[derive(Debug, Default)]
pub struct EffectPipeline {
    pub steps: Vec<EffectStep>,
    pub targets: BTreeMap<String, f32>,
}
pub struct Assets {
    pub document: SceneDocument,
    pub objects: Vec<Object>,
    pub object_index: BTreeMap<i64, usize>,
    pub models: BTreeMap<String, ImageModel>,
    pub materials: BTreeMap<String, Material>,
    pub textures: BTreeMap<String, Texture>,
    pub effects: BTreeMap<i64, EffectPipeline>,
    pub particles: BTreeMap<String, super::particle::Definition>,
    pub text_models: BTreeMap<i64, String>,
    pub sounds: BTreeMap<String, Vec<u8>>,
    pub(super) fonts: super::text::Fonts,
}

struct Loader<'a> {
    resources: &'a dyn SceneResources,
    bytes: usize,
    nodes: BTreeSet<String>,
}
impl Loader<'_> {
    fn read(&mut self, path: &str, limit: usize) -> Result<Vec<u8>> {
        resource_name(path).with_context(|| format!("Scene resource '{path}'"))?;
        self.nodes.insert(path.into());
        ensure!(
            self.nodes.len() <= 4096,
            "Scene resource graph exceeds 4096 nodes"
        );
        let bytes = self
            .resources
            .read(path)
            .with_context(|| format!("Scene resource '{path}'"))?;
        ensure!(
            bytes.len() <= limit,
            "Scene resource '{path}' exceeds budget"
        );
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .context("Scene resource budget overflow")?;
        ensure!(
            self.bytes <= 256 * 1024 * 1024,
            "Scene resource graph exceeds 256 MiB budget"
        );
        Ok(bytes)
    }
    fn json(&mut self, path: &str) -> Result<Value> {
        let value: Value = serde_json::from_slice(&self.read(path, 1024 * 1024)?)
            .with_context(|| format!("Scene JSON resource '{path}'"))?;
        ensure!(value.is_object(), "Scene JSON '{path}' must be an object");
        Ok(value)
    }
}
impl Loader<'_> {
    fn material(
        &mut self,
        material: &str,
        materials: &mut BTreeMap<String, Material>,
        textures: &mut BTreeMap<String, Texture>,
        texture_bytes: &mut usize,
    ) -> Result<()> {
        if materials.contains_key(material) {
            return Ok(());
        }
        let value = self.json(material)?;
        self.material_value(material, value, materials, textures, texture_bytes)
    }
    fn material_value(
        &mut self,
        material: &str,
        value: Value,
        materials: &mut BTreeMap<String, Material>,
        textures: &mut BTreeMap<String, Texture>,
        texture_bytes: &mut usize,
    ) -> Result<()> {
        let mut parsed =
            Material::parse(&value).with_context(|| format!("Scene material '{material}'"))?;
        for pass in &mut parsed.passes {
            use super::shader::{Source, Stage};
            let mut sources = [
                Source::load(&pass.shader, Stage::Vertex, &pass.combos, self.resources)?,
                Source::load(&pass.shader, Stage::Fragment, &pass.combos, self.resources)?,
            ];
            let explicit = pass.textures.clone();
            let explicit_combos = pass.combos.clone();
            for source in &sources {
                for (combo, value) in &source.combos {
                    pass.combos.entry(combo.clone()).or_insert(*value);
                }
            }
            for source in &sources {
                for (uniform, metadata) in &source.metadata {
                    let Some(unit) = uniform
                        .strip_prefix("g_Texture")
                        .and_then(|unit| unit.parse::<u32>().ok())
                    else {
                        continue;
                    };
                    ensure!(unit < 16, "Shader texture slot exceeds 16: {uniform}");
                    if let Some(requirements) = metadata.get("require") {
                        let requirements = requirements
                            .as_object()
                            .context("Shader texture requirements must be an object")?;
                        ensure!(
                            requirements.len() <= 256,
                            "Shader texture requirement budget exceeded"
                        );
                        let any = metadata
                            .get("requireany")
                            .map(|value| {
                                value.as_bool().context("Shader requireany must be boolean")
                            })
                            .transpose()?
                            .unwrap_or(false);
                        let matches = requirements
                            .iter()
                            .map(|(name, value)| -> Result<bool> {
                                ensure!(
                                    identifier(name),
                                    "Invalid shader texture requirement combo '{name}'"
                                );
                                let expected: i32 = value
                                    .as_i64()
                                    .context("Shader texture requirement must be an integer")?
                                    .try_into()?;
                                Ok(pass
                                    .combos
                                    .get(&name.to_ascii_uppercase())
                                    .copied()
                                    .unwrap_or(0)
                                    == expected)
                            })
                            .collect::<Result<Vec<_>>>()?;
                        let required = if any {
                            matches.iter().any(|value| *value)
                        } else {
                            matches.iter().all(|value| *value)
                        };
                        if !required && !explicit.contains_key(&unit) {
                            continue;
                        }
                    }
                    if let Some(combo) = metadata["combo"].as_str() {
                        ensure!(identifier(combo), "Invalid shader texture combo {combo}");
                        if explicit.contains_key(&unit)
                            && !explicit_combos.contains_key(&combo.to_ascii_uppercase())
                        {
                            pass.combos.insert(combo.to_ascii_uppercase(), 1);
                        }
                    }
                    if let std::collections::btree_map::Entry::Vacant(entry) =
                        pass.textures.entry(unit)
                    {
                        if let Some(default) = metadata["default"].as_str() {
                            resource_name(default)
                                .with_context(|| format!("Shader default texture '{default}'"))?;
                            entry.insert(default.into());
                        }
                    }
                }
            }
            // Stage declarations share combo defaults, as one linked program.
            for (source, stage) in sources.iter_mut().zip([Stage::Vertex, Stage::Fragment]) {
                if source.combos != pass.combos {
                    *source = Source::load(&pass.shader, stage, &pass.combos, self.resources)?;
                }
                self.nodes.extend(source.files.iter().cloned());
                self.bytes = self
                    .bytes
                    .checked_add(source.text.len())
                    .context("Scene shader budget overflow")?;
                ensure!(
                    self.nodes.len() <= 4096 && self.bytes <= 256 * 1024 * 1024,
                    "Scene shader resource graph budget exceeded"
                );
            }
            pass.sources = Some(sources);
            for name in pass.textures.values() {
                // Render-target names are resolved by the effect pass graph.
                if name.starts_with("_rt_") {
                    continue;
                }
                let name = format!("materials/{name}.tex");
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    textures.entry(name.clone())
                {
                    let parsed = Texture::parse(&self.read(&name, 256 * 1024 * 1024)?)
                        .with_context(|| format!("Scene texture '{name}'"))?;
                    let bytes: usize = parsed
                        .images
                        .iter()
                        .flatten()
                        .map(|mip| mip.rgba.len())
                        .sum();
                    *texture_bytes = texture_bytes
                        .checked_add(bytes)
                        .context("Scene texture cache size overflow")?;
                    ensure!(
                        *texture_bytes <= 256 * 1024 * 1024,
                        "Scene texture cache exceeds 256 MiB budget"
                    );
                    entry.insert(parsed);
                }
            }
        }
        materials.insert(material.into(), parsed);
        Ok(())
    }
}

fn path(value: &Value) -> Result<String> {
    let name = value.as_str().context("Resource name must be a string")?;
    resource_name(name)?;
    Ok(name.into())
}
fn bound(value: &Value) -> &Value {
    if value.is_object() {
        &value["value"]
    } else {
        value
    }
}
impl Object {
    pub(super) fn parse(value: &Value) -> Result<Self> {
        ensure!(value.is_object(), "Scene object must be an object");
        let id = value["id"]
            .as_i64()
            .context("Scene object id must be an integer")?;
        let name = match &value["name"] {
            Value::String(name) => name.clone(),
            Value::Number(name) => name.to_string(),
            _ => bail!("Scene object {id} must have a name"),
        };
        ensure!(name.len() <= 4096, "Scene object name exceeds budget");
        let mut dependencies = Vec::new();
        let parent = value
            .get("parent")
            .map(|value| value.as_i64().context("Invalid Scene parent id"))
            .transpose()?;
        if let Some(parent) = value.get("parent") {
            dependencies.push(parent.as_i64().context("Invalid Scene parent id")?);
        }
        if let Some(ids) = value.get("dependencies") {
            let ids = ids
                .as_array()
                .context("Scene dependencies must be an array")?;
            ensure!(ids.len() <= 4096, "Scene dependency count exceeds budget");
            for id in ids {
                dependencies.push(id.as_i64().context("Invalid Scene dependency id")?);
            }
        }
        let kinds: Vec<_> = [
            "image", "text", "sound", "particle", "light", "shape", "model",
        ]
        .into_iter()
        .filter(|key| value.get(*key).is_some())
        .collect();
        ensure!(kinds.len() <= 1, "Scene object {id} has conflicting kinds");
        let kind = match kinds.first().copied() {
            Some("image") => ObjectKind::Image(path(&value["image"])?),
            Some("text") => {
                ensure!(
                    bound(&value["text"]).is_string(),
                    "Scene text must be a string or binding"
                );
                ObjectKind::Text(value["text"].clone())
            }
            Some("sound") => {
                let sounds = value["sound"]
                    .as_array()
                    .context("Scene sound must be an array")?;
                ensure!(
                    !sounds.is_empty() && sounds.len() <= 256,
                    "Invalid Scene sound list"
                );
                ObjectKind::Sound(sounds.iter().map(path).collect::<Result<_>>()?)
            }
            Some("particle") => ObjectKind::Particle(path(&value["particle"])?),
            None if value["solid"].as_bool() == Some(true) => ObjectKind::Group,
            _ => bail!(
                "Unsupported required Scene object {id}: {}",
                kinds.first().copied().unwrap_or("unknown")
            ),
        };
        Ok(Self {
            id,
            name,
            dependencies,
            parent,
            origin: vector(bound(&value["origin"]), [0.0; 3])?,
            scale: vector(bound(&value["scale"]), [1.0; 3])?,
            angles: vector(bound(&value["angles"]), [0.0; 3])?,
            size: vector(bound(&value["size"]), [0.0; 2])?,
            kind,
            properties: value.clone(),
        })
    }
}
fn validate_dependencies(objects: &[Object]) -> Result<()> {
    let objects: BTreeMap<_, _> = objects.iter().map(|object| (object.id, object)).collect();
    let mut done = BTreeSet::new();
    let mut active = BTreeSet::new();
    for id in objects.keys() {
        let mut stack = vec![(*id, false)];
        while let Some((id, exiting)) = stack.pop() {
            if exiting {
                active.remove(&id);
                done.insert(id);
                continue;
            }
            if done.contains(&id) {
                continue;
            }
            ensure!(active.insert(id), "Scene dependency cycle at object {id}");
            ensure!(active.len() <= 128, "Scene dependency depth exceeds 128");
            let object = objects
                .get(&id)
                .with_context(|| format!("Missing Scene dependency object {id}"))?;
            stack.push((id, true));
            for dependency in object.dependencies.iter().rev() {
                stack.push((*dependency, false));
            }
        }
    }
    Ok(())
}
fn switch(value: &Value, default: &str, allowed: [&str; 2]) -> Result<bool> {
    let value = if value.is_null() {
        default
    } else {
        value.as_str().context("Material mode must be a string")?
    };
    ensure!(
        allowed.contains(&value),
        "Unsupported required material mode '{value}'"
    );
    Ok(value == allowed[1])
}
impl Material {
    fn parse(value: &Value) -> Result<Self> {
        let passes = value["passes"]
            .as_array()
            .context("Material must have passes")?;
        ensure!(
            !passes.is_empty() && passes.len() <= 64,
            "Invalid material pass count"
        );
        let mut result = Vec::new();
        for (index, value) in passes.iter().enumerate() {
            let pass = (|| -> Result<Pass> {
                ensure!(value.is_object(), "Material pass must be an object");
                let shader = path(&value["shader"])?;
                let constants = &value["constantshadervalues"];
                ensure!(
                    constants.is_null()
                        || constants
                            .as_object()
                            .is_some_and(|values| values.len() <= 256),
                    "Material shader constants must be an object with at most 256 entries"
                );
                let mut textures = BTreeMap::new();
                if let Some(values) = value.get("textures") {
                    let values = values
                        .as_array()
                        .context("Material textures must be an array")?;
                    ensure!(values.len() <= 16, "Material texture count exceeds 16");
                    for (index, value) in values.iter().enumerate() {
                        if value.is_null() || value.as_str() == Some("") {
                            continue;
                        }
                        let name = path(if value.is_object() {
                            &value["name"]
                        } else {
                            value
                        })?;
                        textures.insert(index as u32, name);
                    }
                }
                let mut combos = BTreeMap::new();
                if let Some(values) = value.get("combos") {
                    let values = values
                        .as_object()
                        .context("Material combos must be an object")?;
                    ensure!(values.len() <= 256, "Material combo budget exceeded");
                    for (name, value) in values {
                        ensure!(identifier(name), "Invalid shader combo identifier '{name}'");
                        combos.insert(
                            name.to_ascii_uppercase(),
                            value
                                .as_i64()
                                .context("Shader combo must be an integer")?
                                .try_into()?,
                        );
                    }
                }
                let blending = value
                    .get("blending")
                    .map(|value| value.as_str().context("Material blending must be a string"))
                    .transpose()?
                    .unwrap_or("normal");
                let blend = match blending {
                    "normal" => Blend::Normal,
                    "additive" => Blend::Additive,
                    "translucent" => Blend::Translucent,
                    mode => bail!("Unsupported material blending '{mode}'"),
                };
                Ok(Pass {
                    shader,
                    textures,
                    combos,
                    blend,
                    cull: switch(&value["cullmode"], "nocull", ["nocull", "normal"])?,
                    depth_test: switch(&value["depthtest"], "disabled", ["disabled", "enabled"])?,
                    depth_write: switch(&value["depthwrite"], "disabled", ["disabled", "enabled"])?,
                    constants: value["constantshadervalues"].clone(),
                    sources: None,
                })
            })()
            .with_context(|| format!("Material pass {index}"))?;
            result.push(pass);
        }
        Ok(Self { passes: result })
    }
}
pub(super) fn identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name.len() <= 256
}

impl Assets {
    pub fn load(entry: &str, resources: &dyn SceneResources) -> Result<Self> {
        let mut loader = Loader {
            resources,
            bytes: 0,
            nodes: BTreeSet::new(),
        };
        let document = SceneDocument::parse(&loader.read(entry, 1024 * 1024)?)
            .with_context(|| format!("Scene document '{entry}'"))?;
        let objects: Vec<_> = document
            .objects
            .iter()
            .enumerate()
            .map(|(index, value)| {
                Object::parse(value).with_context(|| format!("Scene '{entry}' object {index}"))
            })
            .collect::<Result<_>>()?;
        let ids: BTreeSet<_> = objects.iter().map(|object| object.id).collect();
        ensure!(ids.len() == objects.len(), "Duplicate Scene object id");
        validate_dependencies(&objects)?;
        let object_index = objects
            .iter()
            .enumerate()
            .map(|(index, object)| (object.id, index))
            .collect();
        let mut assets = Self {
            document,
            objects,
            object_index,
            models: BTreeMap::new(),
            materials: BTreeMap::new(),
            textures: BTreeMap::new(),
            effects: BTreeMap::new(),
            particles: BTreeMap::new(),
            text_models: BTreeMap::new(),
            sounds: BTreeMap::new(),
            fonts: super::text::Fonts::default(),
        };
        let mut texture_bytes = 0usize;
        for object in &assets.objects {
            match &object.kind {
                ObjectKind::Image(name) if !assets.models.contains_key(name) => {
                    let value = loader.json(name)?;
                    let material = path(&value["material"])
                        .with_context(|| format!("Scene model '{name}' material"))?;
                    let dimension = |key: &str| -> Result<Option<u32>> {
                        value
                            .get(key)
                            .map(super::model::dimension)
                            .transpose()
                            .with_context(|| format!("Scene model '{name}' field '{key}'"))
                    };
                    let model = ImageModel {
                        material: material.clone(),
                        width: dimension("width")?,
                        height: dimension("height")?,
                        properties: value,
                    };
                    loader.material(
                        &material,
                        &mut assets.materials,
                        &mut assets.textures,
                        &mut texture_bytes,
                    )?;
                    assets.models.insert(name.clone(), model);
                }
                ObjectKind::Particle(name) if !assets.particles.contains_key(name) => {
                    let definition = super::particle::Definition::parse(&loader.json(name)?)
                        .with_context(|| format!("Scene particle '{name}'"))?;
                    loader.material(
                        &definition.material,
                        &mut assets.materials,
                        &mut assets.textures,
                        &mut texture_bytes,
                    )?;
                    assets.particles.insert(name.clone(), definition);
                }
                ObjectKind::Text(_) => {
                    let texture = super::text::raster(
                        object,
                        super::text::value(object)?,
                        resources,
                        &mut assets.fonts,
                    )
                    .with_context(|| format!("Scene text layer '{}'", object.name))?;
                    texture_bytes = texture_bytes
                        .checked_add(texture.images[0][0].rgba.len())
                        .context("Scene text texture budget overflow")?;
                    ensure!(
                        texture_bytes <= 256 * 1024 * 1024,
                        "Scene texture cache exceeds 256 MiB budget"
                    );
                    let texture_name = format!("__rust_text_{}", object.id);
                    let model_name = format!("__rust_text_{}.json", object.id);
                    let material_name = format!("__rust_text_{}.material", object.id);
                    let source = |text: &str| super::shader::Source {
                        text: text.to_string(),
                        files: Vec::new(),
                        metadata: BTreeMap::new(),
                        combos: BTreeMap::new(),
                    };
                    let sources=[source("#version 330 core\nin vec3 a_Position; in vec2 a_TexCoord; uniform mat4 g_ModelViewProjectionMatrix; out vec2 uv; void main(){uv=a_TexCoord; gl_Position=g_ModelViewProjectionMatrix*vec4(a_Position,1.0);}"),source("#version 330 core\nin vec2 uv; uniform sampler2D g_Texture0; uniform vec3 g_Color; uniform float g_Alpha; out vec4 color; void main(){vec4 p=texture(g_Texture0,uv);color=vec4(p.rgb*g_Color,p.a*g_Alpha);}")];
                    assets.materials.insert(
                        material_name.clone(),
                        Material {
                            passes: vec![Pass {
                                shader: "rust-text".into(),
                                textures: BTreeMap::from([(0, texture_name.clone())]),
                                combos: BTreeMap::new(),
                                blend: Blend::Translucent,
                                cull: false,
                                depth_test: false,
                                depth_write: false,
                                constants: Value::Null,
                                sources: Some(sources),
                            }],
                        },
                    );
                    assets.models.insert(
                        model_name.clone(),
                        ImageModel {
                            material: material_name,
                            width: Some(texture.width),
                            height: Some(texture.height),
                            properties: serde_json::json!({}),
                        },
                    );
                    assets
                        .textures
                        .insert(format!("materials/{texture_name}.tex"), texture);
                    assets.text_models.insert(object.id, model_name);
                }
                ObjectKind::Sound(names) => {
                    for name in names {
                        if !assets.sounds.contains_key(name) {
                            assets
                                .sounds
                                .insert(name.clone(), loader.read(name, 64 * 1024 * 1024)?);
                        }
                    }
                }
                _ => {}
            }
        }
        for object in &assets.objects {
            let model = match &object.kind {
                ObjectKind::Image(name) => name,
                ObjectKind::Text(_) => &assets.text_models[&object.id],
                _ => continue,
            };
            let base = &assets.models[model].material;
            let mut pipeline = EffectPipeline::default();
            for pass in 0..assets.materials[base].passes.len() {
                pipeline.steps.push(EffectStep::Draw {
                    material: base.clone(),
                    pass,
                    source: None,
                    target: None,
                });
            }
            if let Some(effects) = object.properties.get("effects") {
                let effects = effects
                    .as_array()
                    .context("Scene effects must be an array")?;
                ensure!(effects.len() <= 64, "Scene effect count exceeds 64");
                for (index, instance) in effects.iter().enumerate() {
                    ensure!(
                        instance.is_object(),
                        "Scene effect instance must be an object"
                    );
                    let visible = bound(&instance["visible"]);
                    if !visible.is_null() {
                        ensure!(
                            visible.is_boolean(),
                            "Scene effect visibility must be boolean"
                        );
                        if visible == &Value::Bool(false) {
                            continue;
                        }
                    }
                    let file = path(&instance["file"])?;
                    let effect = loader
                        .json(&file)
                        .with_context(|| format!("Scene effect '{file}'"))?;
                    let mut names = BTreeMap::new();
                    if let Some(fbos) = effect.get("fbos") {
                        let fbos = fbos.as_array().context("Effect fbos must be an array")?;
                        ensure!(fbos.len() <= 64, "Effect FBO count exceeds 64");
                        for fbo in fbos {
                            let name = fbo["name"]
                                .as_str()
                                .context("Effect FBO must have a name")?;
                            ensure!(identifier(name), "Invalid effect FBO name '{name}'");
                            ensure!(
                                fbo.get("format")
                                    .is_none_or(|v| v.as_str() == Some("rgba8888")),
                                "Unsupported required effect FBO format"
                            );
                            ensure!(
                                fbo.get("unique").is_none_or(Value::is_boolean),
                                "Effect FBO unique must be boolean"
                            );
                            let scale = fbo
                                .get("scale")
                                .map(|v| v.as_f64().context("Invalid effect FBO scale"))
                                .transpose()?
                                .unwrap_or(1.0) as f32;
                            ensure!(scale.is_finite() && scale > 0.0, "Invalid effect FBO scale");
                            let key = format!("_rt_effect_{}_{}_{}", object.id, index, name);
                            ensure!(
                                names.insert(name.to_string(), key.clone()).is_none(),
                                "Duplicate effect FBO name '{name}'"
                            );
                            pipeline.targets.insert(key, scale);
                        }
                    }
                    let target_name = |v: &Value| -> Result<String> {
                        let name = v.as_str().context("Effect target must be a string")?;
                        names.get(name).cloned().with_context(|| {
                            format!("Effect '{file}' references undeclared target '{name}'")
                        })
                    };
                    if let Some(deps) = effect.get("dependencies") {
                        let deps = deps
                            .as_array()
                            .context("Effect dependencies must be an array")?;
                        ensure!(deps.len() <= 256, "Effect dependency budget exceeded");
                        for dependency in deps {
                            loader.read(&path(dependency)?, 64 * 1024 * 1024)?;
                        }
                    }
                    let passes = effect["passes"]
                        .as_array()
                        .context("Effect must have passes")?;
                    ensure!(
                        !passes.is_empty() && passes.len() <= 64,
                        "Invalid effect pass count"
                    );
                    let overrides = instance
                        .get("passes")
                        .map(|v| v.as_array().context("Effect overrides must be an array"))
                        .transpose()?;
                    ensure!(
                        overrides.is_none_or(|v| v.len() <= passes.len()),
                        "Too many effect pass overrides"
                    );
                    let mut material_index = 0;
                    for (pass_index, pass) in passes.iter().enumerate() {
                        ensure!(pass.is_object(), "Effect pass must be an object");
                        if let Some(command) = pass.get("command") {
                            ensure!(
                                pass.get("material").is_none(),
                                "Effect pass has both material and command"
                            );
                            ensure!(
                                command.as_str() == Some("copy"),
                                "Unsupported required effect command: {command}"
                            );
                            pipeline.steps.push(EffectStep::Copy {
                                source: target_name(&pass["source"])?,
                                target: target_name(&pass["target"])?,
                            });
                            continue;
                        }
                        let material_file = path(&pass["material"])?;
                        let mut value = loader.json(&material_file)?;
                        let material_passes = value["passes"]
                            .as_array_mut()
                            .context("Effect material must have passes")?;
                        let override_pass = overrides.and_then(|v| v.get(material_index));
                        material_index += 1;
                        let mut bindings = BTreeMap::new();
                        if let Some(binds) = pass.get("bind") {
                            let binds = binds.as_array().context("Effect bind must be an array")?;
                            ensure!(binds.len() <= 16, "Effect binding budget exceeded");
                            for bind in binds {
                                let slot = bind["index"]
                                    .as_u64()
                                    .context("Invalid effect bind index")?;
                                ensure!(slot < 16, "Invalid effect bind index");
                                ensure!(
                                    bindings
                                        .insert(slot as usize, target_name(&bind["name"])?)
                                        .is_none(),
                                    "Duplicate effect binding"
                                );
                            }
                        }
                        for material_pass in material_passes.iter_mut() {
                            if let Some(overrides) = override_pass {
                                ensure!(
                                    overrides.is_object(),
                                    "Effect pass override must be an object"
                                );
                                for field in ["combos", "constantshadervalues"] {
                                    if let Some(update) = overrides.get(field) {
                                        let update = update.as_object().context(
                                            "Effect pass parameter override must be an object",
                                        )?;
                                        let object = material_pass
                                            .as_object_mut()
                                            .context("Invalid effect material pass")?;
                                        let prior = object
                                            .entry(field)
                                            .or_insert_with(|| serde_json::json!({}));
                                        if prior.is_null() {
                                            *prior = serde_json::json!({});
                                        }
                                        prior
                                            .as_object_mut()
                                            .context("Invalid effect parameter map")?
                                            .extend(update.clone());
                                    }
                                }
                                for field in ["textures", "usertextures"] {
                                    if let Some(update) = overrides.get(field) {
                                        let update = update
                                            .as_array()
                                            .context("Effect texture override must be an array")?;
                                        ensure!(
                                            update.len() <= 16,
                                            "Effect texture override exceeds 16"
                                        );
                                        let object = material_pass
                                            .as_object_mut()
                                            .context("Invalid effect material pass")?;
                                        let prior = object
                                            .entry("textures")
                                            .or_insert_with(|| serde_json::json!([]));
                                        let prior = prior
                                            .as_array_mut()
                                            .context("Invalid material textures")?;
                                        for (slot, texture) in update.iter().enumerate() {
                                            if texture.is_null() || texture.as_str() == Some("") {
                                                continue;
                                            }
                                            prior.resize(prior.len().max(slot + 1), Value::Null);
                                            prior[slot] = texture.clone();
                                        }
                                    }
                                }
                            }
                            let object = material_pass
                                .as_object_mut()
                                .context("Invalid effect material pass")?;
                            let textures = object
                                .entry("textures")
                                .or_insert_with(|| serde_json::json!([]))
                                .as_array_mut()
                                .context("Invalid material textures")?;
                            for (slot, name) in &bindings {
                                textures.resize(textures.len().max(slot + 1), Value::Null);
                                textures[*slot] = Value::String(name.clone());
                            }
                            for texture in textures {
                                if let Some(name) =
                                    texture.as_str().and_then(|name| names.get(name))
                                {
                                    *texture = Value::String(name.clone());
                                }
                            }
                        }
                        let key = format!(
                            "{material_file}#layer{}-effect{index}-pass{pass_index}",
                            object.id
                        );
                        loader.material_value(
                            &key,
                            value,
                            &mut assets.materials,
                            &mut assets.textures,
                            &mut texture_bytes,
                        )?;
                        for material_pass in 0..assets.materials[&key].passes.len() {
                            pipeline.steps.push(EffectStep::Draw {
                                material: key.clone(),
                                pass: material_pass,
                                source: pass.get("source").map(&target_name).transpose()?,
                                target: pass.get("target").map(&target_name).transpose()?,
                            });
                        }
                    }
                }
            }
            ensure!(
                pipeline.steps.len() <= 4096,
                "Scene layer effect graph exceeds 4096 passes"
            );
            let mut initialized = BTreeSet::new();
            for step in &pipeline.steps {
                match step {
                    EffectStep::Copy { source, target } => {
                        ensure!(
                            initialized.contains(source),
                            "Effect copy reads an unwritten target '{source}'"
                        );
                        ensure!(source != target, "Effect copy cannot read its destination");
                        initialized.insert(target.clone());
                    }
                    EffectStep::Draw {
                        material,
                        pass,
                        source,
                        target,
                    } => {
                        let pass = &assets.materials[material].passes[*pass];
                        for input in source
                            .iter()
                            .chain(pass.textures.values().filter(|v| v.starts_with("_rt_")))
                        {
                            ensure!(
                                initialized.contains(input),
                                "Effect pass reads an undeclared or unwritten target '{input}'"
                            );
                            ensure!(
                                target.as_ref() != Some(input),
                                "Effect pass cannot sample its destination"
                            );
                        }
                        if let Some(target) = target {
                            initialized.insert(target.clone());
                        }
                    }
                }
            }
            assets.effects.insert(object.id, pipeline);
        }
        Ok(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::fixtures::{image_resources, Resources};

    #[test]
    fn generated_models_materials_and_shared_texture_are_typed() {
        let assets = Assets::load("scene.json", &image_resources()).unwrap();
        assert_eq!(assets.objects.len(), 2);
        assert_eq!(assets.models.len(), 1);
        assert_eq!(assets.materials.len(), 1);
        assert_eq!(assets.textures.len(), 1);
        assert_eq!(assets.objects[0].origin, [16.0, 32.0, 0.0]);
        assert_eq!(assets.objects[1].size, [32.0, 64.0]);
        assert_eq!(
            assets.materials["material.json"].passes[0].blend,
            Blend::Normal
        );
    }
    #[test]
    fn loose_package_and_shared_resources_produce_identical_cpu_assets() {
        use crate::scene::fixtures::{export, package};
        use wayvid_library::project::{discover_project, ProjectResources};
        let temp = tempfile::tempdir().unwrap();
        let shared = temp.path().join("shared-assets");
        std::fs::create_dir_all(shared.join("materials")).unwrap();
        let original = image_resources();
        std::fs::write(
            shared.join("materials/fixture.tex"),
            &original.0["materials/fixture.tex"],
        )
        .unwrap();
        let expected = Assets::load("scene.json", &original).unwrap();
        for kind in [
            "loose",
            "packed",
            "shared",
            "missing-shared",
            "escaping-symlink",
        ] {
            let root = temp.path().join(kind);
            if kind == "loose" || kind == "escaping-symlink" {
                export(&original, &root).unwrap();
                if kind == "escaping-symlink" {
                    std::fs::remove_file(root.join("materials/fixture.tex")).unwrap();
                    std::os::unix::fs::symlink(
                        shared.join("materials/fixture.tex"),
                        root.join("materials/fixture.tex"),
                    )
                    .unwrap();
                }
            } else {
                std::fs::create_dir(&root).unwrap();
                std::fs::write(root.join("project.json"), &original.0["project.json"]).unwrap();
                let mut resources = image_resources();
                if kind != "packed" {
                    resources.0.remove("materials/fixture.tex");
                }
                std::fs::write(root.join("scene.pkg"), package(&resources)).unwrap();
            }
            let project = discover_project(
                &root,
                wayvid_library::SourceType::LocalDirectory,
                None,
                if kind == "shared" {
                    vec![shared.clone()]
                } else {
                    vec![]
                },
            )
            .unwrap()
            .project
            .unwrap();
            let resources = ProjectResources::new(&project).unwrap();
            let result = Assets::load("scene.json", &resources);
            if kind == "missing-shared" || kind == "escaping-symlink" {
                let error = result.err().expect("Missing/escaped texture accepted");
                assert!(
                    format!("{error:#}").contains("materials/fixture.tex"),
                    "{error:#}"
                );
            } else {
                let actual = result.unwrap();
                assert_eq!(actual.objects.len(), expected.objects.len());
                assert_eq!(
                    actual.textures["materials/fixture.tex"].images[0][0].rgba,
                    expected.textures["materials/fixture.tex"].images[0][0].rgba
                );
            }
        }
    }
    fn document(resources: &Resources) -> Value {
        serde_json::from_slice(&resources.0["scene.json"]).unwrap()
    }
    fn replace_document(resources: &mut Resources, document: &Value) {
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(document).unwrap());
    }
    #[test]
    fn missing_invalid_and_cyclic_resources_have_asset_diagnostics() {
        for name in [
            "image.json",
            "material.json",
            "shaders/fixture.vert",
            "materials/fixture.tex",
        ] {
            let mut resources = image_resources();
            resources.0.remove(name);
            let error = Assets::load("scene.json", &resources).err().unwrap();
            assert!(format!("{error:#}").contains(name), "{error:#}");
        }
        let mut resources = image_resources();
        let mut value = document(&resources);
        value["objects"][0]["dependencies"] = serde_json::json!([2]);
        value["objects"][1]["parent"] = serde_json::json!(1);
        replace_document(&mut resources, &value);
        assert!(Assets::load("scene.json", &resources)
            .err()
            .unwrap()
            .to_string()
            .contains("cycle"));
        value["objects"][1]
            .as_object_mut()
            .unwrap()
            .remove("parent");
        value["objects"][0]["dependencies"] = serde_json::json!([999]);
        replace_document(&mut resources, &value);
        assert!(Assets::load("scene.json", &resources)
            .err()
            .unwrap()
            .to_string()
            .contains("999"));
        for invalid in [
            serde_json::json!("../image.json"),
            serde_json::json!("/image.json"),
            serde_json::json!(3),
        ] {
            let mut resources = image_resources();
            let mut value = document(&resources);
            value["objects"][0]["image"] = invalid;
            replace_document(&mut resources, &value);
            let error = Assets::load("scene.json", &resources).err().unwrap();
            assert!(format!("{error:#}").contains("object 0"));
        }
        for invalid in [
            serde_json::json!({"passes":[]}),
            serde_json::json!({"passes":[{"shader":1}]}),
            serde_json::json!({"passes":[{"shader":"fixture","blending":2}]}),
            serde_json::json!({"passes":[{"shader":"fixture","combos":{"bad\n#define HACK":1}}]}),
        ] {
            let mut resources = image_resources();
            resources.0.insert(
                "material.json".into(),
                serde_json::to_vec(&invalid).unwrap(),
            );
            let error = Assets::load("scene.json", &resources).err().unwrap();
            assert!(format!("{error:#}").contains("material.json"), "{error:#}");
        }
    }
}
