use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum LayoutMode {
    #[default]
    Fill,
    Contain,
    Stretch,
    Cover,
    Centre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum HwdecMode {
    #[default]
    Auto,
    Force,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderBackend {
    #[default]
    Auto,
    #[serde(rename = "opengl")]
    OpenGL,
    Vulkan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HdrMode {
    #[default]
    Auto,
    Force,
    Disable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum TransferFunction {
    #[default]
    Srgb,
    Pq,
    Hlg,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ColorSpace {
    #[default]
    Sdr,
    Hdr10,
    Hlg,
    DolbyVision,
    Unknown,
}

impl ColorSpace {
    fn is_hdr(self) -> bool {
        matches!(self, Self::Hdr10 | Self::Hlg | Self::DolbyVision)
    }
}

impl TransferFunction {
    fn is_hdr(self) -> bool {
        matches!(self, Self::Pq | Self::Hlg)
    }
}

#[derive(Debug, Clone, Default)]
pub struct HdrMetadata {
    pub color_space: ColorSpace,
    pub transfer_function: TransferFunction,
    pub primaries: String,
    pub peak_luminance: Option<f64>,
    pub avg_luminance: Option<f64>,
    pub min_luminance: Option<f64>,
}

impl HdrMetadata {
    pub fn is_hdr(&self) -> bool {
        self.color_space.is_hdr() || self.transfer_function.is_hdr()
    }

    pub fn format_description(&self) -> String {
        if !self.is_hdr() {
            return "SDR".to_string();
        }
        match (self.color_space, self.transfer_function) {
            (ColorSpace::Hdr10, TransferFunction::Pq) => "HDR10",
            (ColorSpace::Hlg, TransferFunction::Hlg) => "HLG",
            (ColorSpace::DolbyVision, _) => "Dolby Vision",
            _ => "HDR (Unknown)",
        }
        .to_string()
    }
}

pub fn parse_colorspace(value: &str) -> ColorSpace {
    match value.to_lowercase().as_str() {
        "bt.709" | "srgb" => ColorSpace::Sdr,
        "bt.2020-ncl" | "bt.2020-cl" | "bt2020" => ColorSpace::Hdr10,
        _ => ColorSpace::Unknown,
    }
}

pub fn parse_transfer_function(value: &str) -> TransferFunction {
    match value.to_lowercase().as_str() {
        "pq" | "smpte2084" | "st2084" => TransferFunction::Pq,
        "hlg" | "arib-std-b67" => TransferFunction::Hlg,
        "srgb" | "bt.1886" | "bt.709" => TransferFunction::Srgb,
        _ => TransferFunction::Unknown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ToneMappingAlgorithm {
    #[default]
    Hable,
    Mobius,
    Reinhard,
    Bt2390,
    Clip,
}

impl ToneMappingAlgorithm {
    pub fn as_mpv_str(self) -> &'static str {
        match self {
            Self::Hable => "hable",
            Self::Mobius => "mobius",
            Self::Reinhard => "reinhard",
            Self::Bt2390 => "bt.2390",
            Self::Clip => "clip",
        }
    }

    pub fn recommended_param(self) -> f64 {
        match self {
            Self::Hable => 1.0,
            Self::Mobius => 0.3,
            Self::Reinhard => 0.5,
            Self::Bt2390 | Self::Clip => 1.0,
        }
    }

    pub fn uses_param(self) -> bool {
        !matches!(self, Self::Clip | Self::Bt2390)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToneMappingConfig {
    #[serde(default)]
    pub algorithm: ToneMappingAlgorithm,
    #[serde(default = "default_tone_mapping_param")]
    pub param: f64,
    #[serde(default = "default_compute_peak")]
    pub compute_peak: bool,
    #[serde(default = "default_tone_mapping_mode")]
    pub mode: String,
}

impl Default for ToneMappingConfig {
    fn default() -> Self {
        Self {
            algorithm: ToneMappingAlgorithm::default(),
            param: 1.0,
            compute_peak: true,
            mode: "hybrid".to_string(),
        }
    }
}

impl ToneMappingConfig {
    /// Replaces the default parameter with the algorithm's recommended one.
    /// The content metadata is not yet used to tune the value further.
    pub fn optimize_for_content(&mut self, _metadata: &HdrMetadata) {
        if (self.param - 1.0).abs() < 0.01 {
            self.param = self.algorithm.recommended_param();
        }
    }
}

fn default_tone_mapping_param() -> f64 {
    1.0
}
fn default_compute_peak() -> bool {
    true
}
fn default_tone_mapping_mode() -> String {
    "hybrid".to_string()
}

#[derive(Debug, Clone)]
pub struct OutputHdrCapabilities {
    pub hdr_supported: bool,
    pub max_luminance: Option<f64>,
    pub min_luminance: Option<f64>,
    pub supported_eotf: Vec<TransferFunction>,
}

impl Default for OutputHdrCapabilities {
    fn default() -> Self {
        Self {
            hdr_supported: false,
            max_luminance: Some(203.0),
            min_luminance: Some(0.0),
            supported_eotf: vec![TransferFunction::Srgb],
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputInfo {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub position: (i32, i32),
    pub active: bool,
    pub hdr_capabilities: OutputHdrCapabilities,
}

impl OutputInfo {
    pub fn logical_size(&self) -> (f64, f64) {
        (
            self.width as f64 / self.scale,
            self.height as f64 / self.scale,
        )
    }
}

#[derive(Debug, Clone)]
pub struct LayoutTransform {
    pub src_rect: (f64, f64, f64, f64),
    pub dst_rect: (i32, i32, i32, i32),
}

pub fn calculate_layout(
    mode: LayoutMode,
    video_width: i32,
    video_height: i32,
    output_width: i32,
    output_height: i32,
) -> LayoutTransform {
    let video_aspect = video_width as f64 / video_height as f64;
    let output_aspect = output_width as f64 / output_height as f64;
    match mode {
        LayoutMode::Fill | LayoutMode::Cover => {
            if video_aspect > output_aspect {
                let scaled_width = video_width as f64 * output_height as f64 / video_height as f64;
                let crop_width = output_width as f64 / scaled_width;
                LayoutTransform {
                    src_rect: ((1.0 - crop_width) / 2.0, 0.0, crop_width, 1.0),
                    dst_rect: (0, 0, output_width, output_height),
                }
            } else {
                let scaled_height = video_height as f64 * output_width as f64 / video_width as f64;
                let crop_height = output_height as f64 / scaled_height;
                LayoutTransform {
                    src_rect: (0.0, (1.0 - crop_height) / 2.0, 1.0, crop_height),
                    dst_rect: (0, 0, output_width, output_height),
                }
            }
        }
        LayoutMode::Contain => {
            if video_aspect > output_aspect {
                let scaled_height =
                    (video_height as f64 * output_width as f64 / video_width as f64) as i32;
                LayoutTransform {
                    src_rect: (0.0, 0.0, 1.0, 1.0),
                    dst_rect: (
                        0,
                        (output_height - scaled_height) / 2,
                        output_width,
                        scaled_height,
                    ),
                }
            } else {
                let scaled_width =
                    (video_width as f64 * output_height as f64 / video_height as f64) as i32;
                LayoutTransform {
                    src_rect: (0.0, 0.0, 1.0, 1.0),
                    dst_rect: (
                        (output_width - scaled_width) / 2,
                        0,
                        scaled_width,
                        output_height,
                    ),
                }
            }
        }
        LayoutMode::Stretch => LayoutTransform {
            src_rect: (0.0, 0.0, 1.0, 1.0),
            dst_rect: (0, 0, output_width, output_height),
        },
        LayoutMode::Centre => LayoutTransform {
            src_rect: (0.0, 0.0, 1.0, 1.0),
            dst_rect: (
                (output_width - video_width) / 2,
                (output_height - video_height) / 2,
                video_width,
                video_height,
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hdr(color_space: ColorSpace, transfer_function: TransferFunction) -> HdrMetadata {
        HdrMetadata {
            color_space,
            transfer_function,
            ..HdrMetadata::default()
        }
    }

    #[test]
    fn hdr_detection_and_description() {
        assert!(!HdrMetadata::default().is_hdr());
        assert_eq!(HdrMetadata::default().format_description(), "SDR");
        assert_eq!(
            hdr(ColorSpace::Hdr10, TransferFunction::Pq).format_description(),
            "HDR10"
        );
        assert_eq!(
            hdr(ColorSpace::Hlg, TransferFunction::Hlg).format_description(),
            "HLG"
        );
        assert_eq!(
            hdr(ColorSpace::DolbyVision, TransferFunction::Srgb).format_description(),
            "Dolby Vision"
        );
        assert_eq!(
            hdr(ColorSpace::Sdr, TransferFunction::Pq).format_description(),
            "HDR (Unknown)"
        );
        assert!(!hdr(ColorSpace::Unknown, TransferFunction::Unknown).is_hdr());
    }

    #[test]
    fn parses_colorspace_and_transfer_function() {
        assert_eq!(parse_colorspace("BT.709"), ColorSpace::Sdr);
        assert_eq!(parse_colorspace("srgb"), ColorSpace::Sdr);
        assert_eq!(parse_colorspace("bt.2020-ncl"), ColorSpace::Hdr10);
        assert_eq!(parse_colorspace("bt2020"), ColorSpace::Hdr10);
        assert_eq!(parse_colorspace("weird"), ColorSpace::Unknown);

        assert_eq!(parse_transfer_function("SMPTE2084"), TransferFunction::Pq);
        assert_eq!(
            parse_transfer_function("arib-std-b67"),
            TransferFunction::Hlg
        );
        assert_eq!(parse_transfer_function("bt.1886"), TransferFunction::Srgb);
        assert_eq!(parse_transfer_function("?"), TransferFunction::Unknown);
    }

    #[test]
    fn tone_mapping_algorithm_properties() {
        use ToneMappingAlgorithm::*;
        let expected = [
            (Hable, "hable", 1.0, true),
            (Mobius, "mobius", 0.3, true),
            (Reinhard, "reinhard", 0.5, true),
            (Bt2390, "bt.2390", 1.0, false),
            (Clip, "clip", 1.0, false),
        ];
        for (algorithm, name, param, uses_param) in expected {
            assert_eq!(algorithm.as_mpv_str(), name);
            assert_eq!(algorithm.recommended_param(), param);
            assert_eq!(algorithm.uses_param(), uses_param);
        }
    }

    #[test]
    fn tone_mapping_config_defaults_and_optimization() {
        let parsed: ToneMappingConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(parsed.algorithm, ToneMappingAlgorithm::Hable);
        assert_eq!(parsed.param, 1.0);
        assert!(parsed.compute_peak);
        assert_eq!(parsed.mode, "hybrid");

        let mut config = ToneMappingConfig {
            algorithm: ToneMappingAlgorithm::Mobius,
            ..ToneMappingConfig::default()
        };
        config.optimize_for_content(&HdrMetadata {
            peak_luminance: Some(4000.0),
            ..HdrMetadata::default()
        });
        assert_eq!(config.param, 0.3);

        // A user-tuned parameter is left untouched.
        let mut tuned = ToneMappingConfig {
            algorithm: ToneMappingAlgorithm::Reinhard,
            param: 0.8,
            ..ToneMappingConfig::default()
        };
        tuned.optimize_for_content(&HdrMetadata::default());
        assert_eq!(tuned.param, 0.8);
    }

    #[test]
    fn serde_names_match_config_format() {
        assert_eq!(
            serde_json::to_string(&RenderBackend::OpenGL).unwrap(),
            "\"opengl\""
        );
        assert_eq!(
            serde_json::to_string(&HdrMode::Disable).unwrap(),
            "\"disable\""
        );
        assert_eq!(
            serde_json::from_str::<ToneMappingAlgorithm>("\"bt2390\"").unwrap(),
            ToneMappingAlgorithm::Bt2390
        );
    }

    #[test]
    fn output_info_logical_size() {
        let info = OutputInfo {
            name: "DP-1".into(),
            width: 3840,
            height: 2160,
            scale: 2.0,
            position: (0, 0),
            active: true,
            hdr_capabilities: OutputHdrCapabilities::default(),
        };
        assert_eq!(info.logical_size(), (1920.0, 1080.0));
        assert!(!info.hdr_capabilities.hdr_supported);
        assert_eq!(info.hdr_capabilities.max_luminance, Some(203.0));
    }

    #[test]
    fn layout_fill_crops_wider_video_horizontally() {
        let t = calculate_layout(LayoutMode::Fill, 3840, 1080, 1920, 1080);
        assert_eq!(t.dst_rect, (0, 0, 1920, 1080));
        assert_eq!(t.src_rect, (0.25, 0.0, 0.5, 1.0));
    }

    #[test]
    fn layout_cover_crops_taller_video_vertically() {
        let t = calculate_layout(LayoutMode::Cover, 1920, 2160, 1920, 1080);
        assert_eq!(t.dst_rect, (0, 0, 1920, 1080));
        assert_eq!(t.src_rect, (0.0, 0.25, 1.0, 0.5));
    }

    #[test]
    fn layout_contain_letterboxes_and_pillarboxes() {
        let wide = calculate_layout(LayoutMode::Contain, 3840, 1080, 1920, 1080);
        assert_eq!(wide.src_rect, (0.0, 0.0, 1.0, 1.0));
        assert_eq!(wide.dst_rect, (0, 270, 1920, 540));

        let tall = calculate_layout(LayoutMode::Contain, 1080, 1080, 1920, 1080);
        assert_eq!(tall.dst_rect, (420, 0, 1080, 1080));
    }

    #[test]
    fn layout_stretch_and_centre() {
        let stretch = calculate_layout(LayoutMode::Stretch, 640, 480, 1920, 1080);
        assert_eq!(stretch.dst_rect, (0, 0, 1920, 1080));

        let centre = calculate_layout(LayoutMode::Centre, 640, 480, 1920, 1080);
        assert_eq!(centre.dst_rect, (640, 300, 640, 480));
    }
}
