use serde_json::{json, Value};

use super::config::AgentConfig;
use super::error::{AgentError, AgentResult};
use super::tools::BuiltinTool;

pub enum Turn {
    Answer(String),
    Tool { name: String, args: Value },
}

const DECIDE_SYSTEM: &str = "\
You are Coda, a local assistant. Reply with one JSON object only.
{\"action\":\"answer\",\"text\":\"...\"}
or
{\"action\":\"tool\",\"name\":\"WEB_SEARCH\",\"args\":{\"query\":\"...\"}}

Tools: WEB_SEARCH, WEB_FETCH, OPEN_APPLICATION, OPEN_URL, OPEN_FOLDER, PLAY_MEDIA, \
SET_REMINDER, FIND_FILE, SUMMARIZE_FILE, CREATE_DOCUMENT, DRAFT_EMAIL, SEND_EMAIL, \
CALENDAR_EVENT, FILES_LIST, FILES_READ, FILES_WRITE, FILES_SEARCH, CALCULATOR, \
DATETIME, PYTHON_EXECUTE, CODE_EXECUTE, MEMORY_WRITE, MEMORY_SEARCH.

Use a tool for live facts, opening things, reminders, media, PDFs, files, code, or math. \
Never invent an itinerary. Never claim you sent email, paid, or changed the computer \
unless a tool result says success=true.";

const ANSWER_SYSTEM: &str = "\
You are Coda, a local assistant. Be direct and useful. \
Never claim you sent email, paid, or changed permissions unless a tool result says so. \
No preamble.";

pub fn parse_turn(text: &str) -> Option<Turn> {
    let value = extract_json(text)?;
    if let Some(turn) = turn_from_value(&value) {
        return Some(turn);
    }
    None
}

fn turn_from_value(value: &Value) -> Option<Turn> {
    if let Some(name) = value.get("name").or_else(|| value.get("tool")).and_then(Value::as_str) {
        let action = value.get("action").and_then(Value::as_str).unwrap_or("tool");
        if matches!(action, "tool" | "call" | "use") || BuiltinTool::from_name(&normalize_tool(name)).is_some()
        {
            if !matches!(action, "answer" | "final" | "done") {
                let args = value
                    .get("args")
                    .or_else(|| value.get("arguments"))
                    .cloned()
                    .unwrap_or_else(|| args_from_flat(value));
                return Some(Turn::Tool {
                    name: normalize_tool(name),
                    args,
                });
            }
        }
    }

    let action = value.get("action").and_then(Value::as_str).unwrap_or("");
    match action {
        "answer" | "final" | "done" | "" => {
            let text = value
                .get("text")
                .or_else(|| value.get("answer"))
                .or_else(|| value.get("content"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_string();
            if text.len() > 1 {
                Some(Turn::Answer(text))
            } else {
                None
            }
        }
        other if BuiltinTool::from_name(&normalize_tool(other)).is_some() => Some(Turn::Tool {
            name: normalize_tool(other),
            args: value
                .get("args")
                .or_else(|| value.get("arguments"))
                .cloned()
                .unwrap_or_else(|| args_from_flat(value)),
        }),
        _ => None,
    }
}

fn args_from_flat(value: &Value) -> Value {
    let mut map = serde_json::Map::new();
    if let Some(obj) = value.as_object() {
        for (key, item) in obj {
            if matches!(key.as_str(), "action" | "name" | "tool" | "args" | "arguments") {
                continue;
            }
            map.insert(key.clone(), item.clone());
        }
    }
    Value::Object(map)
}

fn normalize_tool(name: &str) -> String {
    name.trim()
        .replace(' ', "_")
        .replace('-', "_")
        .to_ascii_uppercase()
}

fn extract_json(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    let fenced = trimmed
        .split("```")
        .nth(1)
        .map(|block| {
            block
                .trim_start_matches("json")
                .trim_start_matches("JSON")
                .trim()
        })
        .unwrap_or(trimmed);
    if let Ok(value) = serde_json::from_str::<Value>(fenced) {
        return Some(value);
    }
    let start = fenced.find('{')?;
    let end = fenced.rfind('}')?;
    serde_json::from_str(&fenced[start..=end]).ok()
}

pub async fn chat(
    http: &reqwest::Client,
    config: &AgentConfig,
    model: &str,
    system: &str,
    user: &str,
    num_predict: u32,
    json_mode: bool,
) -> AgentResult<String> {
    let url = format!("{}/api/chat", config.ollama_url);
    let mut body = json!({
        "model": model,
        "stream": false,
        "keep_alive": "60m",
        "options": {
            "temperature": 0.2,
            "num_predict": num_predict,
            "num_ctx": 1536,
            "num_gpu": 99,
            "num_batch": 512
        },
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ]
    });
    if json_mode {
        body["format"] = json!("json");
    }
    let response = http
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    if !response.status().is_success() {
        return Err(AgentError::msg(format!("ollama chat failed ({})", response.status())));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    let text = value
        .pointer("/message/content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        return Err(AgentError::msg("empty model output"));
    }
    Ok(text)
}

pub async fn decide(
    http: &reqwest::Client,
    config: &AgentConfig,
    model: &str,
    goal: &str,
    memories: &[String],
    findings: &str,
) -> AgentResult<Turn> {
    let mut user = format!("User request:\n{goal}\n");
    if !memories.is_empty() {
        user.push_str("\nMemory:\n");
        for item in memories.iter().take(4) {
            user.push_str("- ");
            user.push_str(item);
            user.push('\n');
        }
    }
    if !findings.is_empty() {
        user.push_str("\nTool results so far:\n");
        user.push_str(findings);
        user.push_str("\nIf this is enough, answer. Otherwise call another tool.\n");
    }
    let raw = chat(http, config, model, DECIDE_SYSTEM, &user, 120, true).await?;
    if let Some(turn) = parse_turn(&raw) {
        return Ok(turn);
    }
    if raw.chars().count() > 12 {
        return Ok(Turn::Answer(raw));
    }
    Err(AgentError::msg("could not parse model turn"))
}

pub async fn answer(
    http: &reqwest::Client,
    config: &AgentConfig,
    model: &str,
    goal: &str,
    memories: &[String],
    findings: &str,
) -> AgentResult<String> {
    let mut user = format!("Goal:\n{goal}\n");
    if !memories.is_empty() {
        user.push_str("\nRelevant memory:\n");
        for item in memories.iter().take(4) {
            user.push_str("- ");
            user.push_str(item);
            user.push('\n');
        }
    }
    if findings.is_empty() {
        user.push_str("\nWrite a concise useful answer. No preamble.");
    } else {
        user.push_str("\nWhat already happened:\n");
        user.push_str(findings);
        user.push_str("\nWrite a concise useful answer using these results. No preamble.");
    }
    chat(http, config, model, ANSWER_SYSTEM, &user, 160, false).await
}

const DOCUMENT_SYSTEM: &str = "\
You summarize a local document the user asked about.
Write a useful recap in plain text:
- what the document is
- 5-8 concrete points
- any dates, names, amounts, or next steps
Do not invent facts that are not in the text. No JSON.";

pub async fn summarize_document(
    http: &reqwest::Client,
    config: &AgentConfig,
    model: &str,
    goal: &str,
    path: &str,
    text: &str,
) -> AgentResult<String> {
    let user = format!("User asked:\n{goal}\n\nFile: {path}\n\nDocument text:\n{text}");
    let url = format!("{}/api/chat", config.ollama_url);
    let body = json!({
        "model": model,
        "stream": false,
        "keep_alive": "60m",
        "options": {
            "temperature": 0.15,
            "num_predict": 480,
            "num_ctx": 8192,
            "num_gpu": 99,
            "num_batch": 512
        },
        "messages": [
            {"role": "system", "content": DOCUMENT_SYSTEM},
            {"role": "user", "content": user}
        ]
    });
    let response = http
        .post(url)
        .timeout(std::time::Duration::from_secs(90))
        .json(&body)
        .send()
        .await
        .map_err(|err| {
            AgentError::msg(format!(
                "Ollama is not reachable at {}. Set OLLAMA_HOST and run `ollama serve`. ({err})",
                config.ollama_url
            ))
        })?;
    if !response.status().is_success() {
        return Err(AgentError::msg(format!(
            "Ollama chat failed ({}) at {}",
            response.status(),
            config.ollama_url
        )));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|err| AgentError::msg(err.to_string()))?;
    let text = value
        .pointer("/message/content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if text.len() < 20 {
        return Err(AgentError::msg(
            "the local model returned nothing usable. Pull llama3.1:8b (`ollama pull llama3.1:8b`).",
        ));
    }
    Ok(text)
}

/// Prefer an 8B+ local model for agent work. Live coach still uses the small model.
pub async fn resolve_agent_model(
    http: &reqwest::Client,
    host: &str,
    fallback: &str,
) -> String {
    if let Ok(forced) = std::env::var("CODA_AGENT_MODEL")
        .or_else(|_| std::env::var("GHOSTNOTE_AGENT_MODEL"))
    {
        if !forced.is_empty() {
            return forced;
        }
    }
    let url = format!("{host}/api/tags");
    let Ok(response) = http.get(&url).send().await else {
        return fallback.to_string();
    };
    let Ok(body) = response.json::<Value>().await else {
        return fallback.to_string();
    };
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
    pick_agent_model(&names)
        .or_else(|| crate::ollama::pick_fast_model(&names))
        .unwrap_or_else(|| fallback.to_string())
}

const AGENT_PREFERRED: &[&str] = &[
    "llama3.1:8b",
    "llama3.1:8b-instruct-q4_K_M",
    "qwen2.5:14b",
    "qwen2.5:7b",
    "llama3.1:70b",
    "mistral:7b",
    "llama3.1",
    "qwen2.5:32b",
];

pub fn pick_agent_model(names: &[String]) -> Option<String> {
    for preferred in AGENT_PREFERRED {
        if let Some(hit) = names.iter().find(|name| {
            *name == preferred
                || name.starts_with(&format!("{preferred}:"))
                || name.starts_with(preferred)
        }) {
            return Some(hit.clone());
        }
    }
    names.iter().cloned().find(|name| {
        let lower = name.to_ascii_lowercase();
        lower.contains("8b") || lower.contains("7b") || lower.contains("14b") || lower.contains("70b")
    })
}

#[cfg(test)]
mod extra_model_tests {
    use super::pick_agent_model;

    #[test]
    fn prefers_8b_over_3b() {
        let names = vec!["llama3.2:3b".into(), "llama3.1:8b".into(), "qwen2.5:3b".into()];
        assert_eq!(pick_agent_model(&names).as_deref(), Some("llama3.1:8b"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_answer_json() {
        let turn = parse_turn(r#"{"action":"answer","text":"Hello there."}"#).unwrap();
        match turn {
            Turn::Answer(text) => assert!(text.contains("Hello")),
            Turn::Tool { .. } => panic!("expected answer"),
        }
    }

    #[test]
    fn parses_tool_json() {
        let turn = parse_turn(r#"{"action":"tool","name":"web search","args":{"query":"rust jobs"}}"#).unwrap();
        match turn {
            Turn::Tool { name, args } => {
                assert_eq!(name, "WEB_SEARCH");
                assert_eq!(args["query"], "rust jobs");
            }
            Turn::Answer(_) => panic!("expected tool"),
        }
    }

    #[test]
    fn parses_fenced_json() {
        let turn = parse_turn("```json\n{\"action\":\"answer\",\"text\":\"Done.\"}\n```").unwrap();
        assert!(matches!(turn, Turn::Answer(_)));
    }
}
