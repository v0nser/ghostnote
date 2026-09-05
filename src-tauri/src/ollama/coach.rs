//! Live interview answers, driven by what the interviewer just said.
//!
//! The moment a participant utterance is transcribed, the previous suggestion
//! is discarded and a new one is streamed in. In-flight Ollama calls are
//! aborted so a late reply cannot overwrite the current turn.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;

use super::client::{Client, CoachStatus};
use super::parse::Draft;
use crate::audio::types::Speaker;
use crate::transcribe::TranscriptSegment;

/// Emitted whenever a fresh draft is ready (including mid-stream tokens).
pub const POINTS_EVENT: &str = "ghostnote://talking-points";
/// Emitted when generation starts, finishes, or Ollama becomes unavailable.
pub const STATUS_EVENT: &str = "ghostnote://coach-status";

/// Only enough to let a trailing word land on the same turn. The pause that
/// means "they finished" already happened in the segmenter.
const DEBOUNCE: Duration = Duration::from_millis(40);
const MIN_CUE_CHARS: usize = 24;
const MAX_TURNS: usize = 24;
const MAX_MEETING_TURNS: usize = 80;
/// Ollama only sees this much recent transcript. A full meeting dump is the
/// main reason answers felt slow.
const CONTEXT_WINDOW_MS: u64 = 45_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TalkingPoints {
    pub id: u64,
    /// Last interviewer line, used until the model names the question.
    pub cue: String,
    pub question: String,
    pub answer: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveCoachStatus {
    pub available: bool,
    pub generating: bool,
    /// `idle` | `processing` | `writing`
    pub phase: &'static str,
    pub model: Option<String>,
    pub message: Option<String>,
    pub pending_cue: Option<String>,
}

#[derive(Clone)]
struct Turn {
    speaker: Speaker,
    text: String,
    start_ms: u64,
    end_ms: u64,
}

struct Inner {
    client: Client,
    running: AtomicBool,
    generating: AtomicBool,
    generation: AtomicU64,
    next_id: AtomicU64,
    history: Mutex<Vec<Turn>>,
    inflight: Mutex<Option<oneshot::Sender<()>>>,
    last_cue: Mutex<String>,
    meeting_log: Mutex<Vec<Turn>>,
}

/// Managed Tauri state for the live interview copilot.
#[derive(Clone)]
pub struct Coach {
    inner: Arc<Inner>,
}

impl Default for Coach {
    fn default() -> Self {
        Self {
            inner: Arc::new(Inner {
                client: Client::default(),
                running: AtomicBool::new(false),
                generating: AtomicBool::new(false),
                generation: AtomicU64::new(0),
                next_id: AtomicU64::new(0),
                history: Mutex::new(Vec::new()),
                inflight: Mutex::new(None),
                last_cue: Mutex::new(String::new()),
                meeting_log: Mutex::new(Vec::new()),
            }),
        }
    }
}

impl Coach {
    pub async fn status(&self) -> CoachStatus {
        self.inner.client.status().await
    }

    pub async fn warm_model(&self) {
        self.inner.client.warm().await;
    }

    /// Begins a meeting. Previous talking points are discarded so a new
    /// conversation cannot inherit suggestions from the last one.
    pub fn start(&self, app: &AppHandle) {
        self.inner.running.store(true, Ordering::SeqCst);
        self.abort_inflight();
        self.clear_history();
        emit_points(app, None);
        emit_status(app, idle_status());

        let coach = self.clone();
        tauri::async_runtime::spawn(async move {
            coach.inner.client.warm().await;
        });
    }

    pub fn stop(&self, app: &AppHandle) {
        self.inner.running.store(false, Ordering::SeqCst);
        self.inner.generating.store(false, Ordering::SeqCst);
        self.abort_inflight();
        emit_status(app, idle_status());
    }

    /// VAD just closed the interviewer's turn. Show "Processing…" now — do
    /// not wait for Whisper or Ollama.
    pub fn note_speech_ended(&self, app: &AppHandle) {
        if !self.inner.running.load(Ordering::Relaxed) {
            return;
        }
        self.abort_inflight();
        self.inner.generating.store(true, Ordering::SeqCst);
        emit_points(app, None);
        emit_status(app, processing_status());
    }

    /// Called for every recognised utterance. Participant speech may produce
    /// an answer; the user's speech only updates the context window.
    pub fn consider(&self, app: &AppHandle, segment: TranscriptSegment) {
        if !self.inner.running.load(Ordering::Relaxed) {
            return;
        }

        let text = segment.text.trim();
        if text.is_empty() {
            return;
        }

        self.push_turn(
            Turn {
                speaker: segment.speaker,
                text: text.to_string(),
                start_ms: segment.start_ms,
                end_ms: segment.end_ms,
            },
            segment.provisional,
        );

        if segment.speaker != Speaker::Participant {
            return;
        }

        if segment.provisional {
            return;
        }

        if text.chars().count() < MIN_CUE_CHARS {
            self.finish_idle(app);
            return;
        }

        let cue = self.latest_cue();
        if cue.chars().count() < MIN_CUE_CHARS {
            self.finish_idle(app);
            return;
        }

        *self.inner.last_cue.lock().unwrap_or_else(|err| err.into_inner()) = cue.clone();
        self.start_generate(app, cue);
    }

    fn start_generate(&self, app: &AppHandle, cue: String) {
        let generation = self.inner.generation.fetch_add(1, Ordering::SeqCst) + 1;
        let coach = self.clone();
        let app = app.clone();
        let (cancel_tx, cancel_rx) = oneshot::channel();
        *self.inner.inflight.lock().unwrap_or_else(|err| err.into_inner()) = Some(cancel_tx);

        tauri::async_runtime::spawn(async move {
            tokio::select! {
                _ = cancel_rx => {}
                _ = async {
                    tokio::time::sleep(DEBOUNCE).await;
                    if generation != coach.inner.generation.load(Ordering::SeqCst) {
                        return;
                    }
                    if !coach.inner.running.load(Ordering::Relaxed) {
                        return;
                    }
                    coach.inner.generating.store(true, Ordering::SeqCst);
                    emit_status(&app, writing_status(Some(cue.clone())));
                    coach.generate(&app, generation, cue).await;
                } => {}
            }
        });
    }

    async fn generate(&self, app: &AppHandle, generation: u64, cue: String) {
        let transcript = self.render_context();
        if transcript.trim().is_empty() {
            self.finish_idle(app);
            return;
        }

        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let mut last = Draft::default();

        let result = self
            .inner
            .client
            .suggest(&transcript, |draft| {
                if generation != self.inner.generation.load(Ordering::SeqCst) {
                    return;
                }
                last = draft.clone();
                emit_points(
                    app,
                    Some(TalkingPoints {
                        id,
                        cue: cue.clone(),
                        question: draft.question,
                        answer: draft.answer,
                        model: String::new(),
                    }),
                );
            })
            .await;

        if generation != self.inner.generation.load(Ordering::SeqCst) {
            return;
        }

        self.inner.generating.store(false, Ordering::SeqCst);

        match result {
            Ok((model, draft)) => {
                emit_points(
                    app,
                    Some(TalkingPoints {
                        id,
                        cue,
                        question: draft.question,
                        answer: draft.answer,
                        model: model.clone(),
                    }),
                );
                emit_status(
                    app,
                    LiveCoachStatus {
                        available: true,
                        generating: false,
                        phase: "idle",
                        model: Some(model),
                        message: None,
                        pending_cue: None,
                    },
                );
            }
            Err(err) => {
                if !last.answer.is_empty() {
                    emit_status(app, idle_status());
                    return;
                }

                log::warn!("talking-points generation failed: {err}");
                emit_status(
                    app,
                    LiveCoachStatus {
                        available: !matches!(
                            err,
                            super::error::OllamaError::Unavailable | super::error::OllamaError::NoModel
                        ),
                        generating: false,
                        phase: "idle",
                        model: None,
                        message: Some(err.to_string()),
                        pending_cue: None,
                    },
                );
            }
        }
    }

    fn latest_cue(&self) -> String {
        self.lock_history()
            .iter()
            .rev()
            .find(|turn| turn.speaker == Speaker::Participant)
            .map(|turn| turn.text.clone())
            .unwrap_or_default()
    }

    fn push_turn(&self, turn: Turn, provisional: bool) {
        let mut history = self.lock_history();
        if turn.speaker == Speaker::Participant {
            if let Some(last) = history.last_mut() {
                if last.speaker == Speaker::Participant && same_question(&last.text, &turn.text)
                {
                    merge_interviewer(&mut last.text, &turn.text, provisional);
                    last.end_ms = last.end_ms.max(turn.end_ms);
                    if turn.start_ms > 0 {
                        last.start_ms = last.start_ms.min(turn.start_ms);
                    }
                    drop(history);
                    self.append_meeting_log(&turn, true);
                    return;
                }
            }
        }
        history.push(turn.clone());
        let extra = history.len().saturating_sub(MAX_TURNS);
        if extra > 0 {
            history.drain(..extra);
        }
        drop(history);
        if turn.speaker == Speaker::Participant {
            self.append_meeting_log(&turn, false);
        }
    }

    fn append_meeting_log(&self, turn: &Turn, merge_last: bool) {
        let mut log = self
            .inner
            .meeting_log
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        if merge_last {
            if let Some(last) = log.last_mut() {
                merge_interviewer(&mut last.text, &turn.text, false);
                last.end_ms = last.end_ms.max(turn.end_ms);
            } else {
                log.push(turn.clone());
            }
        } else {
            log.push(turn.clone());
        }
        let extra = log.len().saturating_sub(MAX_MEETING_TURNS);
        if extra > 0 {
            log.drain(..extra);
        }
    }

    fn render_context(&self) -> String {
        let history = self.lock_history();
        render_recent(&history, CONTEXT_WINDOW_MS)
    }

    fn render_meeting(&self) -> String {
        self.inner
            .meeting_log
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .iter()
            .map(|turn| format!("Interviewer: {}", turn.text))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub async fn summarize(&self) -> super::error::OllamaResult<MeetingSummary> {
        let transcript = self.render_meeting();
        if transcript.trim().len() < MIN_CUE_CHARS {
            return Err(super::error::OllamaError::Request(
                "nothing to summarize yet — record some questions first".into(),
            ));
        }
        let (model, text) = self.inner.client.summarize(&transcript).await?;
        Ok(MeetingSummary { text, model })
    }

    fn abort_inflight(&self) {
        self.inner.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(cancel) = self
            .inner
            .inflight
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .take()
        {
            let _ = cancel.send(());
        }
    }

    fn finish_idle(&self, app: &AppHandle) {
        self.inner.generating.store(false, Ordering::SeqCst);
        emit_status(app, idle_status());
    }

    fn clear_history(&self) {
        self.lock_history().clear();
        self.inner
            .last_cue
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();
        self.inner
            .meeting_log
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();
    }

    fn lock_history(&self) -> std::sync::MutexGuard<'_, Vec<Turn>> {
        self.inner
            .history
            .lock()
            .unwrap_or_else(|err| err.into_inner())
    }
}

fn idle_status() -> LiveCoachStatus {
    LiveCoachStatus {
        available: true,
        generating: false,
        phase: "idle",
        model: None,
        message: None,
        pending_cue: None,
    }
}

fn processing_status() -> LiveCoachStatus {
    LiveCoachStatus {
        available: true,
        generating: true,
        phase: "processing",
        model: None,
        message: None,
        pending_cue: None,
    }
}

fn writing_status(cue: Option<String>) -> LiveCoachStatus {
    LiveCoachStatus {
        available: true,
        generating: true,
        phase: "writing",
        model: None,
        message: None,
        pending_cue: cue,
    }
}

fn emit_points(app: &AppHandle, points: Option<TalkingPoints>) {
    let _ = app.emit(POINTS_EVENT, points);
}

fn emit_status(app: &AppHandle, status: LiveCoachStatus) {
    let _ = app.emit(STATUS_EVENT, status);
}

/// Last 45 seconds of transcript, oldest first. Nothing older is sent.
fn render_recent(turns: &[Turn], window_ms: u64) -> String {
    let latest = turns.iter().map(|turn| turn.end_ms).max().unwrap_or(0);
    let cutoff = latest.saturating_sub(window_ms);
    let lines: Vec<String> = turns
        .iter()
        .filter(|turn| turn.end_ms >= cutoff)
        .map(|turn| {
            let who = match turn.speaker {
                Speaker::You => "You",
                Speaker::Participant => "Interviewer",
            };
            format!("{who}: {}", turn.text)
        })
        .collect();
    lines.join("\n")
}

/// Merge a later clip of the same interviewer turn into the text we already have.
fn merge_interviewer(existing: &mut String, incoming: &str, replace: bool) {
    let incoming = incoming.trim();
    if incoming.is_empty() {
        return;
    }
    if existing.is_empty() || replace || incoming.len() >= existing.len() {
        *existing = incoming.to_string();
        return;
    }
    if incoming.starts_with(existing.as_str()) {
        *existing = incoming.to_string();
        return;
    }
    if existing.contains(incoming) {
        return;
    }
    *existing = incoming.to_string();
}

/// True when `next` is the same question still being transcribed, not a new one.
fn same_question(previous: &str, next: &str) -> bool {
    let prev = previous.trim().to_lowercase();
    let nxt = next.trim().to_lowercase();
    if prev.is_empty() || nxt.is_empty() {
        return false;
    }
    if nxt.starts_with(&prev) || prev.starts_with(&nxt) || prev.contains(&nxt) || nxt.contains(&prev)
    {
        return true;
    }

    let prev_words: std::collections::HashSet<&str> = prev.split_whitespace().collect();
    let next_words: std::collections::HashSet<&str> = nxt.split_whitespace().collect();
    if prev_words.len() < 4 || next_words.len() < 4 {
        return false;
    }
    let overlap = prev_words.intersection(&next_words).count();
    let shorter = prev_words.len().min(next_words.len());
    overlap * 100 / shorter >= 60
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingSummary {
    pub text: String,
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::{merge_interviewer, render_recent, same_question, Turn, CONTEXT_WINDOW_MS};
    use crate::audio::types::Speaker;

    #[test]
    fn growing_question_replaces() {
        let mut text = "can you walk through".to_string();
        merge_interviewer(&mut text, "can you walk through a project you led", true);
        assert_eq!(text, "can you walk through a project you led");
    }

    #[test]
    fn longer_transcription_wins() {
        let mut text = "can you walk through".to_string();
        merge_interviewer(&mut text, "can you walk through a project you led", false);
        assert_eq!(text, "can you walk through a project you led");
    }

    #[test]
    fn javascript_is_not_the_same_question_as_space() {
        assert!(!same_question(
            "can you explain closures in javascript",
            "welcome to our documentary on space exploration"
        ));
    }

    #[test]
    fn growing_whisper_is_the_same_question() {
        assert!(same_question(
            "can you explain closures",
            "can you explain closures in javascript"
        ));
    }

    #[test]
    fn context_keeps_only_the_last_45_seconds() {
        let turns = vec![
            Turn {
                speaker: Speaker::Participant,
                text: "old question about rust".into(),
                start_ms: 0,
                end_ms: 1_000,
            },
            Turn {
                speaker: Speaker::Participant,
                text: "can you explain closures in javascript".into(),
                start_ms: 80_000,
                end_ms: 82_000,
            },
        ];
        let rendered = render_recent(&turns, CONTEXT_WINDOW_MS);
        assert!(rendered.contains("closures"));
        assert!(!rendered.contains("rust"));
    }
}
