//! Integration tests for `:doctor` (US6, T054).
//!
//! Deterministic report snapshots live in the `doctor` unit tests (which inject
//! a fake probe). These check the binary's exit codes and the missing-agent
//! diagnostic from `shap send`.

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

fn dir_with(config: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("config.toml"), config).unwrap();
    dir
}

fn shap(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("shap").unwrap();
    cmd.env("SHAP_CONFIG", dir.join("config.toml"))
        .env("SHAP_DATA_DIR", dir.join("data"))
        // Force a clean shell-integration probe regardless of the host env.
        .env_remove("SHAP_SHELL_INTEGRATION");
    cmd
}

#[test]
fn doctor_fails_when_agent_command_missing() {
    let dir = dir_with(
        r#"
default_agent = "codex"
[agents.codex]
command = "definitely-not-installed-xyz"
models = ["m"]
default_model = "m"
"#,
    );
    // The missing agent command is a critical failure → exit 1.
    shap(dir.path())
        .arg("doctor")
        .assert()
        .failure()
        .code(1)
        .stdout(predicates::str::contains("not found on PATH"));
}

#[test]
fn doctor_json_is_emitted() {
    let dir = dir_with(
        r#"
default_agent = "codex"
[agents.codex]
command = "sh"
models = ["m"]
default_model = "m"
"#,
    );
    // `sh` exists on PATH, so the agent check passes; output is JSON.
    shap(dir.path())
        .args(["doctor", "--json"])
        .assert()
        .stdout(predicates::str::contains("\"checks\""));
}

#[test]
fn send_missing_agent_diagnostic() {
    let dir = dir_with(
        r#"
default_agent = "codex"
[agents.codex]
command = "definitely-not-installed-xyz"
models = ["m"]
default_model = "m"
"#,
    );
    shap(dir.path())
        .args(["send", "hello"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("was not found on PATH"));
}
