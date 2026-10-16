//! Scene TEX containers decoded without native Scene code.
use anyhow::{bail, ensure, Context, Result};
use std::io::Cursor;

const MIP_BUDGET: usize = 64 * 1024 * 1024;
const TEXTURE_BUDGET: usize = 256 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Rgba,
    R,
    Rg,
    Dxt1,
    Dxt3,
    Dxt5,
}
impl Format {
    fn parse(value: u32) -> Result<Self> {
        Ok(match value {
            0 => Self::Rgba,
            9 => Self::R,
            8 => Self::Rg,
            7 => Self::Dxt1,
            6 => Self::Dxt3,
            4 => Self::Dxt5,
            _ => bail!("Unsupported Scene texture format {value}"),
        })
    }
    fn bytes(self, width: u32, height: u32) -> usize {
        let (w, h) = (width as usize, height as usize);
        match self {
            Self::Rgba => w * h * 4,
            Self::R => w * h,
            Self::Rg => w * h * 2,
            Self::Dxt1 => w.div_ceil(4) * h.div_ceil(4) * 8,
            Self::Dxt3 | Self::Dxt5 => w.div_ceil(4) * h.div_ceil(4) * 16,
        }
    }
}

#[derive(Debug)]
pub struct Mip {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
#[derive(Debug)]
pub struct Frame {
    pub image: u32,
    pub seconds: f32,
    pub rect: [f32; 6],
}
#[derive(Debug)]
pub struct Texture {
    pub flags: u32,
    pub width: u32,
    pub height: u32,
    pub logical_width: u32,
    pub logical_height: u32,
    pub images: Vec<Vec<Mip>>,
    pub frames: Vec<Frame>,
    pub animation_size: [u32; 2],
}

/// A normalized sprite-atlas transform, with the selected uploaded image.
#[derive(Debug, Clone, Copy)]
pub struct AnimationFrame {
    pub image: usize,
    pub translation: [f32; 2],
    pub rotation: [f32; 4],
}
impl Texture {
    pub fn frame_at(&self, seconds: f64) -> Result<AnimationFrame> {
        ensure!(
            seconds.is_finite() && seconds >= 0.0,
            "Invalid texture animation time"
        );
        let mut selected = None;
        if !self.frames.is_empty() {
            ensure!(
                self.frames
                    .iter()
                    .all(|frame| frame.seconds.is_finite() && frame.seconds > 0.0),
                "Invalid texture frame duration"
            );
            let duration: f64 = self
                .frames
                .iter()
                .map(|frame| f64::from(frame.seconds))
                .sum();
            ensure!(
                duration.is_finite() && duration > 0.0,
                "Invalid texture animation duration"
            );
            let mut phase = seconds % duration;
            selected = self.frames.last();
            for frame in &self.frames {
                phase -= f64::from(frame.seconds);
                // Match the reference's inclusive duration boundary.
                if phase <= 0.0 {
                    selected = Some(frame);
                    break;
                }
            }
        }
        if let Some(frame) = selected {
            let mip = &self.images[frame.image as usize][0];
            let rect = frame.rect;
            Ok(AnimationFrame {
                image: frame.image as usize,
                translation: [rect[0] / mip.width as f32, rect[1] / mip.height as f32],
                rotation: [
                    rect[2] / mip.width as f32,
                    rect[3] / mip.width as f32,
                    rect[4] / mip.height as f32,
                    rect[5] / mip.height as f32,
                ],
            })
        } else {
            Ok(AnimationFrame {
                image: 0,
                translation: [0.0; 2],
                rotation: [1.0, 0.0, 0.0, 1.0],
            })
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(count)
            .context("TEX offset overflow")?;
        let value = self.bytes.get(self.position..end).with_context(|| {
            format!(
                "Truncated TEX at byte {} (need {count} bytes)",
                self.position
            )
        })?;
        self.position = end;
        Ok(value)
    }
    fn integer(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?))
    }
    fn dimension(&mut self) -> Result<u32> {
        let value = self.integer()?;
        ensure!(
            (1..=16384).contains(&value),
            "Invalid TEX dimension {value}"
        );
        Ok(value)
    }
    fn magic(&mut self, expected: &[u8]) -> Result<()> {
        ensure!(self.take(expected.len())? == expected, "Invalid TEX header");
        Ok(())
    }
    fn float(&mut self) -> Result<f32> {
        let value = f32::from_bits(self.integer()?);
        ensure!(value.is_finite(), "Non-finite TEX animation value");
        Ok(value)
    }
}
#[derive(Default)]
struct Budget(usize);
impl Budget {
    fn charge(&mut self, count: usize) -> Result<()> {
        self.0 = self
            .0
            .checked_add(count)
            .context("TEX allocation overflow")?;
        ensure!(
            self.0 <= TEXTURE_BUDGET,
            "TEX exceeds 256 MiB allocation budget"
        );
        Ok(())
    }
}
impl Texture {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() <= TEXTURE_BUDGET,
            "TEX input exceeds 256 MiB budget"
        );
        let mut reader = Reader { bytes, position: 0 };
        reader.magic(b"TEXV0005\0")?;
        reader.magic(b"TEXI0001\0")?;
        let format_value = reader.integer()?;
        let flags = reader.integer()?;
        ensure!(flags & !524335 == 0, "Unsupported TEX flags {flags}");
        ensure!(
            flags & 32 == 0,
            "TEX video texture decoder is not integrated yet"
        );
        let width = reader.dimension()?;
        let height = reader.dimension()?;
        let logical_width = reader.dimension()?;
        let logical_height = reader.dimension()?;
        reader.integer()?;
        let container = match reader.take(9)? {
            b"TEXB0001\0" => 1,
            b"TEXB0002\0" => 2,
            b"TEXB0003\0" => 3,
            b"TEXB0004\0" => 4,
            _ => bail!("Unsupported TEXB container"),
        };
        let image_count = reader.integer()? as usize;
        ensure!((1..=4096).contains(&image_count), "Invalid TEX image count");
        let encoded = if container >= 3 {
            reader.integer()?
        } else {
            u32::MAX
        };
        if container == 4 {
            let video = reader.integer()?;
            ensure!(video <= 1, "Invalid TEX video marker");
            ensure!(video == 0, "TEX MP4 texture decoder is not integrated yet");
        }
        let format = if encoded == u32::MAX {
            Some(Format::parse(format_value)?)
        } else {
            None
        };
        let encoded_format = match encoded {
            u32::MAX => None,
            2 => Some(image::ImageFormat::Jpeg),
            13 => Some(image::ImageFormat::Png),
            25 => Some(image::ImageFormat::Gif),
            35 => Some(image::ImageFormat::WebP),
            _ => bail!("Unsupported encoded TEX image format {encoded}"),
        };
        let mut budget = Budget::default();
        let mut images = Vec::new();
        for image in 0..image_count {
            let count = reader.integer()? as usize;
            ensure!(
                (1..=16).contains(&count),
                "Invalid TEX mip count for image {image}"
            );
            let mut mips = Vec::new();
            for level in 0..count {
                let mip = Self::mip(&mut reader, &mut budget, container, format, encoded_format)
                    .with_context(|| format!("TEX image {image}, mip {level}"))?;
                mips.push(mip);
            }
            images.push(mips);
        }
        let mut frames = Vec::new();
        let mut animation_size = [logical_width, logical_height];
        if flags & 4 != 0 {
            let version = match reader.take(9)? {
                b"TEXS0001\0" => 1,
                b"TEXS0002\0" => 2,
                b"TEXS0003\0" => 3,
                _ => bail!("Unsupported TEXS animation"),
            };
            let count = reader.integer()? as usize;
            ensure!((1..=100000).contains(&count), "Invalid TEX frame count");
            if version == 3 {
                animation_size = [reader.dimension()?, reader.dimension()?];
            }
            // Verify all frame bytes before allocating the frame vector.
            ensure!(
                reader.bytes.len() - reader.position >= count * 32,
                "Truncated TEX animation"
            );
            budget.charge(count * std::mem::size_of::<Frame>())?;
            for _ in 0..count {
                let image = reader.integer()?;
                ensure!(
                    (image as usize) < image_count,
                    "Invalid TEX animation image {image}"
                );
                let seconds = reader.float()?;
                ensure!(seconds > 0.0, "Invalid TEX frame duration");
                let mut rect = [0.0; 6];
                for value in &mut rect {
                    *value = if version == 1 {
                        reader.integer()? as f32
                    } else {
                        reader.float()?
                    };
                }
                if version == 1 {
                    rect[3] = 0.0;
                    rect[4] = 0.0;
                }
                ensure!(
                    (rect[2] * rect[5] - rect[3] * rect[4]).abs() > 0.0,
                    "Empty TEX animation rectangle"
                );
                let mip = &images[image as usize][0];
                for u in [0.0, 1.0] {
                    for v in [0.0, 1.0] {
                        let x = rect[0] + u * rect[2] + v * rect[3];
                        let y = rect[1] + u * rect[4] + v * rect[5];
                        ensure!(
                            x.is_finite()
                                && y.is_finite()
                                && (0.0..=mip.width as f32).contains(&x)
                                && (0.0..=mip.height as f32).contains(&y),
                            "TEX animation rectangle leaves its image"
                        );
                    }
                }
                frames.push(Frame {
                    image,
                    seconds,
                    rect,
                });
            }
            if version < 3 {
                let rect = frames[0].rect;
                animation_size = [rect[2].hypot(rect[4]) as u32, rect[3].hypot(rect[5]) as u32];
                ensure!(
                    animation_size.iter().all(|v| (1..=16384).contains(v)),
                    "Invalid TEX animation dimensions"
                );
            }
        }
        ensure!(
            reader.position == bytes.len(),
            "Unexpected trailing TEX data"
        );
        Ok(Self {
            flags,
            width,
            height,
            logical_width,
            logical_height,
            images,
            frames,
            animation_size,
        })
    }
    fn mip(
        reader: &mut Reader<'_>,
        budget: &mut Budget,
        container: u32,
        format: Option<Format>,
        encoded: Option<image::ImageFormat>,
    ) -> Result<Mip> {
        let width = reader.dimension()?;
        let height = reader.dimension()?;
        let compression = if container >= 2 { reader.integer()? } else { 0 };
        ensure!(compression <= 1, "Invalid TEX compression {compression}");
        let declared = if container >= 2 {
            reader.integer()? as usize
        } else {
            0
        };
        let stored = reader.integer()? as usize;
        let decoded = if compression == 0 { stored } else { declared };
        ensure!(
            stored <= MIP_BUDGET && decoded <= MIP_BUDGET,
            "TEX mip exceeds 64 MiB budget"
        );
        let rgba_size = width as usize * height as usize * 4;
        ensure!(
            rgba_size <= MIP_BUDGET,
            "Decoded TEX mip exceeds 64 MiB budget"
        );
        if let Some(format) = format {
            ensure!(
                decoded == format.bytes(width, height),
                "Invalid TEX pixel payload length"
            );
        }
        let data = reader.take(stored)?;
        // Charge both transient buffers and the retained decoded pixels before allocation.
        budget.charge(decoded + rgba_size + if compression != 0 { stored } else { 0 })?;
        let uncompressed;
        let payload = if compression == 1 {
            uncompressed =
                lz4_flex::block::decompress(data, decoded).context("Invalid TEX LZ4 payload")?;
            ensure!(
                uncompressed.len() == decoded,
                "Invalid TEX LZ4 output length"
            );
            uncompressed.as_slice()
        } else {
            data
        };
        let rgba = if let Some(format) = format {
            decode_pixels(format, width, height, payload)?
        } else {
            let mut decoder = image::ImageReader::with_format(
                Cursor::new(payload),
                encoded.context("Missing image format")?,
            );
            let mut limits = image::Limits::default();
            limits.max_alloc = Some(MIP_BUDGET as u64);
            limits.max_image_width = Some(width);
            limits.max_image_height = Some(height);
            decoder.limits(limits);
            let image = decoder.decode().context("Encoded TEX image")?;
            ensure!(
                image.width() == width && image.height() == height,
                "Encoded TEX dimensions disagree with header"
            );
            image.into_rgba8().into_raw()
        };
        Ok(Mip {
            width,
            height,
            rgba,
        })
    }
}

