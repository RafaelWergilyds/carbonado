use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::AppHandle;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

use super::sidecar;
use crate::domain::model::{is_gpu_failure, CompressOptions, Encoder, JobResult, VideoInfo};
use crate::domain::progress::{ProgressEvent, ProgressParser};
use crate::domain::{args, output};
use crate::error::{AppError, AppResult};

const STDERR_TAIL_LINES: usize = 20;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

pub trait ProgressSink: Send + Sync {
    fn progress(&self, event: ProgressEvent);
    fn fallback(&self, reason: &str);
}

#[derive(Default)]
pub struct JobControl {
    child: Mutex<Option<CommandChild>>,
    cancelled: AtomicBool,
    temp_file: Mutex<Option<PathBuf>>,
}

impl JobControl {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Some(child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
        }
    }

    pub fn shutdown(&self) {
        self.cancel();
        let Some(path) = self.temp_file.lock().unwrap().take() else {
            return;
        };
        for _ in 0..20 {
            match std::fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                _ => return,
            }
        }
    }

    fn set_temp_file(&self, path: &Path) {
        *self.temp_file.lock().unwrap() = Some(path.to_path_buf());
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    fn attach(&self, child: CommandChild) {
        *self.child.lock().unwrap() = Some(child);
        if self.is_cancelled() {
            self.cancel();
        }
    }

    fn detach(&self) {
        self.child.lock().unwrap().take();
    }
}

pub enum JobOutcome {
    Finished(JobResult),
    Cancelled,
}

fn remove_partial(path: &Path) {
    let _ = std::fs::remove_file(path);
}

pub async fn run(
    app: &AppHandle,
    info: &VideoInfo,
    opts: &CompressOptions,
    out_path: &Path,
    mut encoder: Encoder,
    control: &JobControl,
    sink: &dyn ProgressSink,
) -> AppResult<JobOutcome> {
    let tmp_path = output::temp_path(out_path);
    control.set_temp_file(&tmp_path);

    loop {
        let cmd_args = args::build(info, &tmp_path, encoder, opts.quality, opts.resolution);
        let attempt = encode_once(app, &cmd_args, info.duration_s, control, sink).await;

        if control.is_cancelled() {
            remove_partial(&tmp_path);
            return Ok(JobOutcome::Cancelled);
        }
        match attempt {
            Ok(()) => break,
            Err(AppError::EncodeFailed { stderr_tail })
                if encoder.is_gpu() && is_gpu_failure(&stderr_tail) =>
            {
                remove_partial(&tmp_path);
                sink.fallback(&stderr_tail);
                encoder = encoder.cpu_equivalent();
            }
            Err(e) => {
                remove_partial(&tmp_path);
                return Err(e);
            }
        }
    }

    let final_bytes = std::fs::metadata(&tmp_path)?.len();
    if final_bytes >= info.size_bytes {
        remove_partial(&tmp_path);
        return Ok(JobOutcome::Finished(JobResult {
            output_path: None,
            original_bytes: info.size_bytes,
            final_bytes,
            skipped: true,
            encoder,
        }));
    }

    if let Err(e) = std::fs::rename(&tmp_path, out_path) {
        remove_partial(&tmp_path);
        return Err(e.into());
    }

    Ok(JobOutcome::Finished(JobResult {
        output_path: Some(out_path.to_string_lossy().into_owned()),
        original_bytes: info.size_bytes,
        final_bytes,
        skipped: false,
        encoder,
    }))
}

async fn encode_once(
    app: &AppHandle,
    cmd_args: &[String],
    duration_s: f64,
    control: &JobControl,
    sink: &dyn ProgressSink,
) -> AppResult<()> {
    let (mut rx, child) = sidecar::ffmpeg(app)?
        .args(cmd_args)
        .spawn()
        .map_err(|e| AppError::FfmpegNotFound(e.to_string()))?;
    control.attach(child);

    let mut parser = ProgressParser::default();
    let mut stderr_tail: VecDeque<String> = VecDeque::with_capacity(STDERR_TAIL_LINES);
    let mut last_emit: Option<Instant> = None;
    let mut exit_code: Option<i32> = None;

    let mut push_stderr = |line: &str| {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        if stderr_tail.len() == STDERR_TAIL_LINES {
            stderr_tail.pop_front();
        }
        stderr_tail.push_back(line.to_string());
    };

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(bytes) => {
                for line in String::from_utf8_lossy(&bytes).lines() {
                    let Some(snap) = parser.feed(line) else { continue };
                    let due = last_emit.is_none_or(|t| t.elapsed() >= PROGRESS_INTERVAL);
                    if snap.finished || due {
                        sink.progress(ProgressEvent::from_snapshot(snap, duration_s));
                        last_emit = Some(Instant::now());
                    }
                }
            }
            CommandEvent::Stderr(bytes) => {
                for line in String::from_utf8_lossy(&bytes).lines() {
                    push_stderr(line);
                }
            }
            CommandEvent::Error(err) => push_stderr(&err),
            CommandEvent::Terminated(payload) => exit_code = payload.code,
            _ => {}
        }
    }
    control.detach();

    if exit_code == Some(0) {
        Ok(())
    } else {
        Err(AppError::EncodeFailed {
            stderr_tail: stderr_tail.into_iter().collect::<Vec<_>>().join("\n"),
        })
    }
}
