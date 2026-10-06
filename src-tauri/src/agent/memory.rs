use super::ids;
use super::types::{MemoryItem, MemoryKind, MemoryScope};

pub fn create(
    content: impl Into<String>,
    kind: MemoryKind,
    scope: MemoryScope,
    source: impl Into<String>,
    importance: u8,
) -> MemoryItem {
    create_with_embedding(content, kind, scope, source, importance, None, Vec::new())
}

pub fn create_with_embedding(
    content: impl Into<String>,
    kind: MemoryKind,
    scope: MemoryScope,
    source: impl Into<String>,
    importance: u8,
    embedding: Option<Vec<f32>>,
    tags: Vec<String>,
) -> MemoryItem {
    let now = ids::now_ms();
    MemoryItem {
        id: ids::new_id("mem"),
        kind,
        content: content.into(),
        source: source.into(),
        confidence: 0.9,
        created_at: now,
        updated_at: now,
        importance: importance.min(10),
        scope,
        expires_at: None,
        embedding,
        tags,
    }
}

/// Keyword + importance + recency. Never dump the full store.
pub fn search<'a>(items: &'a [MemoryItem], query: &str, limit: usize) -> Vec<&'a MemoryItem> {
    let now = ids::now_ms();
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|word| word.to_ascii_lowercase())
        .filter(|word| word.len() > 2)
        .collect();

    let mut scored: Vec<(&MemoryItem, i64)> = items
        .iter()
        .filter(|item| item.expires_at.map(|exp| exp > now).unwrap_or(true))
        .map(|item| {
            let hay = item.content.to_ascii_lowercase();
            let hits = if terms.is_empty() {
                1
            } else {
                terms.iter().filter(|term| hay.contains(term.as_str())).count() as i64
            };
            let recency = (now.saturating_sub(item.updated_at) / 3_600_000) as i64;
            let score = hits * 10 + i64::from(item.importance) - recency.min(20);
            (item, hits, score)
        })
        .filter(|(item, hits, score)| {
            if !terms.is_empty() && *hits == 0 {
                return false;
            }
            *score > 0 || item.importance >= 8
        })
        .map(|(item, _, score)| (item, score))
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().take(limit.max(1)).map(|(item, _)| item).collect()
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let denom = na.sqrt() * nb.sqrt();
    if denom < f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecallHit {
    pub id: String,
    pub text: String,
    pub source: String,
    pub tags: Vec<String>,
    pub score: f32,
}

pub fn recall(
    items: &[MemoryItem],
    query: &str,
    query_embedding: Option<&[f32]>,
    meeting_lines: &[(String, String)],
    k: usize,
) -> Vec<RecallHit> {
    let mut hits: Vec<RecallHit> = Vec::new();
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|word| word.to_ascii_lowercase())
        .filter(|word| word.len() > 2)
        .collect();

    for item in items {
        let mut score = 0.0f32;
        if let (Some(query_vec), Some(item_vec)) = (query_embedding, item.embedding.as_deref()) {
            score = cosine(query_vec, item_vec);
        }
        let hay = item.content.to_ascii_lowercase();
        if terms.is_empty() {
            score = score.max(0.05);
        } else {
            let hits_n = terms.iter().filter(|term| hay.contains(term.as_str())).count() as f32;
            if hits_n > 0.0 {
                score = score.max(0.15 + hits_n * 0.2);
            }
        }
        if score > 0.12 {
            hits.push(RecallHit {
                id: item.id.clone(),
                text: item.content.clone(),
                source: if item.source.is_empty() {
                    "memory".into()
                } else {
                    item.source.clone()
                },
                tags: item.tags.clone(),
                score,
            });
        }
    }

    for (id, text) in meeting_lines {
        let hay = text.to_ascii_lowercase();
        let hits_n = if terms.is_empty() {
            0.0
        } else {
            terms.iter().filter(|term| hay.contains(term.as_str())).count() as f32
        };
        if hits_n > 0.0 || (terms.is_empty() && !text.is_empty()) {
            hits.push(RecallHit {
                id: id.clone(),
                text: text.clone(),
                source: "meeting".into(),
                tags: vec!["action-item".into()],
                score: 0.18 + hits_n * 0.2,
            });
        }
    }

    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hits.dedup_by(|a, b| a.id == b.id);
    hits.truncate(k.max(1));
    hits
}

