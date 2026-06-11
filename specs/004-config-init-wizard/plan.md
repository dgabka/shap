# Implementation Plan: Config Init Wizard & Interactive Config Editing

**Branch**: `004-config-init-wizard` | **Date**: 2026-06-11 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-config-init-wizard/spec.md`

## Summary

Replace the "no config → print a TOML snippet and fail" experience with two interactive flows built
on the prompt library the project already ships (`dialoguer`):

1. **First-run wizard** — when a command needs config and none exists *and* stdin is a TTY, offer a
   short guided wizard (agent preset or custom command → models → default model → default agent →
   accept UI defaults), then write a validated `config.toml` and continue.
2. **Interactive `config edit`** — let an existing user change common settings through prompts; the
   editor loads the current config, mutates the in-memory `Config`, re-validates, and writes it back,
   preserving opaque per-agent passthrough keys.

This introduces the project's first *tool-written* config. Writing is gated behind explicit user
action (never silent), uses the existing atomic temp-file+rename pattern (`state.rs`), and runs the
existing `Config::validate()` before persisting. Non-interactive contexts keep today's behavior
exactly (printed setup instructions, non-zero exit, no prompting). No new dependencies.

## Technical Context

**Language/Version**: Rust (edition 2024, rust-version 1.85). Cargo workspace.

**Primary Dependencies**: All already present — `dialoguer` 0.11 (prompts: `Input`, `Select`,
`Confirm`, `MultiSelect`), `console`/`std::io::IsTerminal` (TTY detection, already used in
`picker.rs`/`app.rs`), `toml` 0.8 (serialize via `toml::to_string_pretty` — `Config` already derives
`Serialize`), `serde`, `miette`/`thiserror` (diagnostics). No new crate is added (Constitution IX).

**Storage**: The single user `config.toml` at the resolved path (`Paths::config()`, honoring
`--config` / `SHAP_CONFIG`). Written atomically (temp file in same dir + `rename`), mirroring
`ActiveState::save` in `crates/shap-core/src/state.rs`.

**Testing**: `cargo test` / `cargo nextest`. Unit tests in `shap-core` for the config serializer,
the wizard→`Config` builder (pure, no I/O), passthrough-key preservation on round-trip, and the
TTY/non-interactive decision (pure function). CLI-level `assert_cmd` tests confirm non-interactive
fallback (piped stdin → printed instructions, non-zero exit, no hang). Interactive prompt rendering
itself is not unit-tested (dialoguer is trusted); logic is extracted into pure functions per
Constitution IV.

**Target Platform**: macOS and Linux terminals (same as existing interactive paths).

**Project Type**: Rust CLI + thin zsh shell integration. Core logic lives in `shap-core`; the
`shap-cli` layer wires commands and the first-run hook.

**Performance Goals**: N/A. The wizard is user-invoked and interactive. The cheap prompt-segment
path is untouched and must remain allocation-light (Constitution VIII).

**Constraints**: Never prompt when stdin is not a TTY (FR-010, FR-011). Never write a partial/invalid
config (FR-005, FR-007); validate-before-persist. Preserve unsurfaced/passthrough keys on edit
(FR-008). The prompt-segment command (`PromptSegment`) and `Config::load` callers in non-interactive
contexts must not change behavior. No panics on write failure (FR-014).

**Scale/Scope**: ~1 new `shap-core` module (`config_wizard` / `config_write`), edits to `cli.rs`
(turn `Config` into a subcommand group: `path` | `schema` | `edit`, or add flags — see research D2),
`app.rs` (`config` handler + first-run hook in `Context::load`), one new `Error` variant or reuse of
existing ones, and a docs update to `config.md`. ~5–7 files touched.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Application | Status |
|-----------|-------------|--------|
| I. KISS | Reuse `dialoguer` + existing atomic-write pattern; a flat sequence of prompts, no wizard framework or state machine. | PASS |
| II. YAGNI | Wizard collects only what validation requires plus the few common UI toggles; no multi-file/layered config, no full TOML editor, no presets infrastructure beyond a small static list. | PASS |
| III. Code Quality | New logic isolated in a named `shap-core` module; pure builder/validate separated from I/O and prompt rendering. | PASS |
| IV. Tests for Meaningful Logic | Unit-test the pure builder, serializer round-trip (passthrough preservation), and the interactivity decision; trust dialoguer's rendering. | PASS |
| V. Readability over perf | No optimization; straightforward sequential prompts. | PASS |
| VI. Fail Clearly | Write failures name the attempted path; cancel exits with setup guidance; non-interactive prints actionable instructions. | PASS |
| VII. Keep User Control | Config is written only on explicit user action (accepting the wizard / confirming an edit); never silent. The wizard shows the resulting config and asks to confirm before writing. | PASS |
| VIII. Respect the Shell | Prompt-segment path and shell hooks untouched; wizard never triggers in non-TTY (shell startup/prompt) contexts. | PASS |
| IX. Minimize Dependencies | Zero new dependencies — `dialoguer`/`console`/`toml` already vendored. | PASS |
| X. Preserve Contributor Clarity | One obvious module owns wizard+write; `config.md` updated so the "never rewrites" note matches reality. | PASS |

**Note on the "never rewrites config" invariant**: `config.rs` and `config.md` currently state the
tool never rewrites the user's config. This feature intentionally changes that, but only via explicit
user-initiated wizard/edit actions. This is a deliberate, documented relaxation (recorded in the spec
Assumptions and applied to `config.md`), not an unjustified violation. No Complexity Tracking entry
required — the change adds no architectural complexity, only a guarded write path.

## Project Structure

### Documentation (this feature)

```text
specs/004-config-init-wizard/
├── plan.md              # This file
├── research.md          # Phase 0 — prompt lib, command shape, write strategy, presets
├── data-model.md        # Phase 1 — WizardDraft, edit actions, write/validate flow
├── quickstart.md        # Phase 1 — build + manual verification walkthrough
├── contracts/
│   └── cli-commands.md  # Phase 1 — `config` subcommand surface + behavior contract
└── checklists/
    └── requirements.md  # Spec quality checklist (from /speckit-specify)
