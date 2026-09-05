//! Audio capture.
//!
//! ```text
//!   microphone (cpal) ─┐
//!                      ├─► FrameSink ─► Pipeline ─► Utterance ─► transcription
//!   system audio ──────┘   (bounded,     resample
//!   (SCK / WASAPI)          lossy)       + segment
//! ```
//!
//! Two design rules run through the whole module:
//!
//! 1. **Nothing blocks a device callback.** The sink drops blocks rather than
//!    apply backpressure, because stalling a capture callback glitches the
//!    audio the user's meeting app is also reading.
//! 2. **Losing one source degrades, it does not fail.** If system audio cannot
//!    start, capture continues microphone-only with the reason reported, since
//!    half a transcript beats none in the middle of a meeting.

pub mod commands;
pub mod error;
mod gate;
mod pipeline;
mod resample;
mod segmenter;
mod vad;
mod source;
pub mod types;
pub mod wav;

use std::sync::mpsc::sync_channel;
use std::sync::Mutex;
use std::time::Instant;

use serde::Deserialize;
use tauri::AppHandle;

pub use error::{AudioError, AudioResult};
pub use pipeline::{UtteranceSink, LEVEL_EVENT, VAD_EVENT};
pub use types::{AudioDevice, CaptureStatus, Speaker, Utterance, TARGET_SAMPLE_RATE};

/// Capture blocks buffered between the device callbacks and the pipeline.
/// At typical 10 ms blocks this is ~2.5 s of slack, which absorbs a GC pause
/// or a slow disk without dropping audio; beyond that the machine is simply
/// too slow and dropping is the correct response.
const QUEUE_CAPACITY: usize = 256;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOptions {
    /// Microphone to record, or `None` for the system default.
    #[serde(default)]
    pub microphone_device_id: Option<String>,
    /// Whether to also capture participant audio.
    #[serde(default = "default_true")]
    pub capture_system_audio: bool,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            microphone_device_id: None,
            capture_system_audio: true,
        }
    }
}

const fn default_true() -> bool {
    true
}

/// A capture session in progress. Dropping this stops every thread it owns,
/// in order: sources first (so no new audio arrives), then the pipeline (which
/// flushes whatever is still buffered).
struct Session {
    _microphone: source::SourceHandle,
    _system: Option<source::SourceHandle>,
    _pipeline: pipeline::Pipeline,
    /// Retained purely to read the drop counter when the session ends.
    sink: source::FrameSink,
    started_at: Instant,
    system_audio: bool,
    system_audio_error: Option<String>,
}

/// Managed Tauri state owning the capture session.
#[derive(Default)]
pub struct AudioEngine {
    session: Mutex<Option<Session>>,
}

impl AudioEngine {
    pub fn status(&self) -> CaptureStatus {
        let session = self.lock();

        match session.as_ref() {
            Some(session) => CaptureStatus {
                running: true,
                microphone: true,
                system_audio: session.system_audio,
                system_audio_error: session.system_audio_error.clone(),
                elapsed_ms: session.started_at.elapsed().as_millis() as u64,
            },
            None => CaptureStatus::default(),
        }
    }

    /// Starts capture.
    ///
    /// The microphone is mandatory: if it cannot be opened the whole call
    /// fails, because a meeting recording with no user audio is not useful.
    /// System audio is best-effort.
    pub fn start(
        &self,
        app: &AppHandle,
        options: CaptureOptions,
        sink: UtteranceSink,
    ) -> AudioResult<CaptureStatus> {
        let mut slot = self.lock();
        if slot.is_some() {
            return Err(AudioError::AlreadyRunning);
        }

        let (tx, rx) = sync_channel(QUEUE_CAPACITY);
        let frame_sink = source::FrameSink::new(tx);

        let microphone = source::microphone::spawn(
            options.microphone_device_id.clone(),
            frame_sink.clone(),
        )?;

        let (system, system_audio_error) = if options.capture_system_audio {
            match source::system::spawn(frame_sink.clone()) {
                Ok(handle) => (Some(handle), None),
                Err(err) => {
                    log::warn!("continuing without system audio: {err}");
                    (None, Some(err.to_string()))
                }
            }
        } else {
            (None, None)
        };

        let pipeline = pipeline::Pipeline::spawn(rx, app.clone(), sink);

        *slot = Some(Session {
            _microphone: microphone,
            system_audio: system.is_some(),
            _system: system,
            _pipeline: pipeline,
            sink: frame_sink,
            started_at: Instant::now(),
            system_audio_error,
        });

        drop(slot);
        log::info!("audio capture started");
        Ok(self.status())
    }

    /// Stops capture and flushes any speech still in the segmenter.
    pub fn stop(&self) -> AudioResult<CaptureStatus> {
        let session = self.lock().take().ok_or(AudioError::NotRunning)?;

        // A non-zero count means the machine could not keep up with realtime
        // and some audio never reached the transcriber. Worth surfacing in
        // logs, since the symptom (a gap in the transcript) looks like a
        // transcription bug rather than a capacity problem.
        let dropped = session.sink.dropped();
        if dropped > 0 {
            log::warn!("{dropped} audio blocks were dropped during this session");
        }

        // Explicit rather than implicit: sources must stop before the pipeline
        // so the flush at the end of the pipeline sees a settled stream.
        drop(session);

        log::info!("audio capture stopped");
        Ok(CaptureStatus::default())
    }

    /// See [`crate::stealth::StealthManager::lock`] — a poisoned mutex here
    /// only means a previous holder panicked, and the guarded state is plain
    /// data, so recovering beats propagating the panic into the UI.
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<Session>> {
        self.session.lock().unwrap_or_else(|err| err.into_inner())
    }
}

/// Input devices available for microphone capture.
pub fn input_devices() -> AudioResult<Vec<AudioDevice>> {
    source::microphone::devices()
}

/// Whether this platform can capture participant audio, and what it needs.
pub fn system_audio_support() -> (bool, Option<&'static str>) {
    (source::system::SUPPORTED, source::system::REQUIRED_PERMISSION)
}
