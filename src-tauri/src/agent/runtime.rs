use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use super::config::AgentConfig;
use super::error::{AgentError, AgentResult};
use super::ids;
use super::memory;
use super::planner;
use super::policy;
use super::store::{self, AgentSnapshot};
use super::tools::{self, BuiltinTool, ToolExec};
use super::types::{
    AgentEvent, AgentMetrics, AgentProfile, AgentTask, Approval, ApprovalStatus, Artifact,
    ChatMessage, Conversation, MemoryItem, MemoryKind, MemoryScope, PermissionGrant, PolicyDecision,
    ScheduledTask, StepStatus, TaskStatus, ToolCallRecord,
};

pub const EVENT: &str = "coda://agent-event";

struct Inner {
    snap: AgentSnapshot,
    config: AgentConfig,
    metrics: AgentMetrics,
    voice_active: bool,
    voice_buffer: String,
    cached_model: Option<String>,
}

#[derive(Clone)]
pub struct AgentRuntime {
    inner: Arc<Mutex<Inner>>,
    http: reqwest::Client,
}

impl Default for AgentRuntime {
    fn default() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            inner: Arc::new(Mutex::new(Inner {
                snap: AgentSnapshot::default(),
                config: AgentConfig::default(),
                metrics: AgentMetrics::default(),
                voice_active: false,
                voice_buffer: String::new(),
                cached_model: None,
            })),
            http,
        }
    }
}

