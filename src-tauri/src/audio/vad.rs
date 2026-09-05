//! Lightweight voice-activity detection for end-of-utterance.
//!
//! This is a WebRTC-style hangover detector: a frame is speech when its energy
//! sits far enough above the adaptive noise floor, and a turn ends only after
//! a short run of non-speech frames. Frame size is 20 ms, so the cut is
//! within one analysis window of the interviewer actually stopping — not a
//! 30-second batch.

use super::types::TARGET_SAMPLE_RATE;

/// Analysis frame. Matches WebRTC VAD's 20 ms mode.
pub const FRAME_MS: usize = 20;
pub const FRAME_SAMPLES: usize = (TARGET_SAMPLE_RATE as usize / 1000) * FRAME_MS;

/// How far above the noise floor a frame must sit to count as speech.
pub const SPEECH_SNR: f32 = 2.5;

pub const ABSOLUTE_SILENCE_RMS: f32 = 0.0015;

const fn ms_to_samples(ms: usize) -> usize {
    (TARGET_SAMPLE_RATE as usize / 1000) * ms
}

/// Trailing silence that closes an interviewer question. Short enough to
/// flush Whisper the moment they stop; long enough not to cut a comma.
pub const INTERVIEWER_HANGOVER_MS: usize = 260;
pub const INTERVIEWER_HANGOVER_SAMPLES: usize = ms_to_samples(INTERVIEWER_HANGOVER_MS);

pub fn rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum: f32 = frame.iter().map(|s| s * s).sum();
    (sum / frame.len() as f32).sqrt()
}

/// True when this frame is speech given the current noise-floor estimate.
pub fn is_speech(rms: f32, noise_floor: f32) -> bool {
    rms > noise_floor.max(ABSOLUTE_SILENCE_RMS) * SPEECH_SNR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_is_not_speech() {
        assert!(!is_speech(0.001, ABSOLUTE_SILENCE_RMS));
    }

    #[test]
    fn loud_frame_is_speech() {
        assert!(is_speech(0.2, ABSOLUTE_SILENCE_RMS));
    }
}
