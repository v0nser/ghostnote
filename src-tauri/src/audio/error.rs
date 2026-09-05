use serde::{Serialize, Serializer};

/// Errors raised by the audio subsystem.
///
/// Audio hardware disappears, permissions get revoked mid-call, and capture
/// backends fail in ways we cannot predict. Every one of those must surface as
/// a value the UI can render — never a panic, because the capture threads run
/// alongside a live meeting.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no {0} device is available")]
    NoDevice(&'static str),

    #[error("audio device `{0}` was not found")]
    UnknownDevice(String),

    #[error("could not read the device's supported formats: {0}")]
    UnsupportedFormat(String),

    #[error("failed to open the audio stream: {0}")]
    StreamOpen(String),

    #[error("the audio stream stopped unexpectedly: {0}")]
    StreamFailed(String),

    #[error("system-audio capture is not supported on this platform")]
    SystemAudioUnsupported,

    #[error("screen-recording permission is required to capture participant audio")]
    PermissionDenied,

    #[error("capture is already running")]
    AlreadyRunning,

    #[error("capture is not running")]
    NotRunning,

    #[error("failed to write the audio chunk: {0}")]
    Io(String),
}

impl Serialize for AudioError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AudioResult<T> = Result<T, AudioError>;
