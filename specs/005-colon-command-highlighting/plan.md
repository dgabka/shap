# Implementation Plan: Colon-Command Syntax Highlighting

**Branch**: `005-colon-command-highlighting` | **Date**: 2026-06-15 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-colon-command-highlighting/spec.md`

## Summary

Command-word highlighters (zsh-syntax-highlighting / fast-syntax-highlighting) color the first word
of a line red when the shell cannot resolve it to a command, alias, function, builtin, or reserved
word. Every shap colon command except `:commit` is defined as a zsh **function** in `shap.zsh`, so
those resolve and highlight as valid. `:commit` is handled **only** by the `accept-line` widget and
has no resolvable command word, so it renders red — signalling "invalid command" even though it works.

Fix: add a thin `:commit` zsh function alongside the existing colon functions. Its mere existence makes
`:commit` resolvable, so any function-aware highlighter stops coloring it red — generically, with no
plugin-specific config. The `accept-line` widget keeps ownership of the bare-`:commit` buffer-rewrite
behavior: it intercepts and rewrites the buffer *before* `.accept-line`, so the function never runs in
the normal flow. The function body only handles fall-through (`:commit <args>`, or contexts without the
widget): it prints actionable guidance and **never executes git** (Constitution VII).

No Rust changes — `shap commit --prefill-shell-buffer` already exists and is tested. Shell-only edit
to `shell/zsh/shap.zsh` plus a docs note in `docs/shell-integration.md`. No new dependencies.

## Technical Context

**Language/Version**: Zsh shell script (the `shell/zsh/shap.zsh` integration layer). No Rust change.
The compiled CLI is Rust (edition 2024, rust-version 1.85) but is untouched by this feature.

**Primary Dependencies**: None added. Relies only on zsh built-ins (`function`, `zle`, parameter
expansion) already used in `shap.zsh`. Highlighting itself is provided by the user's third-party
highlighter (zsh-syntax-highlighting / fast-syntax-highlighting) — not vendored, not a dependency.

**Storage**: N/A — no state read or written.

**Testing**: No Rust logic added, so no new `cargo` tests. The unchanged `shap commit
--prefill-shell-buffer` path is already covered by `crates/shap-cli/tests/commit.rs`. The shell
function's only branch (guidance message on misuse) is verified manually via `quickstart.md`, matching
the project's existing convention of not unit-testing third-party-rendered interactive behavior (cf.
the 004 plan's note on not unit-testing dialoguer rendering).

**Target Platform**: macOS and Linux zsh sessions with the integration sourced.

**Project Type**: Rust CLI + thin zsh shell integration. This feature touches only the shell layer.

**Performance Goals**: N/A. Adding one function definition at source time has no measurable startup or
prompt cost (Constitution VIII). No per-prompt or per-keystroke work is added.

**Constraints**: `:commit` must never auto-execute a commit (FR-002, FR-007, Constitution VII). The
shell layer must stay thin (FR-005, Constitution VIII). The bare `:` builtin and `: <text>` chat path
must not regress (FR-004). No behavior change when no highlighter is present (FR-006).

**Scale/Scope**: One small zsh function (+ comment update) in `shell/zsh/shap.zsh`; one doc note in
`docs/shell-integration.md`. ~2 files touched, no Rust.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Application | Status |
|-----------|-------------|--------|
| I. KISS | A single stub function mirroring the existing `:agent`/`:model` definitions; no plugin-specific highlighter config, no new mechanism. | PASS |
| II. YAGNI | Only the reported `:commit` gap is closed; no speculative per-highlighter theming or new commands. | PASS |
| III. Code Quality | The new function matches the surrounding colon-function idiom and naming; comment block updated to match reality. | PASS |
| IV. Tests for Meaningful Logic | No Rust logic added; unchanged CLI path stays covered by existing tests. The trivial guidance branch is verified manually (no shell test harness in the project). | PASS |
| V. Readability over perf | No optimization; one declarative function. | PASS |
| VI. Fail Clearly | `:commit <args>` / fall-through prints actionable guidance ("type `:commit` then Enter") instead of a bare "command not found". | PASS |
| VII. Keep User Control | The function never runs git; the commit line is still only prefilled for review by the widget, never executed. | PASS |
| VIII. Respect the Shell | One function defined at source time; zero per-prompt/per-keystroke cost; logic stays out of the shell beyond thin delegation. | PASS |
| IX. Minimize Dependencies | Zero new dependencies; uses only zsh built-ins already in `shap.zsh`. | PASS |
| X. Preserve Contributor Clarity | `:commit` now sits visibly beside its siblings; the "handled by widget" comment is updated so the dual function+widget arrangement is obvious. | PASS |

**Result**: PASS (initial and post-design). No violations; Complexity Tracking intentionally empty.

## Project Structure

### Documentation (this feature)

```text
specs/005-colon-command-highlighting/
├── plan.md              # This file
├── research.md          # Phase 0 — why functions highlight valid; widget-vs-function interaction
├── data-model.md        # Phase 1 — `:commit` resolution/behavior states (no data entities)
├── quickstart.md        # Phase 1 — build + manual verification of highlighting and behavior
├── contracts/
│   └── colon-commands.md  # Phase 1 — colon-command resolution + `:commit` behavior contract
└── checklists/
    └── requirements.md  # Spec quality checklist (from /speckit-specify)
```

### Source Code (repository root)

```text
shell/zsh/
└── shap.zsh             # EDIT — add a thin `:commit` function beside the other colon functions;
                         #   update the "handled by the accept-line widget" comment to describe the
                         #   function + widget split. The accept-line widget is NOT changed.

docs/
└── shell-integration.md # EDIT — note that `:commit` is now a function (so it highlights as valid)
                         #   while the widget still owns the buffer-rewrite; clarify `:commit <args>`.

crates/                  # NOT modified — `shap commit --prefill-shell-buffer` already exists and is
                         #   covered by crates/shap-cli/tests/commit.rs.
```

**Structure Decision**: Shell-only change. The integration layer already owns the colon-command
surface; the fix belongs there as one more thin function, keeping all product logic in the (unchanged)
Rust CLI per Constitution VIII/X. No new module or file is introduced.

## Complexity Tracking

> No constitution violations. Section intentionally empty.
