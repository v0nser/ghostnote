use serde::{Deserialize, Serialize};

/// Whisper is trained on 16 kHz mono audio and resamples internally if given
/// anything else. Doing the conversion ourselves keeps the sidecar's work
/// predictable and shrinks the WAV files we hand it.
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Who produced a piece of audio.
///
/// Labelling comes from the capture topology: the microphone is the user,
/// the system-audio tap is everyone else. The two paths still leak into each
/// other (speakers into the mic, the user's voice into the meeting mix), so
/// the pipeline runs an exclusive energy gate before we trust the label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Speaker {
    /// Captured from the microphone.
    You,
    /// Captured from the system-audio tap.
    Participant,
}

impl Speaker {
    pub const ALL: [Speaker; 2] = [Speaker::You, Speaker::Participant];

    pub const fn label(self) -> &'static str {
        match self {
            Speaker::You => "You",
            Speaker::Participant => "Them",
        }
    }
}

/// A block of audio exactly as a capture backend handed it to us: whatever
/// sample rate and channel count the device happens to use, interleaved.
#[derive(Debug)]
pub struct RawFrames {
    pub speaker: Speaker,
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// A contiguous stretch of speech from one source, normalised to 16 kHz mono
/// and ready to hand to Whisper.
#[derive(Debug)]
pub struct Utterance {
    pub speaker: Speaker,
    pub samples: Vec<f32>,
    /// Milliseconds since capture started.
    pub start_ms: u64,
    pub end_ms: u64,
    /// Still being spoken. The coach uses these to start drafting before the
    /// other person has paused, so the answer is ready the moment they stop.
    pub preview: bool,
}

impl Utterance {
    pub fn duration_ms(&self) -> u64 {
        self.end_ms.saturating_sub(self.start_ms)
    }
}

/// An input device the user can pick for microphone capture.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Live capture state, mirrored into the UI.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureStatus {
    pub running: bool,
    /// Microphone capture is active.
    pub microphone: bool,
    /// System-audio capture is active.
    pub system_audio: bool,
    /// Set when system audio could not be started but the microphone could —
    /// capture continues in a degraded, mic-only mode rather than failing.
    pub system_audio_error: Option<String>,
    pub elapsed_ms: u64,
}

/// Emitted periodically so the UI can render a level meter without ever
/// touching raw audio.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEvent {
    pub speaker: Speaker,
    /// Peak amplitude in `0.0..=1.0` over the reporting window.
    pub peak: f32,
}

/// The interviewer started or stopped speaking. Emitted the moment VAD
/// decides, before Whisper has transcribed the clip.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VadEvent {
    pub speaker: Speaker,
    /// `false` means the turn just ended and the clip is already on its way
    /// to Whisper.
    pub speaking: bool,
}
