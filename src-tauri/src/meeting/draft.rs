//! Drafts for follow-ups. Nothing beyond read/summarize runs until approved.

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

use crate::agent::ids;
use crate::agent::skills;

use super::extract::{ActionItem, ExtractedBatch};
use super::memory::{self, MeetingMemory};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    pub id: String,
    pub meeting_id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub when: Option<String>,
    pub to: Option<String>,
    pub status: String,
}

pub fn propose(meeting_id: &str, batch: &ExtractedBatch) -> Vec<Draft> {
    let mut drafts = Vec::new();
    for item in &batch.items {
        if looks_like_email(&item.task) {
            drafts.push(Draft {
                id: ids::new_id("dft"),
                meeting_id: meeting_id.to_string(),
                kind: "email".into(),
                title: format!("Email: {}", item.task),
                body: email_body(item),
                when: None,
                to: item.owner.clone(),
                status: "pending".into(),
            });
        }
        if item.due.is_some() || looks_like_reminder(&item.task) {
            drafts.push(Draft {
                id: ids::new_id("dft"),
                meeting_id: meeting_id.to_string(),
                kind: "reminder".into(),
                title: item.task.clone(),
                body: match &item.owner {
                    Some(owner) => format!("{owner}: {}", item.task),
                    None => item.task.clone(),
                },
                when: item.due.clone(),
                to: item.owner.clone(),
                status: "pending".into(),
            });
        }
    }
    drafts
}

pub fn decide(app: &AppHandle, draft_id: &str, approve: bool) -> Result<Draft, String> {
    let mut memory = memory::load(app);
    let Some(index) = memory.drafts.iter().position(|draft| draft.id == draft_id) else {
        return Err("that draft is gone".into());
    };
    if memory.drafts[index].status != "pending" {
        return Err("that draft was already decided".into());
    }

    if !approve {
        memory.drafts[index].status = "denied".into();
        let draft = memory.drafts[index].clone();
        memory::save(app, &memory);
        super::audit::draft_decision(app, draft_id, "denied", false);
        return Ok(draft);
    }

    let executed = execute(&memory.drafts[index]);
    match executed {
        Ok(()) => {
            memory.drafts[index].status = "approved".into();
            let draft = memory.drafts[index].clone();
            memory::save(app, &memory);
            super::audit::draft_decision(app, draft_id, "approved", true);
            Ok(draft)
        }
        Err(err) => {
            super::audit::draft_decision(app, draft_id, "approve_failed", false);
            Err(err)
        }
    }
}

pub fn pending(memory: &MeetingMemory) -> Vec<Draft> {
    memory
        .drafts
        .iter()
        .filter(|draft| draft.status == "pending")
        .cloned()
        .collect()
}

fn execute(draft: &Draft) -> Result<(), String> {
    match draft.kind.as_str() {
        "email" => {
            skills::execute(
                "DRAFT_EMAIL",
                &json!({
                    "to": draft.to.clone().unwrap_or_default(),
                    "subject": draft.title,
                    "body": draft.body,
                }),
            )
            .map(|_| ())
            .map_err(|err| err.to_string())
        }
        "reminder" => {
            let when = draft
                .when
                .clone()
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| "tomorrow 9am".into());
            skills::execute(
                "SET_REMINDER",
                &json!({
                    "title": draft.title,
                    "when": when,
                }),
            )
            .map(|_| ())
            .map_err(|err| err.to_string())
        }
        other => Err(format!("{other} drafts cannot be executed")),
    }
}

fn looks_like_email(task: &str) -> bool {
    let lower = task.to_ascii_lowercase();
    lower.contains("email")
        || lower.contains("e-mail")
        || lower.contains("send the recap")
        || lower.contains("send a recap")
        || lower.contains("mail ")
        || lower.starts_with("mail")
}

fn looks_like_reminder(task: &str) -> bool {
    let lower = task.to_ascii_lowercase();
    lower.contains("remind") || lower.contains("follow up") || lower.contains("ping")
}

fn email_body(item: &ActionItem) -> String {
    let mut body = item.task.clone();
    if let Some(due) = &item.due {
        body.push_str("\n\nDue: ");
        body.push_str(due);
    }
    body.push_str("\n\n(Draft only — not sent.)");
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meeting::extract::ActionItem;

    #[test]
    fn email_and_due_become_pending_drafts() {
        let batch = ExtractedBatch {
            items: vec![
                ActionItem {
                    owner: Some("Asha".into()),
                    task: "Email the recap to finance".into(),
                    due: Some("Friday".into()),
                },
                ActionItem {
                    owner: None,
                    task: "Update the deck".into(),
                    due: None,
                },
            ],
            topics: Vec::new(),
            people: Vec::new(),
        };
        let drafts = propose("mtg_1", &batch);
        assert!(drafts.iter().any(|draft| draft.kind == "email" && draft.status == "pending"));
        assert!(drafts.iter().any(|draft| draft.kind == "reminder" && draft.status == "pending"));
        assert!(!drafts.iter().any(|draft| draft.title.contains("Update the deck")));
    }

    #[test]
    fn unknown_kind_does_not_execute() {
        let draft = Draft {
            id: "dft_x".into(),
            meeting_id: "mtg_1".into(),
            kind: "sms".into(),
            title: "nope".into(),
            body: String::new(),
            when: None,
            to: None,
            status: "pending".into(),
        };
        assert!(execute(&draft).is_err());
    }
}
