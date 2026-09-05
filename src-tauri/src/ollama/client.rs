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
const PREFERRED_MODEL: &str = "llama3.1";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

const SYSTEM_PROMPT: &str = "\
You are an expert, real-time interview copilot. Your job is to listen to the interviewer's question and provide the user with exactly ONE direct, subtle, and highly appropriate answer.

STRICT RULES:
1. IDENTIFY THE QUESTION: Look at the recent transcript. Identify the core question the interviewer just asked. Ignore small talk or filler words.
2. ONE ANSWER ONLY: Do not provide multiple options. Do not say \"You could say X or Y\". Give exactly ONE confident, direct answer.
3. NO PREAMBLE: Never start with \"Sure\", \"Here is an answer\", \"To answer this\", or \"I suggest\". Start directly with the substance of the answer.
4. CONVERSATIONAL & SUBTLE TONE: Write the answer as if the user is naturally speaking from memory. Use first-person (\"I\", \"my\"). Keep it concise (2-4 sentences max). It should sound like a smart professional recalling a fact, not an AI generating an essay.
5. NO FORMATTING: Do not use bullet points, bold text, or headers unless absolutely strictly necessary for a technical list. Plain text only.

OUTPUT FORMAT:
<detected_question> [Insert the 1-sentence question you identified here] </detected_question>
<answer> [Insert your single, direct, conversational answer here] </answer>";

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
            host: DEFAULT_HOST.to_string(),
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
                    temperature: 0.2,
                    num_predict: 120,
                    num_ctx: 2048,
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

    async fn resolve_model(&self) -> OllamaResult<String> {
        if let Some(cached) = self.cached_model.lock().unwrap_or_else(|e| e.into_inner()).clone()
        {
            return Ok(cached);
        }

        let names = self.list_models().await?;
        if names.is_empty() {
            return Err(OllamaError::NoModel);
        }

        let preferred = names.iter().find(|name| {
            *name == PREFERRED_MODEL || name.starts_with(&format!("{PREFERRED_MODEL}:"))
        });
        let chosen = preferred.cloned().unwrap_or_else(|| names[0].clone());
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
            .map_err(|_| OllamaError::Unavailable)?;

        if !response.status().is_success() {
            return Err(OllamaError::Unavailable);
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
            .map_err(|err| OllamaError::Request(err.to_string()))?;

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