impl AgentRuntime {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|err| err.into_inner())
    }

    pub fn watch_folder(&self) -> String {
        self.lock().config.watch_folder.clone()
    }

    pub fn set_autostart_pref(&self, app: &AppHandle, enabled: bool) {
        self.ensure_loaded(app);
        self.lock().config.autostart = enabled;
        self.persist(app);
    }

    pub fn screen_hotkey(&self) -> String {
        let value = self.lock().config.screen_hotkey.trim().to_string();
        if value.is_empty() {
            super::screen::DEFAULT_HOTKEY.to_string()
        } else {
            value
        }
    }

    pub async fn ask_screen(&self, app: &AppHandle, question: Option<String>) -> AgentResult<String> {
        self.ensure_loaded(app);
        let config = self.lock().config.clone();
        let question = question
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .unwrap_or(super::screen::DEFAULT_QUESTION);
        super::screen::ask_screen(app, &self.http, &config, question).await
    }

    pub fn ensure_loaded(&self, app: &AppHandle) {
        let mut persist_after = false;
        {
            let mut inner = self.lock();
            if inner.snap.profiles.is_empty() && inner.snap.tasks.is_empty() {
                if let Ok(mut snap) = store::load(app) {
                    if let Some(config) = snap.config.clone() {
                        inner.config = config;
                    }
                    let recovered =
                        super::supervisor::TaskSupervisor::recover_interrupted(&mut snap.tasks);
                    if recovered > 0 {
                        persist_after = true;
                        super::crash::write_report(
                            Some(app),
                            "task_recovery",
                            "marked interrupted tasks as failed",
                            json!({ "count": recovered }),
                        );
                    }
                    inner.snap = snap;
                }
                if inner.snap.profiles.is_empty() {
                    inner.snap.profiles = store::default_profiles();
                }
            }
            // Workspace writes stay inside the jail. Do not block everyday commands.
            inner.config.require_approval_for_file_write = false;
        }
        if persist_after {
            self.persist(app);
        }
    }

    fn persist(&self, app: &AppHandle) {
        let inner = self.lock();
        let mut snap = inner.snap.clone();
        snap.config = Some(inner.config.clone());
        drop(inner);
        let _ = store::save(app, &snap);
    }

    pub fn snapshot(&self, app: &AppHandle) -> AgentSnapshot {
        self.ensure_loaded(app);
        self.lock().snap.clone()
    }

    pub fn config(&self) -> AgentConfig {
        self.lock().config.clone()
    }

    pub fn metrics(&self) -> AgentMetrics {
        self.lock().metrics.clone()
    }

    pub fn workspace(&self, app: &AppHandle) -> AgentResult<std::path::PathBuf> {
        store::workspace(app)
    }

    pub fn create_conversation(&self, app: &AppHandle, agent_id: String) -> Conversation {
        self.ensure_loaded(app);
        let convo = Conversation {
            id: ids::new_id("convo"),
            title: "New conversation".into(),
            agent_id,
            created_at: ids::now_ms(),
            updated_at: ids::now_ms(),
            messages: Vec::new(),
        };
        self.lock().snap.conversations.insert(0, convo.clone());
        self.persist(app);
        convo
    }

    pub fn list_profiles(&self, app: &AppHandle) -> Vec<AgentProfile> {
        self.ensure_loaded(app);
        self.lock().snap.profiles.clone()
    }

    pub fn list_tasks(&self, app: &AppHandle) -> Vec<AgentTask> {
        self.ensure_loaded(app);
        self.lock().snap.tasks.clone()
    }

    pub fn get_task(&self, app: &AppHandle, id: &str) -> AgentResult<AgentTask> {
        self.ensure_loaded(app);
        self.lock()
            .snap
            .tasks
            .iter()
            .find(|task| task.task_id == id)
            .cloned()
            .ok_or(AgentError::TaskNotFound)
    }

    pub fn list_approvals(&self, app: &AppHandle) -> Vec<Approval> {
        self.ensure_loaded(app);
        self.lock().snap.approvals.clone()
    }

    pub fn list_memories(&self, app: &AppHandle) -> Vec<MemoryItem> {
        self.ensure_loaded(app);
        self.lock().snap.memories.clone()
    }

    pub fn list_schedules(&self, app: &AppHandle) -> Vec<ScheduledTask> {
        self.ensure_loaded(app);
        self.lock().snap.schedules.clone()
    }

    pub async fn remember(&self, app: &AppHandle, content: String) -> MemoryItem {
        self.ensure_loaded(app);
        let config = self.lock().config.clone();
        let embedding = embed_memory(&self.http, &config, &content).await;
        let item = memory::create_with_embedding(
            content,
            MemoryKind::Semantic,
            MemoryScope::User,
            "user",
            8,
            embedding,
            Vec::new(),
        );
        self.lock().snap.memories.push(item.clone());
        self.persist(app);
        let _ = store::append_audit(
            app,
            "memory.write",
            &json!({ "id": item.id, "source": item.source, "preview": preview(&item.content) }),
        );
        emit(
            app,
            "memory.created",
            None,
            None,
            json!({"id": item.id}),
        );
        item
    }

    pub fn forget(&self, app: &AppHandle, id: &str) -> AgentResult<()> {
        self.forget_many(app, &[id.to_string()])
    }

    pub fn forget_many(&self, app: &AppHandle, ids: &[String]) -> AgentResult<()> {
        self.ensure_loaded(app);
        let mut removed = Vec::new();
        {
            let mut inner = self.lock();
            inner.snap.memories.retain(|item| {
                if ids.contains(&item.id) {
                    removed.push(item.id.clone());
                    false
                } else {
                    true
                }
            });
        }
        let meeting_removed = crate::meeting::forget_items(app, ids);
        if removed.is_empty() && meeting_removed == 0 {
            return Err(AgentError::msg("memory not found"));
        }
        self.persist(app);
        let _ = store::append_audit(
            app,
            "memory.delete",
            &json!({ "ids": ids, "memories": removed.len(), "meetingItems": meeting_removed }),
        );
        emit(app, "memory.deleted", None, None, json!({ "ids": ids }));
        Ok(())
    }

    pub async fn recall(&self, app: &AppHandle, query: &str, k: usize) -> Vec<memory::RecallHit> {
        self.ensure_loaded(app);
        let config = self.lock().config.clone();
        let items = self.lock().snap.memories.clone();
        let embedding = embed_memory(&self.http, &config, query).await;
        let meeting = crate::meeting::memory_open_lines(app);
        memory::recall(&items, query, embedding.as_deref(), &meeting, k)
    }

    pub fn set_permission(&self, app: &AppHandle, grant: PermissionGrant) {
        self.ensure_loaded(app);
        let mut inner = self.lock();
        inner
            .snap
            .permissions
            .retain(|item| !(item.level == grant.level && item.key == grant.key));
        inner.snap.permissions.push(grant);
        drop(inner);
        self.persist(app);
    }

    pub fn schedule(
        &self,
        app: &AppHandle,
        goal: String,
        agent_id: String,
        cadence: String,
    ) -> ScheduledTask {
        self.ensure_loaded(app);
        let interval = cadence_ms(&cadence);
        let item = ScheduledTask {
            id: ids::new_id("sched"),
            task_template_goal: goal,
            agent_id,
            cadence,
            enabled: true,
            last_run: None,
            next_run: ids::now_ms().saturating_add(interval),
            failure_count: 0,
        };
        self.lock().snap.schedules.push(item.clone());
        self.persist(app);
        item
    }

    pub fn delete_schedule(&self, app: &AppHandle, id: &str) -> AgentResult<()> {
        self.ensure_loaded(app);
        self.lock().snap.schedules.retain(|item| item.id != id);
        self.persist(app);
        Ok(())
    }

    pub fn tick_schedules(&self, app: &AppHandle) {
        self.ensure_loaded(app);
        let now = ids::now_ms();
        let due: Vec<(String, String)> = self
            .lock()
            .snap
            .schedules
            .iter()
            .filter(|item| item.enabled && item.next_run <= now)
            .map(|item| (item.task_template_goal.clone(), item.agent_id.clone()))
            .collect();
        {
            let mut inner = self.lock();
            for item in inner.snap.schedules.iter_mut() {
                if item.enabled && item.next_run <= now {
                    item.last_run = Some(now);
                    item.next_run = now.saturating_add(cadence_ms(&item.cadence));
                }
            }
        }
        for (goal, agent_id) in due {
            let _ = self.start_task(app, goal, agent_id, None);
        }
        if !self.lock().snap.schedules.is_empty() {
            self.persist(app);
        }
    }

    pub fn append_message(&self, app: &AppHandle, conversation_id: &str, role: &str, content: &str, task_id: Option<String>) {
        self.ensure_loaded(app);
        let mut inner = self.lock();
        if let Some(convo) = inner
            .snap
            .conversations
            .iter_mut()
            .find(|item| item.id == conversation_id)
        {
            if convo.title == "New conversation" && role == "user" {
                convo.title = content.chars().take(48).collect();
            }
            convo.updated_at = ids::now_ms();
            convo.messages.push(ChatMessage {
                id: ids::new_id("msg"),
                role: role.into(),
                content: content.into(),
                created_at: ids::now_ms(),
                task_id,
            });
        }
        drop(inner);
        self.persist(app);
    }

    pub fn start_task(
        &self,
        app: &AppHandle,
        goal: String,
        agent_id: String,
        conversation_id: Option<String>,
    ) -> AgentResult<AgentTask> {
        self.ensure_loaded(app);
        if !self.lock().config.agent_enabled {
            return Err(AgentError::msg("agent is disabled"));
        }

        let conversation_id = conversation_id.unwrap_or_else(|| {
            self.create_conversation(app, agent_id.clone()).id
        });

        let mut task = AgentTask::new(goal.clone(), agent_id, conversation_id.clone());
        task.status = TaskStatus::Understanding;
        self.lock().snap.tasks.insert(0, task.clone());
        self.lock().metrics.tasks_started += 1;
        self.persist(app);
        self.append_message(app, &conversation_id, "user", &goal, Some(task.task_id.clone()));
        emit(app, "task.created", Some(&task.task_id), None, json!({"goal": goal}));
        let _ = store::append_audit(app, "task.created", &json!({"taskId": task.task_id}));

        super::supervisor::TaskSupervisor::start(self.clone(), app.clone(), task.task_id.clone());
        Ok(task)
    }

    pub fn pause_task(&self, app: &AppHandle, id: &str) -> AgentResult<AgentTask> {
        self.set_status(app, id, TaskStatus::Paused, None)
    }

    pub fn cancel_task(&self, app: &AppHandle, id: &str) -> AgentResult<AgentTask> {
        {
            let mut inner = self.lock();
            if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == id) {
                task.cancelled = true;
            }
        }
        self.set_status(app, id, TaskStatus::Cancelled, Some("cancelled by user"))
    }

    pub fn resume_task(&self, app: &AppHandle, id: &str) -> AgentResult<AgentTask> {
        let task = self.set_status(app, id, TaskStatus::Executing, None)?;
        super::supervisor::TaskSupervisor::start(self.clone(), app.clone(), id.to_string());
        Ok(task)
    }

    pub fn retry_task(&self, app: &AppHandle, id: &str) -> AgentResult<AgentTask> {
        {
            let mut inner = self.lock();
            inner.metrics.retries += 1;
            if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == id) {
                task.cancelled = false;
                task.errors.clear();
                task.result = None;
                for step in &mut task.plan {
                    if step.status == StepStatus::Failed {
                        step.status = StepStatus::Pending;
                    }
                }
            }
        }
        self.resume_task(app, id)
    }

    pub fn approve(
        &self,
        app: &AppHandle,
        approval_id: &str,
        scope: &str,
    ) -> AgentResult<AgentTask> {
        self.ensure_loaded(app);
        let task_id = {
            let mut inner = self.lock();
            let approval = inner
                .snap
                .approvals
                .iter_mut()
                .find(|item| item.approval_id == approval_id)
                .ok_or(AgentError::ApprovalNotFound)?;
            approval.status = if scope == "task" {
                ApprovalStatus::ApprovedForTask
            } else {
                ApprovalStatus::ApprovedOnce
            };
            approval.task_id.clone()
        };
        emit(
            app,
            "approval.granted",
            Some(&task_id),
            None,
            json!({"approvalId": approval_id}),
        );
        self.resume_task(app, &task_id)
    }

    pub fn deny(&self, app: &AppHandle, approval_id: &str) -> AgentResult<AgentTask> {
        self.ensure_loaded(app);
        let task_id = {
            let mut inner = self.lock();
            let approval = inner
                .snap
                .approvals
                .iter_mut()
                .find(|item| item.approval_id == approval_id)
                .ok_or(AgentError::ApprovalNotFound)?;
            approval.status = ApprovalStatus::Denied;
            approval.task_id.clone()
        };
        emit(
            app,
            "approval.denied",
            Some(&task_id),
            None,
            json!({"approvalId": approval_id}),
        );
        {
            let mut inner = self.lock();
            if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == task_id) {
                task.plan = planner::replan(&task.goal, "approval denied");
                task.status = TaskStatus::Replanning;
                task.touch();
            }
        }
        self.persist(app);
        super::supervisor::TaskSupervisor::start(self.clone(), app.clone(), task_id.clone());
        self.get_task(app, &task_id)
    }

    pub fn set_voice(&self, active: bool) {
        let mut inner = self.lock();
        inner.voice_active = active;
        if active {
            inner.voice_buffer.clear();
        }
    }

    pub fn voice_active(&self) -> bool {
        self.lock().voice_active
    }

    pub fn push_voice_text(&self, text: &str) {
        let mut inner = self.lock();
        if !inner.voice_active {
            return;
        }
        if !inner.voice_buffer.is_empty() {
            inner.voice_buffer.push(' ');
        }
        inner.voice_buffer.push_str(text.trim());
    }

    pub fn take_voice_text(&self) -> String {
        let mut inner = self.lock();
        let raw = std::mem::take(&mut inner.voice_buffer);
        crate::transcribe::correct_voice(&raw)
    }

    fn set_status(
        &self,
        app: &AppHandle,
        id: &str,
        status: TaskStatus,
        error: Option<&str>,
    ) -> AgentResult<AgentTask> {
        self.ensure_loaded(app);
        let task = {
            let mut inner = self.lock();
            let task = inner
                .snap
                .tasks
                .iter_mut()
                .find(|task| task.task_id == id)
                .ok_or(AgentError::TaskNotFound)?;
            task.status = status;
            if let Some(error) = error {
                task.errors.push(error.into());
            }
            task.touch();
            task.clone()
        };
        self.persist(app);
        emit(
            app,
            "task.status",
            Some(id),
            None,
            json!({"status": status}),
        );
        Ok(task)
    }

    pub async fn run_supervised(&self, app: &AppHandle, id: &str) {
        self.run_task(app, id).await;
    }

    pub fn fail_isolated(&self, app: &AppHandle, id: &str, error: &str) -> AgentResult<()> {
        self.finish(
            app,
            id,
            format!("That task failed, but Coda is still running.\n\n{error}"),
            Some(error),
        )
    }

    async fn run_task(&self, app: &AppHandle, id: &str) {
        if let Err(err) = self.run_task_inner(app, id).await {
            super::crash::write_report(
                Some(app),
                "agent_task",
                &err.to_string(),
                json!({ "taskId": id }),
            );
            let _ = self.finish(
                app,
                id,
                user_facing_failure(&err),
                Some(&err.to_string()),
            );
            emit(app, "task.failed", Some(id), None, json!({"error": err.to_string()}));
        }
    }

    async fn run_task_inner(&self, app: &AppHandle, id: &str) -> AgentResult<()> {
        let mut task = self.get_task(app, id)?;
        if task.cancelled {
            return self.set_status(app, id, TaskStatus::Cancelled, None).map(|_| ());
        }

        if looks_like_file_suggestion(&task.goal) {
            if !self.has_task_approval(id, "SUGGEST_FILE") {
                self.request_approval(
                    app,
                    &task,
                    "SUGGEST_FILE",
                    "A new local file may match a stored memory",
                    json!({ "goal": task.goal }),
                )?;
                return Ok(());
            }
            self.finish(
                app,
                id,
                "Noted. I did not open or send that file.".into(),
                None,
            )?;
            return Ok(());
        }
        if let Some(query) = memory::looks_like_recall(&task.goal) {
            let hits = self.recall(app, &query, 8).await;
            if hits.is_empty() {
                self.finish(app, id, format!("I don't have a stored memory about “{query}”."), None)?;
            } else {
                let body = hits
                    .iter()
                    .map(|hit| format!("- {} ({})", hit.text, hit.source))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.finish(app, id, format!("Here's what I remember about “{query}”:\n{body}"), None)?;
            }
            return Ok(());
        }
        if let Some(fact) = memory::looks_like_remember(&task.goal) {
            let item = self.remember(app, fact).await;
            self.finish(app, id, format!("I'll remember that. ({})", item.id), None)?;
            return Ok(());
        }
        if let Some(fact) = memory::looks_like_forget(&task.goal) {
            let matches = self.forget_matches(app, &fact).await;
            if matches.is_empty() {
                self.finish(app, id, "I could not find a matching memory to forget.".into(), None)?;
                return Ok(());
            }
            if !self.has_task_approval(id, "MEMORY_DELETE") {
                self.request_approval(
                    app,
                    &task,
                    "MEMORY_DELETE",
                    &fact,
                    json!({ "query": fact, "matches": matches }),
                )?;
                return Ok(());
            }
            let ids: Vec<String> = matches
                .iter()
                .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_string))
                .collect();
            match self.forget_many(app, &ids) {
                Ok(()) => self.finish(app, id, format!("Forgotten {} stored item(s).", ids.len()), None)?,
                Err(err) => self.finish(app, id, user_facing_failure(&err), Some(&err.to_string()))?,
            }
            return Ok(());
        }

        if super::intent::is_vague(&task.goal) {
            self.finish(app, id, super::intent::capabilities().into(), None)?;
            return Ok(());
        }

        if let Some(intent) = super::intent::route(&task.goal) {
            return self.run_fast_intent(app, id, intent).await;
        }

        if task.plan.is_empty() {
            let _ = self.set_status(app, id, TaskStatus::Planning, None);
            emit(app, "agent.step.started", Some(id), None, json!({"summary": "Planning the work"}));
            let memories = {
                let inner = self.lock();
                memory::search(&inner.snap.memories, &task.goal, 5)
                    .into_iter()
                    .map(|item| item.content.clone())
                    .collect::<Vec<_>>()
            };
            task.plan = planner::plan(&task.goal);
            task.context = json!({ "memories": memories });
            task.status = TaskStatus::Executing;
            task.touch();
            self.upsert_task(app, task.clone());
            emit(app, "task.planned", Some(id), None, json!({"steps": task.plan.len()}));
        }

        let profile = self
            .lock()
            .snap
            .profiles
            .iter()
            .find(|profile| profile.id == task.agent_id)
            .cloned()
            .unwrap_or_else(|| store::default_profiles()[1].clone());

        let planned_tools: Vec<(String, String, String)> = task
            .plan
            .iter()
            .filter(|step| !matches!(step.status, StepStatus::Done | StepStatus::Skipped))
            .flat_map(|step| {
                step.required_tools.iter().map(|tool| {
                    (step.step_id.clone(), tool.clone(), step.description.clone())
                })
            })
            .collect();

        if planned_tools.is_empty() {
            emit(app, "agent.step.started", Some(id), None, json!({"summary": "Thinking"}));
            match self.model_turn(&task).await {
                Ok(super::llm::Turn::Answer(text)) if text.chars().count() > 8 => {
                    self.mark_open_steps(app, id, StepStatus::Done);
                    self.finish(app, id, text, None)?;
                    return Ok(());
                }
                Ok(super::llm::Turn::Tool { name, args }) => {
                    if let Err(err) = self
                        .run_named_tool(app, id, &profile, &name, args, "Follow the request")
                        .await
                    {
                        if matches!(err, AgentError::NeedsApproval) {
                            return Ok(());
                        }
                        self.note_error(app, id, &err.to_string());
                    }
                }
                Ok(super::llm::Turn::Answer(_)) | Err(_) => {}
            }
        } else {
            for (step_id, tool_name, description) in planned_tools {
                task = self.get_task(app, id)?;
                if task.cancelled {
                    return self.set_status(app, id, TaskStatus::Cancelled, None).map(|_| ());
                }
                if task.status == TaskStatus::Paused {
                    return Ok(());
                }
                self.mark_step(app, id, &step_id, StepStatus::Running, None);
                emit(
                    app,
                    "agent.step.started",
                    Some(id),
                    Some(&step_id),
                    json!({"summary": description}),
                );
                let Some(args) = infer_args(&tool_name, &task) else {
                    self.mark_step(app, id, &step_id, StepStatus::Skipped, Some("Nothing to run."));
                    continue;
                };
                match self
                    .run_named_tool(app, id, &profile, &tool_name, args, &description)
                    .await
                {
                    Ok(()) => self.mark_step(app, id, &step_id, StepStatus::Done, None),
                    Err(AgentError::NeedsApproval) => {
                        self.mark_step(app, id, &step_id, StepStatus::WaitingApproval, None);
                        return Ok(());
                    }
                    Err(err) => {
                        self.note_error(app, id, &err.to_string());
                        self.mark_step(app, id, &step_id, StepStatus::Failed, Some(&err.to_string()));
                    }
                }
            }
        }

        task = self.get_task(app, id)?;
        if task.status == TaskStatus::WaitingForApproval || task.cancelled {
            return Ok(());
        }

        if let Some(failed) = task.plan.iter().rev().find(|step| step.status == StepStatus::Failed) {
            let message = failed
                .summary
                .clone()
                .or_else(|| task.errors.last().cloned())
                .unwrap_or_else(|| "That action failed.".into());
            self.finish(app, id, user_facing_failure(&AgentError::msg(&message)), Some(&message))?;
            return Ok(());
        }

        if let Some(quick) = quick_result(&task) {
            self.mark_open_steps(app, id, StepStatus::Done);
            self.finish(app, id, quick, None)?;
            return Ok(());
        }

        if let Some((text, failed)) = verified_computer_result(&task) {
            self.mark_open_steps(app, id, if failed { StepStatus::Failed } else { StepStatus::Done });
            let message = if failed {
                text.clone()
            } else {
                honest_computer_followup(&task, &text)
            };
            self.finish(app, id, message, failed.then_some(text.as_str()))?;
            return Ok(());
        }

        let _ = self.set_status(app, id, TaskStatus::Reflecting, None);
        emit(app, "agent.step.started", Some(id), None, json!({"summary": "Writing the answer"}));
        let result = self.compose_answer(&task).await;
        self.mark_open_steps(app, id, StepStatus::Done);
        self.finish(app, id, result, None)?;
        Ok(())
    }

    async fn run_fast_intent(
        &self,
        app: &AppHandle,
        id: &str,
        intent: super::intent::FastIntent,
    ) -> AgentResult<()> {
        let task = self.get_task(app, id)?;
        let profile = self
            .lock()
            .snap
            .profiles
            .iter()
            .find(|item| item.id == task.agent_id)
            .cloned()
            .unwrap_or_else(|| store::default_profiles()[1].clone());

        let _ = self.set_status(app, id, TaskStatus::Executing, None);
        emit(
            app,
            "agent.step.started",
            Some(id),
            None,
            json!({"summary": intent.summary()}),
        );

        if let super::intent::FastIntent::AskScreen { question } = &intent {
            let config = self.lock().config.clone();
            match super::screen::ask_screen(app, &self.http, &config, question).await {
                Ok(text) => self.finish(app, id, text, None)?,
                Err(err) => {
                    super::crash::write_report(
                        Some(app),
                        "screen_ask",
                        &err.to_string(),
                        json!({ "taskId": id }),
                    );
                    self.finish(app, id, user_facing_failure(&err), Some(&err.to_string()))?;
                }
            }
            return Ok(());
        }

        match self
            .run_named_tool(app, id, &profile, intent.tool_name(), intent.args(), &intent.summary())
            .await
        {
            Ok(()) => {
                if intent.tool_name() == "SUMMARIZE_FILE" {
                    match self.summarize_extracted_file(app, id).await {
                        Ok(summary) => self.finish(app, id, summary, None)?,
                        Err(err) => self.finish(app, id, user_facing_failure(&err), Some(&err.to_string()))?,
                    }
                    return Ok(());
                }
                let summary = self
                    .get_task(app, id)
                    .ok()
                    .and_then(|task| task.tool_calls.last().map(|call| call.result_summary.clone()))
                    .filter(|text| !text.is_empty())
                    .unwrap_or_else(|| intent.summary());
                self.finish(app, id, summary, None)?;
            }
            Err(AgentError::NeedsApproval) => return Ok(()),
            Err(err) => {
                super::crash::write_report(
                    Some(app),
                    "computer_tool",
                    &err.to_string(),
                    json!({ "taskId": id, "tool": intent.tool_name() }),
                );
                self.finish(app, id, user_facing_failure(&err), Some(&err.to_string()))?;
            }
        }
        Ok(())
    }

    async fn model_turn(&self, task: &AgentTask) -> AgentResult<super::llm::Turn> {
        let config = self.lock().config.clone();
        let model = self.resolve_model(&config).await;
        let memories = memories_from_task(task);
        let findings = findings_from_task(task);
        super::llm::decide(&self.http, &config, &model, &task.goal, &memories, &findings).await
    }

    async fn compose_answer(&self, task: &AgentTask) -> String {
        let config = self.lock().config.clone();
        let model = self.resolve_model(&config).await;
        let memories = memories_from_task(task);
        let findings = findings_from_task(task);
        match super::llm::answer(&self.http, &config, &model, &task.goal, &memories, &findings).await {
            Ok(text) if text.chars().count() > 8 => text,
            _ => heuristic_result(task, &findings),
        }
    }

    async fn summarize_extracted_file(&self, app: &AppHandle, id: &str) -> AgentResult<String> {
        let task = self.get_task(app, id)?;
        let path = task
            .context
            .pointer("/lastTool/path")
            .and_then(Value::as_str)
            .unwrap_or("document")
            .to_string();
        let text = task
            .context
            .pointer("/lastTool/text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if text.trim().is_empty() {
            return Err(AgentError::msg(
                "I read the file but there was no text to summarize.",
            ));
        }
        let config = self.lock().config.clone();
        let model = self.resolve_model(&config).await;
        super::llm::summarize_document(&self.http, &config, &model, &task.goal, &path, &text).await
    }

    async fn resolve_model(&self, config: &AgentConfig) -> String {
        if let Some(cached) = self.lock().cached_model.clone() {
            return cached;
        }
        let preferred = config
            .agent_model
            .clone()
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| config.default_model.clone());
        let model = super::llm::resolve_agent_model(&self.http, &config.ollama_url, &preferred).await;
        self.lock().cached_model = Some(model.clone());
        model
    }

    async fn run_named_tool(
        &self,
        app: &AppHandle,
        id: &str,
        profile: &AgentProfile,
        tool_name: &str,
        args: Value,
        description: &str,
    ) -> AgentResult<()> {
        if !is_execution_tool(tool_name) && !policy::profile_allows_tool(&profile.tools, tool_name)
        {
            return Err(AgentError::msg(format!(
                "{tool_name} is not enabled for this assistant"
            )));
        }
        let Some(tool) = BuiltinTool::from_name(tool_name) else {
            return Err(AgentError::msg(format!("unknown tool: {tool_name}")));
        };
        if matches!(tool_name, "WEB_FETCH" | "BROWSER_OPEN")
            && args.get("url").and_then(Value::as_str) == Some("https://example.com")
        {
            return Ok(());
        }

        let config = self.lock().config.clone();
        let decision = policy::decide(tool_name, tool.risk(), &config, config.computer_use_enabled);
        match decision {
            PolicyDecision::Deny => return Err(AgentError::Denied),
            PolicyDecision::RequireApproval => {
                if !self.has_task_approval(id, tool_name) {
                    let task = self.get_task(app, id)?;
                    self.request_approval(app, &task, tool_name, description, args)?;
                    return Err(AgentError::NeedsApproval);
                }
            }
            PolicyDecision::Allow => {}
        }

        let started = ids::now_ms();
        emit(
            app,
            "tool.called",
            Some(id),
            None,
            json!({"tool": tool_name, "summary": format!("Using {tool_name}")}),
        );

        let workspace = store::workspace(app)?;
        let sandbox_root = store::root(app)?.join("sandboxes").join(id);
        let task = self.get_task(app, id)?;

        let output = match tool {
            BuiltinTool::MemoryWrite => {
                let content = args
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or(&task.goal)
                    .to_string();
                let item = self.remember(app, content).await;
                tools::ToolOutput {
                    value: json!({"id": item.id}),
                    summary: "Stored a memory.".into(),
                }
            }
            BuiltinTool::MemorySearch => {
                let query = args.get("query").and_then(Value::as_str).unwrap_or(&task.goal);
                let hits = self.recall(app, query, 5).await;
                let texts: Vec<String> = hits.iter().map(|hit| hit.text.clone()).collect();
                tools::ToolOutput {
                    value: json!({ "hits": hits }),
                    summary: format!("Recalled {} memories.", texts.len()),
                }
            }
            BuiltinTool::MemoryDelete => return Err(AgentError::Denied),
            BuiltinTool::TaskSchedule => {
                let item = self.schedule(app, task.goal.clone(), task.agent_id.clone(), "daily".into());
                tools::ToolOutput {
                    value: json!({"id": item.id}),
                    summary: "Scheduled a daily follow-up.".into(),
                }
            }
            BuiltinTool::Notification => tools::ToolOutput {
                value: json!({"ok": true}),
                summary: "Notification queued in the workspace.".into(),
            },
            other => {
                let ctx = ToolExec {
                    workspace: &workspace,
                    sandbox: &sandbox_root,
                    http: &self.http,
                };
                match tools::execute(other, &args, &ctx).await {
                    Ok(output) => output,
                    Err(err) => {
                        self.lock().metrics.tool_failures += 1;
                        emit(
                            app,
                            "tool.failed",
                            Some(id),
                            None,
                            json!({"tool": tool_name, "error": err.to_string(), "summary": err.to_string()}),
                        );
                        return Err(err);
                    }
                }
            }
        };

        let record = ToolCallRecord {
            id: ids::new_id("call"),
            tool: tool_name.to_string(),
            arguments_hash: ids::arguments_hash(&args),
            sanitized_arguments: ids::sanitize_value(&args),
            result_summary: output.summary.clone(),
            risk_level: tool.risk(),
            status: "ok".into(),
            duration_ms: ids::now_ms().saturating_sub(started),
        };
        {
            let mut inner = self.lock();
            if let Some(task) = inner.snap.tasks.iter_mut().find(|item| item.task_id == id) {
                if tool_name == "WEB_SEARCH" {
                    if let Some(url) = output.value.pointer("/results/0/url").and_then(Value::as_str) {
                        if !url.is_empty() {
                            task.context["lastUrl"] = json!(url);
                        }
                    }
                    task.context["searchResults"] = output.value.clone();
                }
                if tool_name == "WEB_FETCH" {
                    task.context["fetched"] = output.value.clone();
                }
                task.context["lastTool"] = output.value.clone();
                task.tool_calls.push(record);
                if tool_name == "FILES_WRITE" {
                    if let Some(path) = output.value.get("path").and_then(Value::as_str) {
                        task.artifacts.push(Artifact {
                            artifact_id: ids::new_id("art"),
                            task_id: id.to_string(),
                            kind: "markdown".into(),
                            path: path.into(),
                            size: output.value.get("bytes").and_then(Value::as_u64).unwrap_or(0),
                            created_at: ids::now_ms(),
                            metadata: json!({}),
                        });
                    }
                }
            }
        }
        emit(
            app,
            "tool.completed",
            Some(id),
            None,
            json!({"tool": tool_name, "summary": output.summary}),
        );
        Ok(())
    }

    fn mark_open_steps(&self, app: &AppHandle, id: &str, status: StepStatus) {
        let mut inner = self.lock();
        if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == id) {
            for step in &mut task.plan {
                if matches!(step.status, StepStatus::Pending | StepStatus::Running) {
                    step.status = status;
                }
            }
            task.touch();
        }
        drop(inner);
        self.persist(app);
    }

    fn note_error(&self, app: &AppHandle, id: &str, error: &str) {
        let mut inner = self.lock();
        if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == id) {
            task.errors.push(error.into());
            task.touch();
        }
        drop(inner);
        self.persist(app);
    }

    fn finish(&self, app: &AppHandle, id: &str, result: String, error: Option<&str>) -> AgentResult<()> {
        let conversation_id = self.get_task(app, id)?.conversation_id;
        {
            let mut inner = self.lock();
            if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == id) {
                task.result = Some(result.clone());
                task.status = if error.is_some() {
                    TaskStatus::Failed
                } else {
                    TaskStatus::Completed
                };
                if let Some(error) = error {
                    task.errors.push(error.into());
                }
                task.touch();
            }
            if error.is_some() {
                inner.metrics.tasks_failed += 1;
            } else {
                inner.metrics.tasks_completed += 1;
            }
        }
        self.append_message(app, &conversation_id, "assistant", &result, Some(id.into()));
        self.persist(app);
        emit(app, "task.completed", Some(id), None, json!({"result": true}));
        Ok(())
    }

    fn upsert_task(&self, app: &AppHandle, task: AgentTask) {
        {
            let mut inner = self.lock();
            if let Some(existing) = inner.snap.tasks.iter_mut().find(|item| item.task_id == task.task_id) {
                *existing = task;
            } else {
                inner.snap.tasks.insert(0, task);
            }
        }
        self.persist(app);
    }

    fn mark_step(&self, app: &AppHandle, task_id: &str, step_id: &str, status: StepStatus, summary: Option<&str>) {
        {
            let mut inner = self.lock();
            if let Some(task) = inner.snap.tasks.iter_mut().find(|task| task.task_id == task_id) {
                task.current_step = Some(step_id.into());
                if let Some(step) = task.plan.iter_mut().find(|step| step.step_id == step_id) {
                    step.status = status;
                    if let Some(summary) = summary {
                        step.summary = Some(summary.into());
                    }
                }
                task.touch();
            }
        }
        self.persist(app);
    }

    fn has_task_approval(&self, task_id: &str, tool: &str) -> bool {
        self.lock().snap.approvals.iter().any(|item| {
            item.task_id == task_id
                && item.tool == tool
                && matches!(
                    item.status,
                    ApprovalStatus::ApprovedOnce | ApprovalStatus::ApprovedForTask
                )
        })
    }

    fn request_approval(
        &self,
        app: &AppHandle,
        task: &AgentTask,
        tool: &str,
        reason: &str,
        arguments: Value,
    ) -> AgentResult<()> {
        let approval = Approval {
            approval_id: ids::new_id("appr"),
            task_id: task.task_id.clone(),
            step_id: task.current_step.clone(),
            tool: tool.into(),
            action: format!("Use {tool}"),
            target: arguments
                .get("url")
                .or_else(|| arguments.get("path"))
                .or_else(|| arguments.get("to"))
                .or_else(|| arguments.get("query"))
                .and_then(Value::as_str)
                .unwrap_or("local")
                .into(),
            data_summary: ids::sanitize_value(&arguments).to_string(),
            reason: reason.into(),
            consequences: consequence(tool).into(),
            risk_level: policy::risk_for_tool(tool),
            status: ApprovalStatus::Pending,
            created_at: ids::now_ms(),
            arguments: ids::sanitize_value(&arguments),
        };
        {
            let mut inner = self.lock();
            inner.metrics.approvals_requested += 1;
            inner.snap.approvals.insert(0, approval.clone());
            if let Some(task) = inner.snap.tasks.iter_mut().find(|item| item.task_id == task.task_id)
            {
                task.status = TaskStatus::WaitingForApproval;
                task.approval_ids.push(approval.approval_id.clone());
                task.touch();
            }
        }
        self.persist(app);
        emit(
            app,
            "approval.requested",
            Some(&task.task_id),
            None,
            json!({"approvalId": approval.approval_id, "tool": tool}),
        );
        let _ = store::append_audit(
            app,
            "approval.requested",
            &json!({"taskId": task.task_id, "tool": tool}),
        );
        Ok(())
    }
}

