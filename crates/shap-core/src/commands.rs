//! Command handlers (the product logic behind each `shap` subcommand).
//!
//! Handlers are SDK-agnostic: they take an injected `&dyn AgentClient` and a
//! `&mut dyn FnMut(&str)` chunk sink, so the binary wires in the real ACP
//! client + renderer while tests use a mock. The binary resolves
//! [`SessionOptions`] (via `shap-agent`'s registry) and passes them in, keeping
//! this crate free of any ACP/shell dependency.

use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

use crate::agent::{AgentClient, AgentRequest, ChunkSink, SessionOptions};
use crate::config::{Config, FileOptions};
use crate::error::{Error, Result};
use crate::files::Attachment;
use crate::picker::{self, PickerKind};
use crate::prompt::CapturedContext;
use crate::session::Session;
use crate::state::ActiveState;
use crate::{files, git, prompt};

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

/// Start a new session (`shap new`), preserving the active agent/model/
/// reasoning (FR-010, SC-008). Returns the new session id.
pub fn new_session(
    config: &Config,
    state: &mut ActiveState,
    sessions_dir: &Path,
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
    let model = match &state.active_model {
        Some(m) if agent.models.contains(m) => m.clone(),
        _ => agent.default_model.clone(),
    };
    let session = Session::create(sessions_dir, &agent_name, &model)?;
    state.active_session_id = Some(session.id().to_string());
    Ok(session.id().to_string())
}

/// A snapshot of the active selections (`shap status`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Status {
    pub agent: Option<String>,
    pub model: Option<String>,
    pub reasoning: Option<String>,
    pub session_id: Option<String>,
}

/// Build the status view from current state.
pub fn status(state: &ActiveState) -> Status {
    Status {
        agent: state.active_agent.clone(),
        model: state.active_model.clone(),
        reasoning: state.active_reasoning.clone(),
        session_id: state.active_session_id.clone(),
    }
}

/// Serialize a [`Status`] to a compact JSON line (`shap status --json`).
pub fn status_json(status: &Status) -> Result<String> {
    serde_json::to_string(status)
        .map_err(|e| Error::AgentProtocol(format!("serializing status: {e}")))
}

/// Serialize a doctor [`Report`](crate::doctor::Report) to JSON (`shap doctor --json`).
pub fn doctor_json(report: &crate::doctor::Report) -> Result<String> {
    serde_json::to_string(report)
        .map_err(|e| Error::AgentProtocol(format!("serializing doctor report: {e}")))
}

/// Result of a successful [`send`].
#[derive(Debug, Clone)]
pub struct SendOutcome {
    pub response: String,
    pub session_id: String,
}

/// Continue/create a session, log the exchange, and run one agent turn. Shared
/// by `send` and `read`. `state` is mutated (active session + last cwd); the
/// caller persists it. A mid-flight failure is recorded as an `error` record.
#[allow(clippy::too_many_arguments)]
async fn dispatch_prompt(
    opts: &SessionOptions,
    sessions_dir: &Path,
    state: &mut ActiveState,
    raw_prompt: &str,
    composed: String,
    attachments: Vec<Attachment>,
    captured_output_ref: Option<String>,
    client: &dyn AgentClient,
    on_chunk: &mut ChunkSink<'_>,
) -> Result<SendOutcome> {
    // Continue the active session if it still exists, else start a fresh one.
    let session = match &state.active_session_id {
        Some(id) if Session::at(sessions_dir, id).exists() => Session::at(sessions_dir, id),
        _ => Session::create(sessions_dir, &opts.agent, &opts.model)?,
    };
    state.active_session_id = Some(session.id().to_string());
    state.last_cwd = Some(opts.cwd.to_string_lossy().into_owned());

    let cwd = opts.cwd.to_string_lossy().into_owned();
    session.log_user_prompt(raw_prompt, &cwd, attachments, captured_output_ref)?;

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

/// Handle `shap send`: expand `@file` refs, then dispatch the prompt.
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
    dispatch_prompt(
        opts,
        sessions_dir,
        state,
        prompt_text,
        composed,
        resolved.attachments,
        None,
        client,
        on_chunk,
    )
    .await
}

