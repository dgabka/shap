//! Prompt composition.
//!
//! Turns a user prompt plus resolved `@file` blocks (and, for `:read`, captured
//! command output) into the single text payload sent to the agent. The `:read`
//! and `:commit` shapes are contract-pinned and snapshot-tested.

use crate::files::FileBlock;

/// Compose the base `send` payload: the user prompt followed by any attached
/// file blocks. With no attachments the prompt is sent verbatim.
pub fn compose_send(user_prompt: &str, blocks: &[FileBlock]) -> String {
    if blocks.is_empty() {
        return user_prompt.to_string();
    }
    let mut out = String::new();
    out.push_str(user_prompt);
    for block in blocks {
        out.push_str("\n\nFile: ");
        out.push_str(&block.path);
        out.push('\n');
        out.push_str("```\n");
        out.push_str(&block.content);
        if !block.content.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("```");
    }
    out
}

/// Captured command output to fold into a `:read` payload.
#[derive(Debug, Clone)]
pub struct CapturedContext<'a> {
    pub command: &'a str,
    pub exit_code: Option<i32>,
    pub output: &'a str,
    pub truncated: bool,
}

/// Compose the `:commit` prompt: ask the agent for a commit message given the
/// branch, short status, and diff. Snapshot-tested.
pub fn compose_commit(branch: &str, status: &str, diff: &str) -> String {
    format!(
        "Write a single concise Git commit message (Conventional Commits style) \
for the following staged changes. Reply with only the commit message — no \
explanation, no code fences.\n\nBranch: {branch}\n\nStatus:\n{status}\n\nDiff:\n{diff}"
    )
}

/// Compose the `:read` payload (contract: `contracts/cli-commands.md`).
pub fn compose_read(user_prompt: &str, captured: &CapturedContext<'_>) -> String {
    let exit = match captured.exit_code {
        Some(code) => code.to_string(),
        None => "-".to_string(),
    };
    let mut output = captured.output.to_string();
    if captured.truncated {
        output.push_str("\n[output truncated]");
    }
    format!(
        "User prompt:\n{prompt}\n\nPrevious command:\n{command}\n\nExit code:\n{exit}\n\nOutput:\n{output}",
        prompt = user_prompt,
        command = captured.command,
        exit = exit,
        output = output,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_prompt_without_attachments_is_verbatim() {
        assert_eq!(compose_send("hello", &[]), "hello");
    }

    #[test]
    fn base_prompt_appends_file_blocks() {
        let blocks = vec![FileBlock {
            path: "src/a.rs".into(),
            content: "fn main() {}".into(),
        }];
        let out = compose_send("explain this", &blocks);
        assert_eq!(
            out,
            "explain this\n\nFile: src/a.rs\n```\nfn main() {}\n```"
        );
    }

    #[test]
    fn read_payload_shape() {
        let ctx = CapturedContext {
            command: "cargo test",
            exit_code: Some(101),
            output: "error[E0277]",
            truncated: false,
        };
        let out = compose_read("fix it", &ctx);
        assert_eq!(
            out,
            "User prompt:\nfix it\n\nPrevious command:\ncargo test\n\nExit code:\n101\n\nOutput:\nerror[E0277]"
        );
    }

    #[test]
    fn read_payload_snapshot_truncated() {
        let ctx = CapturedContext {
            command: "cargo build",
            exit_code: None,
            output: "line1\nline2",
            truncated: true,
        };
        insta::assert_snapshot!(compose_read("what broke?", &ctx));
    }

    #[test]
    fn commit_prompt_snapshot() {
        let out = compose_commit(
            "main",
            " M src/lib.rs",
            "diff --git a/src/lib.rs b/src/lib.rs\n+// new line",
        );
        insta::assert_snapshot!(out);
    }
}
