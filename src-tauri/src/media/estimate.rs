use std::path::PathBuf;
use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_shell::process::CommandEvent;

use super::sidecar;
use crate::domain::args;
use crate::domain::estimate::{extrapolate, sample_windows, SizeEstimate};
use crate::domain::model::{is_gpu_failure, Encoder, Resolution, VideoInfo};
use crate::error::{AppError, AppResult};

const SUPERSEDE_POLL: Duration = Duration::from_millis(100);

fn temp_dir() -> AppResult<PathBuf> {
    let dir = std::env::temp_dir().join("carbonado");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

async fn sample_bytes(
    app: &AppHandle,
    info: &VideoInfo,
    encoder: Encoder,
    quality: u8,
    resolution: Resolution,
    tag: u64,
    is_current: &(dyn Fn() -> bool + Sync),
) -> AppResult<Option<(u64, f64)>> {
    let dir = temp_dir()?;
    let mut total_bytes = 0u64;
    let mut total_s = 0.0;

    for (i, (start, len)) in sample_windows(info.duration_s).into_iter().enumerate() {
        if !is_current() {
            return Ok(None);
        }
        let out = dir.join(format!("estimate-{tag}-{i}.mp4"));
        let cmd_args = args::build_sample(info, &out, encoder, quality, resolution, start, len);
        let run = run_sample(app, cmd_args, is_current).await;
        let size = std::fs::metadata(&out).map(|m| m.len());
        let _ = std::fs::remove_file(&out);

        match run? {
            SampleRun::Superseded => return Ok(None),
            SampleRun::Failed(stderr_tail) => return Err(AppError::EncodeFailed { stderr_tail }),
            SampleRun::Ok => {}
        }
        total_bytes += size?;
        total_s += len;
    }
    Ok(Some((total_bytes, total_s)))
}

enum SampleRun {
    Ok,
    Failed(String),
    Superseded,
}

async fn run_sample(
    app: &AppHandle,
    cmd_args: Vec<String>,
    is_current: &(dyn Fn() -> bool + Sync),
) -> AppResult<SampleRun> {
    let (mut rx, child) = sidecar::ffmpeg(app)?
        .args(cmd_args)
        .spawn()
        .map_err(|e| AppError::FfmpegNotFound(e.to_string()))?;
    let mut child = Some(child);
    let mut stderr = String::new();
    let mut exit_code = None;

    loop {
        match tokio::time::timeout(SUPERSEDE_POLL, rx.recv()).await {
            Ok(Some(CommandEvent::Stderr(bytes))) => stderr.push_str(&String::from_utf8_lossy(&bytes)),
            Ok(Some(CommandEvent::Error(err))) => stderr.push_str(&err),
            Ok(Some(CommandEvent::Terminated(payload))) => exit_code = payload.code,
            Ok(Some(_)) => {}
            Ok(None) => break,
            Err(_) => {
                if !is_current() {
                    if let Some(c) = child.take() {
                        let _ = c.kill();
                    }
                }
            }
        }
    }

    Ok(if !is_current() {
        SampleRun::Superseded
    } else if exit_code == Some(0) {
        SampleRun::Ok
    } else {
        SampleRun::Failed(stderr.trim().to_string())
    })
}

pub async fn estimate(
    app: &AppHandle,
    info: &VideoInfo,
    encoder: Encoder,
    quality: u8,
    resolution: Resolution,
    tag: u64,
    is_current: &(dyn Fn() -> bool + Sync),
) -> AppResult<Option<SizeEstimate>> {
    let mut encoder = encoder;
    let sampled = match sample_bytes(app, info, encoder, quality, resolution, tag, is_current).await {
        Err(AppError::EncodeFailed { stderr_tail })
            if encoder.is_gpu() && is_gpu_failure(&stderr_tail) =>
        {
            encoder = encoder.cpu_equivalent();
            sample_bytes(app, info, encoder, quality, resolution, tag, is_current).await?
        }
        other => other?,
    };

    Ok(sampled.map(|(bytes, secs)| SizeEstimate {
        bytes: extrapolate(bytes, secs, info.duration_s, info.has_audio),
        used_gpu: encoder.is_gpu(),
    }))
}
