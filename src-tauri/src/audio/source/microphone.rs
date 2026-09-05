//! Microphone capture via cpal. This is the [`Speaker::You`] side of a
//! meeting, and it works identically on every desktop platform.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig, SupportedStreamConfig};

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::types::{AudioDevice, RawFrames, Speaker};

use super::{FrameSink, SourceHandle};

/// Lists the input devices the user can choose between.
///
/// The id is cpal's `DeviceId`, which is stable across reboots and
/// reconnections, so a saved preference survives unplugging a headset. The
/// display name is deliberately kept separate, since it is neither stable nor
/// unique.
pub fn devices() -> AudioResult<Vec<AudioDevice>> {
    let host = cpal::default_host();
    let default_id = host
        .default_input_device()
        .and_then(|device| stable_id(&device));

    let devices = host
        .input_devices()
        .map_err(|err| AudioError::UnsupportedFormat(err.to_string()))?
        .filter_map(|device| {
            let id = stable_id(&device)?;
            let name = device
                .description()
                .map(|description| description.name().to_string())
                .unwrap_or_else(|_| id.clone());

            Some(AudioDevice {
                is_default: Some(&id) == default_id.as_ref(),
                id,
                name,
            })
        })
        .collect();

    Ok(devices)
}

fn stable_id(device: &Device) -> Option<String> {
    device.id().ok().map(|id| id.to_string())
}

/// Starts microphone capture on its own thread.
///
/// Returns once the stream is confirmed running, so a failure here (no device,
/// permission denied, unsupported format) surfaces to the caller instead of
/// silently producing no audio.
pub fn spawn(device_id: Option<String>, sink: FrameSink) -> AudioResult<SourceHandle> {
    let stop = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready_rx) = mpsc::channel::<AudioResult<()>>();

    let thread_stop = Arc::clone(&stop);
    let thread = std::thread::Builder::new()
        .name("ghostnote-microphone".into())
        .spawn(move || run(device_id, sink, thread_stop, ready_tx))
        .map_err(|err| AudioError::StreamOpen(err.to_string()))?;

    match ready_rx.recv() {
        Ok(Ok(())) => Ok(SourceHandle::new(stop, thread)),
        Ok(Err(err)) => Err(err),
        Err(_) => Err(AudioError::StreamOpen(
            "the microphone thread exited before it started".into(),
        )),
    }
}

fn run(
    device_id: Option<String>,
    sink: FrameSink,
    stop: Arc<AtomicBool>,
    ready: mpsc::Sender<AudioResult<()>>,
) {
    let stream = match open_stream(device_id.as_deref(), sink) {
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

    // The stream runs on cpal's own thread; this one exists purely to own it,
    // because dropping a `cpal::Stream` is what stops capture.
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(50));
    }

    drop(stream);
    log::debug!("microphone capture stopped");
}

fn open_stream(device_id: Option<&str>, sink: FrameSink) -> AudioResult<cpal::Stream> {
    let device = resolve_device(device_id)?;
    let supported = device
        .default_input_config()
        .map_err(|err| AudioError::UnsupportedFormat(err.to_string()))?;

    let sample_rate = supported.sample_rate();
    let channels = supported.channels();
    let config: StreamConfig = supported.config();

    let emit = move |samples: Vec<f32>| {
        sink.send(RawFrames {
            speaker: Speaker::You,
            samples,
            sample_rate,
            channels,
        });
    };

    let on_error = |err: cpal::StreamError| {
        // Losing the microphone mid-meeting must not take the app down; the
        // pipeline simply stops receiving `You` audio.
        log::error!("microphone stream error: {err}");
    };

    build_stream(&device, &config, &supported, emit, on_error)
}

/// cpal hands us whatever native format the device uses. Everything is
/// normalised to `f32` in `-1.0..=1.0` at the boundary so the rest of the
/// pipeline only ever deals with one representation.
fn build_stream(
    device: &Device,
    config: &StreamConfig,
    supported: &SupportedStreamConfig,
    emit: impl Fn(Vec<f32>) + Send + 'static,
    on_error: impl Fn(cpal::StreamError) + Send + 'static,
) -> AudioResult<cpal::Stream> {
    let format = supported.sample_format();

    let stream = match format {
        SampleFormat::F32 => device.build_input_stream(
            config,
            move |data: &[f32], _| emit(data.to_vec()),
            on_error,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            config,
            move |data: &[i16], _| {
                emit(data.iter().map(|s| f32::from(*s) / 32_768.0).collect())
            },
            on_error,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            config,
            move |data: &[u16], _| {
                emit(data
                    .iter()
                    .map(|s| (f32::from(*s) - 32_768.0) / 32_768.0)
                    .collect())
            },
            on_error,
            None,
        ),
        SampleFormat::I32 => device.build_input_stream(
            config,
            move |data: &[i32], _| {
                emit(data.iter().map(|s| *s as f32 / 2_147_483_648.0).collect())
            },
            on_error,
            None,
        ),
        other => {
            return Err(AudioError::UnsupportedFormat(format!(
                "the microphone reports an unsupported sample format ({other})"
            )))
        }
    };

    stream.map_err(|err| AudioError::StreamOpen(err.to_string()))
}

fn resolve_device(device_id: Option<&str>) -> AudioResult<Device> {
    let host = cpal::default_host();

    match device_id {
        Some(wanted) => host
            .input_devices()
            .map_err(|err| AudioError::UnsupportedFormat(err.to_string()))?
            .find(|device| stable_id(device).is_some_and(|id| id == wanted))
            .ok_or_else(|| AudioError::UnknownDevice(wanted.to_string())),
        None => host
            .default_input_device()
            .ok_or(AudioError::NoDevice("microphone")),
    }
}
