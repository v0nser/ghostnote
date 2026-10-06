//! Talking to a local Ollama daemon.
//!
//! The request and the reply both contain meeting content, so nothing from
//! either is written to the log. Failures are reported as typed errors only.

use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::error::{OllamaError, OllamaResult};
use super::parse::{parse_partial, parse_reply, Draft};

const DEFAULT_HOST: &str = "http://127.0.0.1:11434";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);
const EXTRACT_TIMEOUT: Duration = Duration::from_secs(90);

/// Meeting action-item extraction always uses this local model.
pub const EXTRACT_MODEL: &str = "llama3.1:8b";

/// Smaller / newer instruct models first so the spoken answer starts sooner.
const FAST_PREFERRED: &[&str] = &[
    "llama3.2:3b",
    "llama3.2:1b",
    "qwen2.5:3b",
    "qwen2.5:1.5b",
    "qwen2.5:7b",
    "phi3:mini",
    "phi3",
    "gemma2:2b",
    "llama3.2",
    "qwen2.5",
    "llama3.1:8b",
    "llama3.1",
];

const SYSTEM_PROMPT: &str = "\
You are a live interview copilot. Reply with ONE short spoken answer.

Rules:
- Identify the latest interviewer question.
- One answer, first person, 2 sentences max.
- No preamble, no bullets, no options.

<detected_question> one sentence </detected_question>
<answer> the spoken reply </answer>";

const SUMMARY_PROMPT: &str = "\
You summarize a live interview for the candidate, from the questions that were asked.

Write a concise recap in plain text:
- 4-8 short bullets of what was asked and what it was really testing
- Then 3-5 sentences: overall themes and what to follow up on

No JSON. No preamble. Do not invent questions that are not in the transcript.";

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    host: String,
    cached_model: std::sync::Arc<Mutex<Option<String>>>,
}

impl Default for Client {
    fn default() -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("reqwest client");

        Self {
            http,
            host: resolve_ollama_host(),
            cached_model: std::sync::Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoachStatus {
    pub available: bool,
    pub model: Option<String>,
}

impl Client {
    pub async fn status(&self) -> CoachStatus {
        match self.resolve_model().await {
            Ok(model) => CoachStatus {
                available: true,
                model: Some(model),
            },
            Err(_) => CoachStatus {
                available: false,
                model: None,
            },
        }
    }

    /// Streams a single spoken answer token-by-token as Ollama produces it.
    pub async fn suggest(
        &self,
        transcript: &str,
        mut on_partial: impl FnMut(Draft),
    ) -> OllamaResult<(String, Draft)> {
        let model = self.resolve_model().await?;
        let content = self
            .chat(
                &model,
                SYSTEM_PROMPT,
                transcript,
                ChatOptions {
                    temperature: 0.15,
                    num_predict: 80,
                    num_ctx: 1024,
                    num_batch: 512,
                    num_gpu: 99,
                },
                true,
                None,
                &mut on_partial,
            )
            .await?;
        let draft = parse_reply(&content);
        if draft.answer.is_empty() {
            return Err(OllamaError::UnusableOutput);
        }
        Ok((model, draft))
    }

    pub async fn summarize(&self, transcript: &str) -> OllamaResult<(String, String)> {
        let model = self.resolve_model().await?;
        let mut ignore = |_draft: Draft| {};
        let content = self
            .chat(
                &model,
                SUMMARY_PROMPT,
                &format!("Interview questions, oldest first.\n{transcript}"),
                ChatOptions {
                    temperature: 0.2,
                    num_predict: 420,
                    num_ctx: 4096,
                    num_batch: 512,
                    num_gpu: 99,
                },
                false,
                None,
                &mut ignore,
            )
            .await?;
        let text = collapse_ws(&content);
        if text.len() < 40 {
            return Err(OllamaError::UnusableOutput);
        }
        Ok((model, text))
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub async fn resolve_extract_model(&self) -> OllamaResult<String> {
        let names = self.list_models().await?;
        if let Some(hit) = names.iter().find(|name| extract_model_matches(name)) {
            return Ok(hit.clone());
        }
        Err(OllamaError::MissingExtractModel)
    }

    pub async fn complete_json(
        &self,
        model: &str,
        system: &str,
        user: &str,
        num_predict: u32,
        num_ctx: u32,
    ) -> OllamaResult<String> {
        let url = format!("{}/api/chat", self.host);
        let request = ChatRequest {
            model: model.to_string(),
            stream: false,
            format: Some("json"),
            keep_alive: "60m",
            options: ChatOptions {
                temperature: 0.0,
                num_predict,
                num_ctx,
                num_batch: 512,
                num_gpu: 99,
            },
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system.to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: user.to_string(),
                },
            ],
        };

        let response = self
            .http
            .post(url)
            .timeout(EXTRACT_TIMEOUT)
            .json(&request)
            .send()
            .await
            .map_err(|_| OllamaError::Unavailable {
                host: self.host.clone(),
            })?;

        if !response.status().is_success() {
            return Err(OllamaError::Request(format!(
                "status {} from {}",
                response.status(),
                self.host
            )));
        }

        let body: ChatUnary = response
            .json()
            .await
            .map_err(|err| OllamaError::Request(err.to_string()))?;
        let content = body.message.content.unwrap_or_default();
        if content.trim().is_empty() {
            return Err(OllamaError::UnusableOutput);
        }
        Ok(content)
    }

    async fn resolve_model(&self) -> OllamaResult<String> {
        if let Some(cached) = self.cached_model.lock().unwrap_or_else(|e| e.into_inner()).clone()
        {
            return Ok(cached);
        }

        let names = self.list_models().await?;
        if names.is_empty() {
            return Err(OllamaError::NoModel);
        }

        let chosen = pick_fast_model(&names).unwrap_or_else(|| names[0].clone());
        *self.cached_model.lock().unwrap_or_else(|e| e.into_inner()) = Some(chosen.clone());
        Ok(chosen)
    }

    async fn list_models(&self) -> OllamaResult<Vec<String>> {
        let url = format!("{}/api/tags", self.host);
        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|_| OllamaError::Unavailable {
                host: self.host.clone(),
            })?;

        if !response.status().is_success() {
            return Err(OllamaError::Unavailable {
                host: self.host.clone(),
            });
        }

        let body: TagsResponse = response
            .json()
            .await
            .map_err(|err| OllamaError::Request(err.to_string()))?;

        Ok(body
            .models
            .into_iter()
            .map(|model| model.name)
            .filter(|name| !name.is_empty())
            .collect())
    }

