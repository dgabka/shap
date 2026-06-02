//! Integration tests for session control (US3, T038).
//!
//! Send-path continuity (a follow-up reusing `active_session_id`) is covered by
//! the mock-backed unit test `commands::tests::second_send_continues_same_session`
//! (a real ACP agent is not available here). These tests exercise `:new` and
//! `:status`, which need no agent.

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

const CONFIG: &str = r#"
default_agent = "codex"

[agents.codex]
command = "codex-acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"
"#;

fn setup() -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("config.toml"), CONFIG).unwrap();
    dir
}

fn shap(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("shap").unwrap();
    cmd.env("SHAP_CONFIG", dir.join("config.toml"))
        .env("SHAP_DATA_DIR", dir.join("data"));
    cmd
}

fn status_json(dir: &Path) -> String {
    let out = shap(dir).args(["status", "--json"]).output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn new_preserves_selections_and_creates_fresh_session() {
    let dir = setup();
    let p = dir.path();

    shap(p).args(["agent", "codex"]).assert().success();
    shap(p).args(["model", "gpt-5"]).assert().success();
    shap(p).args(["reasoning", "high"]).assert().success();

    shap(p).arg("new").assert().success();
    let first = status_json(p);
    assert!(first.contains("\"agent\":\"codex\""));
    assert!(first.contains("\"model\":\"gpt-5\""));
    assert!(first.contains("\"reasoning\":\"high\""));

    shap(p).arg("new").assert().success();
    let second = status_json(p);
    // Selections are preserved across `:new` (FR-010, SC-008)...
    assert!(second.contains("\"model\":\"gpt-5\""));
    assert!(second.contains("\"reasoning\":\"high\""));
    // ...but the session id changes.
    assert_ne!(
        session_id(&first),
        session_id(&second),
        "each `:new` starts a fresh session"
    );
}

#[test]
fn selections_persist_across_invocations() {
    let dir = setup();
    let p = dir.path();
    shap(p).args(["agent", "codex"]).assert().success();
    shap(p).args(["model", "gpt-5-thinking"]).assert().success();
    // A separate process sees the persisted selection.
    assert!(status_json(p).contains("\"model\":\"gpt-5-thinking\""));
}

fn session_id(json: &str) -> &str {
    let key = "\"session_id\":\"";
    let start = json.find(key).expect("session_id present") + key.len();
    let rest = &json[start..];
    &rest[..rest.find('"').unwrap()]
}
