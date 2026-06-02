# Quickstart: shap

**Feature**: `001-shell-agent-acp` | **Date**: 2026-05-30

How a contributor builds, configures, and exercises `shap` once the MVP exists. This doubles as the
manual acceptance walkthrough for the spec's user stories.

## Prerequisites

- Rust stable toolchain (`rustup`), `cargo`
- An ACP-compatible agent adapter on PATH (e.g. `codex-acp` or `claude-agent-acp`)
- Zsh (for the shell integration)
- Optional: `fzf` or `skim` for nicer pickers; `git` for `:commit`

## Build

```sh
cargo build --workspace
# binary at target/debug/shap
cargo nextest run --workspace   # or: cargo test --workspace
```

## Configure

Create `~/.config/shap/config.toml`:

```toml
default_agent = "codex"

[agents.codex]
command = "codex-acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"

[ui]
stream = true
picker = "fzf"
show_prompt_segment = true
```

Validate the setup:

```sh
shap doctor
```

`doctor` reports config validity, agent availability, picker/git presence, and session-dir writability,
with a remediation line for each failure.

## Install the Zsh integration

```sh
echo 'source /path/to/shap/shell/zsh/shap.zsh' >> ~/.zshrc
exec zsh
```

This defines the `:` commands, the optional prompt segment, and the ZLE widget used by `:commit`.

## Try it (maps to the spec's user stories)

```sh
# US2 — select agent / model / reasoning (pickers when value omitted)
:agent                 # picker of configured agents → pick codex
:model                 # picker of codex's models only → pick gpt-5
:reasoning             # picker of levels → pick high
# prompt segment now shows:  ~/project [shap codex·gpt-5·high] $

# US1 — chat
: hello                            # streamed reply appears
: fix the error in @test/server.ts # @file contents included

# US3 — sessions
:status                # active agent/model/reasoning/session id
:new                   # fresh session; agent/model/reasoning preserved

# US4 — feed command output
:run pnpm test         # runs + captures output
:read fix the failing test   # sends prompt + captured output

# US5 — commit message (prefill only, never auto-runs)
:commit                # buffer prefilled with: git commit -am "fix(...): ..."
                       # review, edit, press Enter yourself

# US6 — diagnostics
:doctor
```

## Use the CLI directly (no shell layer — FR-031)

Every colon command has a direct equivalent:

```sh
shap agent codex
shap model gpt-5
shap reasoning high
shap send "hello"
shap run -- pnpm test
shap read "fix the failing test"
shap commit --prefill-shell-buffer
shap status --json
shap doctor
pnpm test 2>&1 | shap read "fix the test"   # pipe mode
```

## Acceptance smoke checklist

- [ ] `shap agent` (no arg) opens a picker; `shap agent codex` sets it without one.
- [ ] `shap model` offers only the active agent's models.
- [ ] `shap reasoning` / `:effort` behave identically.
- [ ] `shap send "hello"` returns agent output; streaming toggles via `[ui].stream`.
- [ ] `shap new` starts a new session, preserving agent/model/reasoning.
- [ ] `shap status` shows agent/model/reasoning/session id.
- [ ] `shap run -- pnpm test` captures output; `shap read "…"` includes it.
- [ ] `shap commit` prints a `git commit` line and never executes it.
- [ ] Config + sessions survive a shell restart.
- [ ] Missing agent command → clear error suggesting `shap doctor`; no agents configured → setup help.
- [ ] Prompt segment can be toggled via `[ui].show_prompt_segment`.
- [ ] Prompt-segment rendering is instant (well under 50 ms) and reads only cached state — `shap
      prompt-segment` performs no config parse and never contacts an agent (SC-006).
```
