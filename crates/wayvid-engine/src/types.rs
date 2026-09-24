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
    pub fn optimize_for_content(&mut self, metadata: &HdrMetadata) {
        if (self.param - 1.0).abs() < 0.01 {
            self.param = match metadata.peak_luminance {
                Some(peak) if peak > 1000.0 => self.algorithm.recommended_param(),
                _ => self.algorithm.recommended_param(),
            };
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
