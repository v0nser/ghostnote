//! Exclusive speaker assignment for the two capture paths.
//!
//! The microphone is not "the user" in the acoustic sense — it is a room
//! sensor. When the interviewer plays through laptop speakers, the mic hears
//! them too, and the same sentence is transcribed once as Them (system audio)
//! and again as You. Some meeting apps also play the user back into the
//! system mix, so the reverse leak happens as well.
//!
//! ScreenCaptureKit also lags the speakers by a couple of hundred milliseconds.
//! Without delaying the mic to match, the leak arrives first and is labelled
//! You before system audio has had a chance to claim the turn.

use std::collections::VecDeque;

use super::types::{Speaker, TARGET_SAMPLE_RATE};

/// How long microphone audio is held so system audio can catch up.
const MIC_DELAY_MS: usize = 220;
const MIC_DELAY_SAMPLES: usize = (TARGET_SAMPLE_RATE as usize / 1000) * MIC_DELAY_MS;

/// After the interviewer stops, keep treating the mic as echo so the delayed
/// tail of their sentence cannot become a You line.
const MIC_HOLD_MS: usize = 450;
const MIC_HOLD_SAMPLES: usize = (TARGET_SAMPLE_RATE as usize / 1000) * MIC_HOLD_MS;

/// After the user stops, keep treating system audio as their loopback.
const THEM_HOLD_MS: usize = 300;
const THEM_HOLD_SAMPLES: usize = (TARGET_SAMPLE_RATE as usize / 1000) * THEM_HOLD_MS;

const SPEECH_RMS: f32 = 0.012;
const EMA_ATTACK: f32 = 0.4;
const EMA_RELEASE: f32 = 0.12;
/// Mic is echo when system audio is at least this fraction of mic energy.
const ECHO_RATIO: f32 = 0.55;
/// User owns the turn when their mic is this much louder than system audio.
const YOU_DOMINANCE: f32 = 1.55;

/// Holds microphone samples until system-audio has had time to arrive.
pub struct MicDelay {
    pending: VecDeque<f32>,
}

impl MicDelay {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::with_capacity(MIC_DELAY_SAMPLES + 512),
        }
    }

    /// Buffers `samples` and returns audio that is now old enough to classify.
    pub fn push(&mut self, samples: &[f32]) -> Vec<f32> {
        self.pending.extend(samples.iter().copied());
        let excess = self.pending.len().saturating_sub(MIC_DELAY_SAMPLES);
        if excess == 0 {
            return Vec::new();
        }
        self.pending.drain(..excess).collect()
    }

    pub fn flush(&mut self) -> Vec<f32> {
        self.pending.drain(..).collect()
    }
}

impl Default for MicDelay {
    fn default() -> Self {
        Self::new()
    }
}

/// Energy-based exclusive gate. Call [`EchoGate::filter`] per 16 kHz block
/// after the microphone delay, and feed silence through when the block is
/// echo so the segmenter can still cut the current utterance.
pub struct EchoGate {
    you_rms: f32,
    them_rms: f32,
    /// Remaining mic samples to treat as echo after Them was last in speech.
    mic_hold: usize,
    /// Remaining system-audio samples to treat as the user's loopback.
    them_hold: usize,
}

impl EchoGate {
    pub fn new() -> Self {
        Self {
            you_rms: 0.0,
            them_rms: 0.0,
            mic_hold: 0,
            them_hold: 0,
        }
    }

