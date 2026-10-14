//! Driver shader/program ownership. All methods require the owner's GL context.
use super::assets::Pass;
use super::shader::{Source, Stage};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::ffi::CString;

#[cfg(test)]
thread_local! { static LIVE: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0,0)) }; }
#[cfg(test)]
thread_local! { static FAIL_CREATE_PROGRAM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
#[cfg(test)]
pub(super) fn fail_program_allocation() {
    FAIL_CREATE_PROGRAM.with(|flag| flag.set(true));
}
#[cfg(test)]
pub(super) fn counts() -> (usize, usize) {
    LIVE.with(std::cell::Cell::get)
}

struct Shader(u32);
impl Shader {
    fn compile(source: &Source, stage: Stage) -> Result<Self> {
        let kind = match stage {
            Stage::Vertex => gl::VERTEX_SHADER,
            Stage::Fragment => gl::FRAGMENT_SHADER,
        };
        let id = unsafe { gl::CreateShader(kind) };
        ensure!(id != 0, "Cannot create {stage:?} shader");
        let shader = Self(id);
        #[cfg(test)]
        LIVE.with(|live| {
            let (s, p) = live.get();
            live.set((s + 1, p));
        });
        let text = CString::new(source.text.as_bytes())?;
        let pointer = text.as_ptr();
        unsafe {
            gl::ShaderSource(id, 1, &pointer, std::ptr::null());
            gl::CompileShader(id);
        }
        let mut status = 0;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut status);
        }
        ensure!(
            status != 0,
            "{stage:?} shader compilation failed: {}\nsource files: {:?}",
            shader.log(),
            source.files
        );
        Ok(shader)
    }
    fn log(&self) -> String {
        let mut size = 0;
        unsafe { gl::GetShaderiv(self.0, gl::INFO_LOG_LENGTH, &mut size) };
        let mut bytes = vec![0; size.clamp(1, 64 * 1024) as usize];
        let mut written = 0;
        unsafe {
            gl::GetShaderInfoLog(
                self.0,
                bytes.len() as i32,
                &mut written,
                bytes.as_mut_ptr().cast(),
            );
        }
        String::from_utf8_lossy(&bytes[..(written.max(0) as usize).min(bytes.len())]).into_owned()
    }
}
impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.0) };
        #[cfg(test)]
        LIVE.with(|live| {
            let (s, p) = live.get();
            live.set((s - 1, p));
        });
    }
}
struct Uniform {
    location: i32,
    kind: u32,
    count: usize,
}
pub(super) struct Program {
    pub id: u32,
    uniforms: BTreeMap<String, Uniform>,
    metadata: BTreeMap<String, Value>,
}
impl Program {
    pub fn new(name: &str, vertex: &Source, fragment: &Source) -> Result<Self> {
        let mut metadata = vertex.metadata.clone();
        for (name, value) in &fragment.metadata {
            if let Some(previous) = metadata.insert(name.clone(), value.clone()) {
                ensure!(
                    previous == *value,
                    "Conflicting shader metadata '{name}' in '{name}'"
                );
            }
        }
        let compile = || -> Result<Self> {
            let vertex = Shader::compile(vertex, Stage::Vertex)?;
            let fragment = Shader::compile(fragment, Stage::Fragment)?;
            #[cfg(test)]
            let fail = FAIL_CREATE_PROGRAM.with(|flag| flag.replace(false));
            #[cfg(not(test))]
            let fail = false;
            let id = if fail {
                0
            } else {
                unsafe { gl::CreateProgram() }
            };
            ensure!(id != 0, "Cannot create Scene shader program");
            let mut program = Self {
                id,
                uniforms: BTreeMap::new(),
                metadata: BTreeMap::new(),
            };
            #[cfg(test)]
            LIVE.with(|live| {
                let (s, p) = live.get();
                live.set((s, p + 1));
            });
            unsafe {
                gl::AttachShader(id, vertex.0);
                gl::AttachShader(id, fragment.0);
                gl::BindAttribLocation(id, 0, c"a_Position".as_ptr());
                gl::BindAttribLocation(id, 1, c"a_TexCoord".as_ptr());
                gl::LinkProgram(id);
            }
            let mut status = 0;
            unsafe {
                gl::GetProgramiv(id, gl::LINK_STATUS, &mut status);
            }
            ensure!(status != 0, "Scene shader link failed: {}", program.log());
            unsafe {
                gl::DetachShader(id, vertex.0);
                gl::DetachShader(id, fragment.0);
            }
            program.reflect()?;
            program.metadata = metadata;
            Ok(program)
        };
        compile().with_context(|| format!("Scene shader '{name}'"))
    }
    fn log(&self) -> String {
        let mut size = 0;
        unsafe { gl::GetProgramiv(self.id, gl::INFO_LOG_LENGTH, &mut size) };
        let mut bytes = vec![0; size.clamp(1, 64 * 1024) as usize];
        let mut written = 0;
        unsafe {
            gl::GetProgramInfoLog(
                self.id,
                bytes.len() as i32,
                &mut written,
                bytes.as_mut_ptr().cast(),
            );
        }
        String::from_utf8_lossy(&bytes[..(written.max(0) as usize).min(bytes.len())]).into_owned()
    }
    pub fn uniform(&self, name: &str) -> Result<i32> {
        let name = CString::new(name)?;
        Ok(unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) })
    }
    fn reflect(&mut self) -> Result<()> {
        let mut count = 0;
        let mut max_name = 0;
        unsafe {
            gl::GetProgramiv(self.id, gl::ACTIVE_UNIFORMS, &mut count);
            gl::GetProgramiv(self.id, gl::ACTIVE_UNIFORM_MAX_LENGTH, &mut max_name);
        }
        ensure!(
            (0..=1024).contains(&count) && (0..=4096).contains(&max_name),
            "Shader active uniform budget exceeded"
        );
        let mut bytes = vec![0u8; max_name.max(1) as usize];
        for index in 0..count {
            let (mut length, mut size, mut kind) = (0, 0, 0);
            unsafe {
                gl::GetActiveUniform(
                    self.id,
                    index as u32,
                    bytes.len() as i32,
                    &mut length,
                    &mut size,
                    &mut kind,
                    bytes.as_mut_ptr().cast(),
                );
            }
            ensure!(
                length >= 0 && length as usize <= bytes.len() && (1..=1024).contains(&size),
                "Invalid shader uniform reflection"
            );
            let name = std::str::from_utf8(&bytes[..length as usize])?
                .trim_end_matches("[0]")
                .to_string();
            let location = self.uniform(&name)?;
            self.uniforms.insert(
                name,
                Uniform {
                    location,
                    kind,
                    count: size as usize,
                },
            );
        }
        Ok(())
    }
    fn constant<'a>(&'a self, name: &str, pass: &'a Pass) -> Result<Option<&'a Value>> {
        let metadata = self.metadata.get(name);
        let key = metadata
            .and_then(|value| value["material"].as_str())
            .unwrap_or(name);
        let value = pass
            .constants
            .get(key)
            .or_else(|| metadata.and_then(|value| value.get("default")));
        if value.is_none() && metadata.is_some_and(|value| value.get("material").is_some()) {
            anyhow::bail!("Missing required shader material parameter '{key}' for '{name}'");
        }
        Ok(value.map(|value| {
            if value.is_object() {
                &value["value"]
            } else {
                value
            }
        }))
    }
    pub fn validate_with_input(&self, pass: &Pass, input: bool) -> Result<()> {
        for (name, uniform) in &self.uniforms {
            if matches!(uniform.kind, gl::SAMPLER_2D | gl::SAMPLER_2D_SHADOW) {
                let slot = name
                    .strip_prefix("g_Texture")
                    .and_then(|slot| slot.parse::<u32>().ok())
                    .context("Unsupported required shader sampler name")?;
                ensure!(
                    pass.textures.contains_key(&slot) || input && slot == 0,
                    "Scene shader '{}' missing required sampler '{name}'",
                    pass.shader
                );
                continue;
            }
            if let Some(value) = self.constant(name, pass)? {
                UniformData::parse(uniform, value)
                    .with_context(|| format!("Scene shader '{}' uniform '{name}'", pass.shader))?;
            }
        }
        Ok(())
    }
    pub fn apply_constants(&self, pass: &Pass) -> Result<()> {
        for (name, uniform) in &self.uniforms {
            if uniform.kind == gl::SAMPLER_2D {
                continue;
            }
            if let Some(value) = self.constant(name, pass)? {
                let data = UniformData::parse(uniform, value)
                    .with_context(|| format!("Scene shader '{}' uniform '{name}'", pass.shader))?;
                unsafe {
                    data.upload(uniform);
                }
            }
        }
        Ok(())
    }
}

