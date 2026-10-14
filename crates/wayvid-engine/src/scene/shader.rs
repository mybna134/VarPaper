//! Rust shader resource expansion and compatibility vocabulary. The GL driver
//! checks syntax/types; comments are never rewritten as shader instructions.
use super::{assets::identifier, SceneResources};
use anyhow::{bail, ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use wayvid_library::project::resource_name;

#[derive(Clone, Copy, Debug)]
pub enum Stage {
    Vertex,
    Fragment,
}
#[derive(Debug)]
pub struct Source {
    pub text: String,
    pub files: Vec<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
    pub combos: BTreeMap<String, i32>,
}
struct Expansion<'a> {
    resources: &'a dyn SceneResources,
    active: BTreeSet<String>,
    files: Vec<String>,
    bytes: usize,
    defaults: BTreeMap<String, i32>,
    metadata: BTreeMap<String, serde_json::Value>,
}
impl Source {
    pub fn load(
        shader: &str,
        stage: Stage,
        combos: &BTreeMap<String, i32>,
        resources: &dyn SceneResources,
    ) -> Result<Self> {
        resource_name(shader)?;
        ensure!(combos.len() <= 256, "Shader combo budget exceeded");
        let mut expansion = Expansion {
            resources,
            active: BTreeSet::new(),
            files: Vec::new(),
            bytes: 0,
            defaults: BTreeMap::new(),
            metadata: BTreeMap::new(),
        };
        let extension = match stage {
            Stage::Vertex => "vert",
            Stage::Fragment => "frag",
        };
        let path = std::path::PathBuf::from(format!("shaders/{shader}")).with_extension(extension);
        let path = path.to_str().context("Shader resource path UTF-8")?;
        let body = expansion.expand(path, 0)?;
        let mut defines = expansion.defaults;
        for (name, value) in combos {
            ensure!(identifier(name), "Invalid shader combo '{name}'");
            defines.insert(name.to_ascii_uppercase(), *value);
        }
        let mut text = String::from("#version 330 core\n");
        for (name, value) in &defines {
            text.push_str(&format!("#define {name} {value}\n"));
        }
        // Definitions describe the file-format vocabulary, rather than applying
        // substring replacements that could change identifiers or comments.
        text.push_str(COMPATIBILITY);
        text.push_str(match stage {
            Stage::Vertex => "#define attribute in\n#define varying out\n",
            Stage::Fragment => {
                "out vec4 vp_FragColor;\n#define gl_FragColor vp_FragColor\n#define varying in\n"
            }
        });
        text.push_str(&body);
        ensure!(
            text.len() <= 4 * 1024 * 1024,
            "Expanded shader exceeds 4 MiB"
        );
        Ok(Self {
            text,
            files: expansion.files,
            metadata: expansion.metadata,
            combos: defines,
        })
    }
}
const COMPATIBILITY: &str = "\
#define GLSL 1
#define float2 vec2
#define float3 vec3
#define float4 vec4
#define float3x3 mat3
#define float4x4 mat4
#define int2 ivec2
#define int3 ivec3
#define int4 ivec4
#define lerp mix
#define frac fract
#define mul(x,y) ((y)*(x))
#define saturate(x) clamp((x),0.0,1.0)
#define CAST2(x) vec2(x)
#define CAST3(x) vec3(x)
#define CAST4(x) vec4(x)
#define CAST3X3(x) mat3(x)
#define texSample2D texture
#define texSample2DLod textureLod
#define texture2D texture
#define atan2 atan
#define log10(x) (log2(x)*0.301029995663981)
#define fmod(x,y) ((x)-(y)*trunc((x)/(y)))
#define ddx dFdx
#define ddy(x) dFdy(-(x))
";
impl Expansion<'_> {
    fn append(output: &mut String, text: &str) -> Result<()> {
        ensure!(
            output
                .len()
                .checked_add(text.len())
                .is_some_and(|n| n <= 4 * 1024 * 1024),
            "Expanded shader exceeds 4 MiB"
        );
        output.push_str(text);
        Ok(())
    }
    fn expand(&mut self, path: &str, depth: usize) -> Result<String> {
        resource_name(path)?;
        ensure!(depth < 32, "Shader include depth exceeds 32: {path}");
        ensure!(
            self.active.insert(path.into()),
            "Shader include cycle: {path}"
        );
        let bytes = self
            .resources
            .read(path)
            .with_context(|| format!("Shader source '{path}'"))?;
        ensure!(
            bytes.len() <= 1024 * 1024,
            "Shader source exceeds 1 MiB: {path}"
        );
        self.bytes = self
            .bytes
            .checked_add(bytes.len())
            .context("Shader source budget overflow")?;
        ensure!(
            self.bytes <= 4 * 1024 * 1024,
            "Shader source graph exceeds 4 MiB: {path}"
        );
        ensure!(self.files.len() < 256, "Shader include count exceeds 256");
        let file_id = self.files.len() + 1;
        self.files.push(path.into());
        let text =
            std::str::from_utf8(&bytes).with_context(|| format!("Shader source UTF-8 '{path}'"))?;
        ensure!(!text.contains('\0'), "Shader source contains NUL: {path}");
        let mut output = format!("#line 1 {file_id}\n");
        let mut in_block = false;
        for (line_number, line) in text.lines().enumerate() {
            let was_in_block = in_block;
            let visible = visible_code(line, &mut in_block);
            let code = visible.trim();
            if let Some(include) = directive(code, "include") {
                let name = include
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .context("Shader include must be a quoted resource name")?;
                resource_name(name)
                    .with_context(|| format!("Shader include '{name}' in '{path}'"))?;
                let include_path =
                    std::path::PathBuf::from(format!("shaders/{name}")).with_extension("h");
                let nested = self
                    .expand(
                        include_path.to_str().context("Shader include path UTF-8")?,
                        depth + 1,
                    )
                    .with_context(|| {
                        format!(
                            "Shader include '{name}' in '{path}' line {}",
                            line_number + 1
                        )
                    })?;
                Self::append(&mut output, &nested)?;
                Self::append(
                    &mut output,
                    &format!("\n#line {} {file_id}\n", line_number + 2),
                )?;
            } else if let Some(version) = directive(code, "version") {
                ensure!(
                    matches!(version, "120" | "130" | "150" | "330" | "330 core"),
                    "Unsupported shader version '{version}' in '{path}'"
                );
                Self::append(&mut output, "\n")?;
            } else if let Some(module) = directive(code, "require") {
                bail!("Unsupported required shader module '{module}' in '{path}'");
            } else {
                if !was_in_block {
                    if let Some((declaration, comment)) = line.split_once("//") {
                        let comment = comment.trim();
                        let tokens: Vec<_> = declaration.split_whitespace().collect();
                        if tokens.first() == Some(&"uniform")
                            && declaration.contains(';')
                            && comment.starts_with('{')
                        {
                            let name = tokens
                                .last()
                                .context("Shader uniform declaration")?
                                .trim_end_matches(';')
                                .split('[')
                                .next()
                                .context("Shader uniform name")?;
                            ensure!(
                                identifier(name),
                                "Invalid shader uniform metadata name '{name}'"
                            );
                            let metadata: serde_json::Value = serde_json::from_str(comment)
                                .with_context(|| {
                                    format!("Shader uniform metadata '{name}' in '{path}'")
                                })?;
                            ensure!(
                                metadata.is_object(),
                                "Shader uniform metadata must be an object"
                            );
                            if let Some(previous) =
                                self.metadata.insert(name.into(), metadata.clone())
                            {
                                ensure!(
                                    previous == metadata,
                                    "Conflicting shader uniform metadata '{name}'"
                                );
                            }
                            ensure!(
                                self.metadata.len() <= 1024,
                                "Shader uniform metadata budget exceeded"
                            );
                        }
                    }
                }
                // Combo declarations are comments in the Wallpaper Engine format.
                if let Some(json) = (!was_in_block)
                    .then(|| line.trim().strip_prefix("// [COMBO]"))
                    .flatten()
                {
                    let value: serde_json::Value =
                        serde_json::from_str(json.trim()).context("Shader combo metadata")?;
                    let name = value["combo"]
                        .as_str()
                        .context("Shader combo metadata name")?;
                    ensure!(identifier(name), "Invalid shader combo metadata '{name}'");
                    let default = value
                        .get("default")
                        .map(|value| {
                            value
                                .as_i64()
                                .context("Shader combo default must be an integer")
                        })
                        .transpose()?
                        .unwrap_or(0)
                        .try_into()?;
                    self.defaults
                        .entry(name.to_ascii_uppercase())
                        .or_insert(default);
                    ensure!(
                        self.defaults.len() <= 256,
                        "Shader combo metadata budget exceeded"
                    );
                }
                Self::append(&mut output, &visible)?;
                Self::append(&mut output, "\n")?;
            }
        }
        ensure!(!in_block, "Unterminated shader comment in '{path}'");
        self.active.remove(path);
        Ok(output)
    }
}
fn directive<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let rest = text.strip_prefix('#')?.trim_start().strip_prefix(name)?;
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some(rest.trim())
    } else {
        None
    }
}
/// Strip comments while preserving token boundaries and physical line numbers.
fn visible_code(line: &str, in_block: &mut bool) -> String {
    let bytes = line.as_bytes();
    let mut output = Vec::new();
    let mut position = 0;
    let mut quoted = false;
    while position < bytes.len() {
        let pair = bytes.get(position..position + 2);
        if *in_block {
            if pair == Some(b"*/") {
                *in_block = false;
                position += 2;
                output.push(b' ');
            } else {
                position += 1;
            }
        } else if !quoted && pair == Some(b"//") {
            break;
        } else if !quoted && pair == Some(b"/*") {
            *in_block = true;
            position += 2;
            output.push(b' ');
        } else {
            if bytes[position] == b'"' && (position == 0 || bytes[position - 1] != b'\\') {
                quoted = !quoted;
            }
            output.push(bytes[position]);
            position += 1;
        }
    }
    String::from_utf8(output).expect("Comment boundaries are ASCII")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::fixtures::image_resources;
    #[test]
    fn includes_comments_and_combos_expand_without_identifier_rewrites() {
        let mut resources = image_resources();
        resources.0.insert("shaders/fixture.frag".into(), b"// [COMBO] {\"combo\":\"effect\",\"default\":1}\n/* #include \"missing\" */\n#include \"common.h\"\n// gl_FragColor must stay in comments\nvoid main(){gl_FragColor=vec4(float(EFFECT));}\n".to_vec());
        resources.0.insert(
            "shaders/common.h".into(),
            b"#include \"nested.h\"\n".to_vec(),
        );
        resources.0.insert(
            "shaders/nested.h".into(),
            b"float gl_FragColorSuffix=1.0;\n".to_vec(),
        );
        let source = Source::load(
            "fixture",
            Stage::Fragment,
            &BTreeMap::from([("effect".into(), 2)]),
            &resources,
        )
        .unwrap();
        assert_eq!(source.files.len(), 3);
        assert!(source.text.contains("#define EFFECT 2"));
        assert!(source.text.contains("gl_FragColorSuffix"));
        assert!(!source.text.contains("missing"));
        resources.0.insert(
            "shaders/nested.h".into(),
            b"#include \"common.h\"\n".to_vec(),
        );
        assert!(format!(
            "{:#}",
            Source::load("fixture", Stage::Fragment, &BTreeMap::new(), &resources)
                .err()
                .unwrap()
        )
        .contains("cycle"));
    }
    #[test]
    fn missing_unsafe_unknown_and_oversized_sources_fail_with_paths() {
        for text in [
            "#include \"missing.h\"",
            "#include \"../outside\"",
            "#require Unknown",
            "#version 999",
            "/* unfinished",
        ] {
            let mut resources = image_resources();
            resources
                .0
                .insert("shaders/fixture.vert".into(), text.as_bytes().to_vec());
            assert!(
                Source::load("fixture", Stage::Vertex, &BTreeMap::new(), &resources).is_err(),
                "{text}"
            );
        }
        let mut resources = image_resources();
        resources
            .0
            .insert("shaders/fixture.vert".into(), vec![b' '; 1024 * 1024 + 1]);
        assert!(
            Source::load("fixture", Stage::Vertex, &BTreeMap::new(), &resources)
                .err()
                .unwrap()
                .to_string()
                .contains("fixture.vert")
        );
        assert!(Source::load(
            "fixture",
            Stage::Vertex,
            &BTreeMap::from([("A\nHACK".into(), 1)]),
            &image_resources()
        )
        .is_err());
    }

    #[test]
    fn nested_source_budgets_and_comment_boundaries_are_enforced() {
        let mut resources = image_resources();
        resources.0.insert(
            "shaders/fixture.vert".into(),
            b"/* comment\n*/ #include \"level0\"\nvoid main(){gl_Position=vec4(0.0);}".to_vec(),
        );
        for i in 0..33 {
            resources.0.insert(
                format!("shaders/level{i}.h"),
                format!("#include \"level{}\"\n", i + 1).into_bytes(),
            );
        }
        let error = Source::load("fixture", Stage::Vertex, &BTreeMap::new(), &resources)
            .err()
            .unwrap();
        assert!(format!("{error:#}").contains("depth exceeds 32"));
        resources.0.insert(
            "shaders/level0.h".into(),
            b"/* ignore #require Unknown */\nfloat helper=1.0;".to_vec(),
        );
        let source = Source::load("fixture", Stage::Vertex, &BTreeMap::new(), &resources).unwrap();
        assert!(source.text.contains("float helper=1.0;"));
        assert!(!source.text.contains("*/"));
        resources.0.insert(
            "shaders/fixture.vert".into(),
            "#include \"large\"\n".repeat(5).into_bytes(),
        );
        resources
            .0
            .insert("shaders/large.h".into(), vec![b' '; 1024 * 1024]);
        let error = Source::load("fixture", Stage::Vertex, &BTreeMap::new(), &resources)
            .err()
            .unwrap();
        assert!(format!("{error:#}").contains("4 MiB"));
    }
}
