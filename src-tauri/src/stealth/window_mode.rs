//! Window geometry for the two shells GhostNote can wear.
//!
//! - **Dashboard**: the normal, resizable notepad window.
//! - **Pill**: a small always-on-top widget parked at the top of the active
//!   monitor, sized like a music-player mini bar.
//!
//! Switching to the pill is purely cosmetic — it is capture *exclusion* that
//! makes the window invisible to a screen share. The pill exists so that the
//! sliver of screen the user still sees is unobtrusive.

use serde::{Deserialize, Serialize};
use tauri::{LogicalSize, PhysicalPosition, PhysicalSize, Size, WebviewWindow};

use crate::stealth::error::StealthResult;
use crate::stealth::platform;

/// Logical (DPI-independent) dimensions of the floating pill.
const PILL_WIDTH: f64 = 268.0;
const PILL_HEIGHT: f64 = 52.0;
/// Wider, taller card used when talking points need to be readable at a glance
/// during a screen share.
const PILL_CARD_WIDTH: f64 = 400.0;
const PILL_CARD_HEIGHT: f64 = 240.0;
/// Gap between the top edge of the work area and the pill.
const PILL_TOP_MARGIN: f64 = 18.0;

/// Everything we need to put the dashboard back exactly where the user left it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedGeometry {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
}

impl SavedGeometry {
    fn capture(window: &WebviewWindow) -> StealthResult<Self> {
        let size = window.outer_size()?;
        let position = window.outer_position()?;
        Ok(Self {
            width: size.width,
            height: size.height,
            x: position.x,
            y: position.y,
        })
    }
}

/// Shrinks the window into the floating pill and pins it above other apps.
///
/// Returns the geometry that was replaced so the caller can restore it later.
pub fn enter_pill(window: &WebviewWindow) -> StealthResult<SavedGeometry> {
    let saved = SavedGeometry::capture(window)?;

    // The dashboard declares a minimum size that is far larger than the pill,
    // so the constraint has to be lifted before the resize will be honoured.
    window.set_min_size(None::<Size>)?;
    window.set_resizable(false)?;
    window.set_size(LogicalSize::new(PILL_WIDTH, PILL_HEIGHT))?;

    if let Some(position) = pill_position(window)? {
        window.set_position(position)?;
    }

    window.set_always_on_top(true)?;
    // Keeps the pill reachable while the user is in a full-screen meeting and
    // hides it from the taskbar / alt-tab switcher where it is supported.
    let _ = window.set_visible_on_all_workspaces(true);
    let _ = window.set_skip_taskbar(true);
    super::on_ui_thread(window, |window| {
        platform::set_floating_behavior(window, true)
    })?;

    Ok(saved)
}

/// Restores the dashboard shell. Best-effort: every step is attempted even if
/// an earlier one fails, so a partial failure cannot strand the user with an
/// unusable 268x52 window.
pub fn exit_pill(window: &WebviewWindow, saved: Option<&SavedGeometry>) -> StealthResult<()> {
    let _ = super::on_ui_thread(window, |window| {
        platform::set_floating_behavior(window, false)
    });
    let _ = window.set_skip_taskbar(false);
    let _ = window.set_visible_on_all_workspaces(false);
    let _ = window.set_always_on_top(false);
    let _ = window.set_resizable(true);
    let _ = window.set_min_size(Some(LogicalSize::new(
        super::MIN_DASHBOARD_WIDTH,
        super::MIN_DASHBOARD_HEIGHT,
    )));

    if let Some(saved) = saved {
        window.set_size(PhysicalSize::new(saved.width, saved.height))?;
        window.set_position(PhysicalPosition::new(saved.x, saved.y))?;
    } else {
        window.set_size(LogicalSize::new(
            super::DEFAULT_DASHBOARD_WIDTH,
            super::DEFAULT_DASHBOARD_HEIGHT,
        ))?;
        window.center()?;
    }

    Ok(())
}

/// Grows or shrinks the pill so talking points can be read without leaving
/// stealth. A no-op if the window is not currently in pill mode — the caller
/// is expected to check that, but we still refuse to resize the dashboard.
pub fn set_pill_expanded(window: &WebviewWindow, expanded: bool) -> StealthResult<()> {
    let (width, height) = if expanded {
        (PILL_CARD_WIDTH, PILL_CARD_HEIGHT)
    } else {
        (PILL_WIDTH, PILL_HEIGHT)
    };

    window.set_size(LogicalSize::new(width, height))?;

    if let Some(position) = pill_position_for(window, width)? {
        window.set_position(position)?;
    }

    Ok(())
}

/// Top-centre of whichever monitor the window currently lives on.
fn pill_position(window: &WebviewWindow) -> StealthResult<Option<PhysicalPosition<i32>>> {
    pill_position_for(window, PILL_WIDTH)
}

fn pill_position_for(
    window: &WebviewWindow,
    width: f64,
) -> StealthResult<Option<PhysicalPosition<i32>>> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(None);
    };

    let scale = monitor.scale_factor();
    let monitor_size = monitor.size();
    let monitor_position = monitor.position();

    let pill_width_px = (width * scale).round() as i32;
    let top_margin_px = (PILL_TOP_MARGIN * scale).round() as i32;

    let x = monitor_position.x + (monitor_size.width as i32 - pill_width_px) / 2;
    let y = monitor_position.y + top_margin_px;

    Ok(Some(PhysicalPosition::new(x, y)))
}
