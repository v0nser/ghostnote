//! Read-only audio queries. Starting and stopping capture lives in
//! [`crate::session`], because it has to coordinate audio with transcription.

use serde::Serialize;
use tauri::State;

use super::{AudioDevice, AudioEngine, AudioResult, CaptureStatus};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemAudioSupport {
    pub supported: bool,
    /// OS permission the user must grant first, if any.
    pub required_permission: Option<&'static str>,
}

/// Microphones the user can choose between.
#[tauri::command]
pub async fn list_input_devices() -> AudioResult<Vec<AudioDevice>> {
    super::input_devices()
}

#[tauri::command]
pub async fn capture_status(engine: State<'_, AudioEngine>) -> AudioResult<CaptureStatus> {
    Ok(engine.status())
}

/// Lets the UI explain up front what participant audio will require, rather
/// than surfacing a permission prompt mid-meeting.
#[tauri::command]
pub async fn system_audio_support() -> SystemAudioSupport {
    let (supported, required_permission) = super::system_audio_support();
    SystemAudioSupport {
        supported,
        required_permission,
    }
}
