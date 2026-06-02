//! Thin wrappers over the local `git` CLI (Constitution I: the CLI over a Git
//! library). Used by `:commit` to gather the diff, branch, and status. None of
//! these ever mutate the repository.

use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

fn run_git(cwd: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::GitUnavailable
            } else {
                Error::io("running git", e)
            }
        })
}

fn stdout_string(output: std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

/// Whether `cwd` is inside a Git working tree.
pub fn is_repo(cwd: &Path) -> Result<bool> {
    let out = run_git(cwd, &["rev-parse", "--is-inside-work-tree"])?;
    Ok(out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true")
}

/// Current branch name (empty on an unborn HEAD).
pub fn branch(cwd: &Path) -> Result<String> {
    Ok(stdout_string(run_git(cwd, &["branch", "--show-current"])?))
}

/// `git status --short`.
pub fn status_short(cwd: &Path) -> Result<String> {
    Ok(stdout_string(run_git(cwd, &["status", "--short"])?))
}

/// The diff. `staged` selects `git diff --staged` vs. the working-tree `git diff`.
pub fn diff(cwd: &Path, staged: bool) -> Result<String> {
    let args: &[&str] = if staged {
        &["diff", "--staged"]
    } else {
        &["diff"]
    };
    Ok(stdout_string(run_git(cwd, args)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::tempdir;

    fn git(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?} failed");
    }

    fn init_repo(dir: &Path) {
        git(dir, &["init", "-q"]);
        git(dir, &["config", "user.email", "t@t.t"]);
        git(dir, &["config", "user.name", "t"]);
    }

    #[test]
    fn non_repo_is_detected() {
        let dir = tempdir().unwrap();
        assert!(!is_repo(dir.path()).unwrap());
    }

    #[test]
    fn staged_diff_and_status() {
        let dir = tempdir().unwrap();
        init_repo(dir.path());
        std::fs::write(dir.path().join("a.txt"), "hello\n").unwrap();
        git(dir.path(), &["add", "a.txt"]);

        assert!(is_repo(dir.path()).unwrap());
        let staged = diff(dir.path(), true).unwrap();
        assert!(staged.contains("a.txt"), "staged diff names the file");
        let status = status_short(dir.path()).unwrap();
        assert!(status.contains("a.txt"));
        // Unstaged diff is empty (everything is staged).
        assert!(diff(dir.path(), false).unwrap().trim().is_empty());
    }
}
