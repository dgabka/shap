//! Command handlers (the product logic behind each `shap` subcommand).
//!
//! Handlers are SDK-agnostic: they take an injected `&dyn AgentClient` and a
//! `&mut dyn FnMut(&str)` chunk sink, so the binary wires in the real ACP
//! client + renderer while tests use a mock. The binary resolves
//! [`SessionOptions`] (via `shap-agent`'s registry) and passes them in, keeping
//! this crate free of any ACP/shell dependency.

use std::path::Path;

use crate::agent::{AgentClient, AgentRequest, ChunkSink, SessionOptions};
use crate::config::{Config, FileOptions};
use crate::error::{Error, Result};
use crate::picker::{self, PickerKind};
use crate::session::Session;
use crate::state::ActiveState;
use crate::{files, prompt};

/// Supported reasoning levels (MVP default set).
pub const REASONING_LEVELS: [&str; 3] = ["low", "medium", "high"];

/// Set the active agent (`shap agent`). With no name (or `force_picker`), opens
/// a picker over the configured agents. On switch, the active model is reset to
/// the new agent's `default_model` if the current model is invalid for it.
pub fn set_agent(
    config: &Config,
    state: &mut ActiveState,
    name: Option<String>,
    force_picker: bool,
    picker_kind: PickerKind,
) -> Result<String> {
    let chosen = match name {
        Some(n) if !force_picker => n,
        _ => picker::select(picker_kind, "agent", &config.agent_names())?,
    };
    let agent = config.agent(&chosen).ok_or_else(|| Error::UnknownAgent {
        name: chosen.clone(),
        configured: config.agent_names().join(", "),
    })?;

    state.active_agent = Some(chosen.clone());
    let keep_model = state
        .active_model
        .as_ref()
        .is_some_and(|m| agent.models.contains(m));
    if !keep_model {
        state.active_model = Some(agent.default_model.clone());
    }
    Ok(chosen)
}

/// Set the active model (`shap model`). Offers/sets only the active agent's
/// models (the active agent falls back to `default_agent`).
pub fn set_model(
    config: &Config,
    state: &mut ActiveState,
    name: Option<String>,
    force_picker: bool,
    picker_kind: PickerKind,
) -> Result<String> {
    let agent_name = state
        .active_agent
        .clone()
        .unwrap_or_else(|| config.default_agent.clone());
    let agent = config
        .agent(&agent_name)
        .ok_or_else(|| Error::UnknownAgent {
            name: agent_name.clone(),
            configured: config.agent_names().join(", "),
        })?;

    let chosen = match name {
        Some(n) if !force_picker => n,
        _ => picker::select(picker_kind, "model", &agent.models)?,
    };
    if !agent.models.contains(&chosen) {
        return Err(Error::ModelNotForAgent {
            model: chosen,
            agent: agent_name,
            models: agent.models.join(", "),
        });
    }
    state.active_model = Some(chosen.clone());
    Ok(chosen)
}

/// Set the reasoning effort (`shap reasoning` / `:effort`).
pub fn set_reasoning(
    state: &mut ActiveState,
    level: Option<String>,
    force_picker: bool,
    picker_kind: PickerKind,
) -> Result<String> {
    let levels: Vec<String> = REASONING_LEVELS.iter().map(|s| s.to_string()).collect();
    let chosen = match level {
        Some(l) if !force_picker => l,
        _ => picker::select(picker_kind, "reasoning", &levels)?,
    };
    if !REASONING_LEVELS.contains(&chosen.as_str()) {
        return Err(Error::InvalidReasoning {
            level: chosen,
            valid: REASONING_LEVELS.join(", "),
        });
    }
    state.active_reasoning = Some(chosen.clone());
    Ok(chosen)
}

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

    fn config() -> Config {
        toml::from_str(
            r#"
default_agent = "codex"
[agents.codex]
command = "codex-acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"
[agents.claude]
command = "claude-agent-acp"
models = ["sonnet", "opus"]
default_model = "sonnet"
"#,
        )
        .unwrap()
    }

    #[test]
    fn set_agent_resets_invalid_model_to_default() {
        let cfg = config();
        let mut state = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("gpt-5".into()),
            ..Default::default()
        };
        // Switch to claude: gpt-5 is invalid there → reset to claude's default.
        set_agent(
            &cfg,
            &mut state,
            Some("claude".into()),
            false,
            PickerKind::Builtin,
        )
        .unwrap();
        assert_eq!(state.active_agent.as_deref(), Some("claude"));
        assert_eq!(state.active_model.as_deref(), Some("sonnet"));
    }

    #[test]
    fn set_agent_keeps_valid_model() {
        let cfg = config();
        let mut state = ActiveState {
            active_agent: Some("claude".into()),
            active_model: Some("opus".into()),
            ..Default::default()
        };
        // claude → claude, opus still valid → keep it.
        set_agent(
            &cfg,
            &mut state,
            Some("claude".into()),
            false,
            PickerKind::Builtin,
        )
        .unwrap();
        assert_eq!(state.active_model.as_deref(), Some("opus"));
    }

    #[test]
    fn set_model_rejects_invalid_for_active_agent() {
        let cfg = config();
        let mut state = ActiveState {
            active_agent: Some("codex".into()),
            ..Default::default()
        };
        let err = set_model(
            &cfg,
            &mut state,
            Some("sonnet".into()),
            false,
            PickerKind::Builtin,
        )
        .unwrap_err();
        assert!(matches!(err, Error::ModelNotForAgent { .. }), "{err:?}");
    }

    #[test]
    fn set_model_accepts_valid() {
        let cfg = config();
        let mut state = ActiveState {
            active_agent: Some("codex".into()),
            ..Default::default()
        };
        set_model(
            &cfg,
            &mut state,
            Some("gpt-5".into()),
            false,
            PickerKind::Builtin,
        )
        .unwrap();
        assert_eq!(state.active_model.as_deref(), Some("gpt-5"));
    }

    #[test]
    fn set_reasoning_validates_level() {
        let mut state = ActiveState::default();
        set_reasoning(&mut state, Some("high".into()), false, PickerKind::Builtin).unwrap();
        assert_eq!(state.active_reasoning.as_deref(), Some("high"));

        let err = set_reasoning(&mut state, Some("ultra".into()), false, PickerKind::Builtin)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidReasoning { .. }), "{err:?}");
    }
}
