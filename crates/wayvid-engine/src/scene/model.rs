//! Bounded Scene JSON inputs. Resource confinement is owned by ProjectResources.
use anyhow::{bail, ensure, Context, Result};
use serde_json::Value;

use super::SceneResources;

#[derive(Clone, Debug)]
pub struct SceneDocument {
    pub width: u32,
    pub height: u32,
    pub clear_color: [f32; 3],
    pub objects: Vec<Value>,
    pub camera: Camera,
    pub auto_projection: bool,
    pub general: Value,
}
#[derive(Clone, Debug)]
pub struct Camera {
    pub center: [f32; 3],
    pub eye: [f32; 3],
    pub up: [f32; 3],
    pub near: f32,
    pub far: f32,
}
impl SceneDocument {
    pub fn load(entry: &str, resources: &dyn SceneResources) -> Result<Self> {
        let bytes = resources
            .read(entry)
            .with_context(|| format!("Scene resource '{entry}'"))?;
        ensure!(
            bytes.len() <= 1024 * 1024,
            "Scene JSON exceeds 1 MiB budget: {entry}"
        );
        Self::parse(&bytes).with_context(|| format!("Scene document '{entry}'"))
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() <= 1024 * 1024,
            "Scene JSON exceeds 1 MiB budget"
        );
        let value: Value = serde_json::from_slice(bytes)?;
        ensure!(value.is_object(), "Scene document must be an object");
        let projection = &value["general"]["orthogonalprojection"];
        let auto_projection = projection
            .get("auto")
            .map(|value| value.as_bool().context("Projection auto must be boolean"))
            .transpose()?
            .unwrap_or(false);
        let width = if auto_projection {
            1
        } else {
            dimension(&projection["width"])?
        };
        let height = if auto_projection {
            1
        } else {
            dimension(&projection["height"])?
        };
        let clear_color = vector(bound(&value["general"]["clearcolor"]), [1.0; 3])?;
        ensure!(
            clear_color.iter().all(|v| (0.0..=1.0).contains(v)),
            "Scene clear color outside 0..1"
        );
        let objects = value["objects"]
            .as_array()
            .context("Scene objects must be an array")?;
        ensure!(objects.len() <= 4096, "Scene object count exceeds budget");
        let camera = &value["camera"];
        ensure!(camera.is_object(), "Scene must have a camera object");
        for field in ["center", "eye", "up"] {
            ensure!(!camera[field].is_null(), "Scene camera must have '{field}'");
        }
        let center = vector(&camera["center"], [0.0; 3])?;
        let eye = vector(&camera["eye"], [0.0, 0.0, 1.0])?;
        let up = vector(&camera["up"], [0.0, 1.0, 0.0])?;
        ensure!(
            center
                .iter()
                .chain(&eye)
                .chain(&up)
                .all(|value| value.abs() <= 1000000.0),
            "Scene camera coordinate exceeds budget"
        );
        let direction = glam::Vec3::from_array(center) - glam::Vec3::from_array(eye);
        let normal = direction.cross(glam::Vec3::from_array(up));
        ensure!(
            direction.length_squared().is_finite()
                && normal.length_squared().is_finite()
                && direction.length_squared() > 1e-12
                && normal.length_squared() > 1e-12,
            "Degenerate Scene camera"
        );
        let near = scalar(&camera["nearz"], 0.0)?;
        let far = scalar(&camera["farz"], 1000.0)?;
        ensure!(far > near, "Scene camera far plane must exceed near plane");
        Ok(Self {
            width,
            height,
            clear_color,
            objects: objects.clone(),
            camera: Camera {
                center,
                eye,
                up,
                near,
                far,
            },
            auto_projection,
            general: value["general"].clone(),
        })
    }
}
pub fn bound(value: &Value) -> &Value {
    if value.is_object() {
        &value["value"]
    } else {
        value
    }
}
pub fn scalar(value: &Value, default: f32) -> Result<f32> {
    let value = bound(value);
    if value.is_null() {
        return Ok(default);
    }
    let number = if let Some(text) = value.as_str() {
        text.parse::<f32>()?
    } else {
        value.as_f64().context("Scene scalar must be numeric")? as f32
    };
    ensure!(number.is_finite(), "Non-finite Scene scalar");
    Ok(number)
}
pub fn dimension(value: &Value) -> Result<u32> {
    let number = value
        .as_u64()
        .context("Scene dimension must be a positive integer")?;
    ensure!(
        (1..=16384).contains(&number),
        "Invalid Scene dimension: {number}"
    );
    Ok(number as u32)
}
pub fn vector<const N: usize>(value: &Value, default: [f32; N]) -> Result<[f32; N]> {
    if value.is_null() {
        return Ok(default);
    }
    let numbers: Vec<f32> = if let Some(text) = value.as_str() {
        text.split_whitespace()
            .map(str::parse)
            .collect::<std::result::Result<_, _>>()?
    } else if let Some(array) = value.as_array() {
        array
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|n| n as f32)
                    .context("Non-numeric Scene vector")
            })
            .collect::<Result<_>>()?
    } else {
        bail!("Scene vector must be a string or array");
    };
    ensure!(
        numbers.len() == N && numbers.iter().all(|n| n.is_finite()),
        "Invalid Scene vector"
    );
    Ok(std::array::from_fn(|i| numbers[i]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_dimensions_vectors_and_objects_are_rejected() {
        let mut value: Value =
            serde_json::from_slice(super::super::fixtures::clear_scene().as_bytes()).unwrap();
        for invalid in [
            serde_json::json!(0),
            serde_json::json!(-1),
            serde_json::json!(16385),
            serde_json::json!(2.5),
        ] {
            value["general"]["orthogonalprojection"]["width"] = invalid;
            assert!(SceneDocument::parse(&serde_json::to_vec(&value).unwrap()).is_err());
        }
        assert!(vector::<3>(&serde_json::json!("NaN 0 1"), [0.0; 3]).is_err());
        assert!(vector::<3>(&serde_json::json!([0, 1]), [0.0; 3]).is_err());
        assert!(SceneDocument::parse(&vec![b' '; 1024 * 1024 + 1]).is_err());
    }
}
