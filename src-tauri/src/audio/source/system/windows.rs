//! Participant audio on Windows via WASAPI loopback.
//!
//! Windows is the easy case: cpal can open an *output* device in loopback
//! mode, which yields exactly what the speakers are playing. No extra
//! permission, no recording indicator, no virtual driver.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::source::{FrameSink, SourceHandle};
use crate::audio::types::{RawFrames, Speaker};

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

    if let Err(err) = stream.play() {
        let _ = ready.send(Err(AudioError::StreamOpen(err.to_string())));
        return;
    }

    let _ = ready.send(Ok(()));

    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(50));
    }

    drop(stream);
    log::debug!("system-audio capture stopped");
}

fn open_stream(sink: FrameSink) -> AudioResult<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or(AudioError::NoDevice("system audio output"))?;

    // On WASAPI, cpal exposes loopback by building an *input* stream on an
    // output device.
    let supported = device
        .default_output_config()
        .map_err(|err| AudioError::UnsupportedFormat(err.to_string()))?;

    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels();
    let config = supported.config();

    let emit = move |samples: Vec<f32>| {
        sink.send(RawFrames {
            speaker: Speaker::Participant,
            samples,
            sample_rate,
            channels,
        });
    };

    let on_error = |err: cpal::StreamError| {
        log::error!("system-audio stream error: {err}");
    };

    let stream = match supported.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| emit(data.to_vec()),
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| emit(data.iter().map(|s| f32::from(*s) / 32_768.0).collect()),
            on_error,
            None,
        ),
        SampleFormat::I32 => device.build_input_stream(
            &config,
            move |data: &[i32], _| emit(data.iter().map(|s| *s as f32 / 2_147_483_648.0).collect()),
            on_error,
            None,
        ),
        other => {
            return Err(AudioError::UnsupportedFormat(format!(
                "the output device reports an unsupported sample format ({other})"
            )))
        }
    };

    stream.map_err(|err| AudioError::StreamOpen(err.to_string()))
}
