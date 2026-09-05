//! Fallback for platforms without an OS-level capture-exclusion primitive
//! (Linux/X11 and Wayland today).
//!
//! We deliberately fail loudly rather than silently pretending to be hidden —
//! a stealth app that reports success while still being visible in a screen
//! share is worse than one that admits it cannot help.

use tauri::WebviewWindow;

use crate::stealth::error::{StealthError, StealthResult};

pub const SUPPORTS_CAPTURE_EXCLUSION: bool = false;

pub const BACKEND_NAME: &str = "unsupported";

pub fn set_capture_exclusion(_window: &WebviewWindow, _excluded: bool) -> StealthResult<()> {
    Err(StealthError::UnsupportedPlatform)
}

pub fn is_capture_excluded(_window: &WebviewWindow) -> StealthResult<bool> {
    Err(StealthError::UnsupportedPlatform)
}

/// The pill still works here — only capture exclusion is missing — so the
/// cross-platform `set_always_on_top` path is left to do its job.
pub fn set_floating_behavior(_window: &WebviewWindow, _floating: bool) -> StealthResult<()> {
    Ok(())
}
