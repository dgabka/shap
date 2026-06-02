//! Integration tests for `:run` output capture (US4, T044).
//!
//! `:read` payload composition (including the captured output) is covered by
//! the mock-backed unit test `commands::tests::read_composes_payload_and_marks_capture`;
//! a real agent is not available here, and assert_cmd's stdin is never a TTY
//! (so `read` always takes the pipe branch).

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

const CONFIG: &str = r#"
default_agent = "codex"

[agents.codex]
command = "codex-acp"
models = ["gpt-5"]
default_model = "gpt-5"
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
fn run_captures_output_and_records_exit_code() {
    let dir = setup();
    shap(dir.path())
        .args(["run", "--", "sh", "-c", "echo captured-line"])
        .assert()
        .success();

    let text = std::fs::read_to_string(dir.path().join("data/last-command-output.txt")).unwrap();
    assert!(text.contains("captured-line"));

    let meta = std::fs::read_to_string(dir.path().join("data/last-command-output.json")).unwrap();
    assert!(meta.contains("\"exit_code\": 0"));
}

#[test]
fn run_propagates_nonzero_exit_code() {
    let dir = setup();
    shap(dir.path())
        .args(["run", "--", "sh", "-c", "exit 7"])
        .assert()
        .failure()
        .code(7);
}

#[test]
fn run_captures_stderr_too() {
    let dir = setup();
    shap(dir.path())
        .args(["run", "--", "sh", "-c", "echo oops 1>&2"])
        .assert()
        .success();
    let text = std::fs::read_to_string(dir.path().join("data/last-command-output.txt")).unwrap();
    assert!(text.contains("oops"), "stderr is captured");
}
