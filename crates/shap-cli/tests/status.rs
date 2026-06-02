//! Snapshot test for `shap status` human output (US3, T039).

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

#[test]
fn status_human_output() {
    let dir = setup();
    let p = dir.path();
    shap(p).args(["agent", "codex"]).assert().success();
    shap(p).args(["model", "gpt-5"]).assert().success();
    shap(p).args(["reasoning", "high"]).assert().success();

    // No session started yet → it renders as `-` (deterministic).
    let out = shap(p).arg("status").output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!(stdout);
}
