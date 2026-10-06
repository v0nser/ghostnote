//! Meeting-scoped proactive layer: extract, remember, nudge, audit, confirm.

mod audit;
mod draft;
mod extract;
mod llm;
mod memory;
mod nudge;

use serde::Serialize;
use tauri::AppHandle;

use crate::ollama::client::Client;

pub fn forget_items(app: &AppHandle, ids: &[String]) -> usize {
    memory::delete_items(app, ids)
}

pub fn overdue_count(app: &AppHandle) -> usize {
    nudge::overdue_items(&memory::load(app)).len()
}

pub fn memory_open_lines(app: &AppHandle) -> Vec<(String, String)> {
    memory::load(app)
        .items
        .iter()
        .filter(|item| item.status == "open")
        .map(|item| {
            let owner = item.owner.clone().unwrap_or_else(|| "unassigned".into());
            let due = item.due.clone().unwrap_or_else(|| "no due date".into());
            (
                item.id.clone(),
                format!("{owner}: {} (due {due})", item.task),
            )
        })
        .collect()
}

pub use draft::Draft;
pub use extract::{ActionItem, ExtractedBatch};
pub use nudge::Nudge;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingLayer {
    pub action_items: Vec<ActionItem>,
    pub topics: Vec<String>,
    pub people: Vec<String>,
    pub nudges: Vec<Nudge>,
    pub drafts: Vec<Draft>,
    pub extract_model: Option<String>,
    pub extract_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingSnapshot {
    pub items: Vec<memory::StoredItem>,
    pub topics: Vec<memory::StoredTopic>,
    pub people: Vec<memory::StoredPerson>,
    pub drafts: Vec<Draft>,
    pub nudges: Vec<Nudge>,
}

/// Run after a transcript exists. Extraction failures are returned on the
/// layer, never as a crash.
pub async fn after_transcript(
    app: &AppHandle,
    client: &Client,
    transcript: &str,
) -> MeetingLayer {
    match llm::extract_actions(client, transcript).await {
        Ok(batch) => persist_and_nudge(app, batch, Some(llm::extract_model_name()), None, transcript),
        Err(err) => {
            let message = err.to_string();
            log::warn!("meeting extract failed: {message}");
            persist_and_nudge(
                app,
                ExtractedBatch::default(),
                None,
                Some(message),
                transcript,
            )
        }
    }
}

fn persist_and_nudge(
    app: &AppHandle,
    batch: ExtractedBatch,
    model: Option<&str>,
    error: Option<String>,
    transcript: &str,
) -> MeetingLayer {
    let prior = memory::load(app);
    let nudges = nudge::from_extract(&prior, &batch);
    let (meeting_id, mut memory) = if error.is_none()
        && (!batch.items.is_empty() || !batch.topics.is_empty() || !batch.people.is_empty())
    {
        memory::record(app, &batch)
    } else {
        (String::new(), prior)
    };

    let new_drafts = if meeting_id.is_empty() {
        Vec::new()
    } else {
        draft::propose(&meeting_id, &batch)
    };
    if !new_drafts.is_empty() {
        memory.drafts.extend(new_drafts);
        memory::save(app, &memory);
    }

    audit::extraction(
        app,
        if meeting_id.is_empty() {
            None
        } else {
            Some(meeting_id.as_str())
        },
        model,
        transcript,
        &batch,
        error.as_deref(),
        &nudges,
    );

    MeetingLayer {
        action_items: batch.items,
        topics: batch.topics,
        people: batch.people,
        nudges,
        drafts: draft::pending(&memory),
        extract_model: model.map(str::to_string),
        extract_error: error,
    }
}

pub fn snapshot(app: &AppHandle) -> MeetingSnapshot {
    let memory = memory::load(app);
    let nudges = nudge::from_extract(&memory, &ExtractedBatch::default());
    MeetingSnapshot {
        items: memory::open_items(&memory),
        topics: memory.topics.clone(),
        people: memory.people.clone(),
        drafts: draft::pending(&memory),
        nudges,
    }
}

pub fn decide_draft(app: &AppHandle, draft_id: &str, approve: bool) -> Result<Draft, String> {
    draft::decide(app, draft_id, approve)
}

#[tauri::command]
pub fn meeting_snapshot(app: AppHandle) -> MeetingSnapshot {
    snapshot(&app)
}

#[tauri::command]
pub fn meeting_decide_draft(
    app: AppHandle,
    draft_id: String,
    approve: bool,
) -> Result<Draft, String> {
    decide_draft(&app, &draft_id, approve)
}