enum UniformData {
    Float(Vec<f32>),
    Integer(Vec<i32>),
}
impl UniformData {
    fn parse(uniform: &Uniform, value: &Value) -> Result<Self> {
        let components = match uniform.kind {
            gl::FLOAT | gl::INT | gl::BOOL => 1,
            gl::FLOAT_VEC2 | gl::INT_VEC2 | gl::BOOL_VEC2 => 2,
            gl::FLOAT_VEC3 | gl::INT_VEC3 | gl::BOOL_VEC3 => 3,
            gl::FLOAT_VEC4 | gl::INT_VEC4 | gl::BOOL_VEC4 | gl::FLOAT_MAT2 => 4,
            gl::FLOAT_MAT3 => 9,
            gl::FLOAT_MAT4 => 16,
            kind => anyhow::bail!("Unsupported required shader uniform type {kind:#x}"),
        };
        let numbers: Vec<String> = if let Some(text) = value.as_str() {
            text.split_whitespace().map(str::to_owned).collect()
        } else if let Some(values) = value.as_array() {
            values
                .iter()
                .map(|v| {
                    if let Some(value) = v.as_bool() {
                        u8::from(value).to_string()
                    } else {
                        v.to_string()
                    }
                })
                .collect()
        } else if let Some(value) = value.as_bool() {
            vec![u8::from(value).to_string()]
        } else {
            vec![value.to_string()]
        };
        ensure!(
            numbers.len() == components * uniform.count,
            "Shader uniform value has wrong component count"
        );
        let integer = matches!(
            uniform.kind,
            gl::INT
                | gl::INT_VEC2
                | gl::INT_VEC3
                | gl::INT_VEC4
                | gl::BOOL
                | gl::BOOL_VEC2
                | gl::BOOL_VEC3
                | gl::BOOL_VEC4
        );
        if integer {
            Ok(Self::Integer(
                numbers
                    .iter()
                    .map(|value| {
                        value
                            .parse::<i32>()
                            .context("Invalid integer shader uniform")
                    })
                    .collect::<Result<_>>()?,
            ))
        } else {
            let values = numbers
                .iter()
                .map(|value| {
                    value
                        .parse::<f32>()
                        .context("Invalid floating shader uniform")
                })
                .collect::<Result<Vec<_>>>()?;
            ensure!(
                values.iter().all(|value| value.is_finite()),
                "Non-finite shader uniform"
            );
            Ok(Self::Float(values))
        }
    }
    unsafe fn upload(&self, uniform: &Uniform) {
        let (location, count) = (uniform.location, uniform.count as i32);
        match self {
            Self::Float(values) => match uniform.kind {
                gl::FLOAT => gl::Uniform1fv(location, count, values.as_ptr()),
                gl::FLOAT_VEC2 => gl::Uniform2fv(location, count, values.as_ptr()),
                gl::FLOAT_VEC3 => gl::Uniform3fv(location, count, values.as_ptr()),
                gl::FLOAT_VEC4 => gl::Uniform4fv(location, count, values.as_ptr()),
                gl::FLOAT_MAT2 => gl::UniformMatrix2fv(location, count, gl::FALSE, values.as_ptr()),
                gl::FLOAT_MAT3 => gl::UniformMatrix3fv(location, count, gl::FALSE, values.as_ptr()),
                gl::FLOAT_MAT4 => gl::UniformMatrix4fv(location, count, gl::FALSE, values.as_ptr()),
                _ => unreachable!("Validated float uniform type"),
            },
            Self::Integer(values) => match uniform.kind {
                gl::INT | gl::BOOL => gl::Uniform1iv(location, count, values.as_ptr()),
                gl::INT_VEC2 | gl::BOOL_VEC2 => gl::Uniform2iv(location, count, values.as_ptr()),
                gl::INT_VEC3 | gl::BOOL_VEC3 => gl::Uniform3iv(location, count, values.as_ptr()),
                gl::INT_VEC4 | gl::BOOL_VEC4 => gl::Uniform4iv(location, count, values.as_ptr()),
                _ => unreachable!("Validated integer uniform type"),
            },
        }
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
        #[cfg(test)]
        LIVE.with(|live| {
            let (s, p) = live.get();
            live.set((s, p - 1));
        });
    }
}
