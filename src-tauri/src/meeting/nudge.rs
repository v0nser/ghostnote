//! Proactive meeting nudges. Surface only — never send or execute.

use serde::{Deserialize, Serialize};

use super::extract::{normalize_label, ExtractedBatch};
use super::memory::{similar_open_item, MeetingMemory, StoredItem};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nudge {
    pub kind: String,
    pub title: String,
    pub detail: String,
}

/// Compare this meeting's extract against stored memory.
pub fn from_extract(memory: &MeetingMemory, batch: &ExtractedBatch) -> Vec<Nudge> {
    let mut nudges = Vec::new();

    for item in &batch.items {
        if let Some(prior) = similar_open_item(memory, item) {
            nudges.push(Nudge {
                kind: "duplicate".into(),
                title: format!("Still open: {}", prior.task),
                detail: match &prior.owner {
                    Some(owner) => format!("{owner} already owns this from an earlier meeting."),
                    None => "This action item is still open from an earlier meeting.".into(),
                },
            });
        }
    }

    for item in overdue_items(memory) {
        nudges.push(Nudge {
            kind: "overdue".into(),
            title: format!("Overdue: {}", item.task),
            detail: match (&item.owner, &item.due) {
                (Some(owner), Some(due)) => format!("{owner} said {due}."),
                (None, Some(due)) => format!("Due {due}."),
                (Some(owner), None) => format!("{owner} still has this open."),
                (None, None) => "This is still open.".into(),
            },
        });
    }

    for topic in &batch.topics {
        if let Some(prior) = memory
            .topics
            .iter()
            .find(|stored| stored.count >= 1 && normalize_label(&stored.label) == normalize_label(topic))
        {
            if prior.count >= 1 {
                nudges.push(Nudge {
                    kind: "topic".into(),
                    title: format!("You have talked about {topic} before"),
                    detail: format!("Seen in {} earlier meeting(s).", prior.count),
                });
            }
        }
    }

    dedupe(nudges)
}

pub fn overdue_items(memory: &MeetingMemory) -> Vec<StoredItem> {
    memory
        .items
        .iter()
        .filter(|item| item.status == "open" && due_looks_past(item.due.as_deref()))
        .cloned()
        .collect()
}

fn due_looks_past(due: Option<&str>) -> bool {
    let Some(due) = due.map(str::trim).filter(|text| !text.is_empty()) else {
        return false;
    };
    if let Ok(parsed) = chrono_ymd(due) {
        return parsed < today_ymd();
    }
    let lower = due.to_ascii_lowercase();
    lower.contains("yesterday") || lower.contains("last week") || lower.contains("overdue")
}

fn chrono_ymd(raw: &str) -> Result<(i32, u32, u32), ()> {
    let trimmed = raw.trim();
    let mut parts = trimmed.split('-');
    let year: i32 = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let month: u32 = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    let day: u32 = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(());
    }
    Ok((year, month, day))
}

fn today_ymd() -> (i32, u32, u32) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = now / 86_400;
    civil_from_days(days as i64)
}

/// Howard Hinnant civil-from-days, proleptic Gregorian.
fn civil_from_days(mut z: i64) -> (i32, u32, u32) {
    z += 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}

fn dedupe(nudges: Vec<Nudge>) -> Vec<Nudge> {
    let mut out = Vec::new();
    for nudge in nudges {
        if !out
            .iter()
            .any(|seen: &Nudge| seen.kind == nudge.kind && seen.title == nudge.title)
        {
            out.push(nudge);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meeting::extract::ActionItem;
    use crate::meeting::memory::{StoredItem, StoredMeeting, StoredTopic};

    fn memory_with(item: StoredItem, topic: Option<StoredTopic>) -> MeetingMemory {
        MeetingMemory {
            meetings: vec![StoredMeeting {
                id: "mtg_old".into(),
                created_at: 1,
                topics: Vec::new(),
                people: Vec::new(),
            }],
            items: vec![item],
            topics: topic.into_iter().collect(),
            people: Vec::new(),
            drafts: Vec::new(),
        }
    }

    #[test]
    fn flags_duplicate_open_item() {
        let memory = memory_with(
            StoredItem {
                id: "act_1".into(),
                meeting_id: "mtg_old".into(),
                owner: Some("Sam".into()),
                task: "Send the recap".into(),
                due: None,
                status: "open".into(),
                created_at: 1,
            },
            None,
        );
        let batch = ExtractedBatch {
            items: vec![ActionItem {
                owner: None,
                task: "send the recap".into(),
                due: None,
            }],
            topics: Vec::new(),
            people: Vec::new(),
        };
        let nudges = from_extract(&memory, &batch);
        assert!(nudges.iter().any(|nudge| nudge.kind == "duplicate"));
    }

    #[test]
    fn flags_iso_overdue() {
        let memory = memory_with(
            StoredItem {
                id: "act_2".into(),
                meeting_id: "mtg_old".into(),
                owner: None,
                task: "File expenses".into(),
                due: Some("2020-01-01".into()),
                status: "open".into(),
                created_at: 1,
            },
            None,
        );
        let nudges = from_extract(&memory, &ExtractedBatch::default());
        assert!(nudges.iter().any(|nudge| nudge.kind == "overdue"));
    }

    #[test]
    fn future_iso_is_not_overdue() {
        assert!(!due_looks_past(Some("2099-12-31")));
        assert!(due_looks_past(Some("yesterday")));
    }
}
