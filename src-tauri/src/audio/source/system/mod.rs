//! System-audio ("Participant") capture, dispatched per platform.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::spawn;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::spawn;

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub use unsupported::spawn;

/// Whether this build has a system-audio backend at all.
pub const SUPPORTED: bool = cfg!(any(target_os = "macos", target_os = "windows"));

/// What the user must grant before capture can start, if anything. Surfaced in
/// the UI so the permission prompt is never a surprise.
pub const REQUIRED_PERMISSION: Option<&str> = if cfg!(target_os = "macos") {
    Some("Screen Recording")
} else {
    None
};
