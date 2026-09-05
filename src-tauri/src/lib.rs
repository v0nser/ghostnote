//! GhostNote — private, local-first meeting intelligence.
//!
//! Module map (subsystems land here as the build progresses):
//!
//! - [`stealth`]    — OS-level capture exclusion, the pill shell, notification
//!   silence. *(implemented)*
//! - [`audio`]      — microphone and system-audio capture, resampling, voice
//!   segmentation. *(implemented)*
//! - [`transcribe`] — Whisper.cpp sidecar transcription. *(implemented)*
//! - [`session`]    — wires capture to transcription. *(implemented)*
//! - [`ollama`]  — local LLM interview copilot. *(implemented)*
//! - `memory`  — local vector store for retrieval over past meetings. *(planned)*
//! - `sync`    — Supabase auth and encrypted cloud sync. *(planned)*

pub mod audio;
pub mod ollama;
pub mod session;
pub mod stealth;
pub mod transcribe;

use tauri::{Manager, WindowEvent};

use audio::AudioEngine;
use ollama::Coach;
use stealth::StealthManager;
use transcribe::Transcriber;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Called before the event loop starts, while we are still on the thread
    // that will become the UI thread. The stealth subsystem needs to know it
    // so it can decide whether a native call has to be dispatched.
    stealth::record_main_thread();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(log_plugin())
        .manage(StealthManager::default())
        .manage(AudioEngine::default())
        .manage(Transcriber::default())
        .manage(Coach::default())
        .setup(|app| {
            // Pays the sidecar's one-off startup cost now rather than on the
            // user's first spoken sentence.
            transcribe::warm_up(&app.handle().clone());
            ollama::warm_up(&app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            stealth::commands::stealth_status,
            stealth::commands::set_stealth_enabled,
            stealth::commands::toggle_stealth,
            stealth::commands::verify_capture_exclusion,
            stealth::commands::notifications_suppressed,
            stealth::commands::set_pill_expanded,
            audio::commands::list_input_devices,
            audio::commands::capture_status,
            audio::commands::system_audio_support,
            transcribe::commands::transcription_model_status,
            ollama::coach_status,
            ollama::summarize_meeting,
            session::start_capture,
            session::stop_capture,
            session::session_status,
        ])
        .on_window_event(|window, event| {
            // Moving between monitors, or a display being reconfigured
            // mid-call, can drop the capture-exclusion flag. Re-assert it
            // whenever the window's placement changes, so a user who drags the
            // pill to a second screen does not silently become visible again.
            if !matches!(
                event,
                WindowEvent::Moved(_) | WindowEvent::ScaleFactorChanged { .. }
            ) {
                return;
            }

            let app = window.app_handle();
            if let (Some(manager), Ok(webview)) = (
                app.try_state::<StealthManager>(),
                stealth::main_window(app),
            ) {
                manager.reassert(&webview);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running GhostNote");
}

/// Logging is deliberately conservative: GhostNote handles meeting transcripts
/// and notes, so release builds only ever emit warnings and errors, and no
/// subsystem is permitted to log user content at any level.
fn log_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };

    let mut builder = tauri_plugin_log::Builder::new().level(level);

    // Never write logs to disk in release: a log file is exactly the kind of
    // plaintext artefact this product exists to avoid.
    builder = if cfg!(debug_assertions) {
        builder.target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Stdout,
        ))
    } else {
        builder.target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Stderr,
        ))
    };

    builder.build()
}
