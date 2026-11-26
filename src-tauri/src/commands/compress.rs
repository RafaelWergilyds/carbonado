use std::path::{Path, PathBuf};

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, State};

use super::video::encoder_support;
use crate::domain::model::{CompressOptions, Encoder};
use crate::domain::output;
use crate::domain::progress::ProgressEvent;
use crate::error::{AppError, AppResult, InputProblem};
use crate::media::job::{self, JobOutcome, ProgressSink};
use crate::media::probe;
use crate::state::AppState;

pub const EVT_PROGRESS: &str = "compress://progress";
pub const EVT_FALLBACK: &str = "compress://fallback";
pub const EVT_DONE: &str = "compress://done";
pub const EVT_ERROR: &str = "compress://error";
pub const EVT_CANCELLED: &str = "compress://cancelled";

struct TauriSink(AppHandle);

impl ProgressSink for TauriSink {
    fn progress(&self, event: ProgressEvent) {
        let _ = self.0.emit(EVT_PROGRESS, event);
    }

    fn fallback(&self, reason: &str) {
        let _ = self.0.emit(EVT_FALLBACK, json!({ "reason": reason }));
    }
}

#[tauri::command]
pub async fn compress(
    app: AppHandle,
    state: State<'_, AppState>,
    options: CompressOptions,
) -> AppResult<()> {
    let control = state.begin_job()?;
    state.next_estimate();

    let prepared = async {
        if options.quality > 100 {
            return Err(AppError::InvalidInput(InputProblem::QualityOutOfRange));
        }
        let info = probe::probe(&app, &options.input).await?;
        let out_path = resolve_output(&options)?;
        let wanted = Encoder::select(options.codec, options.use_gpu);
        let encoder = if wanted.is_gpu() && !encoder_support(&app, &state).await.supports(wanted) {
            wanted.cpu_equivalent()
        } else {
            wanted
        };
        Ok((info, out_path, encoder))
    }
    .await;

    let (info, out_path, encoder) = match prepared {
        Ok(v) => v,
        Err(e) => {
            state.end_job();
            return Err(e);
        }
    };

    let app_bg = app.clone();
    tauri::async_runtime::spawn(async move {
        let sink = TauriSink(app_bg.clone());
        let outcome =
            job::run(&app_bg, &info, &options, &out_path, encoder, &control, &sink).await;
        app_bg.state::<AppState>().end_job();
        let _ = match outcome {
            Ok(JobOutcome::Finished(result)) => app_bg.emit(EVT_DONE, result),
            Ok(JobOutcome::Cancelled) => app_bg.emit(EVT_CANCELLED, json!({})),
            Err(err) => app_bg.emit(EVT_ERROR, serde_json::to_value(&err).unwrap_or_default()),
        };
    });

    Ok(())
}

fn resolve_output(options: &CompressOptions) -> AppResult<PathBuf> {
    let input = Path::new(&options.input);
    let out = match options.output.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(o) => output::normalize(Path::new(o)),
        None => output::suggest(input, None, |p| p.exists()),
    };

    if output::same_file(&out, input) {
        return Err(AppError::InvalidInput(InputProblem::OutputIsInput));
    }
    match out.parent() {
        Some(dir) if dir.as_os_str().is_empty() || dir.is_dir() => {}
        _ => return Err(AppError::InvalidInput(InputProblem::OutputDirMissing)),
    }
    Ok(out)
}

#[tauri::command]
pub fn cancel_compression(state: State<'_, AppState>) {
    if let Some(control) = state.current_job() {
        control.cancel();
    }
}