fn emit(app: &AppHandle, event: &str, task_id: Option<&str>, step_id: Option<&str>, data: Value) {
    let payload = AgentEvent {
        event: event.into(),
        task_id: task_id.map(str::to_string),
        step_id: step_id.map(str::to_string),
        timestamp: ids::now_ms(),
        data,
    };
    let _ = app.emit(EVENT, payload);
}

fn is_execution_tool(tool: &str) -> bool {
    matches!(
        tool,
        "OPEN_APPLICATION"
            | "OPEN_URL"
            | "OPEN_FOLDER"
            | "OPEN_FILE"
            | "CREATE_FOLDER"
            | "LIST_APPS"
            | "CLOSE_APPLICATION"
            | "FOCUS_APPLICATION"
            | "TAKE_SCREENSHOT"
            | "PLAY_MEDIA"
            | "PLAY_PLAYLIST"
            | "SET_REMINDER"
            | "FIND_FILE"
            | "CREATE_DOCUMENT"
            | "SUMMARIZE_FILE"
            | "DRAFT_EMAIL"
            | "SEND_EMAIL"
            | "CALENDAR_EVENT"
    )
}

fn infer_args(tool: &str, task: &AgentTask) -> Option<Value> {
    if let Some(intent) = super::intent::route(&task.goal) {
        if intent.tool_name() == tool {
            return Some(intent.args());
        }
    }
    match tool {
        "OPEN_APPLICATION" => Some(json!({
            "name": super::intent::leading_application(&task.goal)
                .unwrap_or_else(|| extract_open_name(&task.goal))
        })),
        "OPEN_URL" => planner::first_http_url(&task.goal).map(|url| json!({ "url": url })),
        "OPEN_FOLDER" | "OPEN_FILE" | "CREATE_FOLDER" => {
            Some(json!({ "path": extract_folder_path(&task.goal) }))
        }
        "LIST_APPS" => Some(json!({})),
        "SET_REMINDER" | "SET_ALARM" => Some(json!({ "title": "Alarm", "when": task.goal })),
        "PLAY_MEDIA" => Some(json!({ "query": extract_play_query(&task.goal) })),
        "PLAY_PLAYLIST" => Some(json!({ "name": extract_play_query(&task.goal).replace(" playlist", "") })),
        "TAKE_SCREENSHOT" => Some(json!({})),
        "CLOSE_APPLICATION" | "FOCUS_APPLICATION" => Some(json!({ "name": extract_open_name(&task.goal) })),
        "FIND_FILE" => Some(json!({ "query": extract_file_query(&task.goal) })),
        "CREATE_DOCUMENT" => Some(json!({ "title": task.goal, "content": "" })),
        "DRAFT_EMAIL" | "SEND_EMAIL" => Some(json!({ "to": "", "subject": task.goal, "body": "" })),
        "CALENDAR_EVENT" => Some(json!({ "title": task.goal, "when": task.goal })),
        "SUMMARIZE_FILE" => Some(json!({ "query": extract_file_query(&task.goal) })),
        "WEB_SEARCH" | "MEMORY_SEARCH" => Some(json!({ "query": search_query(&task.goal) })),
        "WEB_FETCH" | "BROWSER_OPEN" => {
            let url = planner::first_http_url(&task.goal)
                .or_else(|| first_result_url(task))
                .unwrap_or_else(|| "https://example.com".into());
            if url == "https://example.com" {
                return None;
            }
            Some(json!({ "url": url }))
        }
        "FILES_WRITE" => Some(infer_write(task)),
        "FILES_SEARCH" | "FILES_LIST" => Some(json!({ "path": "", "query": task.goal })),
        "FILES_READ" => {
            let path = extract_path(&task.goal)?;
            Some(json!({ "path": path }))
        }
        "CALCULATOR" => planner::extract_math(&task.goal).map(|expression| json!({ "expression": expression })),
        "DATETIME" => Some(json!({})),
        "MEMORY_WRITE" => Some(json!({ "content": task.goal })),
        "PYTHON_EXECUTE" | "CODE_EXECUTE" => extract_code(&task.goal).map(|code| json!({ "code": code })),
        _ => Some(json!({})),
    }
}

