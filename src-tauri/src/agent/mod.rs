//! Local-first personal agent platform.
//!
//! The meeting copilot stays in [`crate::ollama`]. This module adds planning,
//! tools, Sentinel, memory, and a workspace the UI can drive.

mod commands;
mod computer;
mod config;
mod documents;
pub mod crash;
mod error;
pub(crate) mod ids;
mod intent;
mod llm;
mod memory;
mod net;
mod paths;
mod planner;
mod policy;
mod runtime;
pub(crate) mod screen;
pub mod setup;
pub(crate) mod skills;
pub(crate) mod store;
mod supervisor;
mod tools;
mod types;
pub(crate) mod watcher;
pub(crate) mod alerts;

pub use commands::*;
pub use runtime::{extract_action_items, AgentRuntime, EVENT as AGENT_EVENT};
pub use types::AgentEvent;
