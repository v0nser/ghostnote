use serde::Deserialize;
use tauri::{AppHandle, State};

use super::error::AgentResult;
use super::runtime::{extract_action_items, AgentRuntime};
use super::store::AgentSnapshot;
use super::tools::ToolRegistry;
use super::types::{
    AgentMetrics, AgentProfile, AgentTask, Approval, Conversation, MemoryItem, PermissionGrant,
    ScheduledTask, ToolDefinition,
};
use crate::audio::{AudioEngine, CaptureOptions};
use crate::transcribe::Transcriber;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskInput {
    pub goal: String,
    pub agent_id: Option<String>,
    pub conversation_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalInput {
    pub scope: Option<String>,
}

#[tauri::command]
pub async fn agent_overview(app: AppHandle, runtime: State<'_, AgentRuntime>) -> AgentResult<AgentSnapshot> {
    runtime.ensure_loaded(&app);
    runtime.tick_schedules(&app);
    Ok(runtime.snapshot(&app))
}

#[tauri::command]
pub async fn agent_list_profiles(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
) -> AgentResult<Vec<AgentProfile>> {
    Ok(runtime.list_profiles(&app))
}

#[tauri::command]
pub async fn agent_create_conversation(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    agent_id: Option<String>,
) -> AgentResult<Conversation> {
    Ok(runtime.create_conversation(&app, agent_id.unwrap_or_else(|| "personal".into())))
}

#[tauri::command]
pub async fn agent_create_task(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    input: CreateTaskInput,
) -> AgentResult<AgentTask> {
    runtime.start_task(
        &app,
        input.goal,
        input.agent_id.unwrap_or_else(|| "personal".into()),
        input.conversation_id,
    )
}

#[tauri::command]
pub async fn agent_list_tasks(app: AppHandle, runtime: State<'_, AgentRuntime>) -> AgentResult<Vec<AgentTask>> {
    Ok(runtime.list_tasks(&app))
}

#[tauri::command]
pub async fn agent_get_task(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<AgentTask> {
    runtime.get_task(&app, &id)
}

#[tauri::command]
pub async fn agent_pause_task(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<AgentTask> {
    runtime.pause_task(&app, &id)
}

#[tauri::command]
pub async fn agent_resume_task(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<AgentTask> {
    runtime.resume_task(&app, &id)
}

#[tauri::command]
pub async fn agent_cancel_task(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<AgentTask> {
    runtime.cancel_task(&app, &id)
}

#[tauri::command]
pub async fn agent_retry_task(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<AgentTask> {
    runtime.retry_task(&app, &id)
}

#[tauri::command]
pub async fn agent_list_approvals(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
) -> AgentResult<Vec<Approval>> {
    Ok(runtime.list_approvals(&app))
}

#[tauri::command]
pub async fn agent_approve(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
    input: Option<ApprovalInput>,
) -> AgentResult<AgentTask> {
    runtime.approve(&app, &id, input.and_then(|v| v.scope).as_deref().unwrap_or("once"))
}

#[tauri::command]
pub async fn agent_deny(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<AgentTask> {
    runtime.deny(&app, &id)
}

#[tauri::command]
pub async fn agent_list_memories(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
) -> AgentResult<Vec<MemoryItem>> {
    Ok(runtime.list_memories(&app))
}

#[tauri::command]
pub async fn agent_remember(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    content: String,
) -> AgentResult<MemoryItem> {
    Ok(runtime.remember(&app, content).await)
}

#[tauri::command]
pub async fn agent_recall(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    query: String,
    k: Option<usize>,
) -> AgentResult<Vec<super::memory::RecallHit>> {
    Ok(runtime.recall(&app, &query, k.unwrap_or(8)).await)
}

#[tauri::command]
pub async fn agent_forget(app: AppHandle, runtime: State<'_, AgentRuntime>, id: String) -> AgentResult<()> {
    runtime.forget(&app, &id)
}

#[tauri::command]
pub async fn agent_list_tools() -> AgentResult<Vec<ToolDefinition>> {
    Ok(ToolRegistry::definitions())
}

#[tauri::command]
pub async fn agent_set_permission(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    grant: PermissionGrant,
) -> AgentResult<()> {
    runtime.set_permission(&app, grant);
    Ok(())
}

#[tauri::command]
pub async fn agent_list_schedules(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
) -> AgentResult<Vec<ScheduledTask>> {
    Ok(runtime.list_schedules(&app))
}

#[tauri::command]
pub async fn agent_schedule(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    goal: String,
    cadence: Option<String>,
    agent_id: Option<String>,
) -> AgentResult<ScheduledTask> {
    Ok(runtime.schedule(
        &app,
        goal,
        agent_id.unwrap_or_else(|| "personal".into()),
        cadence.unwrap_or_else(|| "daily".into()),
    ))
}

#[tauri::command]
pub async fn agent_delete_schedule(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    id: String,
) -> AgentResult<()> {
    runtime.delete_schedule(&app, &id)
}

#[tauri::command]
pub async fn agent_metrics(runtime: State<'_, AgentRuntime>) -> AgentResult<AgentMetrics> {
    Ok(runtime.metrics())
}

#[tauri::command]
pub async fn agent_action_items(summary: String) -> AgentResult<Vec<String>> {
    Ok(extract_action_items(&summary))
}

#[tauri::command]
pub async fn agent_start_voice(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    audio: State<'_, AudioEngine>,
    transcriber: State<'_, Transcriber>,
) -> AgentResult<crate::audio::CaptureStatus> {
    if audio.status().running {
        let _ = audio.stop();
        transcriber.stop();
        runtime.set_voice(false);
    }
    let model = crate::transcribe::model::status(&app)
        .map_err(|err| super::error::AgentError::msg(err.to_string()))?;
    if !model.installed {
        return Err(super::error::AgentError::ModelMissing);
    }
    runtime.set_voice(true);
    let sink = transcriber.start(&app);
    audio
        .start(
            &app,
            CaptureOptions {
                microphone_device_id: None,
                capture_system_audio: false,
                transcribe_microphone: true,
            },
            sink,
        )
        .map_err(|err| super::error::AgentError::msg(err.to_string()))
}

#[tauri::command]
pub async fn agent_stop_voice(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    audio: State<'_, AudioEngine>,
    transcriber: State<'_, Transcriber>,
    agent_id: Option<String>,
    conversation_id: Option<String>,
) -> AgentResult<Option<AgentTask>> {
    let goal = finish_voice(&runtime, &audio, &transcriber).await;
    if goal.trim().len() < 2 {
        return Ok(None);
    }
    runtime
        .start_task(
            &app,
            goal,
            agent_id.unwrap_or_else(|| "personal".into()),
            conversation_id,
        )
        .map(Some)
}

#[tauri::command]
pub async fn agent_finish_voice(
    runtime: State<'_, AgentRuntime>,
    audio: State<'_, AudioEngine>,
    transcriber: State<'_, Transcriber>,
) -> AgentResult<String> {
    Ok(finish_voice(&runtime, &audio, &transcriber).await)
}

async fn finish_voice(
    runtime: &AgentRuntime,
    audio: &AudioEngine,
    transcriber: &Transcriber,
) -> String {
    let _ = audio.stop();
    transcriber.finish().await;
    let goal = runtime.take_voice_text();
    runtime.set_voice(false);
    goal
}

#[tauri::command]
pub async fn agent_abort_voice(
    runtime: State<'_, AgentRuntime>,
    audio: State<'_, AudioEngine>,
    transcriber: State<'_, Transcriber>,
) -> AgentResult<()> {
    let _ = audio.stop();
    transcriber.stop();
    runtime.set_voice(false);
    let _ = runtime.take_voice_text();
    Ok(())
}

#[tauri::command]
pub async fn agent_ask_screen(
    app: AppHandle,
    runtime: State<'_, AgentRuntime>,
    question: Option<String>,
) -> AgentResult<String> {
    runtime.ask_screen(&app, question).await
}

#[tauri::command]
pub async fn agent_list_audit(
    app: AppHandle,
    query: Option<String>,
    limit: Option<usize>,
) -> AgentResult<Vec<super::store::AuditRow>> {
    super::store::read_audit(&app, query.as_deref(), limit.unwrap_or(400))
}

#[tauri::command]
pub async fn agent_export_audit() -> AgentResult<String> {
    super::store::export_audit_text()
}
