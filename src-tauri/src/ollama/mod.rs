//! Local language-model pipeline.
//!
//! Live interview copilot: when the interviewer stops speaking, a local
//! Ollama model streams one spoken answer. Meeting summaries share the
//! same client.

mod client;
mod coach;
mod error;
mod parse;

use tauri::{AppHandle, Manager};

pub use client::CoachStatus;
pub use coach::{Coach, LiveCoachStatus, MeetingSummary, TalkingPoints, POINTS_EVENT, STATUS_EVENT};

/// Snapshot the UI reads on mount, so it can hide the coach rather than
/// show a dead panel when Ollama is not installed.
#[tauri::command]
pub async fn coach_status(coach: tauri::State<'_, Coach>) -> Result<CoachStatus, error::OllamaError> {
    Ok(coach.status().await)
}

#[tauri::command]
pub async fn summarize_meeting(
    coach: tauri::State<'_, Coach>,
) -> Result<MeetingSummary, error::OllamaError> {
    coach.summarize().await
}

pub fn warm_up(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let Some(coach) = app.try_state::<Coach>() else {
            return;
        };
        let status = coach.status().await;
        if status.available {
            log::debug!("ollama coach is available; loading the model");
            coach.warm_model().await;
        } else {
            log::info!("ollama is not running; talking points will stay idle");
        }
    });
}