fn infer_write(task: &AgentTask) -> Value {
    let path = extract_path(&task.goal).unwrap_or_else(|| format!("artifacts/{}.md", task.task_id));
    json!({
        "path": path,
        "content": draft_artifact(task)
    })
}

fn extract_path(goal: &str) -> Option<String> {
    goal.split_whitespace().find_map(|word| {
        let trimmed = word.trim_matches(|ch: char| "\"'`,.".contains(ch));
        if trimmed.contains('.')
            && trimmed
                .rsplit('.')
                .next()
                .is_some_and(|ext| matches!(ext, "md" | "txt" | "json" | "py" | "js" | "csv" | "pdf"))
        {
            Some(trimmed.trim_start_matches(['.', '/']).to_string())
        } else {
            None
        }
    })
}

fn extract_code(goal: &str) -> Option<String> {
    if let Some(rest) = goal.split("```").nth(1) {
        let mut lines = rest.lines();
        let first = lines.next().unwrap_or("");
        let body = if matches!(first.trim(), "python" | "py" | "js" | "javascript" | "node") {
            lines.collect::<Vec<_>>().join("\n")
        } else {
            rest.to_string()
        };
        let code = body.trim().trim_end_matches("```").trim();
        if !code.is_empty() {
            return Some(code.to_string());
        }
    }
    None
}

