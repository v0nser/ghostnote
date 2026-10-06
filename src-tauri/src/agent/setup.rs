use std::io::Write;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::error::{AgentError, AgentResult};
use super::runtime::AgentRuntime;
use crate::ollama::Coach;
use crate::transcribe::model;

pub const DOWNLOAD_EVENT: &str = "coda://setup-download";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    pub platform: &'static str,
    pub whisper_installed: bool,
    pub whisper_directory: String,
    pub whisper_model: String,
    pub ollama_available: bool,
    pub ollama_model: Option<String>,
    pub first_run: bool,
    pub autostart: bool,
    pub watch_folder: String,
}

#[tauri::command]
pub async fn agent_setup_status(
    app: AppHandle,
    coach: tauri::State<'_, Coach>,
    runtime: tauri::State<'_, AgentRuntime>,
) -> AgentResult<SetupStatus> {
    let model = model::status(&app).map_err(|err| AgentError::msg(err.to_string()))?;
    let coach = coach.status().await;
    Ok(with_prefs(
        &app,
        &runtime,
        SetupStatus {
            platform: std::env::consts::OS,
            whisper_installed: model.installed,
            whisper_directory: model.directory,
            whisper_model: model.name,
            ollama_available: coach.available,
            ollama_model: coach.model,
            first_run: !model.installed || !coach.available,
            autostart: false,
            watch_folder: String::new(),
        },
    ))
}

#[tauri::command]
pub async fn agent_download_whisper_model(app: AppHandle) -> AgentResult<SetupStatus> {
    let dir = model::model_dir(&app).map_err(|err| AgentError::msg(err.to_string()))?;
    std::fs::create_dir_all(&dir).map_err(|err| AgentError::msg(err.to_string()))?;
    let dest = dir.join(model::download_name());
    if dest.is_file() {
        return status_without_coach(&app, true);
    }

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        model::download_name()
    );
    let tmp = dest.with_extension("bin.partial");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .map_err(|err| AgentError::msg(err.to_string()))?;

    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    if !response.status().is_success() {
        return Err(AgentError::msg(format!(
            "could not download the speech model ({})",
            response.status()
        )));
    }

    let total = response.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(&tmp).map_err(|err| AgentError::msg(err.to_string()))?;
    let mut copied = 0u64;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?
    {
        file.write_all(&chunk)
            .map_err(|err| AgentError::msg(err.to_string()))?;
        copied += chunk.len() as u64;
        let _ = app.emit(
            DOWNLOAD_EVENT,
            serde_json::json!({ "received": copied, "total": total }),
        );
    }
    file.flush().map_err(|err| AgentError::msg(err.to_string()))?;
    drop(file);
    std::fs::rename(&tmp, &dest).map_err(|err| AgentError::msg(err.to_string()))?;
    status_without_coach(&app, true)
}

fn status_without_coach(app: &AppHandle, whisper_ok: bool) -> AgentResult<SetupStatus> {
    let model = model::status(app).map_err(|err| AgentError::msg(err.to_string()))?;
    Ok(SetupStatus {
        platform: std::env::consts::OS,
        whisper_installed: whisper_ok || model.installed,
        whisper_directory: model.directory,
        whisper_model: model.name,
        ollama_available: false,
        ollama_model: None,
        first_run: !(whisper_ok || model.installed),
        autostart: false,
        watch_folder: String::new(),
    })
}

fn with_prefs(app: &AppHandle, runtime: &AgentRuntime, mut status: SetupStatus) -> SetupStatus {
    runtime.ensure_loaded(app);
    let config = runtime.config();
    status.autostart = config.autostart;
    status.watch_folder = if config.watch_folder.trim().is_empty() {
        "~/Downloads".into()
    } else {
        config.watch_folder
    };
    status
}

#[tauri::command]
pub async fn agent_set_autostart(
    app: AppHandle,
    runtime: tauri::State<'_, AgentRuntime>,
    enabled: bool,
) -> AgentResult<SetupStatus> {
    runtime.set_autostart_pref(&app, enabled);
    use tauri_plugin_autostart::ManagerExt;
    let launcher = app.autolaunch();
    let result = if enabled {
        launcher.enable()
    } else {
        launcher.disable()
    };
    if let Err(err) = result {
        return Err(AgentError::msg(format!("Could not update login item: {err}")));
    }
    let model = model::status(&app).map_err(|err| AgentError::msg(err.to_string()))?;
    Ok(with_prefs(
        &app,
        &runtime,
        SetupStatus {
            platform: std::env::consts::OS,
            whisper_installed: model.installed,
            whisper_directory: model.directory,
            whisper_model: model.name,
            ollama_available: true,
            ollama_model: None,
            first_run: !model.installed,
            autostart: enabled,
            watch_folder: String::new(),
        },
    ))
}
