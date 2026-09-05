use serde::{Serialize, Serializer};

/// Errors raised by the stealth subsystem.
///
/// Every variant is recoverable: the caller is expected to surface the problem
/// to the UI and leave the window in its previous state rather than panic. A
/// failure here must never take down the main thread, because that would close
/// the user's window in the middle of a meeting.
#[derive(Debug, thiserror::Error)]
pub enum StealthError {
    #[error("window `{0}` is not available")]
    WindowUnavailable(String),

    #[error("could not dispatch the operation to the UI thread")]
    MainThreadDispatch,

    #[error("the operating system returned a null window handle")]
    NullWindowHandle,

    #[error("capture exclusion is not supported on this platform")]
    UnsupportedPlatform,

    #[error("the operating system rejected the capture-exclusion request: {0}")]
    PlatformCall(String),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl Serialize for StealthError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type StealthResult<T> = Result<T, StealthError>;
