//! Platform-specific capture-exclusion backends.
//!
//! Each backend exposes the same three items so the generic layer in
//! [`crate::stealth`] never needs a `cfg` of its own:
//!
//! - `SUPPORTS_CAPTURE_EXCLUSION: bool`
//! - `BACKEND_NAME: &str`
//! - `set_capture_exclusion(&WebviewWindow, bool) -> StealthResult<()>`
//! - `is_capture_excluded(&WebviewWindow) -> StealthResult<bool>`

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use unsupported::*;
