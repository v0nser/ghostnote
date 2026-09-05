//! Notification gate.
//!
//! While Stealth Mode is active every notification GhostNote would raise is
//! dropped at the source. A toast sliding in from the corner of the screen is
//! rendered by the OS *outside* our window, so capture exclusion cannot hide
//! it — suppressing emission is the only reliable defence.
//!
//! All GhostNote notifications must go through [`dispatch`]; nothing should
//! call a notification API directly.

use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

static SUPPRESSED: AtomicBool = AtomicBool::new(false);

/// Event name the frontend listens on for in-window (non-OS) notices.
pub const NOTIFICATION_EVENT: &str = "ghostnote://notification";

#[derive(Debug, Clone, Serialize)]
pub struct Notification {
    pub title: String,
    pub body: String,
}

pub fn set_suppressed(suppressed: bool) {
    SUPPRESSED.store(suppressed, Ordering::SeqCst);
}

pub fn is_suppressed() -> bool {
    SUPPRESSED.load(Ordering::SeqCst)
}

/// Emits a notification unless the gate is closed.
///
/// Returns `true` when the notification was delivered and `false` when it was
/// swallowed because Stealth Mode is active. The notification body is never
/// logged — it can contain meeting content.
pub fn dispatch<R: Runtime>(app: &AppHandle<R>, notification: Notification) -> bool {
    if is_suppressed() {
        log::debug!("notification suppressed by stealth mode");
        return false;
    }

    if let Err(err) = app.emit(NOTIFICATION_EVENT, notification) {
        log::warn!("failed to emit notification event: {err}");
        return false;
    }

    true
}