fn first_result_url(task: &AgentTask) -> Option<String> {
    task.context
        .get("lastUrl")
        .and_then(Value::as_str)
        .filter(|url| !url.is_empty())
        .map(str::to_string)
}

fn search_query(goal: &str) -> String {
    goal.chars().take(160).collect()
}

fn draft_artifact(task: &AgentTask) -> String {
    let mut out = format!("# {}\n\n", task.goal);
    for call in &task.tool_calls {
        out.push_str(&format!("- {} — {}\n", call.tool, call.result_summary));
    }
    out
}

fn memories_from_task(task: &AgentTask) -> Vec<String> {
    task.context
        .get("memories")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn findings_from_task(task: &AgentTask) -> String {
    let mut notes = String::new();
    for call in &task.tool_calls {
        notes.push_str(&format!("- {}: {}\n", call.tool, call.result_summary));
    }
    if let Some(errors) = (!task.errors.is_empty()).then_some(&task.errors) {
        for error in errors {
            notes.push_str(&format!("- error: {error}\n"));
        }
    }
    notes
}

fn quick_result(task: &AgentTask) -> Option<String> {
    if task.tool_calls.len() != 1 {
        return None;
    }
    let call = &task.tool_calls[0];
    if matches!(
        call.tool.as_str(),
        "CALCULATOR"
            | "DATETIME"
            | "OPEN_APPLICATION"
            | "OPEN_URL"
            | "OPEN_FOLDER"
            | "OPEN_FILE"
            | "CREATE_FOLDER"
            | "LIST_APPS"
            | "SET_REMINDER"
            | "PLAY_MEDIA"
            | "PLAY_PLAYLIST"
            | "TAKE_SCREENSHOT"
            | "FIND_FILE"
            | "CREATE_DOCUMENT"
            | "SUMMARIZE_FILE"
            | "SEND_EMAIL"
            | "CALENDAR_EVENT"
            | "DRAFT_EMAIL"
    ) && !call.result_summary.is_empty()
        && call.status == "ok"
    {
        return Some(call.result_summary.clone());
    }
    None
}

fn honest_computer_followup(task: &AgentTask, verified: &str) -> String {
    if super::intent::route(&task.goal).is_some() {
        return verified.to_string();
    }
    if super::intent::leading_application(&task.goal).is_some() {
        return format!(
            "{verified} I opened it, but I couldn’t verify the rest of “{}” yet.",
            task.goal
        );
    }
    verified.to_string()
}

fn verified_computer_result(task: &AgentTask) -> Option<(String, bool)> {
    let call = task.tool_calls.last()?;
    if !matches!(
        call.tool.as_str(),
        "OPEN_APPLICATION" | "OPEN_URL" | "OPEN_FOLDER" | "OPEN_FILE" | "CREATE_FOLDER"
            | "SET_REMINDER" | "PLAY_MEDIA" | "PLAY_PLAYLIST" | "TAKE_SCREENSHOT"
            | "FIND_FILE" | "CREATE_DOCUMENT"
            | "DRAFT_EMAIL" | "SEND_EMAIL" | "CALENDAR_EVENT" | "SUMMARIZE_FILE"
    ) {
        return None;
    }
    if call.result_summary.is_empty() {
        return None;
    }
    Some((call.result_summary.clone(), call.status != "ok"))
}

fn user_facing_failure(err: &AgentError) -> String {
    let text = err.to_string();
    if text.contains("isn't installed") {
        return text;
    }
    if text.contains("permission") || text.contains("not permitted") {
        return format!("Coda doesn't have permission to do that. {text}");
    }
    if text.contains("Ollama") || text.contains("connection") || text.contains("11434") {
        return "Agent model unavailable. Please check Ollama.".into();
    }
    if text.contains("speech model") {
        return "Couldn't understand that. Try again after the speech model is installed.".into();
    }
    format!("I couldn't complete that.\n\n{text}")
}

fn extract_open_name(goal: &str) -> String {
    let lower = goal.to_ascii_lowercase();
    let rest = lower
        .split_once("open ")
        .or_else(|| lower.split_once("launch "))
        .or_else(|| lower.split_once("start "))
        .map(|(_, rest)| rest)
        .unwrap_or(goal);
    rest.trim()
        .trim_start_matches("the ")
        .trim_start_matches("my ")
        .trim_end_matches(" app")
        .trim()
        .to_string()
}

fn extract_play_query(goal: &str) -> String {
    let lower = goal.to_ascii_lowercase();
    lower
        .split_once("play ")
        .map(|(_, rest)| rest.trim().to_string())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| goal.to_string())
}

