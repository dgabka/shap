//! `shap-agent` — ACP integration for shap.
//!
//! Exposes the SDK-agnostic [`client`] surface plus the ACP wrapper, agent
//! registry, and session-id mapping. The binary and tests depend only on the
//! `AgentClient` trait, never on the concrete SDK types.
