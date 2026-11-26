use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputProblem {
    FileNotFound,
    NotAFile,
    NoVideoStream,
    UnknownDuration,
    OutputIsInput,
    OutputDirMissing,
    QualityOutOfRange,
}

impl InputProblem {
    pub fn code(self) -> &'static str {
        match self {
            InputProblem::FileNotFound => "fileNotFound",
            InputProblem::NotAFile => "notAFile",
            InputProblem::NoVideoStream => "noVideoStream",
            InputProblem::UnknownDuration => "unknownDuration",
            InputProblem::OutputIsInput => "outputIsInput",
            InputProblem::OutputDirMissing => "outputDirMissing",
            InputProblem::QualityOutOfRange => "qualityOutOfRange",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("FFmpeg could not be started")]
    FfmpegNotFound(String),
    #[error("Could not read the video")]
    ProbeFailed(String),
    #[error("Compression failed")]
    EncodeFailed { stderr_tail: String },
    #[error("Invalid input: {}", .0.code())]
    InvalidInput(InputProblem),
    #[error("File error: {0}")]
    Io(#[from] std::io::Error),
    #[error("A compression is already running")]
    AlreadyRunning,
}

impl AppError {
    fn kind(&self) -> &'static str {
        match self {
            AppError::FfmpegNotFound(_) => "ffmpegNotFound",
            AppError::ProbeFailed(_) => "probeFailed",
            AppError::EncodeFailed { .. } => "encodeFailed",
            AppError::InvalidInput(_) => "invalidInput",
            AppError::Io(_) => "io",
            AppError::AlreadyRunning => "alreadyRunning",
        }
    }

    fn code(&self) -> &'static str {
        match self {
            AppError::InvalidInput(problem) => problem.code(),
            other => other.kind(),
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            AppError::FfmpegNotFound(d) | AppError::ProbeFailed(d) => Some(d.clone()),
            AppError::EncodeFailed { stderr_tail } => Some(stderr_tail.clone()),
            AppError::Io(e) => Some(e.to_string()),
            _ => None,
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 4)?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.to_string())?;
        s.serialize_field("detail", &self.detail())?;
        s.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_code_for_ui_translation() {
        let v = serde_json::to_value(AppError::InvalidInput(InputProblem::OutputIsInput)).unwrap();
        assert_eq!(v["kind"], "invalidInput");
        assert_eq!(v["code"], "outputIsInput");
        let v = serde_json::to_value(AppError::AlreadyRunning).unwrap();
        assert_eq!(v["code"], "alreadyRunning");
        assert!(v["detail"].is_null());
    }
}
