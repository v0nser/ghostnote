//! Stealth Mode — GhostNote's defining feature.
//!
//! Stealth Mode is three things applied together:
//!
//! 1. **Capture exclusion** — the OS is told to omit our window from every
//!    screen-capture stream ([`platform`]).
//! 2. **The pill shell** — the window shrinks to an unobtrusive floating bar
//!    ([`window_mode`]).
//! 3. **Notification silence** — outgoing notifications are dropped, since the
//!    OS renders them outside our window where exclusion cannot reach
//!    ([`notifications`]).
//!
//! The invariant this module protects: **we never report `enabled` unless
//! capture exclusion actually succeeded.** If the OS call fails we roll the
//! window back and surface the error, because a user who believes they are
//! hidden when they are not is the worst possible outcome.

pub mod commands;
pub mod error;
pub mod notifications;
mod platform;
mod window_mode;

use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

use serde::Serialize;
use tauri::{Manager, WebviewWindow};

pub use error::{StealthError, StealthResult};

pub const MAIN_WINDOW_LABEL: &str = "main";

pub const DEFAULT_DASHBOARD_WIDTH: f64 = 980.0;
pub const DEFAULT_DASHBOARD_HEIGHT: f64 = 680.0;
pub const MIN_DASHBOARD_WIDTH: f64 = 720.0;
pub const MIN_DASHBOARD_HEIGHT: f64 = 480.0;

/// Snapshot of the stealth subsystem, shaped for the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StealthStatus {
    /// Stealth Mode is fully engaged.
    pub enabled: bool,
    /// The OS has confirmed the window is excluded from capture.
    pub capture_excluded: bool,
    /// The window is currently wearing the pill shell.
    pub pill_mode: bool,
    /// Notifications are being dropped.
    pub notifications_suppressed: bool,
    /// This OS has a capture-exclusion primitive at all.
    pub platform_supported: bool,
    /// Human-readable name of the primitive in use, shown in the UI so the
    /// user can verify which mechanism is protecting them.
    pub backend: &'static str,
}

#[derive(Default)]
struct StealthInner {
    enabled: bool,
    capture_excluded: bool,
    saved_geometry: Option<window_mode::SavedGeometry>,
}

/// Managed Tauri state holding the stealth subsystem's bookkeeping.
#[derive(Default)]
pub struct StealthManager {
    inner: Mutex<StealthInner>,
}

impl StealthManager {
    pub fn status(&self) -> StealthStatus {
        let inner = self.lock();
        StealthStatus {
            enabled: inner.enabled,
            capture_excluded: inner.capture_excluded,
            pill_mode: inner.saved_geometry.is_some(),
            notifications_suppressed: notifications::is_suppressed(),
            platform_supported: platform::SUPPORTS_CAPTURE_EXCLUSION,
            backend: platform::BACKEND_NAME,
        }
    }

    /// Engages or disengages Stealth Mode on `window`.
    pub fn set_enabled(
        &self,
        window: &WebviewWindow,
        enabled: bool,
    ) -> StealthResult<StealthStatus> {
        {
            let inner = self.lock();
            if inner.enabled == enabled {
                drop(inner);
                return Ok(self.status());
            }
        }

        if enabled {
            self.engage(window)?;
        } else {
            self.disengage(window)?;
        }

        Ok(self.status())
    }

    fn engage(&self, window: &WebviewWindow) -> StealthResult<()> {
        if !platform::SUPPORTS_CAPTURE_EXCLUSION {
            return Err(StealthError::UnsupportedPlatform);
        }

        // Resize first: on Windows the display affinity can be reset by some
        // window-style changes, so exclusion has to be the last thing applied.
        let saved = window_mode::enter_pill(window)?;

        if let Err(err) = apply_capture_exclusion(window, true) {
            // The OS refused. Undo the pill so the user is not left in a
            // half-applied state that *looks* like stealth but is not.
            let _ = window_mode::exit_pill(window, Some(&saved));
            return Err(err);
        }

        notifications::set_suppressed(true);

        let mut inner = self.lock();
        inner.enabled = true;
        inner.capture_excluded = true;
        inner.saved_geometry = Some(saved);

        log::info!("stealth mode engaged via {}", platform::BACKEND_NAME);
        Ok(())
    }

