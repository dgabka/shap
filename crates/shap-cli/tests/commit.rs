//! Integration tests for `:commit` (US5, T050).
//!
//! The message-generation path needs an agent (covered by the mock-backed unit
//! test `commands::tests::commit_returns_line_and_never_runs_git`, which also
//! asserts no commit is ever created). These exercise the no-agent branches:
//! not-a-repo and nothing-to-commit.

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

#[test]
fn commit_outside_repo_fails() {
    let dir = setup();
    // The data/config tempdir is not a git repo.
    shap(dir.path())
        .arg("--cwd")
        .arg(dir.path())
        .args(["commit", "--prefill-shell-buffer"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains("not inside a Git repository"));
}

#[test]
fn commit_with_nothing_staged_is_clean_noop() {
    let dir = setup();
    let repo = dir.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);

    // Clean repo: exit 0, no `git commit` line on stdout (never executes git).
    shap(dir.path())
        .arg("--cwd")
        .arg(&repo)
        .args(["commit", "--prefill-shell-buffer"])
        .assert()
        .success()
        .stdout(predicates::str::is_empty());
}
