use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub path: String,
    pub file_name: String,
    pub duration_s: f64,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub size_bytes: u64,
    pub bitrate: Option<u64>,
    pub has_audio: bool,
    pub hdr: bool,
}

impl VideoInfo {
    pub fn short_side(&self) -> u32 {
        self.width.min(self.height)
    }

    pub fn is_portrait(&self) -> bool {
        self.height > self.width
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    H265,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "1080p")]
    P1080,
    #[serde(rename = "720p")]
    P720,
    #[serde(rename = "480p")]
    P480,
}

impl Resolution {
    pub fn target_short_side(self) -> Option<u32> {
        match self {
            Resolution::Original => None,
            Resolution::P1080 => Some(1080),
            Resolution::P720 => Some(720),
            Resolution::P480 => Some(480),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressOptions {
    pub input: String,
    pub quality: u8,
    pub codec: Codec,
    pub resolution: Resolution,
    pub use_gpu: bool,
    #[serde(default)]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Encoder {
    X264,
    X265,
    NvencH264,
    NvencHevc,
}

impl Encoder {
    pub fn select(codec: Codec, gpu: bool) -> Self {
        match (codec, gpu) {
            (Codec::H264, false) => Encoder::X264,
            (Codec::H265, false) => Encoder::X265,
            (Codec::H264, true) => Encoder::NvencH264,
            (Codec::H265, true) => Encoder::NvencHevc,
        }
    }

    pub fn ffmpeg_name(self) -> &'static str {
        match self {
            Encoder::X264 => "libx264",
            Encoder::X265 => "libx265",
            Encoder::NvencH264 => "h264_nvenc",
            Encoder::NvencHevc => "hevc_nvenc",
        }
    }

    pub fn is_gpu(self) -> bool {
        matches!(self, Encoder::NvencH264 | Encoder::NvencHevc)
    }

    pub fn codec(self) -> Codec {
        match self {
            Encoder::X264 | Encoder::NvencH264 => Codec::H264,
            Encoder::X265 | Encoder::NvencHevc => Codec::H265,
        }
    }

    pub fn cpu_equivalent(self) -> Self {
        Encoder::select(self.codec(), false)
    }
}

pub fn is_gpu_failure(stderr: &str) -> bool {
    const MARKERS: [&str; 5] = ["nvenc", "cuda", "nvidia", "openencodesession", "no capable devices"];
    let lower = stderr.to_lowercase();
    MARKERS.iter().any(|m| lower.contains(m))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncoderSupport {
    pub h264_nvenc: bool,
    pub hevc_nvenc: bool,
}

impl EncoderSupport {
    pub fn supports(&self, encoder: Encoder) -> bool {
        match encoder {
            Encoder::X264 | Encoder::X265 => true,
            Encoder::NvencH264 => self.h264_nvenc,
            Encoder::NvencHevc => self.hevc_nvenc,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobResult {
    pub output_path: Option<String>,
    pub original_bytes: u64,
    pub final_bytes: u64,
    pub skipped: bool,
    pub encoder: Encoder,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_and_fallback() {
        assert_eq!(Encoder::select(Codec::H265, true), Encoder::NvencHevc);
        assert_eq!(Encoder::NvencHevc.cpu_equivalent(), Encoder::X265);
        assert_eq!(Encoder::NvencH264.cpu_equivalent(), Encoder::X264);
        assert!(!Encoder::X264.is_gpu());
    }

    #[test]
    fn detects_gpu_failures_only() {
        assert!(is_gpu_failure(
            "[h264_nvenc @ 000001c7c72be280] Driver does not support the required nvenc API version."
        ));
        assert!(is_gpu_failure("[hevc_nvenc @ 0x1] OpenEncodeSessionEx failed: out of memory (10)"));
        assert!(is_gpu_failure("Cannot load nvcuda.dll"));
        assert!(!is_gpu_failure("C:/out/v.mp4: Permission denied"));
        assert!(!is_gpu_failure("Error writing trailer: No space left on device"));
        assert!(!is_gpu_failure("[h264 @ 0x2] Invalid NAL unit size"));
    }

    #[test]
    fn options_deserialize_from_ui_payload() {
        let json = r#"{"input":"C:/v.mp4","quality":60,"codec":"h265","resolution":"720p","useGpu":true}"#;
        let o: CompressOptions = serde_json::from_str(json).unwrap();
        assert_eq!(o.codec, Codec::H265);
        assert_eq!(o.resolution, Resolution::P720);
        assert!(o.use_gpu);
        assert_eq!(o.output, None);
    }
}
