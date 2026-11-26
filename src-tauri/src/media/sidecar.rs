use tauri::AppHandle;
use tauri_plugin_shell::{process::Command, ShellExt};

use crate::error::{AppError, AppResult};

fn sidecar(app: &AppHandle, name: &str) -> AppResult<Command> {
    app.shell()
        .sidecar(name)
        .map_err(|e| AppError::FfmpegNotFound(e.to_string()))
}

pub fn ffmpeg(app: &AppHandle) -> AppResult<Command> {
    sidecar(app, "ffmpeg")
}

pub fn ffprobe(app: &AppHandle) -> AppResult<Command> {
    sidecar(app, "ffprobe")
}
