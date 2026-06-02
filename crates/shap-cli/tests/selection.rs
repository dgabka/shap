//! Integration tests for agent/model selection (US2, T030).
//!
//! Each `shap` invocation is a separate process sharing `SHAP_DATA_DIR`, so
//! state persists across calls exactly as it would in a shell session.

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

const CONFIG: &str = r#"
default_agent = "codex"

[agents.codex]
command = "codex-acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"

[agents.claude]
command = "claude-agent-acp"
models = ["sonnet", "opus"]
default_model = "sonnet"
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

#[test]
fn model_invalid_for_active_agent_is_rejected() {
    let dir = setup();
    // default agent is codex; "sonnet" belongs to claude → rejected.
    shap(dir.path())
        .args(["model", "sonnet"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("not valid for agent"));
}

#[test]
fn switching_agent_resets_model_to_new_default() {
    let dir = setup();
    let p = dir.path();

    shap(p).args(["agent", "codex"]).assert().success();
    shap(p).args(["model", "gpt-5"]).assert().success();
    // Switch to claude: gpt-5 is invalid there, so the model resets to sonnet.
    shap(p).args(["agent", "claude"]).assert().success();

    shap(p)
        .arg("prompt-segment")
        .assert()
        .success()
        .stdout(predicates::str::contains("claude·sonnet"));
}

#[test]
fn reasoning_rejects_unknown_level() {
    let dir = setup();
    shap(dir.path())
        .args(["reasoning", "ultra"])
        .assert()
        .failure()
        .code(1);
}
