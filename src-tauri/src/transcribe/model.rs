//! Locating the local Whisper model.
//!
//! The model is not bundled with the app. Prefer `small.en` for accented
//! speech; fall back to `base.en` if that is all that is installed.

use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;

use super::error::{TranscribeError, TranscribeResult};

/// Best default download: much better on accents than `base.en`.
pub const PREFERRED_MODEL_FILE: &str = "ggml-small.en.bin";

/// Last-resort model. Fast, but it hears “play” as “plea” on many accents.
#[allow(dead_code)]
pub const FALLBACK_MODEL_FILE: &str = "ggml-base.en.bin";

/// Search order: accuracy first, then whatever the user already downloaded.
const CANDIDATES: &[&str] = &[
    "ggml-medium.en.bin",
    "ggml-small.en.bin",
    "ggml-small.bin",
    "ggml-base.en.bin",
];

/// Where the user should put a model, and whether one is there.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub installed: bool,
    pub name: String,
    pub directory: String,
}

pub fn model_dir(_app: &AppHandle) -> TranscribeResult<PathBuf> {
    Ok(crate::coda::models_dir())
}

pub fn resolve(app: &AppHandle) -> TranscribeResult<PathBuf> {
    let dir = model_dir(app)?;
    if let Ok(forced) = std::env::var("CODA_WHISPER_MODEL")
        .or_else(|_| std::env::var("GHOSTNOTE_WHISPER_MODEL"))
    {
        if !forced.is_empty() {
            let path = if forced.contains('/') || forced.contains('\\') {
                PathBuf::from(&forced)
            } else {
                dir.join(&forced)
            };
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    for name in CANDIDATES {
        let path = dir.join(name);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(TranscribeError::ModelMissing {
        expected: dir.join(PREFERRED_MODEL_FILE),
    })
}

pub fn status(app: &AppHandle) -> TranscribeResult<ModelStatus> {
    let dir = model_dir(app)?;
    match resolve(app) {
        Ok(path) => Ok(ModelStatus {
            installed: true,
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(PREFERRED_MODEL_FILE)
                .to_string(),
            directory: dir.to_string_lossy().into_owned(),
        }),
        Err(_) => Ok(ModelStatus {
            installed: false,
            name: PREFERRED_MODEL_FILE.to_string(),
            directory: dir.to_string_lossy().into_owned(),
        }),
    }
}

pub fn download_name() -> &'static str {
    PREFERRED_MODEL_FILE
}

#[cfg(test)]
mod tests {
    use super::CANDIDATES;

    #[test]
    fn prefers_small_or_medium_over_base() {
        assert!(CANDIDATES.iter().position(|n| *n == "ggml-small.en.bin")
            < CANDIDATES.iter().position(|n| *n == "ggml-base.en.bin"));
    }
}
