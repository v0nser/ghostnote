//! Persistent meeting memory. JSON on disk — no extra crate.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::agent::ids;

use super::extract::{normalize_label, ActionItem, ExtractedBatch};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MeetingMemory {
    #[serde(default)]
    pub meetings: Vec<StoredMeeting>,
    #[serde(default)]
    pub items: Vec<StoredItem>,
    #[serde(default)]
    pub topics: Vec<StoredTopic>,
    #[serde(default)]
    pub people: Vec<StoredPerson>,
    #[serde(default)]
    pub drafts: Vec<super::draft::Draft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredMeeting {
    pub id: String,
    pub created_at: u64,
    pub topics: Vec<String>,
    pub people: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredItem {
    pub id: String,
    pub meeting_id: String,
    pub owner: Option<String>,
    pub task: String,
    pub due: Option<String>,
    pub status: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredTopic {
    pub label: String,
    pub last_seen: u64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredPerson {
    pub name: String,
    pub last_seen: u64,
    pub count: u32,
}

pub fn load(app: &AppHandle) -> MeetingMemory {
    let path = memory_path(app);
    let Some(path) = path else {
        return MeetingMemory::default();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return MeetingMemory::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save(app: &AppHandle, memory: &MeetingMemory) {
    let Some(path) = memory_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(raw) = serde_json::to_string_pretty(memory) {
        if let Err(err) = fs::write(&path, raw) {
            log::warn!("could not persist meeting memory: {err}");
        }
    }
}

pub fn record(app: &AppHandle, batch: &ExtractedBatch) -> (String, MeetingMemory) {
    let mut memory = load(app);
    let now = ids::now_ms();
    let meeting_id = ids::new_id("mtg");

    memory.meetings.push(StoredMeeting {
        id: meeting_id.clone(),
        created_at: now,
        topics: batch.topics.clone(),
        people: batch.people.clone(),
    });

    for item in &batch.items {
        memory.items.push(StoredItem {
            id: ids::new_id("act"),
            meeting_id: meeting_id.clone(),
            owner: item.owner.clone(),
            task: item.task.clone(),
            due: item.due.clone(),
            status: "open".into(),
            created_at: now,
        });
    }

    for topic in &batch.topics {
        upsert_topic(&mut memory.topics, topic, now);
    }
    for person in &batch.people {
        upsert_person(&mut memory.people, person, now);
    }
    for item in &batch.items {
        if let Some(owner) = &item.owner {
            upsert_person(&mut memory.people, owner, now);
        }
    }

    save(app, &memory);
    (meeting_id, memory)
}

pub fn delete_items(app: &AppHandle, ids: &[String]) -> usize {
    if ids.is_empty() {
        return 0;
    }
    let mut memory = load(app);
    let before = memory.items.len();
    memory.items.retain(|item| !ids.contains(&item.id));
    let removed = before.saturating_sub(memory.items.len());
    if removed > 0 {
        save(app, &memory);
    }
    removed
}

pub fn open_items(memory: &MeetingMemory) -> Vec<StoredItem> {
    memory
        .items
        .iter()
        .filter(|item| item.status == "open")
        .cloned()
        .collect()
}

pub fn similar_open_item<'a>(memory: &'a MeetingMemory, item: &ActionItem) -> Option<&'a StoredItem> {
    let needle = normalize_label(&item.task);
    if needle.is_empty() {
        return None;
    }
    memory.items.iter().find(|stored| {
        stored.status == "open" && normalize_label(&stored.task) == needle
    })
}

fn upsert_topic(topics: &mut Vec<StoredTopic>, label: &str, now: u64) {
    if let Some(existing) = topics
        .iter_mut()
        .find(|topic| normalize_label(&topic.label) == normalize_label(label))
    {
        existing.count += 1;
        existing.last_seen = now;
        return;
    }
    topics.push(StoredTopic {
        label: label.to_string(),
        last_seen: now,
        count: 1,
    });
}

fn upsert_person(people: &mut Vec<StoredPerson>, name: &str, now: u64) {
    if let Some(existing) = people
        .iter_mut()
        .find(|person| normalize_label(&person.name) == normalize_label(name))
    {
        existing.count += 1;
        existing.last_seen = now;
        return;
    }
    people.push(StoredPerson {
        name: name.to_string(),
        last_seen: now,
        count: 1,
    });
}

fn memory_path(_app: &AppHandle) -> Option<PathBuf> {
    Some(crate::coda::agent_dir().join("meeting_memory.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upserts_repeat_topics() {
        let mut topics = Vec::new();
        upsert_topic(&mut topics, "Hiring", 1);
        upsert_topic(&mut topics, "hiring", 2);
        assert_eq!(topics.len(), 1);
        assert_eq!(topics[0].count, 2);
        assert_eq!(topics[0].last_seen, 2);
    }
}
