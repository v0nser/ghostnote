//! Windows capture exclusion via `SetWindowDisplayAffinity`.
//!
//! `WDA_EXCLUDEFROMCAPTURE` (Windows 10 2004 / build 19041 and later) makes the
//! desktop compositor render the window as fully transparent to every capture
//! client while keeping it on screen for the local user. Unlike the older
//! `WDA_MONITOR` flag it leaves no black rectangle behind in the shared frame.

use tauri::WebviewWindow;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
};

use crate::stealth::error::{StealthError, StealthResult};

pub const SUPPORTS_CAPTURE_EXCLUSION: bool = true;

pub const BACKEND_NAME: &str = "SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)";

/// Applies or clears capture exclusion on `window`.
///
/// # Threading
///
/// `SetWindowDisplayAffinity` must be called from the thread that owns the
/// window. The caller hops onto the UI thread before invoking us.
pub fn set_capture_exclusion(window: &WebviewWindow, excluded: bool) -> StealthResult<()> {
    let hwnd = window.hwnd()?;
    if hwnd.0.is_null() {
        return Err(StealthError::NullWindowHandle);
    }

    let affinity = if excluded {
        WDA_EXCLUDEFROMCAPTURE
    } else {
        WDA_NONE
    };

    // SAFETY: `hwnd` is a live top-level window owned by the Tauri runtime and
    // we are executing on its owning thread.
    unsafe { SetWindowDisplayAffinity(HWND(hwnd.0), affinity) }
        .map_err(|err| StealthError::PlatformCall(err.message()))
}

/// No-op on Windows: `set_always_on_top` plus `set_skip_taskbar`, which the
/// generic layer already applies, give the pill the behaviour we want.
pub fn set_floating_behavior(_window: &WebviewWindow, _floating: bool) -> StealthResult<()> {
    Ok(())
}

/// Windows exposes `GetWindowDisplayAffinity`, but it is unreliable across
/// remote-desktop sessions, so we report our own intent instead of reading the
/// value back. The generic layer tracks the requested state.
pub fn is_capture_excluded(_window: &WebviewWindow) -> StealthResult<bool> {
    Err(StealthError::UnsupportedPlatform)
}
