//! ACP wrapper: implements [`AgentClient`] over a child ACP process via stdio.
//!
//! One-shot per `run_prompt`: launch the configured agent, initialize the ACP
//! connection, open a session in the working directory, send the prompt, and
//! read the reply — the whole exchange lives inside the SDK's single
//! `connect_with` scope (matching shap's per-invocation model). The child's
//! stdin/stdout are bridged from Tokio to the SDK's `futures` byte streams via
//! `tokio_util::compat`.
//!
//! The streamed `on_chunk` sink receives the full reply once the turn
//! completes. Token-level streaming is a follow-up: the SDK exposes per-chunk
//! updates through `ActiveSession::read_update`, but routing them through the
//! sink (across the connect_with closure boundary) is deferred.

use async_trait::async_trait;
use tokio::process::Command;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use agent_client_protocol::schema::{InitializeRequest, ProtocolVersion};
use agent_client_protocol::{ByteStreams, Client};

use shap_core::agent::{AgentClient, AgentRequest, AgentResponse, ChunkSink, SessionOptions};
use shap_core::error::{Error, Result};

/// [`AgentClient`] backed by the official ACP SDK over child-process stdio.
#[derive(Debug, Default, Clone)]
pub struct AcpClient;

impl AcpClient {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AgentClient for AcpClient {
    async fn run_prompt(
        &self,
        opts: &SessionOptions,
        request: &AgentRequest,
        on_chunk: &mut ChunkSink<'_>,
    ) -> Result<AgentResponse> {
        let mut child = Command::new(&opts.command)
            .args(&opts.args)
            .current_dir(&opts.cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|source| Error::AgentSpawn {
                agent: opts.agent.clone(),
                command: opts.command.clone(),
                source,
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::AgentProtocol("agent stdin was not piped".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::AgentProtocol("agent stdout was not piped".to_string()))?;

        let transport = ByteStreams::new(stdin.compat_write(), stdout.compat());

        let prompt = request.prompt.clone();
        let cwd = opts.cwd.clone();
        let agent = opts.agent.clone();

        let result = Client
            .builder()
            .name("shap")
            .connect_with(transport, async move |cx| {
                cx.send_request(InitializeRequest::new(ProtocolVersion::V1))
                    .block_task()
                    .await?;
                cx.build_session(&cwd)
                    .block_task()
                    .run_until(async move |mut session| {
                        session.send_prompt(&prompt)?;
                        session.read_to_string().await
                    })
                    .await
            })
            .await;

        // The agent process is single-use; tear it down regardless of outcome.
        let _ = child.start_kill();

        let text = result.map_err(|e| Error::AgentUnavailable {
            agent,
            reason: e.to_string(),
        })?;

        on_chunk(&text);
        Ok(AgentResponse { text })
    }
}
