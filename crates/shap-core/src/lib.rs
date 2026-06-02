//! `shap-core` — SDK-agnostic product logic for shap.
//!
//! Holds configuration, state, sessions, prompt composition, file expansion,
//! output capture, Git helpers, pickers, and diagnostics. No shell specifics
//! and no ACP SDK surface live here; the [`agent::AgentClient`] trait is the
//! seam that `shap-agent` implements.

pub mod agent;
pub mod commands;
pub mod config;
pub mod error;
pub mod files;
pub mod paths;
pub mod picker;
pub mod prompt;
pub mod session;
pub mod state;

pub use error::{Error, Result};
