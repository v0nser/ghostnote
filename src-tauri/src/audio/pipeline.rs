//! The single worker thread that turns raw capture blocks into utterances.
//!
//! Everything after the device callbacks happens here, on one thread, so the
//! realtime audio threads stay free of allocation-heavy work and no locking is
//! needed around the resampler or segmenter state.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use super::gate::{EchoGate, MicDelay};
use super::resample::Resampler;
use super::segmenter::Segmenter;
use super::types::{LevelEvent, RawFrames, Speaker, Utterance, VadEvent};
use crate::ollama::Coach;

/// Event carrying level-meter data to the UI.
pub const LEVEL_EVENT: &str = "coda://audio-level";
/// Interviewer started or stopped speaking. Fires before Whisper runs.
pub const VAD_EVENT: &str = "coda://vad";

/// How often the level meter updates. Fast enough to look live, slow enough
/// that we are not spamming the webview's IPC channel during a long meeting.
const LEVEL_INTERVAL: Duration = Duration::from_millis(100);

/// Where completed utterances go. The transcription layer supplies this, which
/// keeps the audio module unaware of Whisper entirely.
pub type UtteranceSink = Arc<dyn Fn(Utterance) + Send + Sync>;

pub struct Pipeline {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Pipeline {
    pub fn spawn(
        rx: Receiver<RawFrames>,
        app: AppHandle,
        sink: UtteranceSink,
        transcribe_microphone: bool,
        capture_system_audio: bool,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);

        let thread = std::thread::Builder::new()
            .name("coda-audio-pipeline".into())
            .spawn(move || {
                run(
                    rx,
                    app,
                    sink,
                    thread_stop,
                    transcribe_microphone,
                    capture_system_audio,
                )
            })
            .expect("failed to spawn the audio pipeline thread");

        Self {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                log::error!("the audio pipeline thread panicked");
            }
        }
    }
}

fn run(
    rx: Receiver<RawFrames>,
    app: AppHandle,
    sink: UtteranceSink,
    stop: Arc<AtomicBool>,
    transcribe_microphone: bool,
    capture_system_audio: bool,
) {
    let mut resamplers: HashMap<Speaker, Resampler> = HashMap::new();
    let voice_only = transcribe_microphone && !capture_system_audio;
    let mut segmenters: HashMap<Speaker, Segmenter> = Speaker::ALL
        .iter()
        .map(|speaker| {
            let segmenter = if voice_only && *speaker == Speaker::You {
                Segmenter::voice_command(*speaker)
            } else {
                Segmenter::new(*speaker)
            };
            (*speaker, segmenter)
        })
        .collect();
    let mut peaks: HashMap<Speaker, f32> = HashMap::new();
    let mut last_level = Instant::now();
    let mut gate = EchoGate::new();
    let mut mic_delay = MicDelay::new();

    while !stop.load(Ordering::Relaxed) {
        match rx.recv_timeout(LEVEL_INTERVAL) {
            Ok(frames) => {
                let speaker = frames.speaker;

                let peak = frames.samples.iter().fold(0.0f32, |acc, s| acc.max(s.abs()));
                let slot = peaks.entry(speaker).or_insert(0.0);
                *slot = slot.max(peak);

                // A device can change format mid-session (switching from the
                // built-in mic to a headset), so the resampler is rebuilt
                // whenever the incoming format stops matching.
                let resampler = resamplers.entry(speaker).or_insert_with(|| {
                    Resampler::new(frames.sample_rate, frames.channels)
                });
                if !resampler.matches(frames.sample_rate, frames.channels) {
                    *resampler = Resampler::new(frames.sample_rate, frames.channels);
                }

                let mono = resampler.process(&frames.samples);
                match speaker {
                    Speaker::Participant => {
                        consume(
                            speaker,
                            mono,
                            &mut gate,
                            &mut segmenters,
                            &sink,
                            &app,
                            transcribe_microphone,
                        );
                    }
                    Speaker::You => {
                        // Voice commands have no system-audio tap, so the
                        // echo-alignment delay only adds lag.
                        let samples = if capture_system_audio {
                            mic_delay.push(&mono)
                        } else {
                            mono
                        };
                        if !samples.is_empty() {
                            consume(
                                speaker,
                                samples,
                                &mut gate,
                                &mut segmenters,
                                &sink,
                                &app,
                                transcribe_microphone,
                            );
                        }
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        if last_level.elapsed() >= LEVEL_INTERVAL {
            emit_levels(&app, &mut peaks);
            last_level = Instant::now();
        }
    }

    let leftover = mic_delay.flush();
    if !leftover.is_empty() {
        consume(
            Speaker::You,
            leftover,
            &mut gate,
            &mut segmenters,
            &sink,
            &app,
            transcribe_microphone,
        );
    }
    flush(
        &mut resamplers,
        &mut segmenters,
        &sink,
        &app,
        transcribe_microphone,
    );
    log::debug!("audio pipeline stopped");
}

fn consume(
    speaker: Speaker,
    mut samples: Vec<f32>,
    gate: &mut EchoGate,
    segmenters: &mut HashMap<Speaker, Segmenter>,
    sink: &UtteranceSink,
    app: &AppHandle,
    transcribe_microphone: bool,
) {
    gate.filter(speaker, &mut samples);

    let Some(segmenter) = segmenters.get_mut(&speaker) else {
        return;
    };
    for utterance in segmenter.push(&samples) {
        // Meetings keep the mic for echo gating only. Voice commands need
        // those same clips sent to Whisper.
        if speaker == Speaker::You && !transcribe_microphone {
            continue;
        }
        if !utterance.preview {
            signal_turn_ended(app, speaker);
        }
        sink(utterance);
    }
    if speaker == Speaker::Participant {
        if let Some(preview) = segmenter.take_preview() {
            sink(preview);
        }
    }
}

/// Drains the resamplers and segmenters so the last few words spoken before
/// the user hit stop are not silently discarded.
fn flush(
    resamplers: &mut HashMap<Speaker, Resampler>,
    segmenters: &mut HashMap<Speaker, Segmenter>,
    sink: &UtteranceSink,
    app: &AppHandle,
    transcribe_microphone: bool,
) {
    for (speaker, segmenter) in segmenters.iter_mut() {
        if *speaker == Speaker::You && !transcribe_microphone {
            let _ = resamplers.get_mut(speaker).map(|resampler| resampler.flush());
            let _ = segmenter.flush();
            continue;
        }
        if let Some(resampler) = resamplers.get_mut(speaker) {
            for utterance in segmenter.push(&resampler.flush()) {
                signal_turn_ended(app, *speaker);
                sink(utterance);
            }
        }

        if let Some(utterance) = segmenter.flush() {
            signal_turn_ended(app, *speaker);
            sink(utterance);
        }
    }
}

fn signal_turn_ended(app: &AppHandle, speaker: Speaker) {
    let _ = app.emit(
        VAD_EVENT,
        VadEvent {
            speaker,
            speaking: false,
        },
    );
    if let Some(coach) = app.try_state::<Coach>() {
        coach.note_speech_ended(app);
    }
}

fn emit_levels(app: &AppHandle, peaks: &mut HashMap<Speaker, f32>) {
    for speaker in Speaker::ALL {
        let peak = peaks.remove(&speaker).unwrap_or(0.0);
        // Emission failures mean the window is gone; capture continues.
        let _ = app.emit(LEVEL_EVENT, LevelEvent { speaker, peak });
    }
}