    async fn chat(
        &self,
        model: &str,
        system: &str,
        user: &str,
        options: ChatOptions,
        stream: bool,
        format: Option<&'static str>,
        on_partial: &mut impl FnMut(Draft),
    ) -> OllamaResult<String> {
        let url = format!("{}/api/chat", self.host);
        let request = ChatRequest {
            model: model.to_string(),
            stream,
            format,
            keep_alive: "60m",
            options,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system.to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: user.to_string(),
                },
            ],
        };

        let mut response = self
            .http
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|_| OllamaError::Unavailable {
                host: self.host.clone(),
            })?;

        if !response.status().is_success() {
            return Err(OllamaError::Request(format!(
                "status {}",
                response.status()
            )));
        }

        if !stream {
            let body: ChatUnary = response
                .json()
                .await
                .map_err(|err| OllamaError::Request(err.to_string()))?;
            let content = body.message.content.unwrap_or_default();
            if content.trim().is_empty() {
                return Err(OllamaError::UnusableOutput);
            }
            return Ok(content);
        }

        let mut buffer: Vec<u8> = Vec::new();
        let mut accumulated = String::new();
        let mut last = Draft::default();

        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|err| OllamaError::Request(err.to_string()))?
        {
            buffer.extend_from_slice(&chunk);
            while let Some(newline) = buffer.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=newline).collect();
                let line = String::from_utf8_lossy(&line);
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let Ok(piece) = serde_json::from_str::<ChatStreamChunk>(line) else {
                    continue;
                };
                if let Some(delta) = piece.message.and_then(|m| m.content) {
                    accumulated.push_str(&delta);
                    let draft = parse_partial(&accumulated);
                    if draft != last {
                        last = draft.clone();
                        on_partial(draft);
                    }
                }
            }
        }

        if accumulated.trim().is_empty() {
            return Err(OllamaError::UnusableOutput);
        }
        Ok(accumulated)
    }

    /// Loads the model into memory and keeps it there. The first chat of a
    /// meeting otherwise spends several seconds on a cold Metal load, and any
    /// cancelled request during that load aborts it.
    pub async fn warm(&self) {
        let Ok(model) = self.resolve_model().await else {
            return;
        };

        let url = format!("{}/api/generate", self.host);
        let body = serde_json::json!({
            "model": model,
            "prompt": "ok",
            "stream": false,
            "keep_alive": "60m",
            "options": {
                "num_predict": 1,
                "num_gpu": 99,
                "num_batch": 512
            }
        });

        match self.http.post(url).json(&body).send().await {
            Ok(response) if response.status().is_success() => {
                log::info!("ollama model is warm");
            }
            Ok(_) => log::warn!("ollama warm-up returned an error status"),
            Err(err) => log::warn!("ollama warm-up failed: {err}"),
        }
    }
}

