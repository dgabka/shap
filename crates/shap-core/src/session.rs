//! Append-only JSONL session store.
//!
//! One file per session at `<history.dir>/<session_id>.jsonl`. Each line is one
//! tagged [`Record`]. Writers only append; the file is never rewritten. On read,
//! corrupt/partial trailing lines are skipped (with a warning) so a crash
//! mid-write never destroys a session (contract: `session-records.md`).

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::macros::format_description;

use crate::error::{Error, Result};
use crate::files::Attachment;

/// A single session-log line, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Record {
    SessionStarted {
        session_id: String,
        agent: String,
        model: String,
        created_at: String,
    },
    UserPrompt {
        content: String,
        cwd: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attachments: Vec<Attachment>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        captured_output_ref: Option<String>,
    },
    AgentResponse {
        content: String,
    },
    Error {
        message: String,
    },
}

/// A handle to one session's JSONL file.
#[derive(Debug, Clone)]
pub struct Session {
    id: String,
    path: PathBuf,
}

impl Session {
    /// Create a new session: generate a timestamped id, ensure the directory,
    /// and write the `session_started` line.
    pub fn create(dir: &Path, agent: &str, model: &str) -> Result<Session> {
        let now = OffsetDateTime::now_utc();
        let id = format!("{}-{}", format_id(now), agent);
        let created_at = format_timestamp(now);
        let session = Session::at(dir, &id);

        std::fs::create_dir_all(dir)
            .map_err(|e| Error::io(format!("creating {}", dir.display()), e))?;
        session.append(&Record::SessionStarted {
            session_id: id.clone(),
            agent: agent.to_string(),
            model: model.to_string(),
            created_at,
        })?;
        Ok(session)
    }

    /// Handle to an existing session file by id (no I/O).
    pub fn at(dir: &Path, id: &str) -> Session {
        Session {
            id: id.to_string(),
            path: dir.join(format!("{id}.jsonl")),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.is_file()
    }

    pub fn log_user_prompt(
        &self,
        content: &str,
        cwd: &str,
        attachments: Vec<Attachment>,
        captured_output_ref: Option<String>,
    ) -> Result<()> {
        self.append(&Record::UserPrompt {
            content: content.to_string(),
            cwd: cwd.to_string(),
            attachments,
            captured_output_ref,
        })
    }

    pub fn log_agent_response(&self, content: &str) -> Result<()> {
        self.append(&Record::AgentResponse {
            content: content.to_string(),
        })
    }

    pub fn log_error(&self, message: &str) -> Result<()> {
        self.append(&Record::Error {
            message: message.to_string(),
        })
    }

    fn append(&self, record: &Record) -> Result<()> {
        let line = serde_json::to_string(record)
            .map_err(|e| Error::AgentProtocol(format!("serializing record: {e}")))?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| Error::io(format!("opening {}", self.path.display()), e))?;
        file.write_all(line.as_bytes())
            .and_then(|()| file.write_all(b"\n"))
            .map_err(|e| Error::io(format!("appending to {}", self.path.display()), e))?;
        Ok(())
    }

    /// Read all well-formed records in order, skipping corrupt/partial lines
    /// (with a warning). Used by tests now and resume later.
    pub fn read_records(&self) -> Result<Vec<Record>> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(Error::io(format!("reading {}", self.path.display()), e)),
        };
        let mut records = Vec::new();
        for (n, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Record>(line) {
                Ok(r) => records.push(r),
                Err(e) => tracing::warn!(line = n + 1, error = %e, "skipping corrupt session line"),
            }
        }
        Ok(records)
    }
}

fn format_id(now: OffsetDateTime) -> String {
    // ':' is illegal in many filenames, so the id uses '-' for time separators.
    // Millisecond precision keeps ids unique even for rapid `:new` calls.
    let fmt =
        format_description!("[year]-[month]-[day]T[hour]-[minute]-[second]-[subsecond digits:3]Z");
    now.format(&fmt).unwrap_or_else(|_| "session".to_string())
}

fn format_timestamp(now: OffsetDateTime) -> String {
    let fmt = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]Z");
    now.format(&fmt).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_writes_session_started_first() {
        let dir = tempdir().unwrap();
        let s = Session::create(dir.path(), "codex", "gpt-5").unwrap();
        assert!(s.exists());
        assert!(s.id().ends_with("-codex"));
        let records = s.read_records().unwrap();
        assert_eq!(records.len(), 1);
        assert!(matches!(records[0], Record::SessionStarted { .. }));
    }

    #[test]
    fn appends_in_order() {
        let dir = tempdir().unwrap();
        let s = Session::create(dir.path(), "codex", "gpt-5").unwrap();
        s.log_user_prompt("hi", "/tmp", vec![], None).unwrap();
        s.log_agent_response("hello").unwrap();
        let records = s.read_records().unwrap();
        assert_eq!(records.len(), 3);
        assert!(matches!(records[1], Record::UserPrompt { .. }));
        assert!(matches!(records[2], Record::AgentResponse { .. }));
    }

    #[test]
    fn attachments_omitted_when_empty() {
        let dir = tempdir().unwrap();
        let s = Session::create(dir.path(), "a", "m").unwrap();
        s.log_user_prompt("hi", "/tmp", vec![], None).unwrap();
        let line = std::fs::read_to_string(s.path()).unwrap();
        assert!(
            !line.contains("attachments"),
            "empty attachments must be omitted"
        );
        assert!(!line.contains("captured_output_ref"));
    }

    #[test]
    fn corrupt_trailing_line_is_skipped() {
        let dir = tempdir().unwrap();
        let s = Session::create(dir.path(), "a", "m").unwrap();
        s.log_agent_response("ok").unwrap();
        // Simulate a crash mid-write.
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(s.path())
            .unwrap();
        f.write_all(b"{ partial").unwrap();
        let records = s.read_records().unwrap();
        assert_eq!(records.len(), 2, "corrupt line skipped, good ones kept");
    }

    #[test]
    fn id_has_no_colons() {
        let dir = tempdir().unwrap();
        let s = Session::create(dir.path(), "codex", "m").unwrap();
        assert!(!s.id().contains(':'), "session id must be filename-safe");
    }
}
