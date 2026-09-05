use std::path::PathBuf;

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum TranscribeError {
    #[error("the speech model is not installed")]
    ModelMissing { expected: PathBuf },

    #[error("could not locate the application data directory: {0}")]
    DataDir(String),

    #[error("the Whisper sidecar could not be started: {0}")]
    SidecarSpawn(String),

    #[error("the Whisper sidecar failed: {0}")]
    SidecarFailed(String),

    #[error("failed to prepare audio for transcription: {0}")]
    Audio(String),
}

impl Serialize for TranscribeError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type TranscribeResult<T> = Result<T, TranscribeError>;
