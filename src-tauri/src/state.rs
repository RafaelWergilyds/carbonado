use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::domain::model::{EncoderSupport, VideoInfo};
use crate::error::{AppError, AppResult};
use crate::media::job::JobControl;

#[derive(Default)]
pub struct AppState {
    job: Mutex<Option<Arc<JobControl>>>,
    encoders: Mutex<Option<EncoderSupport>>,
    last_probe: Mutex<Option<VideoInfo>>,
    estimate_gen: AtomicU64,
}

impl AppState {
    pub fn begin_job(&self) -> AppResult<Arc<JobControl>> {
        let mut slot = self.job.lock().unwrap();
        if slot.is_some() {
            return Err(AppError::AlreadyRunning);
        }
        let control = Arc::new(JobControl::default());
        *slot = Some(control.clone());
        Ok(control)
    }

    pub fn end_job(&self) {
        self.job.lock().unwrap().take();
    }

    pub fn current_job(&self) -> Option<Arc<JobControl>> {
        self.job.lock().unwrap().clone()
    }

    pub fn cached_encoders(&self) -> Option<EncoderSupport> {
        *self.encoders.lock().unwrap()
    }

    pub fn cache_encoders(&self, support: EncoderSupport) {
        *self.encoders.lock().unwrap() = Some(support);
    }

    pub fn remember_probe(&self, info: &VideoInfo) {
        *self.last_probe.lock().unwrap() = Some(info.clone());
    }

    pub fn cached_probe(&self, path: &str) -> Option<VideoInfo> {
        self.last_probe
            .lock()
            .unwrap()
            .as_ref()
            .filter(|i| i.path == path)
            .cloned()
    }

    pub fn next_estimate(&self) -> u64 {
        self.estimate_gen.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn is_current_estimate(&self, generation: u64) -> bool {
        self.estimate_gen.load(Ordering::SeqCst) == generation
    }
}
