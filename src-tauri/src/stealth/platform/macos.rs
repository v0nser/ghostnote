//! macOS capture exclusion via `NSWindow.sharingType`.
//!
//! Setting `sharingType` to `NSWindowSharingNone` removes the window from the
//! window list that the OS hands to screen-capture clients. This covers both
//! the legacy `CGWindowList` path and the modern ScreenCaptureKit path that
//! Zoom, Teams and Meet use on recent macOS versions, so the window disappears
//! from screen shares and screen recordings while remaining visible locally.

use objc2::msg_send;
use objc2::runtime::AnyObject;
use tauri::WebviewWindow;

use crate::stealth::error::{StealthError, StealthResult};

/// `NSWindowSharingNone` — the window's contents are not shared with any other
/// process.
const NS_WINDOW_SHARING_NONE: usize = 0;
/// `NSWindowSharingReadOnly` — other processes may read the window's contents.
/// This is the default for ordinary application windows.
const NS_WINDOW_SHARING_READ_ONLY: usize = 1;

pub const SUPPORTS_CAPTURE_EXCLUSION: bool = true;

pub const BACKEND_NAME: &str = "NSWindow.sharingType";

/// Applies or clears capture exclusion on `window`.
///
/// # Safety / threading
///
/// AppKit requires `NSWindow` mutation on the main thread. The caller
/// ([`crate::stealth::apply_capture_exclusion`]) guarantees this by hopping
/// through `run_on_main_thread` before invoking us.
pub fn set_capture_exclusion(window: &WebviewWindow, excluded: bool) -> StealthResult<()> {
    let handle = window.ns_window()? as *mut AnyObject;
    if handle.is_null() {
        return Err(StealthError::NullWindowHandle);
    }

    let sharing_type = if excluded {
        NS_WINDOW_SHARING_NONE
    } else {
        NS_WINDOW_SHARING_READ_ONLY
    };

    // SAFETY: `handle` is a non-null `NSWindow` owned by the Tauri runtime and
    // we are executing on the main thread, so `setSharingType:` is valid here.
    unsafe {
        let _: () = msg_send![handle, setSharingType: sharing_type];
    }

    Ok(())
}

/// `NSWindowCollectionBehaviorCanJoinAllSpaces`
const BEHAVIOR_CAN_JOIN_ALL_SPACES: usize = 1 << 0;
/// `NSWindowCollectionBehaviorFullScreenPrimary` — the default for a normal
/// resizable app window; lets the user take the dashboard full screen.
const BEHAVIOR_FULL_SCREEN_PRIMARY: usize = 1 << 7;
/// `NSWindowCollectionBehaviorFullScreenAuxiliary` — allows the window to float
/// on top of *another* app that is in full screen, which is exactly the case
/// when the user has Zoom or Meet maximised.
const BEHAVIOR_FULL_SCREEN_AUXILIARY: usize = 1 << 8;

/// Lets the pill hover above full-screen meeting windows and follow the user
/// across Spaces. `set_always_on_top` alone is not enough on macOS: a normal
/// window is still hidden when another app owns the full-screen Space.
pub fn set_floating_behavior(window: &WebviewWindow, floating: bool) -> StealthResult<()> {
    let handle = window.ns_window()? as *mut AnyObject;
    if handle.is_null() {
        return Err(StealthError::NullWindowHandle);
    }

    let behavior = if floating {
        BEHAVIOR_CAN_JOIN_ALL_SPACES | BEHAVIOR_FULL_SCREEN_AUXILIARY
    } else {
        BEHAVIOR_FULL_SCREEN_PRIMARY
    };

    // SAFETY: see `set_capture_exclusion`.
    unsafe {
        let _: () = msg_send![handle, setCollectionBehavior: behavior];
    }

    Ok(())
}

/// Reads back the effective sharing type so the UI can assert that the OS
/// actually honoured the request instead of trusting our own bookkeeping.
pub fn is_capture_excluded(window: &WebviewWindow) -> StealthResult<bool> {
    let handle = window.ns_window()? as *mut AnyObject;
    if handle.is_null() {
        return Err(StealthError::NullWindowHandle);
    }

    // SAFETY: see `set_capture_exclusion`.
    let sharing_type: usize = unsafe { msg_send![handle, sharingType] };
    Ok(sharing_type == NS_WINDOW_SHARING_NONE)
}