fn rgb565(value: u16) -> [u8; 4] {
    let r = ((value >> 11) & 31) as u8;
    let g = ((value >> 5) & 63) as u8;
    let b = (value & 31) as u8;
    [
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
        255,
    ]
}
fn decode_pixels(format: Format, width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>> {
    ensure!(
        data.len() == format.bytes(width, height),
        "Invalid TEX pixel payload length"
    );
    if format == Format::Rgba {
        return Ok(data.to_vec());
    }
    let mut out = vec![0; width as usize * height as usize * 4];
    if matches!(format, Format::R | Format::Rg) {
        let stride = if format == Format::R { 1 } else { 2 };
        for (pixel, source) in out
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(data.chunks_exact(stride))
        {
            pixel.copy_from_slice(&[source[0], if stride == 2 { source[1] } else { 0 }, 0, 255]);
        }
        return Ok(out);
    }
    let stride = if format == Format::Dxt1 { 8 } else { 16 };
    let columns = width.div_ceil(4) as usize;
    for (index, block) in data.chunks_exact(stride).enumerate() {
        let color = &block[stride - 8..];
        let c0 = u16::from_le_bytes(color[..2].try_into()?);
        let c1 = u16::from_le_bytes(color[2..4].try_into()?);
        let mut palette = [rgb565(c0), rgb565(c1), [0; 4], [0; 4]];
        let a = palette[0];
        let b = palette[1];
        if c0 > c1 || format != Format::Dxt1 {
            palette[2] = std::array::from_fn(|channel| {
                if channel == 3 {
                    255
                } else {
                    ((2 * a[channel] as u16 + b[channel] as u16) / 3) as u8
                }
            });
            palette[3] = std::array::from_fn(|channel| {
                if channel == 3 {
                    255
                } else {
                    ((a[channel] as u16 + 2 * b[channel] as u16) / 3) as u8
                }
            });
        } else {
            palette[2] = std::array::from_fn(|channel| {
                if channel == 3 {
                    255
                } else {
                    ((a[channel] as u16 + b[channel] as u16) / 2) as u8
                }
            });
        }
        let indices = u32::from_le_bytes(color[4..8].try_into()?);
        let mut alphas = [0u8; 8];
        let alpha_indices = if format == Format::Dxt5 {
            alphas[0] = block[0];
            alphas[1] = block[1];
            if alphas[0] > alphas[1] {
                for i in 1..=6 {
                    alphas[i + 1] = (((7 - i) as u16 * alphas[0] as u16
                        + i as u16 * alphas[1] as u16)
                        / 7) as u8;
                }
            } else {
                for i in 1..=4 {
                    alphas[i + 1] = (((5 - i) as u16 * alphas[0] as u16
                        + i as u16 * alphas[1] as u16)
                        / 5) as u8;
                }
                alphas[7] = 255;
            }
            let mut value = 0u64;
            for (i, byte) in block[2..8].iter().enumerate() {
                value |= (*byte as u64) << (i * 8);
            }
            value
        } else {
            0
        };
        for p in 0..16 {
            let x = (index % columns) * 4 + p % 4;
            let y = (index / columns) * 4 + p / 4;
            if x >= width as usize || y >= height as usize {
                continue;
            }
            let mut pixel = palette[((indices >> (p * 2)) & 3) as usize];
            if format == Format::Dxt3 {
                pixel[3] = ((block[p / 2] >> ((p % 2) * 4)) & 15) * 17;
            }
            if format == Format::Dxt5 {
                pixel[3] = alphas[((alpha_indices >> (p * 3)) & 7) as usize];
            }
            let offset = (y * width as usize + x) * 4;
            out[offset..offset + 4].copy_from_slice(&pixel);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::fixtures;

    fn red_payload(format: u32) -> Vec<u8> {
        let color = [0x00, 0xf8, 0x00, 0x00, 0, 0, 0, 0];
        match format {
            0 => [255, 0, 0, 255].repeat(16),
            9 => vec![255; 16],
            8 => [255, 0].repeat(16),
            7 => color.to_vec(),
            6 => [vec![255; 8], color.to_vec()].concat(),
            4 => [vec![255, 0, 0, 0, 0, 0, 0, 0], color.to_vec()].concat(),
            _ => unreachable!(),
        }
    }
    #[test]
    fn twelve_raw_and_lz4_formats_decode_the_same_pixels() {
        for format in [0, 9, 8, 7, 6, 4] {
            for compressed in [false, true] {
                let bytes = fixtures::texture(format, &red_payload(format), compressed);
                let decoded = Texture::parse(&bytes).unwrap();
                assert_eq!(
                    decoded.images[0][0].rgba,
                    [255, 0, 0, 255].repeat(16),
                    "format {format}, compressed {compressed}"
                );
            }
        }
    }
    #[test]
    fn truncated_invalid_and_oversized_containers_are_rejected() {
        let original = fixtures::texture(0, &red_payload(0), false);
        for length in 0..original.len() {
            assert!(
                Texture::parse(&original[..length]).is_err(),
                "truncation {length}"
            );
        }
        // Header dimensions, image count, mip count/dimensions, compression and lengths.
        for (offset, value) in [
            (26, 0),
            (30, 16385),
            (34, u32::MAX),
            (55, 4097),
            (59, 17),
            (63, 0),
            (67, 16385),
            (71, 2),
            (79, 0x80000000),
        ] {
            let mut bytes = original.clone();
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(Texture::parse(&bytes).is_err(), "offset {offset}");
        }
        let mut compressed = fixtures::texture(0, &red_payload(0), true);
        compressed[75..79].copy_from_slice(&63u32.to_le_bytes());
        assert!(Texture::parse(&compressed).is_err());
        compressed[75..79].copy_from_slice(&64u32.to_le_bytes());
        compressed[83] = 0;
        assert!(Texture::parse(&compressed).is_err());
        let mut budget = Budget(TEXTURE_BUDGET - 1);
        assert!(budget.charge(2).is_err());
        let mut budget = Budget(usize::MAX);
        assert!(budget.charge(1).is_err());
    }
    #[test]
    fn legacy_container_and_encoded_png_are_decoded() {
        let original = fixtures::texture(0, &red_payload(0), false);
        let mut legacy = original.clone();
        legacy[53] = b'1';
        legacy.drain(71..79);
        assert_eq!(
            Texture::parse(&legacy).unwrap().images[0][0].rgba,
            red_payload(0)
        );
        let pixels = image::RgbaImage::from_raw(4, 4, red_payload(0)).unwrap();
        let mut png = Cursor::new(Vec::new());
        pixels.write_to(&mut png, image::ImageFormat::Png).unwrap();
        let mut encoded = fixtures::texture(u32::MAX, png.get_ref(), false);
        encoded[53] = b'3';
        encoded.splice(59..59, 13u32.to_le_bytes());
        assert_eq!(
            Texture::parse(&encoded).unwrap().images[0][0].rgba,
            red_payload(0)
        );
        // The decoder must reject a dishonest outer dimension as well.
        encoded[67..71].copy_from_slice(&3u32.to_le_bytes());
        assert!(Texture::parse(&encoded).is_err());
    }
    #[test]
    fn dxt_transparency_alpha_and_partial_blocks() {
        let mut block = [0u8; 8];
        block[4..].fill(255);
        assert_eq!(
            decode_pixels(Format::Dxt1, 1, 1, &block).unwrap(),
            [0, 0, 0, 0]
        );
        let mut block3 = [0u8; 16];
        block3[0] = 0x98;
        block3[8..10].copy_from_slice(&0xf800u16.to_le_bytes());
        let result = decode_pixels(Format::Dxt3, 2, 1, &block3).unwrap();
        assert_eq!(result, [255, 0, 0, 136, 255, 0, 0, 153]);
        let mut block5 = block3;
        block5[0] = 0;
        block5[1] = 255;
        block5[2] = 6 | (7 << 3);
        assert_eq!(
            decode_pixels(Format::Dxt5, 2, 1, &block5).unwrap(),
            [255, 0, 0, 0, 255, 0, 0, 255]
        );
    }

    #[test]
    fn mip_budget_is_checked_before_payload_and_decode_allocation() {
        // A header alone is enough to reject the required output allocation.
        let mut bytes = Vec::new();
        for value in [16384u32, 16384, 0, 1, 1] {
            bytes.extend(value.to_le_bytes());
        }
        let mut reader = Reader {
            bytes: &bytes,
            position: 0,
        };
        let error = Texture::mip(
            &mut reader,
            &mut Budget::default(),
            2,
            Some(Format::Rgba),
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("64 MiB"));

        // A valid tiny mip exceeding the aggregate budget is rejected before decode.
        bytes.clear();
        for value in [1u32, 1, 0, 4, 4] {
            bytes.extend(value.to_le_bytes());
        }
        bytes.extend([255, 0, 0, 255]);
        let mut reader = Reader {
            bytes: &bytes,
            position: 0,
        };
        let error = Texture::mip(
            &mut reader,
            &mut Budget(TEXTURE_BUDGET - 7),
            2,
            Some(Format::Rgba),
            None,
        )
        .unwrap_err();
        assert!(error.to_string().contains("256 MiB"));
    }

    #[test]
    fn animation_and_non_video_v4_are_bounded() {
        let mut bytes = fixtures::texture(0, &red_payload(0), false);
        bytes[53] = b'4';
        bytes.splice(
            59..59,
            [u32::MAX.to_le_bytes(), 0u32.to_le_bytes()].concat(),
        );
        assert!(Texture::parse(&bytes).is_ok());
        bytes[63..67].copy_from_slice(&1u32.to_le_bytes());
        assert!(Texture::parse(&bytes)
            .unwrap_err()
            .to_string()
            .contains("MP4"));

        let mut bytes = fixtures::texture(0, &red_payload(0), false);
        bytes[22..26].copy_from_slice(&7u32.to_le_bytes());
        let trailer = bytes.len();
        bytes.extend(b"TEXS0003\0");
        for value in [1u32, 4, 4, 0] {
            bytes.extend(value.to_le_bytes());
        }
        for value in [0.1f32, 0.0, 0.0, 4.0, 0.0, 0.0, 4.0] {
            bytes.extend(value.to_le_bytes());
        }
        let texture = Texture::parse(&bytes).unwrap();
        assert_eq!(texture.frames.len(), 1);
        assert_eq!(texture.animation_size, [4, 4]);
        let frame = texture.frame_at(0.0).unwrap();
        assert_eq!(frame.rotation, [1.0, 0.0, 0.0, 1.0]);
        assert!(texture.frame_at(f64::NAN).is_err());
        let mut rotated = bytes.clone();
        for (index, value) in [0.0f32, 4.0, 0.0, 4.0, -4.0, 0.0].into_iter().enumerate() {
            let offset = trailer + 29 + index * 4;
            rotated[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        let rotated = Texture::parse(&rotated).unwrap();
        assert_eq!(
            rotated.frame_at(0.0).unwrap().rotation,
            [0.0, 1.0, -1.0, 0.0]
        );
        let mut outside = bytes.clone();
        outside[trailer + 29..trailer + 33].copy_from_slice(&1.0f32.to_le_bytes());
        assert!(Texture::parse(&outside)
            .unwrap_err()
            .to_string()
            .contains("leaves its image"));
        for length in trailer..bytes.len() {
            assert!(Texture::parse(&bytes[..length]).is_err());
        }
        bytes[trailer + 25..trailer + 29].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(Texture::parse(&bytes).is_err());
    }

    #[test]
    fn animation_frame_selection_loops_at_injected_time() {
        let texture = Texture {
            flags: 4,
            width: 1,
            height: 1,
            logical_width: 1,
            logical_height: 1,
            images: vec![vec![Mip {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 255],
            }]],
            frames: vec![
                Frame {
                    image: 0,
                    seconds: 0.25,
                    rect: [0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                },
                Frame {
                    image: 0,
                    seconds: 0.75,
                    rect: [0.0, 0.0, 0.0, 1.0, -1.0, 0.0],
                },
            ],
            animation_size: [1, 1],
        };
        assert_eq!(
            texture.frame_at(0.0).unwrap().rotation,
            [1.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(
            texture.frame_at(0.25).unwrap().rotation,
            [1.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(
            texture.frame_at(0.99).unwrap().rotation,
            [0.0, 1.0, -1.0, 0.0]
        );
        assert_eq!(
            texture.frame_at(1.0).unwrap().rotation,
            [1.0, 0.0, 0.0, 1.0]
        );

        let mut invalid = texture;
        invalid.frames[0].seconds = 0.0;
        assert!(invalid.frame_at(0.0).is_err());
    }
}