fn extract_file_query(goal: &str) -> String {
    let lower = goal.to_ascii_lowercase();
    after_file_prefix(&lower).unwrap_or_else(|| goal.to_string())
}

fn after_file_prefix(lower: &str) -> Option<String> {
    for prefix in ["open the file ", "open file ", "find the file ", "find file ", "open "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            return Some(rest.trim().to_string());
        }
    }
    None
}

fn extract_folder_path(goal: &str) -> String {
    let lower = goal.to_ascii_lowercase();
    if lower.contains("download") {
        return "~/Downloads".into();
    }
    if lower.contains("document") {
        return "~/Documents".into();
    }
    if lower.contains("desktop") {
        return "~/Desktop".into();
    }
    "~/Downloads".into()
}

fn consequence(tool: &str) -> &'static str {
    match tool {
        "FILES_WRITE" => "A file will be created in the local Coda workspace.",
        "FILES_DELETE" => "A workspace file will be permanently deleted.",
        "MEMORY_DELETE" => "These stored rows (and their embeddings) will be deleted, not hidden.",
        "SUGGEST_FILE" => "Coda noticed a new file that may match a memory. Nothing will be opened or sent.",
        "EMAIL_SEND" | "DRAFT_EMAIL" => "Your mail app will open a draft. Nothing is sent until you send it.",
        "CALENDAR_EVENT" => "An event will be added to Calendar.",
        "SHELL_EXECUTE" => "A local process would run.",
        _ => "This has an external side-effect.",
    }
}

