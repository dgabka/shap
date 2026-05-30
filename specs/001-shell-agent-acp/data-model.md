# Phase 1 Data Model: Shell-Native Interface for Coding Agents

**Feature**: `001-shell-agent-acp` | **Date**: 2026-05-30

Entities below are derived from the spec's Key Entities and the FRs. Types are described
language-agnostically; concrete Rust structs live in `shap-core` (config/state/session) and
`shap-agent` (agent runtime). All persistence is local (see [contracts/](./contracts/)).

## Entity overview

```text
Configuration (config.toml)            ActiveState (state.json)
  ├─ default_agent                       ├─ active_agent  ──┐
  ├─ Agent[] (by name)                   ├─ active_model    │ must reference a
  │    ├─ command                        ├─ active_reasoning│ configured Agent/Model
  │    ├─ models[] (Model)               ├─ active_session_id ─┐
  │    ├─ default_model                  └─ last_cwd            │
  │    └─ agent-specific config                                 │
  ├─ UiOptions                                                  ▼
  ├─ HistoryOptions                       Session (sessions/<id>.jsonl)
  └─ FileOptions                            ├─ id, agent, model, created_at
                                            └─ SessionRecord[] (append-only)

CapturedOutput (last-command-output.txt + metadata)
  ├─ command, exit_code, captured_at
  └─ output (bounded by max_output_bytes)
```

## 1. Configuration

User-editable TOML at `~/.config/shap/config.toml`. Loaded read-only by the tool.

| Field | Type | Rules |
|-------|------|-------|
| `default_agent` | string | MUST match a key in `agents`. Used when no active agent is set. |
| `agents` | map<name → Agent> | At least one required for the tool to function (else FR-029 setup message). |
| `ui` | UiOptions | Optional; defaults applied if absent. |
| `history` | HistoryOptions | Optional; defaults applied if absent. |
| `files` | FileOptions | Optional; defaults applied if absent. |

### 1.1 Agent

| Field | Type | Rules |
|-------|------|-------|
| `command` | string | Launch command for the external ACP agent process. Validated by `:doctor` (must resolve on PATH). |
| `models` | string[] | Non-empty. Defines the only valid models for this agent (FR-005, SC-002). |
| `default_model` | string | MUST be a member of `models`. |
| *(passthrough)* | table | Arbitrary agent-specific config forwarded to the agent (FR-022). Not interpreted by `shap`. |

### 1.2 UiOptions

| Field | Type | Default | Rules |
|-------|------|---------|-------|
| `stream` | bool | `true` | Streamed vs. loader-then-final output (FR-003). |
| `picker` | enum(`fzf`,`skim`,`builtin`) | `fzf` | Preference; runtime falls back if unavailable (D6). |
| `show_prompt_segment` | bool | `true` | Toggles the prompt segment (FR-008). |

### 1.3 HistoryOptions

| Field | Type | Default | Rules |
|-------|------|---------|-------|
| `dir` | path | `~/.local/share/shap/sessions` | `~` expanded; must be writable (checked by `:doctor`). |
| `capture_last_output` | bool | `false` | Reserved for future auto-capture; MVP capture is explicit. |
| `max_output_bytes` | int | `200000` | Upper bound on captured output sent to an agent (FR-018). MUST be > 0. |

### 1.4 FileOptions

| Field | Type | Default | Rules |
|-------|------|---------|-------|
| `max_file_bytes` | int | `200000` | Max size of an `@file` inclusion (FR / D9). MUST be > 0. |
| `respect_gitignore` | bool | `true` | When true, ignored files are skipped during `@file` resolution. |

## 2. ActiveState

Machine-written JSON at `~/.local/share/shap/state.json`. Persists selections across shells (FR-012).

| Field | Type | Rules |
|-------|------|-------|
| `active_agent` | string \| null | If set, MUST reference a configured agent. |
| `active_model` | string \| null | If set, MUST be in the active agent's `models`. |
| `active_reasoning` | string \| null | One of the supported reasoning levels (e.g., `low`/`medium`/`high`). |
| `active_session_id` | string \| null | If set, MUST reference an existing session file. |
| `last_cwd` | path \| null | Last working directory forwarded by the shell. |

**Transitions**
- `:agent <name>` → sets `active_agent`; if `active_model` is now invalid for the new agent, reset it
  to that agent's `default_model`.
- `:model <name>` → sets `active_model` (must be valid for `active_agent`).
- `:reasoning|:effort <level>` → sets `active_reasoning`.
- `:new` → creates a new Session, sets `active_session_id`; leaves agent/model/reasoning unchanged (FR-010, SC-008).
- First prompt with no active session → lazily create a session and set `active_session_id`.

## 3. Session

A persisted conversation. One JSONL file per session at `<history.dir>/<id>.jsonl` (FR-013).

| Field | Type | Rules |
|-------|------|-------|
| `id` | string | Unique; timestamped, e.g. `2026-05-30T12-33-10Z-codex`. Filename stem. |
| `agent` | string | Agent the session belongs to. |
| `model` | string | Model at session creation. |
| `created_at` | timestamp (RFC3339) | Set once. |
| `records` | SessionRecord[] | Append-only event log (see below). |

### 3.1 SessionRecord (JSONL line)

Tagged by `type`:

| `type` | Fields | Meaning |
|--------|--------|---------|
| `session_started` | `session_id`, `agent`, `model`, `created_at` | First line of the file. |
| `user_prompt` | `content`, `cwd`, optional `attachments` (resolved `@files`), optional `captured_output_ref` | A prompt the user sent. |
| `agent_response` | `content` | The agent's reply (final text; streaming is reassembled). |
| `error` | `message` | A recorded failure (e.g., agent unavailable). |

**Rules**: append-only; the file is never rewritten; resume (future) replays records in order.

## 4. CapturedOutput

The most recent `:run`/pipe capture, available to `:read` (FR-015/016/017).

| Field | Type | Rules |
|-------|------|-------|
| `command` | string | The command that was run (omitted/`<pipe>` in pipe mode). |
| `exit_code` | int \| null | Process exit code (null if unknown, e.g., pipe mode). |
| `captured_at` | timestamp | When captured. |
| `output` | text | Combined stdout+stderr, truncated to `history.max_output_bytes` (note truncation in the payload). |

**Storage**: output text in `~/.local/share/shap/last-command-output.txt`; metadata alongside (e.g., a
sibling `.json` or a header). Overwritten by each new capture (MVP keeps only the latest).

## 5. Runtime (non-persisted) entities

These exist only during a command invocation; listed for design clarity, not stored.

| Entity | Owned by | Purpose |
|--------|----------|---------|
| `AgentRequest` | `shap-agent` | Composed prompt + context (attachments, captured output) sent to the agent. |
| `AgentResponse` / `AgentStream` | `shap-agent` | Final response, or a stream of chunks for streamed mode. |
| `SessionOptions` | `shap-agent` | Inputs to start an ACP session (agent, model, reasoning, cwd). |
| `PromptPayload` | `shap-core::prompt` | The composed text for `:read`/`:commit` (snapshot-tested). |

## Validation summary (enforced in `shap-core`)

- A configured agent's `default_model` ∈ its `models`.
- `active_model` (if set) ∈ `active_agent.models` — repaired on agent switch.
- `max_output_bytes`, `max_file_bytes` > 0.
- `default_agent` ∈ `agents`.
- `@file` inclusions: path exists, not binary, ≤ `max_file_bytes`, not gitignored (when `respect_gitignore`).
- Missing/invalid config or agent → actionable diagnostic, never a silent failure (FR-028/029/030).
