//! CEF browser owner and OpenGL presentation; requires the native runtime.
use super::*;
use crate::mpv::VideoConfig;
use crate::renderer::{RendererCapabilities, WallpaperRenderer};
use std::ffi::CString;

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
    drawn_size: Option<(i32, i32)>,
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
            drawn_size: None,
            paused: false,
            closed: false,
            config: config.clone(),
        };
        #[cfg(test)]
        track(1, true);
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
            #[cfg(test)]
            if self.program != 0 {
                track(2, true);
            }
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
            #[cfg(test)]
            {
                if self.texture != 0 {
                    track(3, true);
                }
                if self.vao != 0 {
                    track(4, true);
                }
            }
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
            if self.presented && self.drawn_size != Some((width, height)) {
                self.draw(&[], width, height, framebuffer)?;
                self.drawn_size = Some((width, height));
                return Ok(true);
            }
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
        self.drawn_size = Some((width, height));
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
        let merged = crate::properties::merge(&self.properties, updates)?;
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
                #[cfg(test)]
                track(3, false);
            }
            if self.vao != 0 {
                gl::DeleteVertexArrays(1, &self.vao);
                #[cfg(test)]
                track(4, false);
            }
            if self.program != 0 {
                gl::DeleteProgram(self.program);
                #[cfg(test)]
                track(2, false);
            }
        }
        #[cfg(test)]
        track(1, false);
    }
}
