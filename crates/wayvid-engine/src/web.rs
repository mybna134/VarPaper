//! Shared out-of-process CEF owner and per-output OpenGL presentation.
use crate::mpv::VideoConfig;
use crate::renderer::{RendererCapabilities, WallpaperRenderer};
use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::ffi::CString;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

const MAX_JSON: usize = 65536;
const MAX_FRAME: usize = 64 * 1024 * 1024;
const IPC_VERSION: u64 = 2;
const RUNTIME_MANIFEST: &str = include_str!("../../../native/web/runtime.json");
thread_local! { static HOST: RefCell<Weak<RefCell<Host>>> = const { RefCell::new(Weak::new()) }; }

pub fn runtime_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VARPAPER_CEF_RUNTIME") {
        return Ok(path.into());
    }
    let root = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .context("Missing application data directory")?;
    Ok(root.join("varpaper/components/web/runtime"))
}
pub fn host_path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VARPAPER_WEB_HOST") {
        return Ok(path.into());
    }
    let executable = std::env::current_exe()?;
    Ok(executable
        .parent()
        .context("Missing executable directory")?
        .join("libexec/varpaper-web-host"))
}
pub fn available() -> bool {
    runtime_path()
        .is_ok_and(|root| root.join("installed.json").is_file() && root.join("libcef.so").is_file())
        && validated_host_path().is_ok()
}
pub fn validate_host_installation() -> Result<()> {
    validated_host_path().map(|_| ())
}
fn validate_host(host: &Path, manifest: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let executable = std::fs::metadata(host).context("Web host is missing")?;
    ensure!(
        executable.is_file() && executable.permissions().mode() & 0o111 != 0,
        "Web host is not executable"
    );
    let file = std::fs::File::open(manifest).context("Web host manifest is missing")?;
    let mut bytes = Vec::new();
    file.take(MAX_JSON as u64 + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= MAX_JSON,
        "Web host manifest exceeds size budget"
    );
    let actual: Value = serde_json::from_slice(&bytes).context("Invalid Web host manifest")?;
    let expected: Value = serde_json::from_str(RUNTIME_MANIFEST)?;
    ensure!(
        expected["ipc_version"] == IPC_VERSION && actual == expected,
        "Web host version or ABI mismatch; reinstall the application"
    );
    Ok(())
}
fn validated_host_path() -> Result<PathBuf> {
    let host = host_path()?;
    let parent = host.parent().context("Missing Web host directory")?;
    let manifest = if std::env::var_os("VARPAPER_WEB_HOST").is_some() {
        parent.join("runtime.json")
    } else {
        parent
            .parent()
            .context("Missing application directory")?
            .join("data/runtime.json")
    };
    validate_host(&host, &manifest)?;
    Ok(host)
}
struct Host {
    socket: UnixStream,
    child: Child,
    request: u64,
    next_id: u64,
    dead: bool,
}
impl Host {
    fn shared() -> Result<Rc<RefCell<Self>>> {
        HOST.with(|slot| {
            if let Some(host) = slot.borrow().upgrade().filter(|host| !host.borrow().dead) {
                return Ok(host);
            }
            let host = Rc::new(RefCell::new(Self::spawn()?));
            *slot.borrow_mut() = Rc::downgrade(&host);
            Ok(host)
        })
    }
    fn spawn() -> Result<Self> {
        let runtime = runtime_path()?
            .canonicalize()
            .context("Install Web wallpaper support in Settings")?;
        ensure!(
            runtime.join("libcef.so").is_file(),
            "Chromium runtime library is missing"
        );
        let (socket, child_socket) = UnixStream::pair()?;
        socket.set_read_timeout(Some(Duration::from_secs(8)))?;
        socket.set_write_timeout(Some(Duration::from_secs(8)))?;
        let fd = child_socket.as_raw_fd();
        let mut command = Command::new(validated_host_path()?);
        command
            .arg(format!("--ipc-fd={fd}"))
            .env("VARPAPER_CEF_RUNTIME", &runtime)
            .env("LD_LIBRARY_PATH", &runtime)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit());
        unsafe {
            command.pre_exec(move || {
                if libc::setsid() < 0 || libc::fcntl(fd, libc::F_SETFD, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command
            .spawn()
            .context("Cannot launch the Web wallpaper host")?;
        drop(child_socket);
        Ok(Self {
            socket,
            child,
            request: 0,
            next_id: 0,
            dead: false,
        })
    }
    fn call(&mut self, mut message: Value) -> Result<(Value, Vec<u8>)> {
        ensure!(
            !self.dead,
            "Web host exited; reapply the wallpaper to retry"
        );
        self.request += 1;
        message["request"] = self.request.into();
        let result = self.exchange(&message);
        if result.is_err() {
            self.dead = true;
            self.terminate();
        }
        let (response, pixels) = result?;
        if response["ok"] != true {
            bail!(
                "{}",
                response["error"]
                    .as_str()
                    .unwrap_or("Web wallpaper host failed")
            );
        }
        Ok((response, pixels))
    }
    fn exchange(&mut self, message: &Value) -> Result<(Value, Vec<u8>)> {
        let data = serde_json::to_vec(message)?;
        ensure!(data.len() <= MAX_JSON, "Web command exceeds size budget");
        self.socket.write_all(&(data.len() as u32).to_le_bytes())?;
        self.socket.write_all(&data)?;
        let mut header = [0; 4];
        self.socket.read_exact(&mut header)?;
        let size = u32::from_le_bytes(header) as usize;
        ensure!(
            size > 0 && size <= MAX_JSON,
            "Invalid Web host response size"
        );
        let mut bytes = vec![0; size];
        self.socket.read_exact(&mut bytes)?;
        let response: Value = serde_json::from_slice(&bytes)?;
        ensure!(
            response["request"] == message["request"]
                && response["id"] == message["id"]
                && response["generation"] == message["generation"],
            "Stale Web host response"
        );
        ensure!(
            response["ipc_version"] == IPC_VERSION,
            "Web host IPC version mismatch"
        );
        let count = response["bytes"]
            .as_u64()
            .context("Missing Web frame size")?;
        ensure!(count <= MAX_FRAME as u64, "Web frame exceeds size budget");
        let mut pixels = vec![0; count as usize];
        self.socket.read_exact(&mut pixels)?;
        Ok((response, pixels))
    }
    fn terminate(&mut self) {
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        let _ = self.child.wait();
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        if !self.dead {
            let _ = self.call(json!({"op":"shutdown", "id":0,"generation":0}));
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        self.terminate();
    }
}

pub struct WebRenderer {
    audio: Option<crate::audio::AudioConsumer>,
    properties: Value,
    host: Rc<RefCell<Host>>,
    id: u64,
    generation: u64,
    sequence: u64,
    width: i32,
    height: i32,
    texture: u32,
    program: u32,
    vao: u32,
    presented: bool,
    paused: bool,
    closed: bool,
    config: VideoConfig,
}
impl WebRenderer {
    pub fn new(
        root: &Path,
        entry: &str,
        width: i32,
        height: i32,
        config: &VideoConfig,
        properties: &Value,
    ) -> Result<Self> {
        let host = Host::shared()?;
        let id = {
            let mut host = host.borrow_mut();
            host.next_id += 1;
            host.next_id
        };
        let mut result = Self {
            audio: Some(crate::audio::AudioConsumer::acquire()),
            properties: properties.clone(),
            host,
            id,
            generation: id,
            sequence: 0,
            width,
            height,
            texture: 0,
            program: 0,
            vao: 0,
            presented: false,
            paused: false,
            closed: false,
            config: config.clone(),
        };
        let mut command = result.command("create");
        command["root"] = root.to_string_lossy().as_ref().into();
        command["entry"] = entry.into();
        command["width"] = width.into();
        command["height"] = height.into();
        command["properties"] = properties.clone();
        result.host.borrow_mut().call(command)?;
        result.update_config(config)?;
        Ok(result)
    }
    fn command(&self, op: &str) -> Value {
        json!({"op":op, "id":self.id,"generation":self.generation})
    }
    fn gpu_init(&mut self) -> Result<()> {
        if self.texture != 0 {
            return Ok(());
        }
        unsafe {
            let vertex = shader(gl::VERTEX_SHADER, "#version 130\nout vec2 uv;void main(){vec2 p=vec2((gl_VertexID==1)?3.0:-1.0,(gl_VertexID==2)?3.0:-1.0);uv=(p+1.0)*0.5;gl_Position=vec4(p,0,1);}")?;
            let fragment = match shader(gl::FRAGMENT_SHADER, "#version 130\nuniform sampler2D image;in vec2 uv;out vec4 color;void main(){color=texture(image,vec2(uv.x,1.0-uv.y));}") {
                Ok(shader) => shader, Err(error) => { gl::DeleteShader(vertex); return Err(error); }
            };
            self.program = gl::CreateProgram();
            gl::AttachShader(self.program, vertex);
            gl::AttachShader(self.program, fragment);
            gl::LinkProgram(self.program);
            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);
            let mut success = 0;
            gl::GetProgramiv(self.program, gl::LINK_STATUS, &mut success);
            ensure!(success != 0, "Cannot link Web presentation shader");
            gl::GenTextures(1, &mut self.texture);
            gl::GenVertexArrays(1, &mut self.vao);
        }
        Ok(())
    }
    fn draw(&mut self, pixels: &[u8], width: i32, height: i32, framebuffer: i32) -> Result<()> {
        self.gpu_init()?;
        unsafe {
            let mut program = 0;
            let mut vao = 0;
            let mut fbo = 0;
            let mut active = 0;
            let mut texture = 0;
            let mut sampler = 0;
            let mut color_mask = [0_u8; 4];
            let mut unpack = 0;
            let mut pbo = 0;
            let mut row = 0;
            let mut skip_rows = 0;
            let mut skip_pixels = 0;
            let mut viewport = [0; 4];
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut program);
            gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut vao);
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut fbo);
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            gl::GetIntegerv(gl::ACTIVE_TEXTURE, &mut active);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut texture);
            if gl::BindSampler::is_loaded() {
                gl::GetIntegerv(gl::SAMPLER_BINDING, &mut sampler);
            }
            gl::GetBooleanv(gl::COLOR_WRITEMASK, color_mask.as_mut_ptr());
            if gl::BindSampler::is_loaded() {
                gl::BindSampler(0, 0);
            }
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::GetIntegerv(gl::UNPACK_ALIGNMENT, &mut unpack);
            gl::GetIntegerv(gl::PIXEL_UNPACK_BUFFER_BINDING, &mut pbo);
            gl::GetIntegerv(gl::UNPACK_ROW_LENGTH, &mut row);
            gl::GetIntegerv(gl::UNPACK_SKIP_ROWS, &mut skip_rows);
            gl::GetIntegerv(gl::UNPACK_SKIP_PIXELS, &mut skip_pixels);
            let blend = gl::IsEnabled(gl::BLEND);
            let depth = gl::IsEnabled(gl::DEPTH_TEST);
            let scissor = gl::IsEnabled(gl::SCISSOR_TEST);
            let cull = gl::IsEnabled(gl::CULL_FACE);
            let srgb = gl::IsEnabled(gl::FRAMEBUFFER_SRGB);
            gl::Disable(gl::BLEND);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::SCISSOR_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::FRAMEBUFFER_SRGB);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
            gl::PixelStorei(gl::UNPACK_SKIP_ROWS, 0);
            gl::PixelStorei(gl::UNPACK_SKIP_PIXELS, 0);
            if !pixels.is_empty() {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
                if self.presented && width == self.width && height == self.height {
                    gl::TexSubImage2D(
                        gl::TEXTURE_2D,
                        0,
                        0,
                        0,
                        width,
                        height,
                        gl::BGRA,
                        gl::UNSIGNED_BYTE,
                        pixels.as_ptr().cast(),
                    );
                } else {
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        gl::RGBA8 as i32,
                        width,
                        height,
                        0,
                        gl::BGRA,
                        gl::UNSIGNED_BYTE,
                        pixels.as_ptr().cast(),
                    );
                }
            }
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, framebuffer as u32);
            gl::Viewport(0, 0, width, height);
            gl::UseProgram(self.program);
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::UseProgram(program as u32);
            gl::BindVertexArray(vao as u32);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, fbo as u32);
            gl::Viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
            gl::BindTexture(gl::TEXTURE_2D, texture as u32);
            if gl::BindSampler::is_loaded() {
                gl::BindSampler(0, sampler as u32);
            }
            gl::ColorMask(color_mask[0], color_mask[1], color_mask[2], color_mask[3]);
            gl::ActiveTexture(active as u32);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, pbo as u32);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, unpack);
            gl::PixelStorei(gl::UNPACK_ROW_LENGTH, row);
            gl::PixelStorei(gl::UNPACK_SKIP_ROWS, skip_rows);
            gl::PixelStorei(gl::UNPACK_SKIP_PIXELS, skip_pixels);
            for (flag, enabled) in [
                (gl::BLEND, blend),
                (gl::DEPTH_TEST, depth),
                (gl::SCISSOR_TEST, scissor),
                (gl::CULL_FACE, cull),
                (gl::FRAMEBUFFER_SRGB, srgb),
            ] {
                if enabled != 0 {
                    gl::Enable(flag);
                }
            }
        }
        Ok(())
    }
}
unsafe fn shader(kind: u32, source: &str) -> Result<u32> {
    let shader = gl::CreateShader(kind);
    let source = CString::new(source)?;
    gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
    gl::CompileShader(shader);
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        gl::DeleteShader(shader);
        bail!("Cannot compile Web presentation shader");
    }
    Ok(shader)
}
impl WallpaperRenderer for WebRenderer {
    fn render(&mut self, width: i32, height: i32, framebuffer: i32) -> Result<bool> {
        ensure!(
            width > 0 && height > 0 && (width as u64) * (height as u64) * 4 <= MAX_FRAME as u64,
            "Web viewport exceeds budget"
        );
        let mut max_texture = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_texture);
        }
        ensure!(
            width <= max_texture && height <= max_texture,
            "Web viewport exceeds GPU texture limits"
        );
        if self.paused {
            return Ok(false);
        }
        let mut command = self.command("frame");
        command["width"] = width.into();
        command["height"] = height.into();
        command["sequence"] = self.sequence.into();
        if let Some(audio) = &self.audio {
            command["spectrum"] = serde_json::to_value(audio.snapshot().spectrum.as_slice())?;
        }
        let (response, pixels) = self.host.borrow_mut().call(command)?;
        ensure!(
            response["width"] == width && response["height"] == height,
            "Stale Web frame dimensions"
        );
        let sequence = response["sequence"]
            .as_u64()
            .context("Missing Web frame sequence")?;
        if pixels.is_empty() {
            return Ok(false);
        }
        ensure!(
            pixels.len() == width as usize * height as usize * 4 && sequence > self.sequence,
            "Invalid Web frame payload"
        );
        self.draw(&pixels, width, height, framebuffer)?;
        self.sequence = sequence;
        self.width = width;
        self.height = height;
        self.presented = true;
        Ok(true)
    }
    fn pause(&mut self) -> Result<()> {
        if !self.paused {
            self.host.borrow_mut().call(self.command("pause"))?;
            self.paused = true;
            self.audio = None;
        }
        Ok(())
    }
    fn resume(&mut self) -> Result<()> {
        if self.paused {
            self.host.borrow_mut().call(self.command("resume"))?;
            self.paused = false;
            self.audio = Some(crate::audio::AudioConsumer::acquire());
        }
        Ok(())
    }
    fn set_volume(&mut self, volume: f32) -> Result<()> {
        let mut config = self.config.clone();
        config.volume = f64::from(volume);
        self.update_config(&config)
    }
    fn update_config(&mut self, config: &VideoConfig) -> Result<()> {
        let mut command = self.command("audio");
        command["mute"] = config.mute.into();
        command["volume"] = config.volume.into();
        self.host.borrow_mut().call(command)?;
        self.config = config.clone();
        Ok(())
    }
    fn close(&mut self) -> Result<()> {
        self.audio = None;
        if !self.closed {
            let result = if self.host.borrow().dead {
                Ok((Value::Null, vec![]))
            } else {
                self.host.borrow_mut().call(self.command("close"))
            };
            self.closed = true;
            result?;
        }
        Ok(())
    }
    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            video_settings: false,
            project_properties: true,
            audio_visualization: true,
        }
    }
    fn is_web(&self) -> bool {
        true
    }
    fn set_properties(&mut self, updates: &Value) -> Result<()> {
        let updates = updates
            .as_object()
            .context("Properties must be an object")?;
        for (name, update) in updates {
            let declared = self
                .properties
                .get(name)
                .context("Undeclared wallpaper property")?;
            let value = update.get("value").context("Property value missing")?;
            let original = declared
                .get("value")
                .context("Declared property value missing")?;
            ensure!(
                (original.is_boolean() && value.is_boolean())
                    || (original.is_number() && value.is_number())
                    || (original.is_string() && value.is_string()),
                "Wallpaper property value has incorrect type"
            );
            if declared.get("type").and_then(Value::as_str) == Some("color") {
                let components = value
                    .as_str()
                    .context("Color must be a string")?
                    .split_whitespace()
                    .map(str::parse::<f32>)
                    .collect::<std::result::Result<Vec<_>, _>>()?;
                ensure!(
                    components.len() == 3
                        && components
                            .iter()
                            .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
                    "Color requires three values in 0..1"
                );
            }
            if let Some(options) = declared.get("options").and_then(Value::as_array) {
                ensure!(
                    options
                        .iter()
                        .any(
                            |option| option.get("value").is_some_and(|allowed| allowed == value
                                || (allowed.is_number()
                                    && value.is_number()
                                    && allowed.as_f64() == value.as_f64()))
                        ),
                    "Property choice is not declared"
                );
            }
            if let Some(number) = value.as_f64() {
                ensure!(
                    number.is_finite(),
                    "Wallpaper property value must be finite"
                );
                if let Some(min) = declared.get("min").and_then(Value::as_f64) {
                    ensure!(number >= min, "Property below minimum");
                }
                if let Some(max) = declared.get("max").and_then(Value::as_f64) {
                    ensure!(number <= max, "Property above maximum");
                }
            }
            ensure!(
                update.as_object().is_some_and(|obj| obj.len() == 1),
                "Only property values may be updated"
            );
        }
        let mut merged = self.properties.clone();
        for (name, update) in updates {
            merged[name]["value"] = update["value"].clone();
        }
        let mut command = self.command("properties");
        command["properties"] = merged.clone();
        self.host.borrow_mut().call(command)?;
        self.properties = merged;
        Ok(())
    }
}
impl Drop for WebRenderer {
    fn drop(&mut self) {
        let _ = self.close();
        unsafe {
            if self.texture != 0 {
                gl::DeleteTextures(1, &self.texture);
            }
            if self.vao != 0 {
                gl::DeleteVertexArrays(1, &self.vao);
            }
            if self.program != 0 {
                gl::DeleteProgram(self.program);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn helper_metadata_rejects_missing_corrupt_and_incompatible_installations() {
        let directory = tempfile::tempdir().unwrap();
        let host = directory.path().join("varpaper-web-host");
        let manifest = directory.path().join("runtime.json");
        std::fs::write(&host, b"fixture").unwrap();
        std::fs::set_permissions(&host, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("missing"));
        std::fs::write(&manifest, b"broken").unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("Invalid"));
        let mut incompatible: Value = serde_json::from_str(RUNTIME_MANIFEST).unwrap();
        for key in ["ipc_version", "version", "architecture", "sha256"] {
            let original = incompatible[key].clone();
            incompatible[key] = json!("incompatible");
            std::fs::write(&manifest, serde_json::to_vec(&incompatible).unwrap()).unwrap();
            assert!(validate_host(&host, &manifest)
                .unwrap_err()
                .to_string()
                .contains("mismatch"));
            incompatible[key] = original;
        }
        std::fs::write(&manifest, vec![b' '; MAX_JSON + 1]).unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("budget"));
        std::fs::write(&manifest, RUNTIME_MANIFEST).unwrap();
        validate_host(&host, &manifest).unwrap();
        std::fs::set_permissions(&host, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(validate_host(&host, &manifest)
            .unwrap_err()
            .to_string()
            .contains("executable"));
    }

    #[test]
    fn response_abi_mismatch_terminates_the_helper_without_consuming_pixels() {
        let (socket, mut peer) = UnixStream::pair().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let child = unsafe {
            Command::new("sleep")
                .arg("10")
                .pre_exec(|| {
                    if libc::setsid() < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                })
                .spawn()
                .unwrap()
        };
        let thread = std::thread::spawn(move || {
            let mut header = [0; 4];
            peer.read_exact(&mut header).unwrap();
            let mut bytes = vec![0; u32::from_le_bytes(header) as usize];
            peer.read_exact(&mut bytes).unwrap();
            let mut response: Value = serde_json::from_slice(&bytes).unwrap();
            response["ipc_version"] = json!(IPC_VERSION - 1);
            response["bytes"] = json!(4);
            response["ok"] = json!(true);
            let bytes = serde_json::to_vec(&response).unwrap();
            peer.write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
            peer.write_all(&bytes).unwrap();
            // No pixels: validation must reject the protocol before reading them.
        });
        let mut host = Host {
            socket,
            child,
            request: 0,
            next_id: 0,
            dead: false,
        };
        assert!(host
            .call(json!({"id":0,"generation":0}))
            .unwrap_err()
            .to_string()
            .contains("IPC version mismatch"));
        assert!(host.dead);
        assert!(host.child.try_wait().unwrap().is_some());
        thread.join().unwrap();
    }
}
