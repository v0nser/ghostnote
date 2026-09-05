//! Capture backends.
//!
//! Two logical sources feed the pipeline, and which OS primitive backs each
//! one differs per platform:
//!
//! | Source            | macOS             | Windows              |
//! |-------------------|-------------------|----------------------|
//! | [`Speaker::You`]  | cpal (microphone) | cpal (microphone)    |
//! | `Participant`     | ScreenCaptureKit  | cpal WASAPI loopback |
//!
//! macOS has no loopback input device, which is why participant audio needs a
//! completely different mechanism there rather than another cpal stream.

pub mod microphone;
pub mod system;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::JoinHandle;

use super::types::RawFrames;

/// Where capture backends push audio.
///
/// Sends are non-blocking and drop on overflow. This is called directly from
/// realtime audio callbacks, where blocking on a slow consumer would stall the
/// device and produce audible glitches in the user's *own* meeting audio.
/// Losing a block of transcription input is by far the lesser evil.
#[derive(Clone)]
pub struct FrameSink {
    tx: SyncSender<RawFrames>,
    dropped: Arc<AtomicU64>,
}

impl FrameSink {
    pub fn new(tx: SyncSender<RawFrames>) -> Self {
        Self {
            tx,
            dropped: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn send(&self, frames: RawFrames) {
        match self.tx.try_send(frames) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.dropped.fetch_add(1, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }

    /// Number of blocks dropped because the pipeline could not keep up.
    /// Surfaced in logs only — a persistently rising count means the machine
    /// cannot sustain transcription in realtime.
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }
}

/// Owns a running capture thread and stops it on drop.
///
/// Each backend owns its stream on a dedicated thread because `cpal::Stream`
/// is `!Send` and `SCStream` wants a stable owner; the handle is the only
/// thing that crosses thread boundaries.
pub struct SourceHandle {
    stop: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl SourceHandle {
    pub fn new(stop: Arc<std::sync::atomic::AtomicBool>, thread: JoinHandle<()>) -> Self {
        Self {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for SourceHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            // A capture thread that will not join is not worth deadlocking the
            // UI over; it exits on the stop flag and holds no shared state.
            if thread.join().is_err() {
                log::error!("a capture thread panicked while shutting down");
            }
        }
    }
}