```

### Source Code (repository root)

```text
crates/shap-core/src/
├── config.rs            # EXISTING — add a config serializer + (optional) draft→Config builder;
│                        #   relax the "never rewrites" module doc
├── config_wizard.rs     # NEW — wizard prompt flow + interactive edit actions (dialoguer),
│                        #   pure draft/builder helpers split from prompt I/O
├── error.rs             # EXISTING — add write-failure / cancelled-setup variants as needed
├── state.rs             # EXISTING — atomic-write reference pattern (not modified)
└── lib.rs               # EXISTING — export the new module

crates/shap-cli/src/
├── cli.rs               # EXISTING — extend `Config` subcommand: path (default) | --schema | edit
├── app.rs               # EXISTING — `config` handler (interactive edit); first-run wizard hook
│                        #   in Context::load when Config::load → ConfigNotFound and stdin is a TTY
└── main.rs              # EXISTING — dispatch updates for the new config subcommand shape

docs/
└── config.md            # EXISTING — document the wizard + interactive edit; correct the
                         #   "tool never rewrites it" statement

crates/shap-shell/       # source of truth for `:` commands — reviewed, not required to change
shell/zsh/shap.zsh       # NOT modified — non-interactive paths must stay unchanged
```

**Structure Decision**: Keep core logic in `shap-core` (Constitution VIII/X). Add one cohesive
`config_wizard` module owning both the first-run wizard and the interactive editor, since they share
the same prompt helpers, the same draft→`Config` construction, and the same validate-then-atomic-write
step. The CLI layer stays thin: it detects the no-config/TTY condition and dispatches. The first-run
hook lives at the single existing config-load chokepoint (`Context::load`, `app.rs:26`).

## Complexity Tracking

> No unjustified constitution violations. The documented relaxation of the read-only-config invariant
> is intentional and user-gated (see Constitution Check note). Section intentionally empty.
