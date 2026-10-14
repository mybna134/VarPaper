//! Context-bound GPU owners; callers must bind their surface before release.
use anyhow::{ensure, Result};

#[cfg(test)]
thread_local! {
    static COUNTS: std::cell::Cell<(usize,usize)> = const { std::cell::Cell::new((0,0)) };
    static FAIL_AFTER_TEXTURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
#[cfg(test)]
pub(super) fn counts() -> (usize, usize) {
    COUNTS.with(std::cell::Cell::get)
}
#[cfg(test)]
pub(super) fn fail_after_texture() {
    FAIL_AFTER_TEXTURE.with(|flag| flag.set(true));
}

pub(super) struct State {
    draw: i32,
    read: i32,
    viewport: [i32; 4],
    scissor: bool,
    mask: [u8; 4],
    clear: [f32; 4],
    active_texture: i32,
    active_binding: i32,
    textures: [i32; 16],
    program: i32,
    vao: i32,
    array_buffer: i32,
    enabled: [bool; 3],
    blend: [i32; 6],
    depth_mask: u8,
    depth_func: i32,
    cull_mode: i32,
    front_face: i32,
    unpack_buffer: i32,
    unpack: [i32; 6],
}
const UNPACK_FIELDS: [u32; 6] = [
    gl::UNPACK_ALIGNMENT,
    gl::UNPACK_ROW_LENGTH,
    gl::UNPACK_IMAGE_HEIGHT,
    gl::UNPACK_SKIP_PIXELS,
    gl::UNPACK_SKIP_ROWS,
    gl::UNPACK_SKIP_IMAGES,
];
impl State {
    pub fn capture() -> Self {
        let mut state = Self {
            draw: 0,
            read: 0,
            viewport: [0; 4],
            scissor: false,
            mask: [0; 4],
            clear: [0.0; 4],
            active_texture: 0,
            active_binding: 0,
            textures: [0; 16],
            program: 0,
            vao: 0,
            array_buffer: 0,
            enabled: [false; 3],
            blend: [0; 6],
            depth_mask: 0,
            depth_func: 0,
            cull_mode: 0,
            front_face: 0,
            unpack_buffer: 0,
            unpack: [0; 6],
        };
        unsafe {
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut state.draw);
            gl::GetIntegerv(gl::READ_FRAMEBUFFER_BINDING, &mut state.read);
            gl::GetIntegerv(gl::VIEWPORT, state.viewport.as_mut_ptr());
            gl::GetBooleanv(gl::COLOR_WRITEMASK, state.mask.as_mut_ptr());
            gl::GetFloatv(gl::COLOR_CLEAR_VALUE, state.clear.as_mut_ptr());
            gl::GetIntegerv(gl::ACTIVE_TEXTURE, &mut state.active_texture);
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut state.active_binding);
            for (index, binding) in state.textures.iter_mut().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + index as u32);
                gl::GetIntegerv(gl::TEXTURE_BINDING_2D, binding);
            }
            gl::ActiveTexture(state.active_texture as u32);
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut state.program);
            gl::GetIntegerv(gl::VERTEX_ARRAY_BINDING, &mut state.vao);
            gl::GetIntegerv(gl::ARRAY_BUFFER_BINDING, &mut state.array_buffer);
            for (field, enabled) in [gl::BLEND, gl::DEPTH_TEST, gl::CULL_FACE]
                .into_iter()
                .zip(&mut state.enabled)
            {
                *enabled = gl::IsEnabled(field) == gl::TRUE;
            }
            for (field, value) in [
                gl::BLEND_SRC_RGB,
                gl::BLEND_DST_RGB,
                gl::BLEND_SRC_ALPHA,
                gl::BLEND_DST_ALPHA,
                gl::BLEND_EQUATION_RGB,
                gl::BLEND_EQUATION_ALPHA,
            ]
            .into_iter()
            .zip(&mut state.blend)
            {
                gl::GetIntegerv(field, value);
            }
            gl::GetBooleanv(gl::DEPTH_WRITEMASK, &mut state.depth_mask);
            gl::GetIntegerv(gl::DEPTH_FUNC, &mut state.depth_func);
            gl::GetIntegerv(gl::CULL_FACE_MODE, &mut state.cull_mode);
            gl::GetIntegerv(gl::FRONT_FACE, &mut state.front_face);
            gl::GetIntegerv(gl::PIXEL_UNPACK_BUFFER_BINDING, &mut state.unpack_buffer);
            for (field, value) in UNPACK_FIELDS.iter().zip(&mut state.unpack) {
                gl::GetIntegerv(*field, value);
            }
            state.scissor = gl::IsEnabled(gl::SCISSOR_TEST) == gl::TRUE;
        }
        state
    }
}
impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.draw as u32);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.read as u32);
            gl::Viewport(
                self.viewport[0],
                self.viewport[1],
                self.viewport[2],
                self.viewport[3],
            );
            gl::ColorMask(self.mask[0], self.mask[1], self.mask[2], self.mask[3]);
            gl::ClearColor(self.clear[0], self.clear[1], self.clear[2], self.clear[3]);
            for (index, texture) in self.textures.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + index as u32);
                gl::BindTexture(gl::TEXTURE_2D, *texture as u32);
            }
            gl::ActiveTexture(self.active_texture as u32);
            gl::BindTexture(gl::TEXTURE_2D, self.active_binding as u32);
            gl::UseProgram(self.program as u32);
            gl::BindVertexArray(self.vao as u32);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.array_buffer as u32);
            for (field, enabled) in [gl::BLEND, gl::DEPTH_TEST, gl::CULL_FACE]
                .into_iter()
                .zip(self.enabled)
            {
                if enabled {
                    gl::Enable(field);
                } else {
                    gl::Disable(field);
                }
            }
            gl::BlendFuncSeparate(
                self.blend[0] as u32,
                self.blend[1] as u32,
                self.blend[2] as u32,
                self.blend[3] as u32,
            );
            gl::BlendEquationSeparate(self.blend[4] as u32, self.blend[5] as u32);
            gl::DepthMask(self.depth_mask);
            gl::DepthFunc(self.depth_func as u32);
            gl::CullFace(self.cull_mode as u32);
            gl::FrontFace(self.front_face as u32);
            gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, self.unpack_buffer as u32);
            for (field, value) in UNPACK_FIELDS.iter().zip(self.unpack) {
                gl::PixelStorei(*field, value);
            }
            if self.scissor {
                gl::Enable(gl::SCISSOR_TEST);
            } else {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }
    }
}
pub(super) struct Target {
    pub framebuffer: u32,
    texture: u32,
    pub width: u32,
    pub height: u32,
}
impl Target {
    pub fn texture(&self) -> u32 {
        self.texture
    }
    pub fn new(width: u32, height: u32) -> Result<Self> {
        ensure!(
            (1..=16384).contains(&width) && (1..=16384).contains(&height),
            "Invalid Scene target dimensions"
        );
        let bytes = u64::from(width) * u64::from(height) * 4;
        ensure!(
            bytes <= 256 * 1024 * 1024,
            "Scene target exceeds 256 MiB budget"
        );
        let _state = State::capture();
        let mut max = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max);
        }
        ensure!(
            width <= max as u32 && height <= max as u32,
            "Scene dimensions exceed GPU limit"
        );
        let mut target = Self {
            framebuffer: 0,
            texture: 0,
            width,
            height,
        };
        unsafe {
            gl::GenTextures(1, &mut target.texture);
        }
        ensure!(target.texture != 0, "Cannot allocate Scene target texture");
        #[cfg(test)]
        COUNTS.with(|counts| {
            let (f, t) = counts.get();
            counts.set((f, t + 1));
        });
        #[cfg(test)]
        ensure!(
            !FAIL_AFTER_TEXTURE.with(|flag| flag.replace(false)),
            "Injected Scene allocation failure"
        );
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, target.texture);
            reset_unpack();
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::GenFramebuffers(1, &mut target.framebuffer);
        }
        ensure!(target.framebuffer != 0, "Cannot allocate Scene framebuffer");
        #[cfg(test)]
        COUNTS.with(|counts| {
            let (f, t) = counts.get();
            counts.set((f + 1, t));
        });
        unsafe {
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, target.framebuffer);
            gl::FramebufferTexture2D(
                gl::DRAW_FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                target.texture,
                0,
            );
            ensure!(
                gl::CheckFramebufferStatus(gl::DRAW_FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE,
                "Incomplete Scene framebuffer"
            );
        }
        Ok(target)
    }
}

