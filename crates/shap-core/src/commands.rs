//! Command handlers (the product logic behind each `shap` subcommand).
//!
//! Handlers are SDK-agnostic: they take an injected `&dyn AgentClient` and a
//! `&mut dyn FnMut(&str)` chunk sink, so the binary wires in the real ACP
//! client + renderer while tests use a mock. The binary resolves
//! [`SessionOptions`] (via `shap-agent`'s registry) and passes them in, keeping
//! this crate free of any ACP/shell dependency.

use std::path::Path;

use crate::agent::{AgentClient, AgentRequest, ChunkSink, SessionOptions};
use crate::config::FileOptions;
use crate::error::Result;
use crate::session::Session;
use crate::state::ActiveState;
use crate::{files, prompt};

/// Result of a successful [`send`].
#[derive(Debug, Clone)]
pub struct SendOutcome {
    pub response: String,
    pub session_id: String,
}

/// Handle `shap send`: expand `@file` refs, ensure/continue a session, send the
/// prompt to the agent (streaming via `on_chunk`), and log the exchange.
///
/// `state` is mutated in place (active session + last cwd); the caller persists
/// it. A mid-flight agent failure is recorded as an `error` session record
/// before the error propagates.
pub async fn send(
    opts: &SessionOptions,
    file_opts: &FileOptions,
    sessions_dir: &Path,
    state: &mut ActiveState,
    prompt_text: &str,
    client: &dyn AgentClient,
    on_chunk: &mut ChunkSink<'_>,
) -> Result<SendOutcome> {
    let resolved = files::resolve(prompt_text, &opts.cwd, file_opts)?;
    let composed = prompt::compose_send(prompt_text, &resolved.blocks);

    // Continue the active session if it still exists, else start a fresh one.
    let session = match &state.active_session_id {
        Some(id) if Session::at(sessions_dir, id).exists() => Session::at(sessions_dir, id),
        _ => Session::create(sessions_dir, &opts.agent, &opts.model)?,
    };
    state.active_session_id = Some(session.id().to_string());
    state.last_cwd = Some(opts.cwd.to_string_lossy().into_owned());

    let cwd = opts.cwd.to_string_lossy().into_owned();
    session.log_user_prompt(prompt_text, &cwd, resolved.attachments, None)?;

    let request = AgentRequest { prompt: composed };
    let response = match client.run_prompt(opts, &request, on_chunk).await {
        Ok(r) => r,
        Err(e) => {
            let _ = session.log_error(&e.to_string());
            return Err(e);
        }
    };
    session.log_agent_response(&response.text)?;

    Ok(SendOutcome {
        response: response.text,
        session_id: session.id().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentResponse;
    use crate::error::Error;
    use crate::session::Record;
    use async_trait::async_trait;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use tempfile::tempdir;

    struct MockAgent {
        reply: String,
        chunks: Vec<String>,
        fail: bool,
    }

    #[async_trait]
    impl AgentClient for MockAgent {
        async fn run_prompt(
            &self,
            _opts: &SessionOptions,
            request: &AgentRequest,
            on_chunk: &mut ChunkSink<'_>,
        ) -> Result<AgentResponse> {
            if self.fail {
                return Err(Error::AgentUnavailable {
                    agent: "codex".into(),
                    reason: "broken pipe".into(),
                });
            }
            // Echo a marker so we can assert the composed prompt reached us.
            assert!(request.prompt.contains("hi"));
            for c in &self.chunks {
                on_chunk(c);
            }
            Ok(AgentResponse {
                text: self.reply.clone(),
            })
        }
    }

    fn opts(cwd: PathBuf) -> SessionOptions {
        SessionOptions {
            agent: "codex".into(),
            command: "x".into(),
            args: vec![],
            model: "gpt-5".into(),
            reasoning: None,
            cwd,
            extra: BTreeMap::new(),
        }
    }

    #[tokio::test]
    async fn send_streams_logs_and_returns() {
        let dir = tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        let mut state = ActiveState::default();
        let mock = MockAgent {
            reply: "hello world".into(),
            chunks: vec!["hel".into(), "lo".into()],
            fail: false,
        };

        let mut streamed = String::new();
        let outcome;
        {
            let mut on_chunk = |s: &str| streamed.push_str(s);
            outcome = send(
                &opts(dir.path().to_path_buf()),
                &FileOptions::default(),
                &sessions,
                &mut state,
                "hi",
                &mock,
                &mut on_chunk,
            )
            .await
            .unwrap();
        }

        assert_eq!(outcome.response, "hello world");
        assert_eq!(streamed, "hello", "chunks streamed to the sink");
        assert_eq!(
            state.active_session_id.as_deref(),
            Some(outcome.session_id.as_str())
        );

        let records = Session::at(&sessions, &outcome.session_id)
            .read_records()
            .unwrap();
        assert!(matches!(records[0], Record::SessionStarted { .. }));
        assert!(matches!(records[1], Record::UserPrompt { .. }));
        assert!(
            matches!(records[2], Record::AgentResponse { ref content } if content == "hello world")
        );
    }

    #[tokio::test]
    async fn send_attaches_referenced_file() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("note.txt"), "secret-marker").unwrap();
        let sessions = dir.path().join("sessions");
        let mut state = ActiveState::default();

        struct Capture;
        #[async_trait]
        impl AgentClient for Capture {
            async fn run_prompt(
                &self,
                _opts: &SessionOptions,
                request: &AgentRequest,
                _on_chunk: &mut ChunkSink<'_>,
            ) -> Result<AgentResponse> {
                assert!(
                    request.prompt.contains("secret-marker"),
                    "file content inlined"
                );
                Ok(AgentResponse { text: "ok".into() })
            }
        }

        let mut noop = |_: &str| {};
        send(
            &opts(dir.path().to_path_buf()),
            &FileOptions::default(),
            &sessions,
            &mut state,
            "look at @note.txt",
            &Capture,
            &mut noop,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn agent_failure_is_logged_and_propagated() {
        let dir = tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        let mut state = ActiveState::default();
        let mock = MockAgent {
            reply: String::new(),
            chunks: vec![],
            fail: true,
        };
        let mut noop = |_: &str| {};
        let err = send(
            &opts(dir.path().to_path_buf()),
            &FileOptions::default(),
            &sessions,
            &mut state,
            "hi",
            &mock,
            &mut noop,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, Error::AgentUnavailable { .. }));

        let id = state.active_session_id.unwrap();
        let records = Session::at(&sessions, &id).read_records().unwrap();
        assert!(
            records.iter().any(|r| matches!(r, Record::Error { .. })),
            "agent failure recorded as an error record"
        );
    }

    #[tokio::test]
    async fn second_send_continues_same_session() {
        let dir = tempdir().unwrap();
        let sessions = dir.path().join("sessions");
        let mut state = ActiveState::default();
        let mock = MockAgent {
            reply: "ok".into(),
            chunks: vec![],
            fail: false,
        };
        let mut noop = |_: &str| {};
        let first = send(
            &opts(dir.path().into()),
            &FileOptions::default(),
            &sessions,
            &mut state,
            "hi",
            &mock,
            &mut noop,
        )
        .await
        .unwrap();
        let second = send(
            &opts(dir.path().into()),
            &FileOptions::default(),
            &sessions,
            &mut state,
            "hi again",
            &mock,
            &mut noop,
        )
        .await
        .unwrap();
        assert_eq!(
            first.session_id, second.session_id,
            "follow-up reuses the session"
        );
    }
}