    fn disengage(&self, window: &WebviewWindow) -> StealthResult<()> {
        // Clearing exclusion is the one step we refuse to swallow errors on:
        // failing here leaves a window the user cannot see in a screen share
        // even though the UI claims stealth is off.
        apply_capture_exclusion(window, false)?;

        let saved = {
            let mut inner = self.lock();
            inner.enabled = false;
            inner.capture_excluded = false;
            inner.saved_geometry.take()
        };

        window_mode::exit_pill(window, saved.as_ref())?;
        notifications::set_suppressed(false);

        log::info!("stealth mode disengaged");
        Ok(())
    }

    /// Re-applies capture exclusion after an event that may have cleared it
    /// (window recreation, monitor change, display reconfiguration).
    ///
    /// Fire-and-forget by design: this is called from window-event handlers,
    /// which already run on the UI thread, so there is nothing to wait for.
    pub fn reassert(&self, window: &WebviewWindow) {
        if !self.lock().enabled {
            return;
        }

        let target = window.clone();
        let dispatched = window.run_on_main_thread(move || {
            match platform::set_capture_exclusion(&target, true) {
                Ok(()) => log::debug!("re-asserted capture exclusion"),
                Err(err) => log::error!("failed to re-assert capture exclusion: {err}"),
            }
        });

        if let Err(err) = dispatched {
            log::error!("failed to dispatch capture-exclusion re-assert: {err}");
        }
    }

    /// Asks the OS whether the window really is excluded. Platforms without a
    /// trustworthy read-back fall back to our tracked intent.
    pub fn verify(&self, window: &WebviewWindow) -> bool {
        match on_ui_thread(window, platform::is_capture_excluded) {
            Ok(excluded) => {
                self.lock().capture_excluded = excluded;
                excluded
            }
            Err(_) => self.lock().capture_excluded,
        }
    }

    /// Grows the pill into a readable card, or shrinks it back. Refuses to
    /// touch the dashboard: talking points live in the sidebar there.
    pub fn set_pill_expanded(
        &self,
        window: &WebviewWindow,
        expanded: bool,
    ) -> StealthResult<StealthStatus> {
        if self.lock().saved_geometry.is_none() {
            return Ok(self.status());
        }

        window_mode::set_pill_expanded(window, expanded)?;
        Ok(self.status())
    }

    /// A poisoned mutex only means a previous holder panicked; the state it
    /// guards is plain data, so recovering is strictly better than propagating
    /// a panic into the UI thread.
    fn lock(&self) -> std::sync::MutexGuard<'_, StealthInner> {
        self.inner.lock().unwrap_or_else(|err| err.into_inner())
    }
}

static MAIN_THREAD: OnceLock<ThreadId> = OnceLock::new();

/// Records the UI thread. Must be called from `main` before the Tauri event
/// loop starts, so [`on_ui_thread`] can tell whether a hop is needed.
pub fn record_main_thread() {
    let _ = MAIN_THREAD.set(std::thread::current().id());
}

fn already_on_main_thread() -> bool {
    MAIN_THREAD
        .get()
        .is_some_and(|id| *id == std::thread::current().id())
}

/// Runs `work` on the UI thread and returns its result.
///
/// Every native window call in this module has a thread affinity: AppKit
/// requires `NSWindow` mutation on the main thread, and
/// `SetWindowDisplayAffinity` must run on the thread owning the window.
/// Violating this aborts the process with a `Must only be used from the main
/// thread` trap rather than returning an error, so the hop is not optional.
///
/// If the caller is *already* on the UI thread the closure runs inline —
/// dispatching and then blocking on the reply would deadlock, which is exactly
/// what a window-event handler would otherwise do.
pub(crate) fn on_ui_thread<T, F>(window: &WebviewWindow, work: F) -> StealthResult<T>
where
    F: FnOnce(&WebviewWindow) -> StealthResult<T> + Send + 'static,
    T: Send + 'static,
{
    if already_on_main_thread() {
        return work(window);
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let target = window.clone();

    window.run_on_main_thread(move || {
        let _ = tx.send(work(&target));
    })?;

    rx.recv().map_err(|_| StealthError::MainThreadDispatch)?
}

fn apply_capture_exclusion(window: &WebviewWindow, excluded: bool) -> StealthResult<()> {
    on_ui_thread(window, move |window| {
        platform::set_capture_exclusion(window, excluded)
    })
}

/// Resolves the main window, turning a missing window into a typed error
/// instead of an `unwrap`.
pub fn main_window(app: &tauri::AppHandle) -> StealthResult<WebviewWindow> {
    app.get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| StealthError::WindowUnavailable(MAIN_WINDOW_LABEL.to_string()))
}
