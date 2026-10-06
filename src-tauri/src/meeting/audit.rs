//! Local audit log for meeting extraction and suggestions.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;

use crate::agent::ids;

use super::extract::ExtractedBatch;
use super::nudge::Nudge;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent<'a> {
    pub timestamp: u64,
    pub event: &'a str,
    pub meeting_id: Option<&'a str>,
    pub model: Option<&'a str>,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

pub fn append(app: &AppHandle, event: AuditEvent<'_>) {
    let Some(path) = audit_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) else {
        return;
    };
    if let Ok(line) = serde_json::to_string(&event) {
        let _ = writeln!(file, "{line}");
    }
    let _ = crate::agent::store::append_audit(
        app,
        event.event,
        &serde_json::json!({
            "meetingId": event.meeting_id,
            "model": event.model,
            "input": event.input,
            "output": event.output,
        }),
    );
}

pub fn extraction(
    app: &AppHandle,
    meeting_id: Option<&str>,
    model: Option<&str>,
    transcript: &str,
    batch: &ExtractedBatch,
    error: Option<&str>,
    nudges: &[Nudge],
) {
    append(
        app,
        AuditEvent {
            timestamp: ids::now_ms(),
            event: "meeting.extract",
            meeting_id,
            model,
            input: serde_json::json!({
                "transcriptChars": transcript.chars().count(),
                "transcriptPreview": preview(transcript, 240),
            }),
            output: serde_json::json!({
                "items": batch.items,
                "topics": batch.topics,
                "people": batch.people,
                "nudges": nudges,
                "error": error,
            }),
        },
    );
}

pub fn draft_decision(app: &AppHandle, draft_id: &str, decision: &str, executed: bool) {
    append(
        app,
        AuditEvent {
            timestamp: ids::now_ms(),
            event: "meeting.draft",
            meeting_id: None,
            model: None,
            input: serde_json::json!({ "draftId": draft_id, "decision": decision }),
            output: serde_json::json!({ "executed": executed }),
        },
    );
}

fn preview(text: &str, max: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= max {
        return trimmed.to_string();
    }
    trimmed.chars().take(max).collect::<String>() + "…"
}

fn audit_path(_app: &AppHandle) -> Option<PathBuf> {
    Some(crate::coda::agent_dir().join("meeting_audit.jsonl"))
}

#[cfg(test)]
mod tests {
    use super::preview;

    #[test]
    fn preview_truncates() {
        let long = "word ".repeat(80);
        let cut = preview(&long, 20);
        assert!(cut.ends_with('…'));
        assert!(cut.chars().count() <= 21);
    }
}
