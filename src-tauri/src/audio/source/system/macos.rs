//! Participant audio on macOS via ScreenCaptureKit.
//!
//! macOS exposes no loopback input device, so unlike Windows there is no cpal
//! path here. ScreenCaptureKit's `SCStream` can deliver system audio alongside
//! (or instead of) video, which is the supported way to hear what the meeting
//! app is playing.
//!
//! Two consequences the rest of the app has to live with:
//!
//! - It requires **Screen Recording** permission, and macOS shows the orange
//!   recording indicator in the menu bar while the stream is live. GhostNote's
//!   window stays out of the captured frame, but the indicator itself is
//!   visible to anyone looking at the user's screen.
//! - `SCStream` always captures a display, even when only audio is wanted. We
//!   therefore request the smallest legal frame size and never register a
//!   video handler, so the frames are discarded before they cost anything.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use screencapturekit::cm::AudioBufferList;
use screencapturekit::prelude::*;

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::source::{FrameSink, SourceHandle};
use crate::audio::types::{RawFrames, Speaker};

/// ScreenCaptureKit delivers float PCM at whatever rate we ask for. 48 kHz is
/// the system mixer's native rate, so asking for it avoids a resample inside
/// CoreAudio before our own.
const CAPTURE_SAMPLE_RATE: u32 = 48_000;
const CAPTURE_CHANNELS: u16 = 2;

/// Smallest frame we can ask for. We never read the video, but the stream
/// requires a display and a size.
const MIN_FRAME_DIMENSION: u32 = 2;

pub fn spawn(sink: FrameSink) -> AudioResult<SourceHandle> {
    let stop = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready_rx) = mpsc::channel::<AudioResult<()>>();

    let thread_stop = Arc::clone(&stop);
    let thread = std::thread::Builder::new()
        .name("ghostnote-system-audio".into())
        .spawn(move || run(sink, thread_stop, ready_tx))
        .map_err(|err| AudioError::StreamOpen(err.to_string()))?;

    match ready_rx.recv() {
        Ok(Ok(())) => Ok(SourceHandle::new(stop, thread)),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(AudioError::StreamOpen(
            "the system-audio thread exited before it started".into(),
        )),
    }
}

fn run(sink: FrameSink, stop: Arc<AtomicBool>, ready: mpsc::Sender<AudioResult<()>>) {
    let stream = match open_stream(sink) {
        Ok(stream) => stream,
        Err(err) => {
            let _ = ready.send(Err(err));
            return;
        }
    };

    if let Err(err) = stream.start_capture() {
        let _ = ready.send(Err(classify(&err)));
        return;
    }

    let _ = ready.send(Ok(()));

    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(50));
    }

    if let Err(err) = stream.stop_capture() {
        log::error!("failed to stop the system-audio stream: {err}");
    }
    log::debug!("system-audio capture stopped");
}

fn open_stream(sink: FrameSink) -> AudioResult<SCStream> {
    // This is the call that triggers the Screen Recording permission check.
    let content = SCShareableContent::get().map_err(|err| classify(&err))?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or(AudioError::NoDevice("display"))?;

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build();

    let config = SCStreamConfiguration::new()
        .with_width(MIN_FRAME_DIMENSION)
        .with_height(MIN_FRAME_DIMENSION)
        .with_captures_audio(true)
        .with_sample_rate(CAPTURE_SAMPLE_RATE as i32)
        .with_channel_count(i32::from(CAPTURE_CHANNELS))
        // Without this we would hear ourselves: any audio GhostNote plays
        // would be captured, transcribed and fed back into the meeting notes.
        .with_excludes_current_process_audio(true);

    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(AudioTap { sink }, SCStreamOutputType::Audio);

    Ok(stream)
}

struct AudioTap {
    sink: FrameSink,
}

impl SCStreamOutputTrait for AudioTap {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, of_type: SCStreamOutputType) {
        if of_type != SCStreamOutputType::Audio {
            return;
        }

        let Some(list) = sample.audio_buffer_list() else {
            return;
        };

        let samples = downmix(&list);
        if samples.is_empty() {
            return;
        }

        self.sink.send(RawFrames {
            speaker: Speaker::Participant,
            samples,
            sample_rate: CAPTURE_SAMPLE_RATE,
            // Already mixed to mono below, so the pipeline sees one channel.
            channels: 1,
        });
    }
}

/// Collapses a CoreAudio buffer list to mono `f32`.
///
/// CoreAudio may hand back either layout, and which one you get depends on the
/// stream description rather than anything we control:
///
/// - **planar** — one buffer per channel, each single-channel
/// - **interleaved** — a single buffer with all channels woven together
fn downmix(list: &AudioBufferList) -> Vec<f32> {
    let count = list.num_buffers();
    if count == 0 {
        return Vec::new();
    }

    if count > 1 {
        // Planar: one buffer per channel, summed into mono.
        let buffers: Vec<&[u8]> = (0..count)
            .filter_map(|index| list.buffer(index).map(|buffer| buffer.data()))
            .collect();

        let frames = buffers.iter().map(|data| data.len() / 4).min().unwrap_or(0);
        let scale = 1.0 / buffers.len() as f32;

        let mut mono = vec![0.0f32; frames];
        for data in &buffers {
            for (slot, chunk) in mono.iter_mut().zip(data.chunks_exact(4)) {
                *slot += read_f32(chunk) * scale;
            }
        }
        return mono;
    }

    // Interleaved (or genuinely mono).
    let Some(buffer) = list.buffer(0) else {
        return Vec::new();
    };
    let channels = buffer.number_channels().max(1) as usize;
    let data = buffer.data();

    if channels == 1 {
        return data.chunks_exact(4).map(read_f32).collect();
    }

    let scale = 1.0 / channels as f32;
    data.chunks_exact(4 * channels)
        .map(|frame| frame.chunks_exact(4).map(read_f32).sum::<f32>() * scale)
        .collect()
}

fn read_f32(bytes: &[u8]) -> f32 {
    f32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// ScreenCaptureKit reports a missing TCC grant as an ordinary stream error.
/// Recognising it matters because the remedy is completely different from a
/// transient failure: the user has to visit System Settings.
fn classify(err: &SCError) -> AudioError {
    let message = err.to_string();
    let lowered = message.to_lowercase();

    if lowered.contains("permission")
        || lowered.contains("declined")
        || lowered.contains("not authorized")
        || lowered.contains("unauthorized")
    {
        AudioError::PermissionDenied
    } else {
        AudioError::StreamOpen(message)
    }
}
