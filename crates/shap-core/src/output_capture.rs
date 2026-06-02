//! Captured command output store.
//!
//! Holds the most recent `:run`/pipe capture: combined stdout+stderr (truncated
//! to `max_output_bytes`), the command, its exit code, and a timestamp. The
//! text lives in `last-command-output.txt`; metadata in a sibling JSON file.
//! Each capture overwrites the previous one (MVP keeps only the latest).

use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::macros::format_description;

use crate::error::{Error, Result};

/// On-disk metadata sitting beside the captured text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMeta {
    pub command: String,
    pub exit_code: Option<i32>,
    pub captured_at: String,
    pub truncated: bool,
}

/// A loaded capture (metadata + text).
#[derive(Debug, Clone)]
pub struct CapturedOutput {
    pub command: String,
    pub exit_code: Option<i32>,
    pub captured_at: String,
    pub output: String,
    pub truncated: bool,
}

/// Persist a capture, truncating `output` to `max_output_bytes` (at a UTF-8
/// boundary). Returns whether truncation occurred.
pub fn save(
    output_path: &Path,
    meta_path: &Path,
    command: &str,
    exit_code: Option<i32>,
    output: &str,
    max_output_bytes: u64,
) -> Result<bool> {
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
    }

    let (text, truncated) = truncate(output, max_output_bytes);
    std::fs::write(output_path, text.as_bytes())
        .map_err(|e| Error::io(format!("writing {}", output_path.display()), e))?;

    let meta = CaptureMeta {
        command: command.to_string(),
        exit_code,
        captured_at: now_timestamp(),
        truncated,
    };
    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| Error::AgentProtocol(format!("serializing capture meta: {e}")))?;
    std::fs::write(meta_path, meta_json)
        .map_err(|e| Error::io(format!("writing {}", meta_path.display()), e))?;
    Ok(truncated)
}

/// Load the most recent capture. A missing capture is [`Error::NoCapturedOutput`].
pub fn load(output_path: &Path, meta_path: &Path) -> Result<CapturedOutput> {
    let meta_text = match std::fs::read_to_string(meta_path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(Error::NoCapturedOutput),
        Err(e) => return Err(Error::io(format!("reading {}", meta_path.display()), e)),
    };
    let meta: CaptureMeta = serde_json::from_str(&meta_text)
        .map_err(|e| Error::AgentProtocol(format!("parsing capture meta: {e}")))?;
    let output = std::fs::read_to_string(output_path).unwrap_or_default();
    Ok(CapturedOutput {
        command: meta.command,
        exit_code: meta.exit_code,
        captured_at: meta.captured_at,
        output,
        truncated: meta.truncated,
    })
}

fn truncate(text: &str, max_bytes: u64) -> (&str, bool) {
    let max = max_bytes as usize;
    if text.len() <= max {
        return (text, false);
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    (&text[..end], true)
}

fn now_timestamp() -> String {
    let fmt = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    OffsetDateTime::now_utc().format(&fmt).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("out.txt");
        let meta = dir.path().join("out.json");
        let truncated = save(&out, &meta, "cargo test", Some(101), "error here", 1000).unwrap();
        assert!(!truncated);
        let loaded = load(&out, &meta).unwrap();
        assert_eq!(loaded.command, "cargo test");
        assert_eq!(loaded.exit_code, Some(101));
        assert_eq!(loaded.output, "error here");
        assert!(!loaded.truncated);
    }

    #[test]
    fn truncation_flagged() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("out.txt");
        let meta = dir.path().join("out.json");
        let big = "x".repeat(500);
        let truncated = save(&out, &meta, "yes", None, &big, 100).unwrap();
        assert!(truncated);
        let loaded = load(&out, &meta).unwrap();
        assert_eq!(loaded.output.len(), 100);
        assert!(loaded.truncated);
    }

    #[test]
    fn missing_capture_errors() {
        let dir = tempdir().unwrap();
        let err = load(&dir.path().join("none.txt"), &dir.path().join("none.json")).unwrap_err();
        assert!(matches!(err, Error::NoCapturedOutput), "{err:?}");
    }

    #[test]
    fn truncation_respects_utf8_boundary() {
        let dir = tempdir().unwrap();
        let out = dir.path().join("o.txt");
        let meta = dir.path().join("o.json");
        // 'é' is 2 bytes; cut at an odd boundary must not split it.
        let text = "aéaéaé";
        save(&out, &meta, "c", None, text, 3).unwrap();
        let loaded = load(&out, &meta).unwrap();
        assert!(loaded.output.is_char_boundary(loaded.output.len()));
    }
}