    /// Updates energy estimates and zeroes `samples` when they belong to the
    /// other speaker's leak. Returns whether the block was muted.
    pub fn filter(&mut self, speaker: Speaker, samples: &mut [f32]) -> bool {
        let rms = rms(samples);

        match speaker {
            Speaker::Participant => {
                self.them_hold = self.them_hold.saturating_sub(samples.len());
                // Loopback of the user is quieter than the mic. A loud new
                // interviewer frame must still get through, even shortly after
                // the user stopped talking.
                if self.them_hold > 0
                    && self.you_rms >= SPEECH_RMS
                    && self.you_rms > rms.max(self.them_rms) * YOU_DOMINANCE
                {
                    samples.fill(0.0);
                    return true;
                }
                self.them_rms = ema(self.them_rms, rms);
                if self.them_rms >= SPEECH_RMS {
                    self.mic_hold = MIC_HOLD_SAMPLES;
                }
                false
            }
            Speaker::You => {
                self.mic_hold = self.mic_hold.saturating_sub(samples.len());
                if self.mic_is_echo(rms) {
                    samples.fill(0.0);
                    // Echo must not look like the user starting a turn.
                    self.you_rms = ema(self.you_rms, 0.0);
                    return true;
                }
                self.you_rms = ema(self.you_rms, rms);
                if self.you_rms >= SPEECH_RMS && self.you_rms > self.them_rms * YOU_DOMINANCE {
                    self.them_hold = THEM_HOLD_SAMPLES;
                }
                false
            }
        }
    }

    fn mic_is_echo(&self, you_now: f32) -> bool {
        if self.mic_hold > 0 {
            return true;
        }
        if self.them_rms < SPEECH_RMS {
            return false;
        }
        let you = you_now.max(self.you_rms);
        self.them_rms >= you * ECHO_RATIO
    }
}

impl Default for EchoGate {
    fn default() -> Self {
        Self::new()
    }
}

fn ema(previous: f32, sample: f32) -> f32 {
    let alpha = if sample >= previous {
        EMA_ATTACK
    } else {
        EMA_RELEASE
    };
    previous + (sample - previous) * alpha
}

fn rms(frame: &[f32]) -> f32 {
    if frame.is_empty() {
        return 0.0;
    }
    let sum: f32 = frame.iter().map(|s| s * s).sum();
    (sum / frame.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(samples: usize, amplitude: f32) -> Vec<f32> {
        (0..samples)
            .map(|i| amplitude * ((i as f32) * 0.1).sin())
            .collect()
    }

    fn silence(samples: usize) -> Vec<f32> {
        vec![0.0; samples]
    }

    #[test]
    fn delay_holds_then_releases() {
        let mut delay = MicDelay::new();
        let first = delay.push(&tone(MIC_DELAY_SAMPLES / 2, 0.4));
        assert!(first.is_empty(), "should still be buffering");

        let second = delay.push(&tone(MIC_DELAY_SAMPLES, 0.4));
        assert!(!second.is_empty());
        assert!(
            delay.pending.len() <= MIC_DELAY_SAMPLES,
            "buffer should stay near the delay length"
        );
    }

    #[test]
    fn mutes_mic_while_interviewer_is_loud() {
        let mut gate = EchoGate::new();
        let mut them = tone(1_600, 0.25);
        gate.filter(Speaker::Participant, &mut them);

        let mut you = tone(1_600, 0.08);
        let muted = gate.filter(Speaker::You, &mut you);

        assert!(muted, "mic leak of the interviewer must be dropped");
        assert!(you.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn keeps_mic_when_user_is_clearly_louder() {
        let mut gate = EchoGate::new();
        let mut them = tone(1_600, 0.01);
        gate.filter(Speaker::Participant, &mut them);

        let mut you = tone(1_600, 0.3);
        let muted = gate.filter(Speaker::You, &mut you);

        assert!(!muted, "the user's own speech must survive");
        assert!(you.iter().any(|s| *s != 0.0));
    }

    #[test]
    fn mutes_system_audio_when_user_is_speaking_into_loopback() {
        let mut gate = EchoGate::new();
        for _ in 0..8 {
            let mut you = tone(1_600, 0.35);
            gate.filter(Speaker::You, &mut you);
        }

        let mut them = tone(1_600, 0.12);
        let muted = gate.filter(Speaker::Participant, &mut them);

        assert!(muted, "user's voice playing back through the meeting mix");
        assert!(them.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn hold_keeps_mic_muted_after_them_stops() {
        let mut gate = EchoGate::new();
        let mut them = tone(1_600, 0.25);
        gate.filter(Speaker::Participant, &mut them);

        let mut quiet_them = silence(800);
        gate.filter(Speaker::Participant, &mut quiet_them);

        let mut you = tone(800, 0.08);
        let muted = gate.filter(Speaker::You, &mut you);
        assert!(muted, "echo tail after the interviewer pauses");
    }
}