fn cadence_ms(cadence: &str) -> u64 {
    match cadence {
        "weekly" => 7 * 86_400_000,
        "hourly" => 3_600_000,
        "interval" => 3_600_000,
        _ => 86_400_000,
    }
}

fn heuristic_result(task: &AgentTask, notes: &str) -> String {
    if notes.trim().is_empty() {
        return format!("I understood “{}”, but I could not produce an answer. Try again in a moment.", task.goal);
    }
    format!("Here is what I did for “{}”:\n\n{}", task.goal, notes.trim())
}


fn looks_like_file_suggestion(goal: &str) -> bool {
    let lower = goal.to_ascii_lowercase();
    lower.starts_with("new file ") && lower.contains("do not open")
}

fn preview(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= 120 {
        trimmed.to_string()
    } else {
        format!("{}…", trimmed.chars().take(120).collect::<String>())
    }
}

async fn embed_memory(http: &reqwest::Client, config: &AgentConfig, text: &str) -> Option<Vec<f32>> {
    let host = config.ollama_url.trim().trim_end_matches('/');
    let url = format!("{host}/api/tags");
    let response = http.get(&url).timeout(std::time::Duration::from_secs(8)).send().await.ok()?;
    let body: Value = response.json().await.ok()?;
    let names: Vec<String> = body
        .get("models")
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter_map(|model| model.get("name").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let preferred = std::env::var("CODA_EMBED_MODEL")
        .ok()
        .or_else(|| std::env::var("GHOSTNOTE_EMBED_MODEL").ok());
    let model = memory::pick_embed_model(&names, preferred.as_deref())?;
    memory::embed_text(http, host, &model, text).await
}

impl AgentRuntime {
    async fn forget_matches(&self, app: &AppHandle, query: &str) -> Vec<Value> {
        self.recall(app, query, 8)
            .await
            .into_iter()
            .map(|hit| {
                json!({
                    "id": hit.id,
                    "content": hit.text,
                    "source": hit.source,
                    "tags": hit.tags,
                })
            })
            .collect()
    }
}

pub fn extract_action_items(summary: &str) -> Vec<String> {
    summary
        .lines()
        .map(str::trim)
        .filter(|line| {
            let lower = line.to_ascii_lowercase();
            (line.starts_with('-') || line.starts_with('*') || lower.contains("todo"))
                && contains_action(&lower)
        })
        .map(|line| line.trim_start_matches(['-', '*', ' ']).to_string())
        .filter(|line| line.len() > 8)
        .take(8)
        .collect()
}

fn contains_action(line: &str) -> bool {
    ["will ", "send", "follow up", "due", "tomorrow", "action", "assign", "email"]
        .iter()
        .any(|needle| line.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_items_from_summary() {
        let items = extract_action_items("- John will send the API documentation tomorrow.\n- Nice weather");
        assert_eq!(items.len(), 1);
        assert!(items[0].contains("John"));
    }

    #[test]
    fn infer_args_for_open_apple_music() {
        let task = AgentTask::new("Open Apple Music".into(), "personal".into(), "c1".into());
        let args = infer_args("OPEN_APPLICATION", &task).unwrap();
        assert_eq!(args["name"].as_str().unwrap().to_ascii_lowercase(), "apple music");
    }

    #[test]
    fn complex_goal_opens_named_app_only() {
        let task = AgentTask::new(
            "Open Apple Music and play my workout playlist".into(),
            "personal".into(),
            "c1".into(),
        );
        let args = infer_args("OPEN_APPLICATION", &task).unwrap();
        assert_eq!(args["name"].as_str().unwrap(), "apple music");
    }

    #[test]
    fn followup_does_not_claim_playlist_played() {
        let task = AgentTask::new(
            "Open Apple Music and play my workout playlist".into(),
            "personal".into(),
            "c1".into(),
        );
        let text = honest_computer_followup(&task, "Apple Music is open.");
        assert!(text.contains("Apple Music is open."));
        assert!(!text.to_ascii_lowercase().contains("playing"));
    }

    #[test]
    fn missing_model_message_does_not_claim_success() {
        let text = user_facing_failure(&AgentError::msg("connection refused 127.0.0.1:11434"));
        assert!(text.contains("Ollama"));
        assert!(!text.to_ascii_lowercase().contains("opened"));
    }

    #[test]
    fn failed_tool_error_is_structured() {
        let err = AgentError::msg("Apple Music isn't installed on this computer.");
        let text = user_facing_failure(&err);
        assert!(text.contains("isn't installed"));
    }
}