/// Handle `shap read`: compose the prompt + captured command output, then
/// dispatch it (the captured output is recorded as `captured_output_ref`).
#[allow(clippy::too_many_arguments)]
pub async fn read(
    opts: &SessionOptions,
    sessions_dir: &Path,
    state: &mut ActiveState,
    command: &str,
    exit_code: Option<i32>,
    output: &str,
    truncated: bool,
    prompt_text: &str,
    client: &dyn AgentClient,
    on_chunk: &mut ChunkSink<'_>,
) -> Result<SendOutcome> {
    let captured = CapturedContext {
        command,
        exit_code,
        output,
        truncated,
    };
    let composed = prompt::compose_read(prompt_text, &captured);
    dispatch_prompt(
        opts,
        sessions_dir,
        state,
        prompt_text,
        composed,
        vec![],
        Some(command.to_string()),
        client,
        on_chunk,
    )
    .await
}

/// Outcome of running a command under `shap run`.
#[derive(Debug, Clone)]
pub struct CommandRun {
    pub exit_code: i32,
    pub output: String,
}

/// Handle `shap run`: execute `argv` under Tokio, streaming combined
/// stdout+stderr to the terminal live while capturing it for `:read`.
pub async fn run(cwd: &Path, argv: &[String]) -> Result<CommandRun> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| Error::AgentProtocol("no command to run".to_string()))?;

    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| Error::io(format!("running `{program}`"), e))?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let sink = Arc::new(Mutex::new(Vec::<u8>::new()));

    let mut readers = Vec::new();
    if let Some(out) = stdout {
        readers.push(tokio::spawn(tee(out, false, sink.clone())));
    }
    if let Some(err) = stderr {
        readers.push(tokio::spawn(tee(err, true, sink.clone())));
    }

    let status = child
        .wait()
        .await
        .map_err(|e| Error::io("waiting for command", e))?;
    for r in readers {
        let _ = r.await;
    }

    let bytes = Arc::try_unwrap(sink)
        .map(|m| m.into_inner().unwrap_or_default())
        .unwrap_or_default();
    Ok(CommandRun {
        exit_code: status.code().unwrap_or(-1),
        output: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

/// Handle `shap commit --prefill-shell-buffer`: build a commit message from the
/// diff and return the `git commit -am "<message>"` line for the shell to
/// insert. **Never executes `git commit`** (FR-020, SC-003).
///
/// Returns `Ok(None)` when there is nothing to commit (caller prints a note,
/// exit 0); `Err(NotAGitRepo)` outside a repo (exit 1). Prefers the staged diff,
/// falling back to the unstaged diff.
pub async fn commit(
    opts: &SessionOptions,
    client: &dyn AgentClient,
    on_chunk: &mut ChunkSink<'_>,
) -> Result<Option<String>> {
    let cwd = &opts.cwd;
    if !git::is_repo(cwd)? {
        return Err(Error::NotAGitRepo);
    }

    let staged = git::diff(cwd, true)?;
    let diff = if !staged.trim().is_empty() {
        staged
    } else {
        let unstaged = git::diff(cwd, false)?;
        if unstaged.trim().is_empty() {
            return Ok(None);
        }
        unstaged
    };

    let branch = git::branch(cwd)?;
    let status = git::status_short(cwd)?;
    let composed = prompt::compose_commit(&branch, &status, &diff);

    let message = client
        .run_prompt(opts, &AgentRequest { prompt: composed }, on_chunk)
        .await?
        .text;

    Ok(Some(format!(
        "git commit -am \"{}\"",
        escape_double_quoted(message.trim())
    )))
}

/// Escape a string for safe inclusion inside a double-quoted shell word.
fn escape_double_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '\\' | '"' | '`' | '$') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Copy a child stream to our terminal live while appending it to `sink`.
async fn tee<R>(mut reader: R, to_stderr: bool, sink: Arc<Mutex<Vec<u8>>>)
where
    R: AsyncReadExt + Unpin,
{
    let mut buf = [0u8; 8192];
    loop {
        let n = match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let chunk = &buf[..n];
        if to_stderr {
            let mut w = tokio::io::stderr();
            let _ = w.write_all(chunk).await;
            let _ = w.flush().await;
        } else {
            let mut w = tokio::io::stdout();
            let _ = w.write_all(chunk).await;
            let _ = w.flush().await;
        }
        if let Ok(mut s) = sink.lock() {
            s.extend_from_slice(chunk);
        }
    }
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

    #[tokio::test]
    async fn run_captures_output_and_exit_code() {
        let dir = tempdir().unwrap();
        let argv = vec![
            "sh".to_string(),
            "-c".to_string(),
            "printf hello".to_string(),
        ];
        let result = run(dir.path(), &argv).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "hello");
    }

    #[tokio::test]
    async fn run_reports_nonzero_exit() {
        let dir = tempdir().unwrap();
        let argv = vec!["sh".to_string(), "-c".to_string(), "exit 3".to_string()];
        let result = run(dir.path(), &argv).await.unwrap();
        assert_eq!(result.exit_code, 3);
    }

    #[tokio::test]
    async fn read_composes_payload_and_marks_capture() {
        let dir = tempdir().unwrap();
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
                assert!(request.prompt.contains("Previous command:"));
                assert!(request.prompt.contains("error[E0277]"), "output included");
                Ok(AgentResponse {
                    text: "fixed".into(),
                })
            }
        }

        let mut noop = |_: &str| {};
        let out = read(
            &opts(dir.path().to_path_buf()),
            &sessions,
            &mut state,
            "cargo test",
            Some(101),
            "error[E0277]",
            false,
            "fix it",
            &Capture,
            &mut noop,
        )
        .await
        .unwrap();
        assert_eq!(out.response, "fixed");

        let records = Session::at(&sessions, &out.session_id)
            .read_records()
            .unwrap();
        assert!(
            records.iter().any(|r| matches!(
                r,
                Record::UserPrompt {
                    captured_output_ref: Some(_),
                    ..
                }
            )),
            "user_prompt records the captured output reference"
        );
    }

    fn git(dir: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?} failed");
    }

    #[tokio::test]
    async fn commit_returns_line_and_never_runs_git() {
        let dir = tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        git(dir.path(), &["config", "user.email", "t@t.t"]);
        git(dir.path(), &["config", "user.name", "t"]);
        std::fs::write(dir.path().join("f.txt"), "hello\n").unwrap();
        git(dir.path(), &["add", "f.txt"]);

        struct Msg;
        #[async_trait]
        impl AgentClient for Msg {
            async fn run_prompt(
                &self,
                _opts: &SessionOptions,
                request: &AgentRequest,
                _on_chunk: &mut ChunkSink<'_>,
            ) -> Result<AgentResponse> {
                assert!(request.prompt.contains("Diff:"), "diff is in the prompt");
                Ok(AgentResponse {
                    text: "feat: add f".into(),
                })
            }
        }

        let mut noop = |_: &str| {};
        let line = commit(&opts(dir.path().to_path_buf()), &Msg, &mut noop)
            .await
            .unwrap()
            .expect("a commit line");
        assert_eq!(line, "git commit -am \"feat: add f\"");

        // Critical: `:commit` must never create a commit (FR-020, SC-003).
        let log = std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["rev-list", "--count", "--all"])
            .output()
            .unwrap();
        let count = String::from_utf8_lossy(&log.stdout).trim().to_string();
        assert_eq!(count, "0", "no commit was created");
    }

    #[tokio::test]
    async fn commit_outside_repo_errors() {
        let dir = tempdir().unwrap();
        struct Never;
        #[async_trait]
        impl AgentClient for Never {
            async fn run_prompt(
                &self,
                _o: &SessionOptions,
                _r: &AgentRequest,
                _c: &mut ChunkSink<'_>,
            ) -> Result<AgentResponse> {
                panic!("agent must not be called outside a repo");
            }
        }
        let mut noop = |_: &str| {};
        let err = commit(&opts(dir.path().to_path_buf()), &Never, &mut noop)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotAGitRepo), "{err:?}");
    }

    #[tokio::test]
    async fn commit_escapes_quotes() {
        let dir = tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        git(dir.path(), &["config", "user.email", "t@t.t"]);
        git(dir.path(), &["config", "user.name", "t"]);
        std::fs::write(dir.path().join("f.txt"), "x\n").unwrap();
        git(dir.path(), &["add", "f.txt"]);

        struct Q;
        #[async_trait]
        impl AgentClient for Q {
            async fn run_prompt(
                &self,
                _o: &SessionOptions,
                _r: &AgentRequest,
                _c: &mut ChunkSink<'_>,
            ) -> Result<AgentResponse> {
                Ok(AgentResponse {
                    text: r#"fix: handle "quoted" $var"#.into(),
                })
            }
        }
        let mut noop = |_: &str| {};
        let line = commit(&opts(dir.path().to_path_buf()), &Q, &mut noop)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(line, r#"git commit -am "fix: handle \"quoted\" \$var""#);
    }
}
