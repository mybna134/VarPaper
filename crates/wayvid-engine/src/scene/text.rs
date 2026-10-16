//! Bounded Rust font selection, layout and glyph coverage rasterization.
use super::{
    assets::Object,
    model::{bound, scalar},
    texture::{Mip, Texture},
    SceneResources,
};
use ab_glyph::{point, Font, FontArc, PxScale, ScaleFont};
use anyhow::{ensure, Context, Result};
use std::collections::BTreeMap;

const FONT_BUDGET: usize = 64 * 1024 * 1024;
const PIXEL_BUDGET: usize = 16 * 1024 * 1024;
#[derive(Default)]
pub(super) struct Fonts {
    cache: BTreeMap<String, FontArc>,
    bytes: usize,
}
impl Fonts {
    fn retain(&mut self, key: String, bytes: Vec<u8>) -> Result<FontArc> {
        ensure!(
            self.bytes
                .checked_add(bytes.len())
                .is_some_and(|v| v <= FONT_BUDGET),
            "Scene font cache exceeds 64 MiB"
        );
        let length = bytes.len();
        let font =
            FontArc::try_from_vec(bytes).with_context(|| format!("Invalid Scene font '{key}'"))?;
        self.bytes += length;
        self.cache.insert(key, font.clone());
        Ok(font)
    }
    fn system(&mut self) -> Result<FontArc> {
        if let Some(font) = self.cache.get("system:sans") {
            return Ok(font.clone());
        }
        for path in [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ] {
            if let Ok(metadata) = std::fs::metadata(path) {
                ensure!(
                    metadata.len() <= FONT_BUDGET as u64,
                    "System font exceeds Scene font budget"
                );
                return self.retain("system:sans".into(), std::fs::read(path)?);
            }
        }
        anyhow::bail!("No Scene fallback font installed (DejaVu Sans or Liberation Sans)")
    }
    fn select(&mut self, object: &Object, resources: &dyn SceneResources) -> Result<FontArc> {
        let name = object
            .properties
            .get("font")
            .map(|v| v.as_str().context("Scene font must be a string"))
            .transpose()?
            .unwrap_or("");
        ensure!(name.len() <= 4096, "Scene font name exceeds budget");
        if !name.is_empty() {
            wayvid_library::project::resource_name(name)
                .context("Invalid Scene project font path")?;
            for path in [
                name.to_string(),
                format!("materials/fonts/{name}"),
                format!("fonts/{name}"),
            ] {
                if let Some(font) = self.cache.get(&path) {
                    return Ok(font.clone());
                }
                if resources.exists(&path)? {
                    return self.retain(path.clone(), resources.read(&path)?);
                }
            }
        }
        self.system()
    }
}
/// CPU coverage is kept in an ordinary Scene texture, so the same GPU owners
/// and state guard cover text and image layers.
pub(super) fn raster(
    object: &Object,
    text: &str,
    resources: &dyn SceneResources,
    fonts: &mut Fonts,
) -> Result<Texture> {
    ensure!(
        text.len() <= 65536 && text.chars().count() <= 4096,
        "Scene text exceeds glyph budget"
    );
    let size = scalar(&object.properties["pointsize"], 32.0)?;
    ensure!(
        (1.0..=512.0).contains(&size),
        "Invalid Scene text pointsize"
    );
    let font = fonts.select(object, resources)?;
    let scale = PxScale::from(size);
    let scaled = font.as_scaled(scale);
    let padding = scalar(&object.properties["padding"], 0.0)?;
    ensure!(
        (0.0..=1024.0).contains(&padding),
        "Invalid Scene text padding"
    );
    let padding = padding.ceil() as u32;
    let explicit = object.size;
    ensure!(
        explicit
            .iter()
            .all(|v| v.is_finite() && (0.0..=4096.0).contains(v)),
        "Scene text canvas exceeds dimensions"
    );
    let wrap = if explicit[0] > 0.0 {
        (explicit[0] - 2.0 * padding as f32).max(1.0)
    } else {
        4096.0 - 2.0 * padding as f32
    };
    let mut lines: Vec<Vec<ab_glyph::Glyph>> = vec![Vec::new()];
    let mut widths = vec![0.0f32];
    let mut x = 0.0;
    let mut previous = None;
    for character in text.chars() {
        if character == '\r' {
            continue;
        }
        if character == '\n' {
            lines.push(Vec::new());
            widths.push(x);
            x = 0.0;
            previous = None;
            continue;
        }
        let id = scaled.glyph_id(character);
        let kerning = previous
            .map(|id_before| scaled.kern(id_before, id))
            .unwrap_or(0.0);
        let advance = scaled.h_advance(id);
        if x + kerning + advance > wrap && !lines.last().unwrap().is_empty() {
            widths.push(x);
            lines.push(Vec::new());
            x = 0.0;
            previous = None;
        }
        if previous.is_some() {
            x += kerning;
        }
        lines
            .last_mut()
            .unwrap()
            .push(id.with_scale_and_position(scale, point(x, scaled.ascent())));
        x += advance;
        previous = Some(id);
    }
    // Widths track finalized lines plus the last line, without allocating pixels.
    widths.remove(0);
    widths.push(x);
    let line_height = scaled.height() + scaled.line_gap();
    let content_width = widths.iter().copied().fold(0.0f32, f32::max);
    let content_height = line_height * lines.len() as f32;
    let width = if explicit[0] > 0.0 {
        explicit[0].ceil()
    } else {
        (content_width + 2.0 * padding as f32).ceil().max(1.0)
    } as u32;
    let height = if explicit[1] > 0.0 {
        explicit[1].ceil()
    } else {
        (content_height + 2.0 * padding as f32).ceil().max(1.0)
    } as u32;
    ensure!(
        (1..=4096).contains(&width) && (1..=4096).contains(&height),
        "Scene text layout exceeds dimensions"
    );
    let pixels = width as usize * height as usize;
    ensure!(
        pixels <= PIXEL_BUDGET,
        "Scene text raster exceeds 16 Mi pixels"
    );
    let horizontal = object
        .properties
        .get("horizontalalign")
        .or_else(|| object.properties.get("alignment"))
        .map(|v| {
            bound(v)
                .as_str()
                .context("Scene text alignment must be a string")
        })
        .transpose()?
        .unwrap_or("center");
    let vertical = object
        .properties
        .get("verticalalign")
        .map(|v| {
            bound(v)
                .as_str()
                .context("Scene text alignment must be a string")
        })
        .transpose()?
        .unwrap_or("center");
    ensure!(
        ["left", "center", "right"].contains(&horizontal)
            && ["top", "center", "bottom"].contains(&vertical),
        "Unsupported required Scene text alignment"
    );
    let available_height = height as f32 - 2.0 * padding as f32;
    let shift_y = padding as f32
        + match vertical {
            "top" => 0.0,
            "bottom" => available_height - content_height,
            _ => (available_height - content_height) / 2.0,
        };
    let mut rgba = vec![0u8; pixels * 4];
    let mut raster_work = 0usize;
    for (index, line) in lines.into_iter().enumerate() {
        let available = width as f32 - 2.0 * padding as f32;
        let shift_x = padding as f32
            + match horizontal {
                "left" => 0.0,
                "right" => available - widths[index],
                _ => (available - widths[index]) / 2.0,
            };
        for mut glyph in line {
            glyph.position.x += shift_x;
            glyph.position.y += shift_y + index as f32 * line_height;
            if let Some(outline) = font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                ensure!(
                    bounds.min.x.is_finite()
                        && bounds.min.y.is_finite()
                        && bounds.max.x.is_finite()
                        && bounds.max.y.is_finite(),
                    "Invalid Scene glyph bounds"
                );
                if bounds.max.x <= 0.0
                    || bounds.max.y <= 0.0
                    || bounds.min.x >= width as f32
                    || bounds.min.y >= height as f32
                {
                    continue;
                }
                let work = (bounds.width().ceil().max(0.0) as usize)
                    .checked_mul(bounds.height().ceil().max(0.0) as usize)
                    .context("Scene glyph raster budget overflow")?;
                raster_work = raster_work
                    .checked_add(work)
                    .context("Scene glyph raster budget overflow")?;
                ensure!(
                    raster_work <= 64 * 1024 * 1024,
                    "Scene glyph raster work exceeds budget"
                );
                outline.draw(|x, y, coverage| {
                    let x = x as i32 + bounds.min.x as i32;
                    let y = y as i32 + bounds.min.y as i32;
                    if x >= 0 && y >= 0 && x < width as i32 && y < height as i32 {
                        let offset = (y as usize * width as usize + x as usize) * 4;
                        let alpha = (coverage * 255.0).round() as u8;
                        rgba[offset..offset + 3].fill(255);
                        rgba[offset + 3] = rgba[offset + 3].max(alpha);
                    }
                });
            }
        }
    }
    Ok(Texture {
        flags: 2,
        width,
        height,
        logical_width: width,
        logical_height: height,
        images: vec![vec![Mip {
            width,
            height,
            rgba,
        }]],
        frames: Vec::new(),
        animation_size: [width, height],
    })
}
pub(super) fn value(object: &Object) -> Result<&str> {
    bound(&object.properties["text"])
        .as_str()
        .context("Scene text must be a string")
}

