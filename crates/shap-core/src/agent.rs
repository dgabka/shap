//! SDK-agnostic agent surface.
//!
//! The [`AgentClient`] trait and its DTOs live here (not in `shap-agent`) so
//! that command handlers in `shap-core` can depend on the trait without a
//! dependency cycle — `shap-agent` *implements* this trait and the binary
//! injects the concrete ACP client. Tests use an in-memory mock of the trait,
//! never the real SDK.

use std::collections::BTreeMap;
use std::path::PathBuf;

use async_trait::async_trait;

use crate::error::Result;

/// Opaque handle to an agent-side session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionId(pub String);

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Inputs needed to launch and start an ACP session.
#[derive(Debug, Clone)]
pub struct SessionOptions {
    /// Configured agent name (for diagnostics/logging).
    pub agent: String,
    /// Launch command for the external ACP process.
    pub command: String,
    /// Extra launch arguments (currently unused; reserved).
    pub args: Vec<String>,
    /// Selected model.
    pub model: String,
    /// Selected reasoning effort, if any.
    pub reasoning: Option<String>,
    /// Working directory the agent should operate in.
    pub cwd: PathBuf,
    /// Opaque agent-specific passthrough config (FR-022).
    pub extra: BTreeMap<String, String>,
}

/// A composed prompt ready to send to the agent.
#[derive(Debug, Clone)]
pub struct AgentRequest {
    /// Fully composed prompt text (prompt + attachments/output already inlined).
    pub prompt: String,
}

/// The agent's final reply.
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// Final assembled text (streamed chunks reassembled).
    pub text: String,
}

/// SDK-agnostic agent client.
///
/// Implementations stream response text via `on_chunk`; the UI layer decides
/// whether to render chunks live (stream mode) or wait for the final
/// [`AgentResponse`] (spinner mode). Errors map into the core [`Error`] type.
///
/// [`Error`]: crate::error::Error
#[async_trait]
pub trait AgentClient: Send {
    /// Launch the agent process (if needed) and open a session.
    async fn start_session(&mut self, opts: &SessionOptions) -> Result<SessionId>;

    /// Send a prompt and stream the reply. `on_chunk` is invoked for each text
    /// delta as it arrives; the returned [`AgentResponse`] holds the full text.
    async fn prompt(
        &mut self,
        session: &SessionId,
        request: &AgentRequest,
        on_chunk: &mut (dyn FnMut(&str) + Send),
    ) -> Result<AgentResponse>;
}
