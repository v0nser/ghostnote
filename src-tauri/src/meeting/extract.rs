//! Structured action-item extraction. Parse and chunking are local; the
//! model call lives in [`super::llm`].

use serde::{Deserialize, Serialize};

/// One concrete next step taken from a meeting transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionItem {
    pub owner: Option<String>,
    pub task: String,
    pub due: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedBatch {
    #[serde(default)]
    pub items: Vec<ActionItem>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub people: Vec<String>,
}

const CHUNK_CHARS: usize = 1800;

pub const EXTRACT_SYSTEM: &str = "\
Extract action items from a meeting transcript.
Reply with JSON only. No markdown. No commentary.
Shape:
{\"items\":[{\"owner\":null,\"task\":\"\",\"due\":null}],\"topics\":[],\"people\":[]}
Rules:
- owner is a person name if stated, else null
- task is a concrete next step (verb + object), not a wish
- due is the spoken date or null
- topics are 1-5 short nouns from the discussion
- people are names mentioned as participants or owners
- skip already-done work
- if none, {\"items\":[],\"topics\":[],\"people\":[]}";

pub const EXTRACT_RETRY: &str = "\
Your previous reply was not valid JSON. Reply again with ONLY this object and nothing else:
{\"items\":[{\"owner\":null,\"task\":\"example\",\"due\":null}],\"topics\":[],\"people\":[]}";

/// Split a transcript into short chunks so an 8B model stays in context.
pub fn chunk_transcript(transcript: &str) -> Vec<String> {
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if trimmed.len() <= CHUNK_CHARS {
        return vec![trimmed.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !current.is_empty() && current.len() + line.len() + 1 > CHUNK_CHARS {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Parse a model reply into items. Accepts a raw object, a fenced block,
/// or prose wrapped around a JSON object.
pub fn parse_extracted(raw: &str) -> Result<ExtractedBatch, String> {
    let slice = json_slice(raw).ok_or_else(|| "no JSON object in the model reply".to_string())?;
    let value: serde_json::Value =
        serde_json::from_str(slice).map_err(|err| format!("malformed JSON: {err}"))?;
    let batch = coerce_batch(&value)?;
    Ok(normalize_batch(batch))
}

pub fn merge_batches(batches: impl IntoIterator<Item = ExtractedBatch>) -> ExtractedBatch {
    let mut out = ExtractedBatch::default();
    for batch in batches {
        for item in batch.items {
            if !out.items.iter().any(|seen| same_task(&seen.task, &item.task)) {
                out.items.push(item);
            }
        }
        for topic in batch.topics {
            if !out.topics.iter().any(|seen| same_label(seen, &topic)) {
                out.topics.push(topic);
            }
        }
        for person in batch.people {
            if !out.people.iter().any(|seen| same_label(seen, &person)) {
                out.people.push(person);
            }
        }
    }
    out
}

fn json_slice(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    let unfenced = strip_fence(trimmed);
    let start = unfenced.find('{')?;
    let end = unfenced.rfind('}')?;
    if end <= start {
        return None;
    }
    Some(&unfenced[start..=end])
}

fn strip_fence(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("```json") {
        return rest.strip_suffix("```").unwrap_or(rest).trim();
    }
    if let Some(rest) = trimmed.strip_prefix("```") {
        return rest.strip_suffix("```").unwrap_or(rest).trim();
    }
    trimmed
}

fn coerce_batch(value: &serde_json::Value) -> Result<ExtractedBatch, String> {
    if let Some(items) = value.get("items").and_then(|v| v.as_array()) {
        return Ok(ExtractedBatch {
            items: items.iter().filter_map(coerce_item).collect(),
            topics: string_list(value.get("topics")),
            people: string_list(value.get("people")),
        });
    }
    if let Some(items) = value.as_array() {
        return Ok(ExtractedBatch {
            items: items.iter().filter_map(coerce_item).collect(),
            topics: Vec::new(),
            people: Vec::new(),
        });
    }
    if value.get("task").is_some() {
        return Ok(ExtractedBatch {
            items: coerce_item(value).into_iter().collect(),
            topics: Vec::new(),
            people: Vec::new(),
        });
    }
    Err("JSON did not contain an items array".into())
}

fn coerce_item(value: &serde_json::Value) -> Option<ActionItem> {
    let task = value
        .get("task")
        .or_else(|| value.get("action"))
        .and_then(|v| v.as_str())
        .map(clean_text)
        .filter(|text| !text.is_empty())?;
    Some(ActionItem {
        owner: optional_text(value.get("owner")),
        task,
        due: optional_text(value.get("due").or_else(|| value.get("dueDate"))),
    })
}

fn string_list(value: Option<&serde_json::Value>) -> Vec<String> {
    match value {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(clean_text))
            .filter(|text| !text.is_empty())
            .collect(),
        Some(serde_json::Value::String(text)) => {
            let cleaned = clean_text(text);
            if cleaned.is_empty() {
                Vec::new()
            } else {
                vec![cleaned]
            }
        }
        _ => Vec::new(),
    }
}

fn optional_text(value: Option<&serde_json::Value>) -> Option<String> {
    let text = value.and_then(|v| {
        if v.is_null() {
            None
        } else {
            v.as_str().map(clean_text)
        }
    })?;
    if text.is_empty() || text.eq_ignore_ascii_case("null") || text.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(text)
    }
}

fn clean_text(raw: &str) -> String {
    raw.trim().trim_matches('"').trim().to_string()
}

fn normalize_batch(mut batch: ExtractedBatch) -> ExtractedBatch {
    batch.items.retain(|item| !item.task.is_empty());
    batch.topics.retain(|topic| !topic.is_empty());
    batch.people.retain(|person| !person.is_empty());
    batch
}

fn same_task(left: &str, right: &str) -> bool {
    normalize_label(left) == normalize_label(right)
}

fn same_label(left: &str, right: &str) -> bool {
    normalize_label(left) == normalize_label(right)
}

pub(crate) fn normalize_label(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric() || ch.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_object() {
        let raw = r#"{"items":[{"owner":"Asha","task":"Send the recap","due":"Friday"}],"topics":["hiring"],"people":["Asha"]}"#;
        let batch = parse_extracted(raw).expect("parse");
        assert_eq!(batch.items[0].owner.as_deref(), Some("Asha"));
        assert_eq!(batch.items[0].task, "Send the recap");
        assert_eq!(batch.items[0].due.as_deref(), Some("Friday"));
        assert_eq!(batch.topics, vec!["hiring"]);
    }

    #[test]
    fn parses_fenced_and_prose() {
        let raw = "Here you go:\n```json\n{\"items\":[{\"owner\":null,\"task\":\"Book a room\",\"due\":null}]}\n```\n";
        let batch = parse_extracted(raw).expect("parse");
        assert_eq!(batch.items[0].task, "Book a room");
        assert!(batch.items[0].owner.is_none());
    }

    #[test]
    fn rejects_non_json() {
        assert!(parse_extracted("we should maybe follow up sometime").is_err());
    }

    #[test]
    fn chunks_long_transcript() {
        let line = "Interviewer: please send the contract by Monday\n";
        let transcript = line.repeat(80);
        let chunks = chunk_transcript(&transcript);
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|chunk| chunk.len() <= CHUNK_CHARS + line.len()));
    }

    #[test]
    fn empty_transcript_has_no_chunks() {
        assert!(chunk_transcript("   ").is_empty());
    }

    #[test]
    fn merge_drops_duplicate_tasks() {
        let a = parse_extracted(r#"{"items":[{"owner":"Sam","task":"Send the recap","due":null}]}"#)
            .unwrap();
        let b = parse_extracted(r#"{"items":[{"owner":null,"task":"send the recap!","due":"Fri"}]}"#)
            .unwrap();
        let merged = merge_batches([a, b]);
        assert_eq!(merged.items.len(), 1);
        assert_eq!(merged.items[0].owner.as_deref(), Some("Sam"));
    }
}
