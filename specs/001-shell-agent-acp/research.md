# Phase 0 Research: Shell-Native Interface for Coding Agents

**Feature**: `001-shell-agent-acp` | **Date**: 2026-05-30

The planning input specified the stack in full, so there are no open `NEEDS CLARIFICATION` items.
This document records each decision, why it was chosen, and the alternatives weighed — so the choices
are reviewable against the constitution (especially Principles I, II, and IX).

## D1. Language & runtime — Rust + Tokio

- **Decision**: Rust (latest stable) with Tokio (`process`, `io-util`, `macros`, `rt-multi-thread`)
  for spawning agent/command processes, streaming stdio, cancellation, and timeouts.
- **Rationale**: A fast, portable, single-binary CLI with low startup overhead — required for a
  native-feeling shell tool. Async is needed to stream agent output and child-process I/O concurrently.
- **Alternatives**: Go (good single-binary story, but no official ACP SDK and weaker type guarantees
  for protocol modelling); Node/TS (heavier startup, conflicts with the "native, fast" goal).

## D2. ACP integration — official SDK, wrapped behind a trait

- **Decision**: Depend on `agent-client-protocol` (its built-in stdio transport bridged to Tokio child
  pipes via `tokio-util` `compat`). Wrap it behind an `AgentClient` trait in **`shap-core::agent`**
  (so `shap-core::commands` can call it without a cycle); `shap-agent` implements the trait and is the
  only crate that touches SDK types.
  - **Revised during implementation**: `agent-client-protocol-tokio` proved unnecessary, and
    `agent-client-protocol-test` does not exist on crates.io — ACP integration is tested by mocking the
    `AgentClient` trait. The shipped SDK is `agent-client-protocol` 0.12.
- **Rationale**: Principle IX (don't hand-roll the protocol) and Principle III/X (isolate vendor
  surface so command handlers stay SDK-agnostic and FR-035 stays achievable).
- **Alternatives**: Hand-rolling ACP message types (more code, more drift risk, rejected); coupling
  handlers directly to SDK types (breaks the abstraction boundary, rejected).
- **Open item for implementation**: confirm the exact `agent-client-protocol` 0.11 surface for session
  start, prompt send, and streaming, and shape the `AgentClient` trait to match. Verified against the
  SDK during step "ACP SDK integration", not now.

## D3. Agents as external, configured processes

- **Decision**: Agents are external commands declared in config (`[agents.<name>].command`), launched
  as child processes and connected over stdio. No agent is hard-coded in handlers.
- **Rationale**: FR-021/022/035 — pluggable, vendor-neutral, agent-specific config preserved.
- **Alternatives**: Built-in per-agent adapters compiled in (couples the binary to vendors, rejected).

## D4. CLI surface — clap derive subcommands

- **Decision**: `clap` (derive, env) with explicit subcommands: `send`, `agent`, `model`, `reasoning`,
  `new`, `status`, `commit`, `run`, `read`, `doctor` (+ `--picker` flags and `commit --prefill-shell-buffer`).
  `clap_complete` generates completions.
- **Rationale**: The shell layer maps `:` commands to these subcommands (FR-032); explicit subcommands
  keep the binary usable standalone (FR-031).
- **Alternatives**: A single positional dispatcher parsed by hand (worse help/errors, rejected).

## D5. Shell integration — thin Zsh layer + ZLE widget for `:commit`

- **Decision**: A single `shell/zsh/shap.zsh`: defines `:`-prefixed functions that forward to `shap`,
  an optional prompt segment, and a ZLE widget that inserts the generated `git commit` command into the
  buffer (`print -z` / `BUFFER`) for the user to review and run.
- **Rationale**: Principle VIII (respect the shell) and VII (user control — never auto-execute). Logic
  stays in Rust; the shell only maps and inserts.
- **Performance note**: The prompt segment must not spawn `shap` on every prompt render (Principle V,
  NFR-2). It reads the small `state.json`/a cached segment string instead. Verified by a render-latency
  check during the prompt-segment task.
- **Alternatives**: A heavier shell framework or per-prompt subprocess (slows prompt rendering, rejected).

## D6. Pickers — external first, built-in fallback

- **Decision**: Resolve a picker at runtime in priority order: `fzf` → `skim` → built-in numbered prompt
  (`dialoguer`). `which` detects availability; `[ui].picker` expresses preference. Ratatui/crossterm are
  feature-gated and deferred.
- **Rationale**: Principle I/II — avoid building a TUI before it's needed; reuse what the user already has.
- **Alternatives**: Ship a Ratatui picker in the MVP (premature, rejected for now).

## D7. Git — local `git` CLI, not a library

