//! Meeting session control — the seam where capture meets transcription.
//!
//! Neither the audio module nor the transcription module knows about the
//! other: audio produces [`Utterance`](crate::audio::Utterance)s and
//! transcription consumes them. This module is the only place that connects
//! the two, which is what keeps each side independently testable.

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::audio::{AudioEngine, AudioError, CaptureOptions, CaptureStatus};
use crate::ollama::{Coach, CoachStatus};
use crate::transcribe::{model, TranscribeError, Transcriber};

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error(transparent)]
    Audio(#[from] AudioError),

    #[error(transparent)]
    Transcribe(#[from] TranscribeError),
}

impl Serialize for SessionError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

/// Starts recording and transcribing a meeting.
#[tauri::command]
pub async fn start_capture(
    app: AppHandle,
    audio: State<'_, AudioEngine>,
    transcriber: State<'_, Transcriber>,
    coach: State<'_, Coach>,
    options: Option<CaptureOptions>,
) -> Result<CaptureStatus, SessionError> {
    // Fail before touching the microphone. Starting capture without a model
    // would record a whole meeting and produce nothing.
    model::resolve(&app)?;

    coach.start(&app);
    let sink = transcriber.start(&app);

    match audio.start(&app, options.unwrap_or_default(), sink) {
        Ok(status) => Ok(status),
        Err(err) => {
            // Leaving the worker running after a failed start would strand a
            // task waiting on a queue nothing will ever write to.
            transcriber.stop();
            coach.stop(&app);
            Err(err.into())
        }
    }
}

/// Stops recording. Speech already captured is still transcribed, so the tail
/// of the meeting is not lost.
#[tauri::command]
pub async fn stop_capture(
    app: AppHandle,
    audio: State<'_, AudioEngine>,
    transcriber: State<'_, Transcriber>,
    coach: State<'_, Coach>,
) -> Result<CaptureStatus, SessionError> {
    let status = audio.stop()?;
    transcriber.stop();
    coach.stop(&app);
    Ok(status)
}

/// Combined snapshot for the UI on mount.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub capture: CaptureStatus,
    pub model_installed: bool,
    pub coach: CoachStatus,
}

#[tauri::command]
pub async fn session_status(
    app: AppHandle,
    audio: State<'_, AudioEngine>,
    coach: State<'_, Coach>,
) -> Result<SessionStatus, SessionError> {
    Ok(SessionStatus {
        capture: audio.status(),
        model_installed: model::status(&app)?.installed,
        coach: coach.status().await,
    })
}
