# Contract: CLI Commands

**Feature**: `001-shell-agent-acp` | **Date**: 2026-05-30

The `shap` binary is the contract surface. The Zsh layer maps `:` commands to these subcommands and
adds nothing semantically (FR-031/032). Every subcommand is usable directly without the shell.

Conventions:
- Exit code `0` = success; `1` = handled error (actionable message printed); `2` = usage error (clap).
- `--cwd <path>` is accepted by all commands; the shell forwards the current directory. Defaults to the
  process cwd when omitted.
- `--json` (where noted) prints machine-readable output for the shell/scripts.
- Human errors are rendered via `miette`; the message names a next step (often "run `shap doctor`").

## Mapping table

| Colon command | `shap` invocation | Notes |
|---------------|-------------------|-------|
| `: <prompt>` | `shap send "<prompt>"` | Free-form prompt to active agent. |
| `:agent [name]` | `shap agent [name] [--picker]` | No name → picker. |
| `:model [name]` | `shap model [name] [--picker]` | No name → picker (active agent's models only). |
| `:reasoning [level]` | `shap reasoning [level] [--picker]` | No level → picker. |
| `:effort [level]` | `shap reasoning [level]` | Alias — identical behavior. |
| `:new` | `shap new` | New session; keeps agent/model/reasoning. |
| `:status` | `shap status [--json]` | Show active agent/model/reasoning/session id. |
| `:commit` | `shap commit --prefill-shell-buffer` | Prints `git commit …`; shell inserts into buffer. |
| `:run <cmd>` | `shap run -- <cmd...>` | Run + capture output. |
| `:read <prompt>` | `shap read "<prompt>"` | Send prompt + last captured output. |
| `:doctor` | `shap doctor [--json]` | Validate installation/agents. |

## Subcommand contracts

### `shap send <prompt>`
- **Args**: `<prompt>` (positional, required); `--cwd`.
- **Pre**: an agent is active (or `default_agent` resolvable); agent command available.
- **Behavior**: expands `@file` refs in prompt; ensures/creates active session; starts/continues an ACP
  session; streams response if `ui.stream`, else spinner then final text; appends `user_prompt` and
  `agent_response` records.
- **Errors**: no agent configured → setup instructions (exit 1); agent command missing → "run shap
  doctor" (exit 1); agent unavailable mid-stream → readable partial-state message (exit 1).

### `shap agent [name] [--picker]`
- **Args**: optional `<name>`; `--picker` forces the picker.
- **Behavior**: with `name` → set active agent (must be configured); without → open picker of configured
  agents. On switch, reset `active_model` to the new agent's `default_model` if the current model is invalid.
- **Errors**: unknown name → list configured agents (exit 1).

### `shap model [name] [--picker]`
- **Behavior**: offers/sets a model from the **active agent's** `models` only (SC-002). No agent active →
  prompt to select an agent first.
- **Errors**: model not in active agent's list → reject with the valid list (exit 1).

### `shap reasoning [level] [--picker]`
- **Behavior**: offers/sets reasoning effort from supported levels (`low`/`medium`/`high` by default).
- **Alias**: invoked by both `:reasoning` and `:effort`.

### `shap new`
- **Behavior**: create a new Session (new id + file), set `active_session_id`; agent/model/reasoning
  unchanged (SC-008). Prints the new session id.

### `shap status [--json]`
- **Behavior**: print active agent, model, reasoning, and session id. `--json` emits an object with
  those fields (consumed by the prompt segment). Unset fields shown as `-`/null.

### `shap commit --prefill-shell-buffer`
- **Pre**: inside a Git repo; `git` available.
- **Behavior**: read diff (staged preferred, else unstaged) + branch + short status; ask active agent for
  a commit message; print a single line `git commit -am "<message>"` to stdout for buffer insertion.
  **Never executes `git commit`** (FR-020, SC-003).
- **Errors**: not a repo → clear message (exit 1); nothing to commit → message, no command (exit 0).

### `shap run -- <command...>`
- **Behavior**: execute `<command...>` via the shell-words-split argv under Tokio; stream stdout+stderr
  to the terminal live; capture combined output (truncated to `history.max_output_bytes`), exit code, and
  command metadata to the capture store. Returns the child's exit code.
- **Note**: `--` separates `shap` flags from the user command.

### `shap read <prompt>`
- **Args**: `<prompt>` (positional, required); also accepts piped stdin (`… | shap read "<prompt>"`).
- **Behavior**: load latest CapturedOutput (or stdin in pipe mode), compose `PromptPayload`
  (user prompt + command + exit code + output), send to the agent like `send`.
- **Errors**: no captured output and no stdin → "nothing captured; run `:run <cmd>` first" (exit 1).

### `shap doctor [--json]`
- **Behavior**: run all checks from research D11; print a per-check pass/fail report with remediation for
  failures. `--json` emits structured results. Exit `0` if all critical checks pass, else `1`.

## Prompt-payload contract (`:read`)

`shap read` MUST compose this exact shape (snapshot-tested):

```text
User prompt:
<prompt>

Previous command:
<command>

Exit code:
<exit_code>

Output:
<captured output, possibly truncated>
```
