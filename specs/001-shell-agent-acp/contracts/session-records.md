# Contract: Session JSONL Records

**Feature**: `001-shell-agent-acp` | **Date**: 2026-05-30

Defines the append-only session log format. One file per session at `<history.dir>/<session_id>.jsonl`.
Each line is one JSON object tagged by `type`. The file is append-only and never rewritten; resume
(future) replays records in order.

## File naming

```text
<history.dir>/<session_id>.jsonl
# session_id: <RFC3339-ish timestamp with ':' → '-'>-<agent>
# example:
~/.local/share/shap/sessions/2026-05-30T12-33-10Z-codex.jsonl
```

## Record types

### `session_started` (always the first line)

```json
{"type":"session_started","session_id":"2026-05-30T12-33-10Z-codex","agent":"codex","model":"gpt-5-thinking","created_at":"2026-05-30T12:33:10Z"}
```

| Field | Type | Required |
|-------|------|----------|
| `type` | `"session_started"` | yes |
| `session_id` | string | yes |
| `agent` | string | yes |
| `model` | string | yes |
| `created_at` | RFC3339 string | yes |

### `user_prompt`

```json
{"type":"user_prompt","content":"fix the error in @test/server.ts","cwd":"/Users/dawid/project","attachments":[{"path":"test/server.ts","bytes":1234,"truncated":false}],"captured_output_ref":null}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `type` | `"user_prompt"` | yes | |
| `content` | string | yes | Raw user prompt (with original `@refs` text preserved). |
| `cwd` | string | yes | Working dir at send time. |
| `attachments` | array | no | Resolved `@file` inclusions: `{path, bytes, truncated}`. Omitted/empty if none. |
| `captured_output_ref` | string \| null | no | Marker that captured command output was included (`:read`). |

### `agent_response`

```json
{"type":"agent_response","content":"The error is caused by..."}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `type` | `"agent_response"` | yes | |
| `content` | string | yes | Final text. Streamed chunks are reassembled before writing. |

### `error`

```json
{"type":"error","message":"agent 'codex' became unavailable: broken pipe"}
```

| Field | Type | Required |
|-------|------|----------|
| `type` | `"error"` | yes |
| `message` | string | yes |

## Rules

- **Append-only**: writers only append lines; corrupt/partial trailing lines are tolerated on read
  (skip + warn) so a crash mid-write never destroys a session.
- **Ordering**: records reflect chronological order; resume replays them as-is.
- **Forward compatibility**: unknown `type` values and unknown fields are ignored on read (not an error),
  so new record types can be added later without breaking old sessions.
- **Privacy**: files are local-only and contain prompt/response text and file excerpts; no remote sync
  in the MVP.

## Forward-compatibility note (resume — future)

Resume is out of scope for the MVP (FR-014) but the format supports it: replaying
`session_started` + ordered `user_prompt`/`agent_response` records reconstructs conversation context
for a later ACP session. No schema change is anticipated to enable resume.
