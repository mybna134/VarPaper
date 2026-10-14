//! Assignment-local camera and layer geometry in Scene's top-down coordinates.
use super::{
    assets::{Assets, Object},
    model::{bound, scalar, vector},
};
use anyhow::{ensure, Context, Result};
use glam::{Mat4, Quat, Vec3};

pub(super) struct Geometry {
    pub model: Mat4,
    pub projection: Mat4,
    pub uv: [f32; 2],
    pub visible: bool,
    pub color: [f32; 3],
    pub alpha: f32,
    pub brightness: f32,
    pub size: [f32; 2],
}
impl Geometry {
    pub fn layer_transform(object: &Object, assets: &Assets) -> Result<(Mat4, Mat4, bool)> {
        let camera = &assets.document.camera;
        let (width, height) = (assets.document.width as f32, assets.document.height as f32);
        let projection = Mat4::orthographic_rh_gl(
            -width / 2.0,
            width / 2.0,
            -height / 2.0,
            height / 2.0,
            camera.near,
            camera.far,
        ) * Mat4::from_translation(Vec3::from_array(camera.eye))
            * Mat4::look_at_rh(
                Vec3::from_array(camera.eye),
                Vec3::from_array(camera.center),
                Vec3::from_array(camera.up),
            )
            * Mat4::from_translation(Vec3::new(-width / 2.0, height / 2.0, 0.0))
            * Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0));
        let mut parents = vec![object];
        let mut current = object;
        while let Some(id) = current.parent {
            ensure!(
                parents.len() <= 128,
                "Scene parent transform depth exceeds 128"
            );
            let index = assets
                .object_index
                .get(&id)
                .context("Missing Scene parent")?;
            current = &assets.objects[*index];
            parents.push(current);
        }
        let mut transform = Mat4::IDENTITY;
        let mut visible = true;
        for parent in parents.into_iter().rev() {
            let property = bound(&parent.properties["visible"]);
            if !property.is_null() {
                visible &= property
                    .as_bool()
                    .context("Scene visibility must be boolean")?;
            }
            // Scene angles are radians, including parent transforms.
            let angles = parent.angles;
            transform *= Mat4::from_scale_rotation_translation(
                Vec3::from_array(parent.scale),
                Quat::from_euler(glam::EulerRot::XYZ, angles[0], angles[1], angles[2]),
                Vec3::from_array(parent.origin),
            );
        }
        ensure!(
            transform.is_finite() && projection.is_finite(),
            "Scene layer transform overflow: {}",
            object.name
        );
        Ok((transform, projection, visible))
    }
    pub fn image(object: &Object, name: &str, assets: &Assets) -> Result<Self> {
        let model = &assets.models[name];
        let (mut transform, projection, visible) = Self::layer_transform(object, assets)?;
        let (width, height) = (assets.document.width as f32, assets.document.height as f32);
        let pass = &assets.materials[&model.material].passes[0];
        let texture = pass
            .textures
            .get(&0)
            .and_then(|name| assets.textures.get(&format!("materials/{name}.tex")));
        let mut size = object.size;
        if size[0] == 0.0 || size[1] == 0.0 {
            size = if let Some(texture) = texture {
                [texture.logical_width as f32, texture.logical_height as f32]
            } else {
                [
                    model.width.unwrap_or(assets.document.width) as f32,
                    model.height.unwrap_or(assets.document.height) as f32,
                ]
            };
        }
        if model.properties["fullscreen"].as_bool() == Some(true) {
            size = [width, height];
            transform = Mat4::from_translation(Vec3::new(width / 2.0, height / 2.0, 0.0));
        }
        ensure!(
            size.iter().all(|v| *v > 0.0 && v.is_finite()),
            "Invalid Scene image size: {}",
            object.name
        );
        let alignment = object
            .properties
            .get("horizontalalign")
            .or_else(|| object.properties.get("alignment"));
        let alignment = alignment
            .map(|value| {
                value
                    .as_str()
                    .context("Scene image alignment must be a string")
            })
            .transpose()?
            .unwrap_or("center");
        ensure!(
            [
                "center",
                "left",
                "right",
                "top",
                "bottom",
                "topleft",
                "topright",
                "bottomleft",
                "bottomright"
            ]
            .contains(&alignment),
            "Unsupported Scene image alignment '{alignment}'"
        );
        let shift_x = if alignment.contains("left") {
            size[0] / 2.0
        } else if alignment.contains("right") {
            -size[0] / 2.0
        } else {
            0.0
        };
        let shift_y = if alignment.contains("top") {
            size[1] / 2.0
        } else if alignment.contains("bottom") {
            -size[1] / 2.0
        } else {
            0.0
        };
        transform *= Mat4::from_translation(Vec3::new(shift_x, shift_y, 0.0))
            * Mat4::from_scale(Vec3::new(size[0], size[1], 1.0));
        ensure!(
            transform.is_finite() && projection.is_finite(),
            "Scene transform overflow: {}",
            object.name
        );
        let uv = texture
            .map(|texture| {
                if !texture.frames.is_empty() {
                    return [1.0; 2];
                }
                [
                    texture.logical_width as f32 / texture.images[0][0].width as f32,
                    texture.logical_height as f32 / texture.images[0][0].height as f32,
                ]
            })
            .unwrap_or([1.0; 2]);
        let value = bound(&object.properties["color"]);
        let components = value
            .as_array()
            .map(Vec::len)
            .or_else(|| value.as_str().map(|v| v.split_whitespace().count()));
        let (color, color_alpha) = if components == Some(4) {
            let v = vector(value, [1.0; 4])?;
            ([v[0], v[1], v[2]], v[3])
        } else {
            (vector(value, [1.0; 3])?, 1.0)
        };
        ensure!(
            (0.0..=1.0).contains(&color_alpha),
            "Invalid Scene color alpha"
        );
        let alpha = scalar(&object.properties["alpha"], 1.0)? * color_alpha;
        let brightness = scalar(&object.properties["brightness"], 1.0)?;
        ensure!(
            (0.0..=1.0).contains(&alpha)
                && color.iter().all(|v| (0.0..=1.0).contains(v))
                && brightness >= 0.0,
            "Invalid Scene image color/alpha/brightness"
        );
        Ok(Self {
            model: transform,
            projection,
            uv,
            visible,
            color,
            alpha,
            brightness,
            size,
        })
    }
}
