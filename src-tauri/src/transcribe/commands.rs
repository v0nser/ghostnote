use tauri::AppHandle;

use super::model::{self, ModelStatus};
use super::TranscribeResult;

/// Whether the local speech model is installed, and where it belongs.
///
/// The UI checks this before offering to record: starting a meeting and only
/// then discovering there is no model would lose the recording.
#[tauri::command]
pub async fn transcription_model_status(app: AppHandle) -> TranscribeResult<ModelStatus> {
    model::status(&app)
}
