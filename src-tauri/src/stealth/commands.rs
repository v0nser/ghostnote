//! Tauri commands exposing the stealth subsystem to the frontend.
//!
//! Every command here is `async` on purpose. Tauri runs synchronous commands
//! on the main thread, and [`super::apply_capture_exclusion`] blocks while it
//! waits for work it dispatched *to* the main thread — doing that from the
//! main thread would deadlock. Declaring the commands async moves them onto
//! the async runtime, which keeps the hop safe and keeps the UI responsive
//! while the OS call is in flight.

use tauri::{AppHandle, State};

use super::{main_window, notifications, StealthManager, StealthResult, StealthStatus};

/// Current stealth state. Cheap; safe to poll on mount.
#[tauri::command]
pub async fn stealth_status(
    app: AppHandle,
    manager: State<'_, StealthManager>,
) -> StealthResult<StealthStatus> {
    if let Ok(window) = main_window(&app) {
        manager.verify(&window);
    }
    Ok(manager.status())
}

/// Turns Stealth Mode on or off.
///
/// On failure the window is left in its previous state and the error is
/// returned verbatim so the UI can tell the user *why* they are still visible.
#[tauri::command]
pub async fn set_stealth_enabled(
    app: AppHandle,
    manager: State<'_, StealthManager>,
    enabled: bool,
) -> StealthResult<StealthStatus> {
    let window = main_window(&app)?;
    manager.set_enabled(&window, enabled)
}

/// Flips Stealth Mode. Wired to the global shortcut and the UI switch.
#[tauri::command]
pub async fn toggle_stealth(
    app: AppHandle,
    manager: State<'_, StealthManager>,
) -> StealthResult<StealthStatus> {
    let window = main_window(&app)?;
    let next = !manager.status().enabled;
    manager.set_enabled(&window, next)
}

/// Asks the OS to confirm the window is genuinely excluded from capture,
/// rather than trusting our own bookkeeping. Backs the "verify" affordance in
/// the UI so the user can sanity-check before sharing their screen.
#[tauri::command]
pub async fn verify_capture_exclusion(
    app: AppHandle,
    manager: State<'_, StealthManager>,
) -> StealthResult<bool> {
    let window = main_window(&app)?;
    Ok(manager.verify(&window))
}

/// Whether notifications are currently being dropped.
#[tauri::command]
pub async fn notifications_suppressed() -> bool {
    notifications::is_suppressed()
}

/// Grows the stealth pill so talking points can be read during a screen share.
#[tauri::command]
pub async fn set_pill_expanded(
    app: AppHandle,
    manager: State<'_, StealthManager>,
    expanded: bool,
) -> StealthResult<StealthStatus> {
    let window = main_window(&app)?;
    manager.set_pill_expanded(&window, expanded)
}
