//! `@file` reference detection and resolution.
//!
//! A prompt may mention files as `@path` tokens. Resolution turns each into an
//! attachment (metadata for the session log) plus a content block (for prompt
//! composition), subject to guards: the path must exist as a regular file, not
//! be binary, fit within `max_file_bytes`, and — when `respect_gitignore` — not
//! be ignored. Unresolved or skipped refs are left as visible text in the
//! prompt (FR / data-model § Validation).

use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, Serialize};

use crate::config::FileOptions;
use crate::error::{Error, Result};

/// Metadata recorded in the session log for an included file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub path: String,
    pub bytes: u64,
    pub truncated: bool,
}

/// A resolved file's content, for prompt composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileBlock {
    pub path: String,
    pub content: String,
}

/// The result of resolving `@file` refs in a prompt.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    pub attachments: Vec<Attachment>,
    pub blocks: Vec<FileBlock>,
}

/// Detect `@path` tokens in `text`, returning each token's path (without the
/// leading `@`). A token starts at `@` preceded by start-of-string or
/// whitespace and runs to the next whitespace; common trailing punctuation is
/// trimmed so `see @a/b.ts.` yields `a/b.ts`.
pub fn detect_refs(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let at = bytes[i] == b'@';
        let boundary = i == 0 || bytes[i - 1].is_ascii_whitespace();
        if at && boundary {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && !bytes[end].is_ascii_whitespace() {
                end += 1;
            }
            let raw = &text[start..end];
            let trimmed = raw.trim_end_matches(['.', ',', ';', ':', ')', ']', '}', '"', '\'']);
            if !trimmed.is_empty() {
                refs.push(trimmed.to_string());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    refs
}

/// Resolve every `@file` ref in `prompt`, relative to `cwd`. The prompt text is
/// never modified — refs stay visible; this only produces attachments/blocks
/// for refs that pass every guard. A too-large file is a hard error (so the
/// user learns why); missing / directory / binary / gitignored refs are skipped.
pub fn resolve(prompt: &str, cwd: &Path, opts: &FileOptions) -> Result<Resolved> {
    let gitignore = if opts.respect_gitignore {
        build_gitignore(cwd)
    } else {
        None
    };

    let mut resolved = Resolved::default();
    let mut seen = std::collections::BTreeSet::new();

    for raw in detect_refs(prompt) {
        if !seen.insert(raw.clone()) {
            continue;
        }
        let candidate = resolve_path(cwd, &raw);

        let meta = match std::fs::metadata(&candidate) {
            Ok(m) if m.is_file() => m,
            // Missing or a directory → leave the ref visible.
            _ => continue,
        };

        if let Some(gi) = &gitignore {
            if gi.matched(&candidate, false).is_ignore() {
                continue;
            }
        }

        if meta.len() > opts.max_file_bytes {
            return Err(Error::FileTooLarge {
                path: candidate,
                bytes: meta.len(),
                max: opts.max_file_bytes,
            });
        }

        let data = std::fs::read(&candidate)
            .map_err(|e| Error::io(format!("reading {}", candidate.display()), e))?;
        if is_binary(&data) {
            continue;
        }
        let content = String::from_utf8_lossy(&data).into_owned();

        resolved.attachments.push(Attachment {
            path: raw.clone(),
            bytes: meta.len(),
            truncated: false,
        });
        resolved.blocks.push(FileBlock { path: raw, content });
    }

    Ok(resolved)
}

fn resolve_path(cwd: &Path, raw: &str) -> PathBuf {
    let p = Path::new(raw);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    }
}

fn build_gitignore(cwd: &Path) -> Option<Gitignore> {
    let gitignore_path = cwd.join(".gitignore");
    if !gitignore_path.is_file() {
        return None;
    }
    let mut builder = GitignoreBuilder::new(cwd);
    builder.add(&gitignore_path);
    builder.build().ok()
}

/// Heuristic: a NUL byte in the first 8 KiB marks the file as binary.
fn is_binary(data: &[u8]) -> bool {
    let sample = &data[..data.len().min(8192)];
    sample.contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn opts() -> FileOptions {
        FileOptions {
            max_file_bytes: 1000,
            respect_gitignore: true,
        }
    }

    #[test]
    fn detects_refs_at_boundaries() {
        let refs = detect_refs("fix @src/a.rs and email a@b.com but @./c.ts.");
        // `a@b.com` is not a ref (no whitespace/start boundary before `@`).
        assert_eq!(refs, vec!["src/a.rs".to_string(), "./c.ts".to_string()]);
    }

    #[test]
    fn resolves_relative_to_cwd() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let r = resolve("see @a.txt", dir.path(), &opts()).unwrap();
        assert_eq!(r.blocks.len(), 1);
        assert_eq!(r.blocks[0].content, "hello");
        assert_eq!(r.attachments[0].path, "a.txt");
        assert_eq!(r.attachments[0].bytes, 5);
    }

    #[test]
    fn unresolved_ref_is_left_visible() {
        let dir = tempdir().unwrap();
        let r = resolve("see @nope.txt", dir.path(), &opts()).unwrap();
        assert!(r.blocks.is_empty());
        assert!(r.attachments.is_empty());
    }

    #[test]
    fn binary_file_is_skipped() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("bin"), [0u8, 1, 2, 3]).unwrap();
        let r = resolve("@bin", dir.path(), &opts()).unwrap();
        assert!(r.blocks.is_empty(), "binary should not be included");
    }

    #[test]
    fn oversize_file_errors() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("big"), vec![b'x'; 2000]).unwrap();
        let err = resolve("@big", dir.path(), &opts()).unwrap_err();
        assert!(matches!(err, Error::FileTooLarge { .. }), "{err:?}");
    }

    #[test]
    fn gitignored_file_is_skipped_when_enabled() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "secret.txt\n").unwrap();
        fs::write(dir.path().join("secret.txt"), "shh").unwrap();
        let r = resolve("@secret.txt", dir.path(), &opts()).unwrap();
        assert!(r.blocks.is_empty(), "gitignored file should be skipped");

        let mut o = opts();
        o.respect_gitignore = false;
        let r = resolve("@secret.txt", dir.path(), &o).unwrap();
        assert_eq!(r.blocks.len(), 1, "included when gitignore disabled");
    }

    #[test]
    fn duplicate_refs_resolved_once() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        let r = resolve("@a.txt @a.txt", dir.path(), &opts()).unwrap();
        assert_eq!(r.blocks.len(), 1);
    }
}