/// Dynamic text uses font data already retained during candidate creation.
pub(super) struct CachedOnly;
impl SceneResources for CachedOnly {
    fn exists(&self, _: &str) -> Result<bool> {
        Ok(false)
    }
    fn read(&self, name: &str) -> Result<Vec<u8>> {
        anyhow::bail!("Scene font not cached: '{name}'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::fixtures::{image_resources, Resources};
    use serde_json::json;

    const FONT: &[u8] = include_bytes!("testdata/minimal.ttf");
    fn object() -> Object {
        let mut object = super::super::assets::Assets::load("scene.json", &image_resources())
            .unwrap()
            .objects
            .remove(0);
        object.size = [100.0, 80.0];
        object.properties = json!({"font":"fixture.ttf", "pointsize":20, "padding":2,
            "text":{"value":"A V"}, "horizontalalign":"left", "verticalalign":"top"});
        object
    }
    fn resources() -> Resources {
        Resources(BTreeMap::from([("fixture.ttf".into(), FONT.to_vec())]))
    }
    fn ink(texture: &Texture) -> (u32, u32, u32, u32) {
        let mut bounds = (texture.width, texture.height, 0, 0);
        for (i, pixel) in texture.images[0][0]
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
        {
            if pixel[3] > 0 {
                assert_eq!(&pixel[..3], &[255; 3]);
                let (x, y) = (i as u32 % texture.width, i as u32 / texture.width);
                bounds = (
                    bounds.0.min(x),
                    bounds.1.min(y),
                    bounds.2.max(x),
                    bounds.3.max(y),
                );
            }
        }
        assert!(bounds.0 <= bounds.2 && bounds.1 <= bounds.3);
        bounds
    }

    #[test]
    fn embedded_font_raster_alignment_wrapping_and_cached_updates() {
        let mut object = object();
        let resources = resources();
        let mut fonts = Fonts::default();
        assert_eq!(value(&object).unwrap(), "A V");
        let first = raster(&object, "AV", &resources, &mut fonts).unwrap();
        let left = ink(&first);
        assert_eq!((first.width, first.height), (100, 80));
        assert_eq!(fonts.bytes, FONT.len());
        for (horizontal, vertical) in [("center", "center"), ("right", "bottom")] {
            object.properties["horizontalalign"] = json!({"value":horizontal});
            object.properties["verticalalign"] = json!(vertical);
            let next = raster(&object, "AV", &CachedOnly, &mut fonts).unwrap();
            let aligned = ink(&next);
            assert!(aligned.0 > left.0 && aligned.1 > left.1);
        }
        assert_eq!(fonts.cache.len(), 1);
        object.size = [0.0; 2];
        object.properties["horizontalalign"] = json!("left");
        object.properties["verticalalign"] = json!("top");
        let single = raster(&object, "AV", &CachedOnly, &mut fonts).unwrap();
        let multiline = raster(&object, "A\r\nV", &CachedOnly, &mut fonts).unwrap();
        assert!(multiline.height > single.height && multiline.width < single.width);
        object.size = [16.0, 0.0];
        let wrapped = raster(&object, "AVAV", &CachedOnly, &mut fonts).unwrap();
        assert!(wrapped.height > single.height);
        object.size = [1.0, 1.0];
        let clipped = raster(&object, "A", &CachedOnly, &mut fonts).unwrap();
        assert_eq!(clipped.images[0][0].rgba.len(), 4);
        object.size = [0.0; 2];
        let blank = raster(&object, "", &CachedOnly, &mut fonts).unwrap();
        assert!(blank.images[0][0].rgba.iter().all(|v| *v == 0));
        assert!(CachedOnly.read("missing.ttf").is_err());
    }

    #[test]
    fn font_resolution_cache_and_resource_limits_fail_closed() {
        let mut object = object();
        for name in [
            "fixture.ttf",
            "materials/fonts/fixture.ttf",
            "fonts/fixture.ttf",
        ] {
            let resources = Resources(BTreeMap::from([(name.into(), FONT.to_vec())]));
            let mut fonts = Fonts::default();
            fonts.select(&object, &resources).unwrap();
            fonts.select(&object, &CachedOnly).unwrap();
            assert_eq!(fonts.bytes, FONT.len());
        }
        let mut fonts = Fonts::default();
        fonts.retain("system:sans".into(), FONT.to_vec()).unwrap();
        object.properties["font"] = json!("not-found.ttf");
        fonts.select(&object, &CachedOnly).unwrap();
        object.properties["font"] = json!("");
        fonts.select(&object, &CachedOnly).unwrap();
        for invalid in [
            json!(false),
            json!("../escape.ttf"),
            json!("a".repeat(4097)),
        ] {
            object.properties["font"] = invalid;
            assert!(fonts.select(&object, &CachedOnly).is_err());
        }
        let mut fonts = Fonts::default();
        assert!(fonts.retain("broken".into(), vec![0; 16]).is_err());
        assert_eq!(fonts.bytes, 0);
        fonts.bytes = FONT_BUDGET;
        assert!(fonts.retain("too-big".into(), FONT.to_vec()).is_err());
        fonts.bytes = usize::MAX;
        assert!(fonts.retain("overflow".into(), FONT.to_vec()).is_err());
    }

    #[test]
    fn invalid_text_dimensions_and_properties_are_rejected_without_a_display() {
        let resources = resources();
        let mut fonts = Fonts::default();
        let mut object = object();
        for text in ["A".repeat(4097), "A".repeat(65537)] {
            assert!(raster(&object, &text, &resources, &mut fonts).is_err());
        }
        for (key, invalid) in [
            ("pointsize", json!(0)),
            ("pointsize", json!(513)),
            ("padding", json!(-1)),
            ("padding", json!(1025)),
            ("horizontalalign", json!(3)),
            ("horizontalalign", json!("diagonal")),
            ("verticalalign", json!(false)),
            ("verticalalign", json!("diagonal")),
        ] {
            let mut candidate = object.clone();
            candidate.properties[key] = invalid;
            assert!(
                raster(&candidate, "A", &resources, &mut fonts).is_err(),
                "{key}"
            );
        }
        for size in [[-1.0, 10.0], [4097.0, 10.0], [f32::NAN, 10.0]] {
            object.size = size;
            assert!(raster(&object, "A", &resources, &mut fonts).is_err());
        }
        object.size = [1.0, 0.0];
        assert!(raster(&object, &"A\n".repeat(2048), &resources, &mut fonts).is_err());
        object.properties["text"] = json!(123);
        assert!(value(&object).is_err());
    }
}
