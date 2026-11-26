use std::path::{Path, PathBuf};

use serde::Deserialize;
use tauri::{AppHandle, State};

use crate::domain::estimate::SizeEstimate;
use crate::domain::model::{Codec, Encoder, EncoderSupport, Resolution, VideoInfo};
use crate::domain::output;
use crate::error::AppResult;
use crate::media::{estimate, gpu, probe};
use crate::state::AppState;

#[tauri::command]
pub async fn probe_video(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> AppResult<VideoInfo> {
    let info = probe::probe(&app, &path).await?;
    state.remember_probe(&info);
    Ok(info)
}

#[tauri::command]
pub fn suggest_output(input: String, dir: Option<String>) -> String {
    let dir = dir.map(PathBuf::from).filter(|d| d.is_dir());
    output::suggest(Path::new(&input), dir.as_deref(), |p| p.exists())
        .to_string_lossy()
        .into_owned()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateRequest {
    input: String,
    quality: u8,
    codec: Codec,
    resolution: Resolution,
    use_gpu: bool,
}

#[tauri::command]
pub async fn estimate_size(
    app: AppHandle,
    state: State<'_, AppState>,
    request: EstimateRequest,
) -> AppResult<Option<SizeEstimate>> {
    let generation = state.next_estimate();
    if state.current_job().is_some() {
        return Ok(None);
    }

    let info = match state.cached_probe(&request.input) {
        Some(info) => info,
        None => {
            let info = probe::probe(&app, &request.input).await?;
            state.remember_probe(&info);
            info
        }
    };

    let wanted = Encoder::select(request.codec, request.use_gpu);
    let encoder = if wanted.is_gpu() && !encoder_support(&app, &state).await.supports(wanted) {
        wanted.cpu_equivalent()
    } else {
        wanted
    };

    let is_current = || state.is_current_estimate(generation);
    estimate::estimate(
        &app,
        &info,
        encoder,
        request.quality.min(100),
        request.resolution,
        generation,
        &is_current,
    )
    .await
}

#[tauri::command]
pub async fn detect_encoders(app: AppHandle, state: State<'_, AppState>) -> AppResult<EncoderSupport> {
    Ok(encoder_support(&app, &state).await)
}

pub(crate) async fn encoder_support(app: &AppHandle, state: &AppState) -> EncoderSupport {
    if let Some(cached) = state.cached_encoders() {
        return cached;
    }
    let support = gpu::detect(app).await;
    state.cache_encoders(support);
    support
}
