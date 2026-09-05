//! Locating the local Whisper model.
//!
//! The model is not bundled with the app: `ggml-base.en` alone is ~141 MB, and
//! shipping it would triple the installer. It lives in the app's data
//! directory instead, where the user can swap in a larger model without a
//! reinstall.

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use super::error::{TranscribeError, TranscribeResult};

/// Default model: the best accuracy/latency trade-off for meeting speech on a
/// laptop. `small.en` is noticeably better but ~3.5x the size and runtime.
pub const DEFAULT_MODEL_FILE: &str = "ggml-base.en.bin";

/// Where the user should put a model, and whether one is there.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub name: String,
    pub directory: String,
}

pub fn model_dir(app: &AppHandle) -> TranscribeResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("models"))
        .map_err(|err| TranscribeError::DataDir(err.to_string()))
}

pub fn resolve(app: &AppHandle) -> TranscribeResult<PathBuf> {
    let path = model_dir(app)?.join(DEFAULT_MODEL_FILE);

    if !path.is_file() {
        return Err(TranscribeError::ModelMissing { expected: path });
    }

    Ok(path)
}

pub fn status(app: &AppHandle) -> TranscribeResult<ModelStatus> {
    let dir = model_dir(app)?;

    Ok(ModelStatus {
        installed: dir.join(DEFAULT_MODEL_FILE).is_file(),
        name: DEFAULT_MODEL_FILE.to_string(),
        directory: dir.to_string_lossy().into_owned(),
    })
}
