//! Rust-owned audio-only mpv streams; project bytes never become external URLs.
use anyhow::{ensure, Context, Result};
use std::{
    ffi::CString,
    fs::File,
    io::{Seek, SeekFrom, Write},
    os::fd::{AsRawFd, FromRawFd},
    time::{Duration, Instant},
};
#[cfg(test)]
thread_local! {static LIVE:std::cell::Cell<usize>=const {std::cell::Cell::new(0)};}
#[cfg(test)]
pub(super) fn counts() -> usize {
    LIVE.with(std::cell::Cell::get)
}
pub(super) struct Stream {
    handle: *mut libmpv_sys::mpv_handle,
    _file: File,
}
impl Stream {
    pub fn new(name: &str, bytes: &[u8], repeat: bool) -> Result<Self> {
        ensure!(
            !bytes.is_empty() && bytes.len() <= 64 * 1024 * 1024,
            "Invalid Scene sound allocation '{name}'"
        );
        let fd = unsafe { libc::memfd_create(c"varpaper-scene-sound".as_ptr(), libc::MFD_CLOEXEC) };
        ensure!(fd >= 0, "Cannot allocate Scene sound storage");
        let mut file = unsafe { File::from_raw_fd(fd) };
        file.write_all(bytes)?;
        file.seek(SeekFrom::Start(0))?;
        super::super::mpv::ensure_c_numeric_locale()?;
        let handle = unsafe { libmpv_sys::mpv_create() };
        ensure!(!handle.is_null(), "Cannot create Scene sound decoder");
        let stream = Self {
            handle,
            _file: file,
        };
        #[cfg(test)]
        LIVE.with(|v| v.set(v.get() + 1));
        for (key, value) in [
            ("config", "no"),
            ("terminal", "no"),
            ("load-scripts", "no"),
            ("ytdl", "no"),
            ("vid", "no"),
            ("pause", "yes"),
            ("audio-display", "no"),
            ("volume", "0"),
            ("mute", "yes"),
            ("loop-file", if repeat { "inf" } else { "no" }),
        ] {
            let key = CString::new(key)?;
            let value = CString::new(value)?;
            ensure!(
                unsafe { libmpv_sys::mpv_set_option_string(handle, key.as_ptr(), value.as_ptr()) }
                    >= 0,
                "Cannot configure Scene sound decoder"
            );
        }
        ensure!(
            unsafe { libmpv_sys::mpv_initialize(handle) } >= 0,
            "Cannot initialize Scene sound decoder"
        );
        let source = CString::new(format!("/proc/self/fd/{}", stream._file.as_raw_fd()))?;
        let mut command = [
            c"loadfile".as_ptr(),
            source.as_ptr(),
            c"replace".as_ptr(),
            std::ptr::null(),
        ];
        ensure!(
            unsafe { libmpv_sys::mpv_command(handle, command.as_mut_ptr()) } >= 0,
            "Cannot load Scene sound '{name}'"
        );
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            ensure!(
                Instant::now() < deadline,
                "Scene sound loading timed out: '{name}'"
            );
            let event = unsafe { libmpv_sys::mpv_wait_event(handle, 0.05) };
            ensure!(!event.is_null(), "Scene sound decoder event unavailable");
            let event = unsafe { &*event };
            if event.event_id as u32 == 8 {
                break;
            }
            if event.event_id as u32 == 7 {
                anyhow::bail!("Cannot decode Scene sound '{name}'");
            }
        }
        Ok(stream)
    }
    pub fn controls(&self, paused: bool, muted: bool, volume: f32) -> Result<()> {
        ensure!(
            volume.is_finite() && (0.0..=1.0).contains(&volume),
            "Invalid Scene sound volume"
        );
        for (key, value) in [
            ("volume", (volume * 100.0).to_string()),
            ("mute", if muted { "yes" } else { "no" }.into()),
            ("pause", if paused { "yes" } else { "no" }.into()),
        ] {
            let key = CString::new(key)?;
            let value = CString::new(value)?;
            ensure!(
                unsafe {
                    libmpv_sys::mpv_set_property_string(self.handle, key.as_ptr(), value.as_ptr())
                } >= 0,
                "Cannot update Scene sound controls"
            );
        }
        Ok(())
    }
    #[cfg(test)]
    pub fn property(&self, name: &str) -> Result<String> {
        let name = CString::new(name)?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let value = unsafe { libmpv_sys::mpv_get_property_string(self.handle, name.as_ptr()) };
            if !value.is_null() {
                let result = unsafe { std::ffi::CStr::from_ptr(value) }
                    .to_string_lossy()
                    .into_owned();
                unsafe {
                    libmpv_sys::mpv_free(value.cast());
                }
                return Ok(result);
            }
            ensure!(
                Instant::now() < deadline,
                "Scene sound property unavailable"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            libmpv_sys::mpv_terminate_destroy(self.handle);
        }
        #[cfg(test)]
        LIVE.with(|v| v.set(v.get() - 1));
    }
}
pub(super) fn load(assets: &super::assets::Assets) -> Result<Vec<Stream>> {
    let count: usize = assets
        .objects
        .iter()
        .map(|object| match &object.kind {
            super::assets::ObjectKind::Sound(names) => names.len(),
            _ => 0,
        })
        .sum();
    ensure!(count <= 64, "Scene sound stream count exceeds 64");
    let mut streams = Vec::with_capacity(count);
    for object in &assets.objects {
        if let super::assets::ObjectKind::Sound(names) = &object.kind {
            let mode = object
                .properties
                .get("playbackmode")
                .map(|v| v.as_str().context("Sound playbackmode must be a string"))
                .transpose()?
                .unwrap_or("once");
            ensure!(
                ["once", "loop"].contains(&mode),
                "Unsupported required sound playbackmode '{mode}'"
            );
            for name in names {
                streams.push(Stream::new(name, &assets.sounds[name], mode == "loop")?);
            }
        }
    }
    Ok(streams)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_sound_controls_pcm_pause_mute_volume_and_cleanup() {
        if std::env::var_os("VARPAPER_TEST_SCENE_SOUND").is_none() {
            return;
        }
        use crate::{
            audio::AudioConsumer,
            scene::{fixtures, runtime::RustSceneRenderer},
        };
        use std::rc::Rc;
        let server = std::env::var("PULSE_SERVER").unwrap();
        assert!(
            server.contains("varpaper-audio-check-"),
            "Private audio server required"
        );
        let native = fixtures::NativeWindow::new();
        let context = native.context();
        let window = Rc::new(context.create_x11_window(native.window, 64, 64).unwrap());
        window.make_current().unwrap();
        context.load_gl_functions();
        let baseline = (
            counts(),
            super::super::gpu::counts(),
            super::super::program::counts(),
            crate::audio::live_counts(),
        );
        let mut resources = fixtures::clear_resources();
        let mut document: serde_json::Value =
            serde_json::from_slice(&resources.0["scene.json"]).unwrap();
        document["objects"] =
            serde_json::json!([{"id":1,"name":"Tone","sound":["tone.wav"],"playbackmode":"loop"}]);
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
        let mut wave = fixtures::wave();
        let samples = 48000u32;
        wave.truncate(44);
        wave[4..8].copy_from_slice(&(36 + samples * 2).to_le_bytes());
        wave[24..28].copy_from_slice(&48000u32.to_le_bytes());
        wave[28..32].copy_from_slice(&96000u32.to_le_bytes());
        wave[40..44].copy_from_slice(&(samples * 2).to_le_bytes());
        for sample in 0..samples {
            let value = (16384.0
                * (std::f64::consts::TAU * 375.0 * f64::from(sample) / 48000.0).sin())
                as i16;
            wave.extend(value.to_le_bytes());
        }
        resources.0.insert("tone.wav".into(), wave.clone());
        let consumer = AudioConsumer::acquire();
        let wait_level = |low: f32, high: f32| {
            let deadline = Instant::now() + Duration::from_secs(8);
            loop {
                let snapshot = consumer.snapshot();
                let value = snapshot.level[0];
                if snapshot.monitor.is_some() && value >= low && value <= high {
                    return value;
                }
                assert!(
                    Instant::now() < deadline,
                    "Expected RMS {low}..{high}: {snapshot:?}"
                );
                std::thread::sleep(Duration::from_millis(20));
            }
        };
        // Driver properties and actual monitor amplitude are both checked.
        let stream = Stream::new("tone.wav", &wave, true).unwrap();
        stream.controls(true, true, 0.5).unwrap();
        assert_eq!(stream.property("pause").unwrap(), "yes");
        assert_eq!(stream.property("mute").unwrap(), "yes");
        assert_eq!(stream.property("volume").unwrap(), "50.000000");
        assert_eq!(stream.property("audio-params/samplerate").unwrap(), "48000");
        drop(stream);
        for _ in 0..3 {
            let mut scene =
                RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                    .unwrap();
            scene.set_audio(false, false, 1.0).unwrap();
            let loud = wait_level(0.2, 0.5);
            scene.set_audio(false, false, 0.25).unwrap();
            let quiet = wait_level(0.001, 0.03);
            assert!(
                quiet < loud * 0.2,
                "Lower volume must attenuate audible PCM"
            );
            scene.set_audio(false, true, 1.0).unwrap();
            wait_level(0.0, 0.002);
            scene.set_audio(false, false, 1.0).unwrap();
            wait_level(0.2, 0.5);
            scene.set_audio(true, false, 1.0).unwrap();
            wait_level(0.0, 0.002);
            scene.set_audio(false, false, 1.0).unwrap();
            wait_level(0.2, 0.5);
            scene.close().unwrap();
            drop(scene);
            wait_level(0.0, 0.002);
            assert_eq!(counts(), baseline.0);
            assert_eq!(super::super::gpu::counts(), baseline.1);
            assert_eq!(super::super::program::counts(), baseline.2);
        }
        document["objects"][0]["sound"] = serde_json::json!(["tone.wav", "bad.wav"]);
        resources
            .0
            .insert("bad.wav".into(), b"broken audio".to_vec());
        resources
            .0
            .insert("scene.json".into(), serde_json::to_vec(&document).unwrap());
        assert!(
            RustSceneRenderer::new("scene.json", &resources, context.clone(), window.clone())
                .is_err()
        );
        assert_eq!(counts(), baseline.0);
        drop(consumer);
        assert_eq!(crate::audio::live_counts(), baseline.3);
    }
}
