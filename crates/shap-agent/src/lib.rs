//! `shap-agent` — ACP integration for shap.
//!
//! Implements `shap_core::agent::AgentClient` (the ACP wrapper lands at T021)
//! and exposes the agent [`registry`]. The binary and tests depend only on the
//! `AgentClient` trait, never on the concrete SDK types.

pub mod acp;
pub mod registry;

pub use acp::AcpClient;