pub fn pick_embed_model(names: &[String], preferred: Option<&str>) -> Option<String> {
    let mut wants = Vec::new();
    if let Some(value) = preferred.map(str::trim).filter(|value| !value.is_empty()) {
        wants.push(value.to_string());
    }
    wants.push("nomic-embed-text".into());
    for want in wants {
        if let Some(hit) = names.iter().find(|name| {
            let name = name.to_ascii_lowercase();
            let want = want.to_ascii_lowercase();
            name == want || name.starts_with(&format!("{want}:")) || name.starts_with(&want)
        }) {
            return Some(hit.clone());
        }
    }
    None
}

pub async fn embed_text(
    http: &reqwest::Client,
    host: &str,
    model: &str,
    text: &str,
) -> Option<Vec<f32>> {
    let url = format!("{}/api/embed", host.trim().trim_end_matches('/'));
    let payload = serde_json::json!({ "model": model, "input": text });
    if let Ok(response) = http.post(&url).json(&payload).send().await {
        if let Ok(value) = response.json::<serde_json::Value>().await {
            if let Some(vec) = embedding_from_value(&value) {
                return Some(vec);
            }
        }
    }
    let url = format!("{}/api/embeddings", host.trim().trim_end_matches('/'));
    let payload = serde_json::json!({ "model": model, "prompt": text });
    let response = http.post(url).json(&payload).send().await.ok()?;
    let value = response.json::<serde_json::Value>().await.ok()?;
    embedding_from_value(&value)
}

fn embedding_from_value(value: &serde_json::Value) -> Option<Vec<f32>> {
    if let Some(arr) = value.get("embedding").and_then(|v| v.as_array()) {
        return Some(arr.iter().filter_map(|n| n.as_f64().map(|n| n as f32)).collect());
    }
    value
        .get("embeddings")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.as_array())
        .map(|arr| arr.iter().filter_map(|n| n.as_f64().map(|n| n as f32)).collect())
}

pub fn looks_like_recall(text: &str) -> Option<String> {
    let lower = text.trim().to_ascii_lowercase();
    for prefix in [
        "what do you remember about ",
        "what do you know about ",
        "recall ",
        "remember about ",
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let fact = rest.trim();
            if fact.len() > 2 {
                return Some(fact.to_string());
            }
        }
    }
    None
}

pub fn looks_like_remember(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    for prefix in ["remember this:", "remember that:", "remember:", "remember "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let start = trimmed.len().saturating_sub(rest.len());
            let fact = trimmed[start..].trim();
            if fact.len() > 2 {
                return Some(fact.to_string());
            }
        }
    }
    None
}

pub fn looks_like_forget(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    for prefix in ["forget this:", "forget that:", "forget:", "forget "] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let start = trimmed.len().saturating_sub(rest.len());
            let fact = trimmed[start..].trim();
            if fact.len() > 2 {
                return Some(fact.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_returns_relevant_only() {
        let items = vec![
            create("I prefer Rust jobs in Bangalore", MemoryKind::Preference, MemoryScope::User, "user", 8),
            create("Bought milk", MemoryKind::Episodic, MemoryScope::Session, "user", 2),
        ];
        let hits = search(&items, "rust jobs", 5);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].content.contains("Rust"));
    }

    #[test]
    fn remember_phrase() {
        assert_eq!(
            looks_like_remember("Remember this: I use Neovim"),
            Some("I use Neovim".into())
        );
    }

    #[test]
    fn cosine_identical_is_one() {
        let a = vec![1.0, 0.0, 0.0];
        assert!((cosine(&a, &a) - 1.0).abs() < 1e-5);
        assert!(cosine(&a, &[0.0, 1.0, 0.0]).abs() < 1e-5);
    }

    #[test]
    fn recall_ranks_embedded_match() {
        let rust = create_with_embedding(
            "I prefer Rust jobs in Bangalore",
            MemoryKind::Preference,
            MemoryScope::User,
            "user",
            8,
            Some(vec![1.0, 0.0]),
            vec!["work".into()],
        );
        let milk = create("Bought milk", MemoryKind::Episodic, MemoryScope::Session, "user", 2);
        let hits = recall(
            &[rust, milk],
            "rust work",
            Some(&[1.0, 0.0]),
            &[("act1".into(), "Asha owns the Rust hiring plan".into())],
            5,
        );
        assert!(!hits.is_empty());
        assert!(hits[0].text.contains("Rust"));
        assert!(hits.iter().any(|hit| hit.source == "meeting"));
    }

    #[test]
    fn forget_and_recall_phrases() {
        assert_eq!(
            looks_like_forget("forget that I mentioned my manager's name"),
            Some("that I mentioned my manager's name".into())
        );
        assert_eq!(
            looks_like_recall("what do you remember about Bangalore"),
            Some("bangalore".into())
        );
    }
}
