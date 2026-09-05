//! Writes utterances to the 16-bit PCM WAV files the Whisper sidecar reads.
//!
//! These files hold raw meeting audio, so they are written to the app's own
//! cache directory (never the shared system temp directory, which is
//! world-readable on some setups) and deleted the moment transcription
//! finishes — see [`TempAudio`].

use std::path::{Path, PathBuf};

use crate::audio::error::{AudioError, AudioResult};
use crate::audio::types::TARGET_SAMPLE_RATE;

/// A WAV file that deletes itself when dropped.
///
/// Transcription can fail, the sidecar can be killed, the app can hit an error
/// path — and in every one of those cases the audio must still be cleaned up.
/// Tying deletion to the value's lifetime means no error path can forget.
pub struct TempAudio {
    path: PathBuf,
}

impl TempAudio {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempAudio {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_file(&self.path) {
            if err.kind() != std::io::ErrorKind::NotFound {
                // Deliberately does not log the path: it embeds a meeting id.
                log::warn!("failed to delete a temporary audio file");
            }
        }
    }
}

/// Writes 16 kHz mono `f32` samples as a 16-bit PCM WAV.
pub fn write(dir: &Path, name: &str, samples: &[f32]) -> AudioResult<TempAudio> {
    std::fs::create_dir_all(dir).map_err(|err| AudioError::Io(err.to_string()))?;

    let path = dir.join(format!("{name}.wav"));
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer =
        hound::WavWriter::create(&path, spec).map_err(|err| AudioError::Io(err.to_string()))?;

    for sample in samples {
        // Clamp before scaling: a sample slightly outside the nominal range
        // would otherwise wrap around and turn a loud word into a click.
        let clamped = sample.clamp(-1.0, 1.0);
        let encoded = (clamped * i16::MAX as f32) as i16;
        writer
            .write_sample(encoded)
            .map_err(|err| AudioError::Io(err.to_string()))?;
    }

    writer
        .finalize()
        .map_err(|err| AudioError::Io(err.to_string()))?;

    Ok(TempAudio { path })
}
