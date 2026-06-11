//! Integration tests for the config wizard's non-interactive guardrails (US3)
//! and `shap config` back-compat (FR-010/011/012).
//!
//! `assert_cmd` runs the binary with a null stdin, so `IsTerminal` is false —
//! these exercise exactly the non-interactive paths that must never prompt.

use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

fn valid_config() -> &'static str {
    r#"
default_agent = "codex"
[agents.codex]
command = "codex-acp"
models = ["gpt-5"]
default_model = "gpt-5"
"#
}

/// A scratch dir; `config.toml` is created only if `config` is `Some`.
fn scratch(config: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    if let Some(c) = config {
        std::fs::write(dir.path().join("config.toml"), c).unwrap();
    }
    dir
}

fn shap(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("shap").unwrap();
    cmd.env("SHAP_CONFIG", dir.join("config.toml"))
        .env("SHAP_DATA_DIR", dir.join("data"))
        .env_remove("SHAP_SHELL_INTEGRATION");
    cmd
}

#[test]
fn missing_config_non_interactive_falls_back_and_writes_nothing() {
    // No config + non-TTY stdin → ConfigNotFound diagnostic, non-zero, no file.
    let dir = scratch(None);
    shap(dir.path())
        .args(["send", "hi"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("no config file"));
    assert!(
        !dir.path().join("config.toml").exists(),
        "non-interactive run must not create a config"
    );
}

#[test]
fn config_path_is_printed_non_interactive() {
    // Bare `config` on a non-TTY prints the resolved path (back-compat).
    let dir = scratch(None);
    let expected = dir.path().join("config.toml");
    shap(dir.path())
        .arg("config")
        .assert()
        .success()
        .stdout(predicates::str::contains(expected.to_string_lossy()));
}

#[test]
fn config_path_subcommand_prints_path() {
    let dir = scratch(None);
    let expected = dir.path().join("config.toml");
    shap(dir.path())
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicates::str::contains(expected.to_string_lossy()));
}

#[test]
fn config_schema_flag_emits_schema() {
    let dir = scratch(None);
    shap(dir.path())
        .args(["config", "--schema"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"default_agent\""));
}

#[test]
fn config_schema_subcommand_emits_schema() {
    let dir = scratch(None);
    shap(dir.path())
        .args(["config", "schema"])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"default_agent\""));
}

#[test]
fn config_edit_non_interactive_errors() {
    // `config edit` without a terminal must refuse, not hang.
    let dir = scratch(Some(valid_config()));
    shap(dir.path())
        .args(["config", "edit"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no terminal"));
}

#[test]
fn prompt_segment_with_no_config_stays_silent() {
    // The cheap prompt hook must never trigger the wizard (FR-011).
    let dir = scratch(None);
    shap(dir.path()).arg("prompt-segment").assert().success();
    assert!(
        !dir.path().join("config.toml").exists(),
        "prompt-segment must not create a config"
    );
}
