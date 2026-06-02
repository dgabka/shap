# Implementation Plan: Shell-Native Interface for Coding Agents

**Branch**: `001-shell-agent-acp` | **Date**: 2026-05-30 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-shell-agent-acp/spec.md`

## Summary

`shap` is a native CLI binary (Rust + Tokio) plus a thin Zsh integration that lets developers talk
to ACP-compatible coding agents from the terminal via colon-prefixed commands. All product logic
lives in the Rust binary: configuration, state, sessions, agent registry, ACP communication (via the
official Agent Client Protocol SDK), command-output capture, `@file` expansion, Git commit-message
generation, and output rendering. The shell layer only maps `:` commands to `shap` subcommands,
renders an optional prompt segment, and inserts generated commands into the shell buffer for review.

Technical approach: a Cargo workspace with a thin CLI binary over reusable libraries; agents launched
as external ACP processes over child-process stdio; local file persistence (TOML config, JSON state,
JSONL sessions); external pickers (fzf/skim) with a built-in fallback; the local `git` CLI for Git
work; `miette`-based diagnostics and a `:doctor` self-check.

## Technical Context

**Language/Version**: Rust, latest stable toolchain (edition 2021; adopt 2024 if toolchain allows)

**Primary Dependencies**:
- Async/process: `tokio` (process, io-util, macros, rt-multi-thread), `tokio-util`, `futures`, `async-trait`
- CLI: `clap` (derive, env), `clap_complete`
- ACP: `agent-client-protocol` (its stdio transport bridged to Tokio child pipes via `tokio-util`'s
  `compat`). Note: `agent-client-protocol-tokio` is not needed, and `agent-client-protocol-test` does
  not exist on crates.io — ACP integration is tested by mocking the `AgentClient` trait.
- Serialization: `serde`, `serde_json`, `toml`, `schemars`
- Diagnostics/logging: `anyhow`, `thiserror`, `miette` (fancy), `tracing`, `tracing-subscriber`
- Pickers/UX: `which`, `dialoguer`, `indicatif`, `console`, `anstream`, `anstyle`, `terminal_size`
- Files/Git: `ignore`, `globset`, `dunce`, `shell-words`
- Optional/feature-gated (not MVP-default): `ratatui`, `crossterm`, `duct`, `walkdir`, `gix`, `rusqlite`

**Storage**: Local files only — `~/.config/shap/config.toml` (user-editable), `~/.local/share/shap/state.json`
(machine-written), `~/.local/share/shap/sessions/*.jsonl` (conversation logs),
`~/.local/share/shap/last-command-output.txt` (captured output). No database in the MVP.

**Testing**: `cargo test` + `cargo nextest`; `assert_cmd` + `predicates` for CLI integration; `insta`
for snapshots (diagnostics, prompts, commit messages, status, doctor); `tempfile` for isolated FS;
`test-case` for table tests; ACP integration tested by mocking the `AgentClient` trait (the
`agent-client-protocol-test` crate referenced in earlier drafts does not exist); `wiremock` if any HTTP arises.

**Target Platform**: macOS (`aarch64`/`x86_64`) and Linux (`x86_64`/`aarch64`) for the MVP; Windows
(`x86_64-pc-windows-msvc`) deferred. Shell: Zsh first; Bash/Fish later.

**Project Type**: Single Cargo workspace — CLI binary + reusable libraries + a thin shell integration.

**Performance Goals**: Native-feel startup (binary cold-start in the low tens of ms). Shell prompt
segment must add no perceptible delay to prompt rendering (target < 50 ms): the shell hook caches the
segment once per prompt via a lightweight `shap prompt-segment` call that reads only `state.json` — no
config parse, no agent contact — and a fully shell-native read (no subprocess) remains a future
optimization. Streamed responses surface text to the user as the turn completes (per-token streaming
via the SDK's `read_update` is a follow-up).

**Constraints**: Shell layer stays thin (command mapping, prompt segment, buffer insertion only).
No measurable slowdown to shell startup or prompt rendering. Local-only persistence. Never execute
destructive commands or `git commit` automatically. Captured output and `@file` contents bounded by
configurable byte limits.

**Scale/Scope**: Single-user, local, interactive tool. MVP = ~11 colon commands mapped to `shap`
subcommands, one or more configured agents, JSONL session history. No multi-user, no remote sync.

*No NEEDS CLARIFICATION remain — the tech stack and architecture were fully specified in the planning
input. Phase 0 records the decisions and their rationale.*

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against Constitution v1.0.0 (10 principles).

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Keep It Simple (KISS) | ✅ Pass | External pickers over a custom TUI; `git` CLI over a Git library; plain output over rich markdown; official ACP SDK over hand-rolled protocol. |
| II | Keep It Lean (YAGNI) | ⚠ Watch | SQLite, Ratatui, `gix`, Windows, Bash/Fish, resume UI all explicitly deferred. **Risk**: 4-crate workspace + large dependency list. Mitigation: introduce dependencies incrementally per MVP priority order; keep `shap-shell` minimal (see Complexity Tracking). |
| III | Code Quality | ✅ Pass | `clap` derive subcommands; `thiserror` domain errors + `anyhow` at the app edge; a small `AgentClient` trait isolates the SDK. |
| IV | Tests for Meaningful Logic | ✅ Pass | Test areas enumerated (parsing, config, state, sessions, prompt composition, capture, diagnostics); snapshot tests for generated text; ACP integration tested via a mocked `AgentClient` trait. |
| V | Readability Over Performance Tricks | ✅ Pass | No caching/concurrency tricks beyond what async I/O requires; faithful output rendering. Prompt-segment latency is the one measured concern and is handled by reading cached state. |
| VI | Fail Clearly | ✅ Pass | `miette` diagnostics with actionable messages; `:doctor` self-check; missing-agent error names the fix. |
| VII | Keep User Control | ✅ Pass | `:commit` prefills the shell buffer via a ZLE widget and never runs `git commit`; no automatic destructive actions. Core to the design. |
| VIII | Respect the Shell | ✅ Pass | Zsh layer limited to mapping/prompt/buffer-insert; all logic in the Rust binary; the prompt segment spawns no heavy work — `prompt-segment` reads only `state.json` (no config parse, no agent). |
| IX | Minimize Dependencies | ⚠ Watch | Official ACP SDK and `git` CLI avoid hand-rolling. **Risk**: dependency breadth. Mitigation: add each crate only when its MVP step needs it; feature-gate optional ones (`ratatui`, `crossterm`, `gix`, `rusqlite`); run `cargo deny`/`cargo udeps` in CI. |
| X | Preserve Contributor Clarity | ✅ Pass | Clear crate responsibilities and single-purpose modules; the workspace split makes behavior locatable. |

**Gate result**: PASS with two "watch" items (II, IX), both documented in Complexity Tracking. No
unjustified violations. Re-evaluated after Phase 1 — still PASS (design did not add new complexity;
`@file` expansion and prompt composition live as focused modules in `shap-core`).

## Project Structure

### Documentation (this feature)

```text
specs/001-shell-agent-acp/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output (decisions + rationale)
├── data-model.md        # Phase 1 output (entities)
├── quickstart.md        # Phase 1 output (build/run/try)
├── contracts/           # Phase 1 output (CLI command + config/session contracts)
│   ├── cli-commands.md
│   ├── config-schema.md
│   └── session-records.md
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

```text
shap/
├── Cargo.toml                  # workspace manifest
├── crates/
│   ├── shap-cli/               # binary: clap definitions, dispatch, exit codes
│   │   └── src/main.rs
│   ├── shap-core/              # product logic (no shell, no SDK specifics)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── paths.rs        # XDG/SHAP_* path resolution + ~/$XDG expansion
│   │       ├── error.rs        # thiserror + miette domain errors
│   │       ├── agent.rs        # AgentClient trait + DTOs (SDK-agnostic seam)
│   │       ├── config.rs       # TOML load/validate
│   │       ├── state.rs        # state.json read/write
│   │       ├── session.rs      # JSONL session create/append/track
│   │       ├── commands.rs     # command handlers (send, new, status, …)
│   │       ├── git.rs          # git CLI helpers (status/diff/branch)
│   │       ├── output_capture.rs # :run capture + :read composition
│   │       ├── files.rs        # @file detection, resolution, size/binary guards
│   │       ├── picker.rs       # fzf/skim/builtin selection
│   │       ├── doctor.rs       # :doctor checks
│   │       └── prompt.rs       # prompt composition (read/commit payloads)
│   ├── shap-agent/             # ACP integration (implements shap-core::agent)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── acp.rs          # AcpClient: official ACP SDK over child stdio
│   │       └── registry.rs     # configured agents → launchable processes
│   └── shap-shell/             # shell-facing helpers (kept minimal)
│       └── src/
│           ├── lib.rs
│           ├── render.rs       # streamed/spinner output helpers
│           └── prompt.rs       # prompt-segment string builder
├── shell/
│   └── zsh/
│       └── shap.zsh            # thin Zsh integration + ZLE :commit widget
├── docs/
│   ├── config.md
│   ├── shell-integration.md
│   └── agents.md
└── tests/
    └── integration/            # assert_cmd-based end-to-end CLI tests
```

**Structure Decision**: Cargo workspace. The binary (`shap-cli`) stays thin; reusable logic lives in
libraries (`shap-core`, `shap-agent`) so the CLI is testable and usable without the shell layer
(FR-031), and so additional shells (FR-034) and agents (FR-035) can be added without touching command
handlers. `shap-shell` holds only rendering/prompt-segment helpers and is intentionally the smallest
crate (see Complexity Tracking).

The SDK-agnostic `AgentClient` trait + DTOs live in **`shap-core::agent`** (not `shap-agent`):
`shap-core::commands` must call the trait, and `shap-agent` depends on `shap-core`, so placing the
trait in `shap-agent` would create a dependency cycle. `shap-agent` *implements* the trait
(`AcpClient`); the binary injects the concrete client into the core handlers (dependency injection).
Streaming uses an `on_chunk` callback rather than an `AgentStream` type, keeping `futures` out of
`shap-core`.

## Complexity Tracking

> Two "watch" items from the Constitution Check are recorded here per the Complexity-justification gate.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 4-crate workspace (vs. a single crate) | Separates SDK-agnostic product logic (`shap-core`) and ACP integration (`shap-agent`) from the binary so the CLI is independently testable (FR-031) and pluggable for new shells/agents (FR-034/FR-035). | A single crate would mix the ACP SDK surface, product logic, and CLI wiring, making FR-031 testing and FR-035 vendor-neutrality harder to enforce. |
| `shap-shell` as its own crate | Holds prompt-segment + render helpers shared by the binary and future shells. | **Accepted risk**: if it stays this thin through the MVP it SHOULD be merged into `shap-core` as a `render` module. Tracked as a post-MVP simplification, not built out speculatively. |
| Large dependency surface | The stack covers async, ACP, serialization, diagnostics, pickers, and file handling — each maps to a concrete MVP requirement. | Hand-rolling ACP, a TUI picker, or Git access would be more code and more risk than mature crates. Mitigation: add crates per MVP-priority step, feature-gate optional ones, enforce with `cargo deny`/`udeps`. |