- **Decision**: Shell out to `git status --short`, `git diff --staged`, `git diff`,
  `git branch --show-current`. For `:commit`: prefer staged diff, else unstaged; include branch + status
  as context; ask the agent for a message; prefill the buffer; never run `git commit`.
- **Rationale**: Respects the user's existing Git config, hooks, aliases, signing, and worktrees
  (Principle VII). `gix` deferred behind a "later" flag.
- **Alternatives**: `gix`/`git2` (reimplements behavior users already configured, heavier, rejected for MVP).

## D8. Persistence — local files (TOML / JSON / JSONL)

- **Decision**: TOML for user config (`~/.config/shap/config.toml`), JSON for machine state
  (`state.json`), JSONL for session logs (`sessions/*.jsonl`), plain text for captured output
  (`last-command-output.txt`). `serde` ecosystem; `schemars` to publish a config schema.
- **Rationale**: Human-editable config (NFR-5, FR-024); append-friendly session logs; trivial to debug.
  Principle II — no database until session search is actually needed.
- **Alternatives**: SQLite now (`rusqlite` deferred — only justified once search/timeline/indexed history
  is required).

## D9. Command-output capture & `@file` references

- **Decision**: `:run <cmd>` executes via Tokio, streams to the terminal, captures stdout+stderr, exit
  code, and metadata to `last-command-output.txt`; `:read <prompt>` composes prompt + captured output.
  Pipe mode (`… | shap read "…"`) is an additional explicit capture path. `@path` tokens are detected,
  resolved against cwd, read if present (binary rejected, size-bounded by `[files].max_file_bytes`,
  `respect_gitignore` honored via `ignore`/`globset`); unresolved `@refs` stay visible in the prompt.
- **Rationale**: Explicit, predictable capture (Principle VII); bounded payloads (Principle VI, no
  surprise huge sends). Automatic scrollback capture is out of scope for the MVP.
- **Alternatives**: Auto-capture every command via shell hooks (invasive, deferred per NFR-8).

## D10. Output rendering & diagnostics

- **Decision**: Plain terminal output by default; `indicatif` spinner for non-streamed mode; `anstream`/
  `anstyle`/`console`/`terminal_size` for styling/width. Errors via `miette` (fancy) with actionable
  messages; `tracing` for logs gated by env filter. Faithful passthrough of agent output (no rich
  markdown engine in the MVP).
- **Rationale**: Principle VI (clear, actionable failures) and Principle I/V (no over-built rendering).
- **Alternatives**: A full markdown renderer (premature, rejected).

## D11. `:doctor` self-check

- **Decision**: `:doctor` validates: config exists/parses; configured agent commands exist on PATH;
  selected agent available; selected model valid for the agent; picker (`fzf`/`skim`) present if
  configured; `git` available; session dir writable; an agent process can launch; shell integration
  installed.
- **Rationale**: Principle VI — turn misconfiguration into a guided fix; backs FR-027/028/029/030.

## D12. Testing strategy

- **Decision**: Unit tests for parsing/config/state/session/prompt composition/capture; `assert_cmd` +
  `predicates` integration tests on the binary; `insta` snapshots for diagnostics, generated prompts,
  commit messages, status, and doctor output; `tempfile` for isolated FS; the ACP wrapper is tested by
  mocking the `AgentClient` trait (the `agent-client-protocol-test` crate does not exist); `wiremock`
  only if HTTP appears.
- **Rationale**: Principle IV — meaningful logic and generated text are exactly what regressions hide in.
- **Alternatives**: Manual testing only (rejected — generated prompts/messages need snapshot guards).

## D13. Packaging & CI

- **Decision**: `cargo-dist` for release binaries (macOS arm64/x86_64, Linux x86_64/arm64; Windows
  later). GitHub Actions runs `cargo fmt --check`, `clippy -D warnings`, `cargo test`/`nextest`,
  `cargo audit`, `cargo deny check`.
- **Rationale**: Reproducible cross-platform releases; CI enforces the dependency-hygiene mitigations
  for Principle IX.
- **Alternatives**: Manual release builds (error-prone, rejected).

## Resolved unknowns

| Question | Resolution |
|----------|------------|
| Reasoning-effort levels | Start with a small fixed set (`low`/`medium`/`high`); allow per-agent override where an agent exposes different levels. Confirmed as an assumption in the spec. |
| Picker when none available | Fall back to a built-in numbered prompt; if non-interactive, instruct the user to pass the value directly. |
| Edition | Target stable; use edition 2024 if the pinned toolchain supports it, else 2021. Decided at workspace-setup time. |
| Exact ACP 0.11 API shape | Pinned against the SDK during the ACP-integration step; `AgentClient` trait adapts to it. |
