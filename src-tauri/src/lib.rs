//! Coda — local-first desktop assistant.
//!
//! "Your work, resolved. Your mind, at peace."
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
//! - [`agent`]   — orchestrator, tools, Sentinel, memory. *(implemented)*

pub mod agent;
pub mod audio;
pub mod coda;
pub mod meeting;
pub mod ollama;
pub mod session;
pub mod stealth;
pub mod transcribe;

use tauri::{Manager, WindowEvent};

use agent::AgentRuntime;
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
    agent::crash::install_panic_hook();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(log_plugin())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(StealthManager::default())
        .manage(AudioEngine::default())
        .manage(Transcriber::default())
        .manage(Coach::default())
        .manage(AgentRuntime::default())
        .setup(|app| {
            match crate::coda::migrate_from_ghostnote() {
                Ok(report) if report.copied => {
                    log::info!("copied Ghost Note data into ~/.coda (source left in place)");
                }
                Ok(_) => {}
                Err(err) => log::warn!("Ghost Note data copy skipped: {err}"),
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            // Pays the sidecar's one-off startup cost now rather than on the
            // user's first spoken sentence.
            transcribe::warm_up(&app.handle().clone());
            ollama::warm_up(&app.handle().clone());
            crate::agent::screen::register_hotkey(app);
            crate::agent::watcher::spawn(&app.handle());
            crate::agent::alerts::spawn(&app.handle());
            sync_autostart(&app.handle());
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
            meeting::meeting_snapshot,
            meeting::meeting_decide_draft,
            session::start_capture,
            session::stop_capture,
            session::session_status,
            agent::agent_overview,
            agent::agent_list_profiles,
            agent::agent_create_conversation,
            agent::agent_create_task,
            agent::agent_list_tasks,
            agent::agent_get_task,
            agent::agent_pause_task,
            agent::agent_resume_task,
            agent::agent_cancel_task,
            agent::agent_retry_task,
            agent::agent_list_approvals,
            agent::agent_approve,
            agent::agent_deny,
            agent::agent_list_memories,
            agent::agent_remember,
            agent::agent_recall,
            agent::agent_forget,
            agent::agent_list_tools,
            agent::agent_set_permission,
            agent::agent_list_schedules,
            agent::agent_schedule,
            agent::agent_delete_schedule,
            agent::agent_metrics,
            agent::agent_action_items,
            agent::agent_start_voice,
            agent::agent_stop_voice,
            agent::agent_finish_voice,
            agent::agent_abort_voice,
            agent::agent_ask_screen,
            agent::agent_list_audit,
            agent::agent_export_audit,
            agent::setup::agent_setup_status,
            agent::setup::agent_download_whisper_model,
            agent::setup::agent_set_autostart,
        ])
        .on_window_event(|window, event| {
            match event {
                // The custom chrome X used to destroy the process. Hide instead
                // so a stray click does not look like a crash. Dock click
                // brings the window back (see run() Reopen).
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }
                WindowEvent::Moved(_) | WindowEvent::ScaleFactorChanged { .. } => {}
                _ => return,
            }

            // Moving between monitors, or a display being reconfigured
            // mid-call, can drop the capture-exclusion flag. Re-assert it
            // whenever the window's placement changes, so a user who drags the
            // pill to a second screen does not silently become visible again.
            let app = window.app_handle();
            if let (Some(manager), Ok(webview)) = (
                app.try_state::<StealthManager>(),
                stealth::main_window(app),
            ) {
                manager.reassert(&webview);
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Coda")
        .run(|app, event| {
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}

/// Logging is deliberately conservative: Coda handles meeting transcripts
/// and notes, so release builds only ever emit warnings and errors, and no
/// subsystem is permitted to log user content at any level.
fn log_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };

    let mut builder = tauri_plugin_log::Builder::new()
        .level(level)
        .clear_targets();

    // Never write logs to disk: a log file is exactly the kind of
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

fn sync_autostart(app: &tauri::AppHandle) {
    use tauri_plugin_autostart::ManagerExt;
    let runtime = app.state::<AgentRuntime>();
    runtime.ensure_loaded(app);
    let want = runtime.config().autostart;
    let launcher = app.autolaunch();
    let result = if want {
        launcher.enable()
    } else {
        launcher.disable()
    };
    if let Err(err) = result {
        log::warn!("autostart: {err}");
    }
}
