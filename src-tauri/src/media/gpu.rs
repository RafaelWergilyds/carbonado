use tauri::AppHandle;

use super::sidecar;
use crate::domain::model::{Encoder, EncoderSupport};

async fn works(app: &AppHandle, encoder: Encoder) -> bool {
    let Ok(cmd) = sidecar::ffmpeg(app) else {
        return false;
    };
    cmd.args([
        "-hide_banner", "-v", "error",
        "-f", "lavfi", "-i", "color=black:s=256x256:d=0.1",
        "-c:v", encoder.ffmpeg_name(),
        "-f", "null", "-",
    ])
    .output()
    .await
    .map(|o| o.status.success())
    .unwrap_or(false)
}

pub async fn detect(app: &AppHandle) -> EncoderSupport {
    EncoderSupport {
        h264_nvenc: works(app, Encoder::NvencH264).await,
        hevc_nvenc: works(app, Encoder::NvencHevc).await,
    }
}