/// Prevent caller PBO/strides from changing the meaning of a CPU upload pointer.
unsafe fn reset_unpack() {
    gl::BindBuffer(gl::PIXEL_UNPACK_BUFFER, 0);
    for field in UNPACK_FIELDS {
        gl::PixelStorei(field, if field == gl::UNPACK_ALIGNMENT { 1 } else { 0 });
    }
}

pub(super) struct TextureUpload {
    pub images: Vec<u32>,
}
impl TextureUpload {
    pub fn new(texture: &super::texture::Texture) -> Result<Self> {
        let _state = State::capture();
        let mut upload = Self { images: Vec::new() };
        let mut max = 0;
        unsafe {
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max);
        }
        for image in &texture.images {
            let mut id = 0;
            unsafe {
                gl::GenTextures(1, &mut id);
            }
            ensure!(id != 0, "Cannot allocate Scene texture");
            upload.images.push(id);
            #[cfg(test)]
            COUNTS.with(|counts| {
                let (f, t) = counts.get();
                counts.set((f, t + 1));
            });
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, id);
                reset_unpack();
            }
            for (level, mip) in image.iter().enumerate() {
                ensure!(
                    mip.width <= max as u32 && mip.height <= max as u32,
                    "Scene texture exceeds GPU dimension limit"
                );
                unsafe {
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        level as i32,
                        gl::RGBA8 as i32,
                        mip.width as i32,
                        mip.height as i32,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        mip.rgba.as_ptr().cast(),
                    );
                    ensure!(
                        gl::GetError() == gl::NO_ERROR,
                        "Scene texture upload failed at mip {level}"
                    );
                }
            }
            let filter = if texture.flags & 1 != 0 {
                gl::NEAREST
            } else {
                gl::LINEAR
            };
            let wrap = if texture.flags & 8 != 0 {
                gl::CLAMP_TO_BORDER
            } else if texture.flags & 2 != 0 {
                gl::CLAMP_TO_EDGE
            } else {
                gl::REPEAT
            };
            unsafe {
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap as i32);
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MAX_LEVEL,
                    image.len() as i32 - 1,
                );
            }
        }
        Ok(upload)
    }
}
impl Drop for TextureUpload {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.images.len() as i32, self.images.as_ptr());
        }
        #[cfg(test)]
        COUNTS.with(|counts| {
            let (f, t) = counts.get();
            counts.set((f, t - self.images.len()));
        });
    }
}
impl Drop for Target {
    fn drop(&mut self) {
        unsafe {
            if self.framebuffer != 0 {
                gl::DeleteFramebuffers(1, &self.framebuffer);
            }
            if self.texture != 0 {
                gl::DeleteTextures(1, &self.texture);
            }
        }
        #[cfg(test)]
        COUNTS.with(|counts| {
            let (f, t) = counts.get();
            counts.set((
                f - usize::from(self.framebuffer != 0),
                t - usize::from(self.texture != 0),
            ));
        });
    }
}
