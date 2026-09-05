//! Turning utterances into transcript segments with the local Whisper sidecar.
//!
//! Utterances arrive from the audio pipeline as they are spoken and are
//! transcribed **one at a time** on a background task. Running several Whisper
//! processes concurrently would each load their own copy of the model and
//! contend for the same GPU, making every one of them slower — serial is both
//! simpler and faster here.

pub mod commands;
pub mod error;
pub mod model;
mod sidecar;

use std::collections::{HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use crate::audio::types::{Speaker, Utterance};
use crate::audio::{wav, UtteranceSink};
use crate::ollama::Coach;

pub use error::{TranscribeError, TranscribeResult};

/// Emitted for every recognised utterance.
pub const SEGMENT_EVENT: &str = "ghostnote://transcript-segment";
/// Emitted when transcription fails, so the UI can show a degraded state
/// rather than silently stop producing text.
pub const ERROR_EVENT: &str = "ghostnote://transcript-error";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub id: u64,
    pub speaker: Speaker,
    /// Pre-resolved display label, so the UI never has to map the enum.
    pub speaker_label: &'static str,
    pub text: String,
    /// Milliseconds from the start of capture.
    pub start_ms: u64,
    pub end_ms: u64,
    /// The other person is still talking. Replaced when the next preview or
    /// the finished utterance arrives.
    pub provisional: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptError {
    pub message: String,
}

/// Managed Tauri state owning the transcription worker.
#[derive(Default)]
pub struct Transcriber {
    tx: Mutex<Option<UnboundedSender<Utterance>>>,
    next_id: Arc<AtomicU64>,
}

impl Transcriber {
    /// Starts the worker and returns the sink to hand to the audio engine.
    ///
    /// The queue is unbounded on purpose. Whisper runs faster than realtime,
    /// so the queue only grows during a transient stall — and dropping a
    /// user's words to save a few kilobytes would be the wrong trade.
    pub fn start(&self, app: &AppHandle) -> UtteranceSink {
        let (tx, mut rx) = unbounded_channel::<Utterance>();
        *self.lock() = Some(tx.clone());

        let app = app.clone();
        let next_id = Arc::clone(&self.next_id);

        tauri::async_runtime::spawn(async move {
            // Participant clips jump the queue. The user's mic is transcribed
            // too, but talking points have to land the moment the other side
            // stops — waiting behind a You clip would miss that window.
            let mut them: VecDeque<Utterance> = VecDeque::new();
            let mut recent: Vec<(Speaker, String)> = Vec::new();

            loop {
                if them.is_empty() {
                    match rx.recv().await {
                        Some(utterance) => enqueue(&mut them, utterance),
                        None => break,
                    }
                }

                while let Ok(utterance) = rx.try_recv() {
                    enqueue(&mut them, utterance);
                }

                let Some(utterance) = them.pop_front() else {
                    break;
                };

                // A finished question always beats a preview we already popped.
                // Otherwise Whisper finishes the stale clip, then runs again,
                // and the answer is late.
                let utterance = prefer_finished(&mut them, utterance);

                let id = next_id.fetch_add(1, Ordering::Relaxed);
                match handle(&app, id, utterance).await {
                    Ok(Some(segment)) => {
                        if is_echo(&recent, &segment) {
                            log::debug!("dropped a line that duplicated the other speaker");
                            continue;
                        }
                        recent.push((segment.speaker, segment.text.clone()));
                        if recent.len() > 8 {
                            recent.remove(0);
                        }
                        publish(&app, segment);
                    }
                    Ok(None) => {}
                    Err(err) => report(&app, &err),
                }
            }
            log::debug!("transcription worker stopped");
        });

        Arc::new(move |utterance: Utterance| {
            // Fails only once the worker has shut down, which happens after
            // capture stops; a late utterance at that point is expected.
            let _ = tx.send(utterance);
        })
    }

    /// Stops the worker. Queued utterances already sent are still transcribed.
    pub fn stop(&self) {
        self.lock().take();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Option<UnboundedSender<Utterance>>> {
        self.tx.lock().unwrap_or_else(|err| err.into_inner())
    }
}

fn enqueue(them: &mut VecDeque<Utterance>, utterance: Utterance) {
    if utterance.speaker != Speaker::Participant {
        return;
    }
    them.retain(|queued| !queued.preview);
    if utterance.preview {
        them.push_back(utterance);
    } else {
        them.push_front(utterance);
    }
}

fn prefer_finished(them: &mut VecDeque<Utterance>, current: Utterance) -> Utterance {
    if !current.preview {
        return current;
    }
    if let Some(index) = them.iter().position(|queued| !queued.preview) {
        let finished = them.remove(index).unwrap_or(current);
        them.retain(|queued| !queued.preview);
        finished
    } else {
        current
    }
}

async fn handle(
    app: &AppHandle,
    id: u64,
    utterance: Utterance,
) -> TranscribeResult<Option<TranscriptSegment>> {
    let model = model::resolve(app)?;

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|err| TranscribeError::DataDir(err.to_string()))?
        .join("utterances");

    // `TempAudio` deletes the clip on drop, including on every error path
    // below, so raw meeting audio never outlives the transcription of it.
    let clip = wav::write(&cache_dir, &format!("utterance-{id}"), &utterance.samples)
        .map_err(|err| TranscribeError::Audio(err.to_string()))?;

    let Some(text) = sidecar::transcribe(app, &model, clip.path()).await? else {
        log::debug!("utterance {id} contained no speech");
        return Ok(None);
    };

    Ok(Some(TranscriptSegment {
        id,
        speaker: utterance.speaker,
        speaker_label: utterance.speaker.label(),
        text,
        start_ms: utterance.start_ms,
        end_ms: utterance.end_ms,
        provisional: utterance.preview,
    }))
}

fn publish(app: &AppHandle, segment: TranscriptSegment) {
    // Never log `segment.text` — it is meeting content.
    if let Err(err) = app.emit(SEGMENT_EVENT, &segment) {
        log::warn!("failed to emit transcript segment: {err}");
    }

    if let Some(coach) = app.try_state::<Coach>() {
        coach.consider(app, segment);
    }
}

/// True when this line is the same words we already captured from the other
/// source — acoustic echo on the mic, or the user's voice playing back
/// through the meeting mix.
fn is_echo(recent: &[(Speaker, String)], segment: &TranscriptSegment) -> bool {
    recent.iter().rev().take(4).any(|(speaker, text)| {
        *speaker != segment.speaker && same_words(text, &segment.text)
    })
}

fn same_words(a: &str, b: &str) -> bool {
    let left = words(a);
    let right = words(b);
    if left.len() < 4 || right.len() < 4 {
        return false;
    }
    let overlap = left.intersection(&right).count();
    let shorter = left.len().min(right.len());
    overlap * 100 / shorter >= 70
}

fn words(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|word| word.len() > 1)
        .collect()
}

fn report(app: &AppHandle, err: &TranscribeError) {
    log::error!("transcription failed: {err}");

    let _ = app.emit(
        ERROR_EVENT,
        TranscriptError {
            message: err.to_string(),
        },
    );
}

/// Warms the sidecar in the background at startup. See
/// [`sidecar::warm_up`] for why this matters.
pub fn warm_up(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        sidecar::warm_up(&app).await;
    });
}

#[cfg(test)]
mod tests {
    use super::same_words;

    #[test]
    fn echo_of_the_same_sentence() {
        assert!(same_words(
            "can you walk me through a project you led recently",
            "Can you walk me through a project you led recently?"
        ));
    }

    #[test]
    fn different_questions_are_not_echo() {
        assert!(!same_words(
            "can you walk me through a project you led recently",
            "what is your notice period and when can you start"
        ));
    }
}