fn collapse_ws(raw: &str) -> String {
    raw.trim().to_string()
}

pub fn resolve_ollama_host() -> String {
    let raw = std::env::var("OLLAMA_HOST")
        .or_else(|_| std::env::var("OLLAMA_URL"))
        .unwrap_or_else(|_| DEFAULT_HOST.to_string());
    normalize_host(&raw)
}

fn normalize_host(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return DEFAULT_HOST.to_string();
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    }
}

fn extract_model_matches(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == EXTRACT_MODEL
        || lower.starts_with("llama3.1:8b")
        || lower == "llama3.1:latest"
        || lower == "llama3.1"
}

/// Prefer a small local model so first tokens arrive quickly.
pub fn pick_fast_model(names: &[String]) -> Option<String> {
    if let Ok(forced) = std::env::var("CODA_MODEL").or_else(|_| std::env::var("GHOSTNOTE_MODEL")) {
        if !forced.is_empty() {
            if let Some(hit) = names.iter().find(|name| {
                *name == &forced || name.starts_with(&format!("{forced}:")) || name.starts_with(&forced)
            }) {
                return Some(hit.clone());
            }
            return Some(forced);
        }
    }

    for preferred in FAST_PREFERRED {
        if let Some(hit) = names.iter().find(|name| {
            *name == preferred
                || name.starts_with(&format!("{preferred}:"))
                || name.split(':').next() == Some(preferred.split(':').next().unwrap_or(preferred))
                    && name.contains(preferred.split(':').nth(1).unwrap_or("\0"))
        }) {
            return Some(hit.clone());
        }
    }

    names
        .iter()
        .min_by_key(|name| model_weight(name))
        .cloned()
}

fn model_weight(name: &str) -> i32 {
    let lower = name.to_ascii_lowercase();
    if lower.contains("70b") || lower.contains("32b") || lower.contains("14b") {
        return 80;
    }
    if lower.contains("13b") || lower.contains("8b") {
        return 40;
    }
    if lower.contains("7b") {
        return 25;
    }
    if lower.contains("3b") || lower.contains("4b") {
        return 10;
    }
    if lower.contains("1b") || lower.contains("2b") || lower.contains("mini") {
        return 5;
    }
    30
}

#[cfg(test)]
mod tests {
    use super::{extract_model_matches, normalize_host, pick_fast_model};

    #[test]
    fn prefers_small_qwen_over_70b() {
        let names = vec!["llama3.1:70b".into(), "qwen2.5:3b".into()];
        assert_eq!(pick_fast_model(&names).as_deref(), Some("qwen2.5:3b"));
    }

    #[test]
    fn host_adds_scheme() {
        assert_eq!(normalize_host("127.0.0.1:11434"), "http://127.0.0.1:11434");
        assert_eq!(
            normalize_host("http://127.0.0.1:11434/"),
            "http://127.0.0.1:11434"
        );
    }

    #[test]
    fn extract_model_accepts_llama31_8b() {
        assert!(extract_model_matches("llama3.1:8b"));
        assert!(extract_model_matches("llama3.1:8b-instruct"));
        assert!(!extract_model_matches("llama3.2:3b"));
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    #[serde(default)]
    name: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    keep_alive: &'static str,
    options: ChatOptions,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatOptions {
    temperature: f32,
    num_predict: u32,
    num_ctx: u32,
    num_batch: u32,
    num_gpu: u32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct ChatStreamChunk {
    message: Option<ChatDelta>,
}

#[derive(Deserialize)]
struct ChatUnary {
    #[serde(default)]
    message: ChatMessageBody,
}

#[derive(Deserialize, Default)]
struct ChatMessageBody {
    #[serde(default)]
    content: Option<String>,
}

type ChatDelta = ChatMessageBody;
