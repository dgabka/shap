# Phase 0 Research: Colon-Command Syntax Highlighting

## R1. Why `:commit` highlights red while the other colon commands don't

**Decision**: Treat the root cause as "no resolvable command word for `:commit`", not "missing
highlighter config".

**Findings**:
- Command-word highlighters (zsh-syntax-highlighting "main" highlighter; fast-syntax-highlighting)
  classify the first word of a simple command. They color it as a recognized command when zsh can
  resolve it to one of: external command on `PATH`, alias, **function**, builtin, or reserved word.
  Otherwise they apply the unknown-command style (red by default).
- In `shell/zsh/shap.zsh`, `:agent`, `:model`, `:reasoning`, `:effort`, `:new`, `:status`, `:doctor`,
  `:run`, `:read` are all defined with `function :name { … }`. Their names live in zsh's function
  table, so the highlighter resolves them → not red.
- `:commit` has **no** function. It is recognized only by the `accept-line` widget
  (`_shap_accept_line`), which compares `${BUFFER}` to `:commit` / `: commit` at Enter time. The
  highlighter never consults widgets, so at type time `:commit` is an unresolved word → red.
- The bare `:` and `: <text>` chat path start with the `:` zsh **builtin**, which always resolves →
  never red. These are out of the problem.

**Rationale**: The asymmetry is fully explained by "function defined vs. not". The simplest, most
robust fix is to make `:commit` resolvable the same way its siblings already are.

## R2. How to make `:commit` resolvable (options)

**Decision**: Define a thin `:commit` zsh function alongside the other colon functions (Option A).

| Option | Approach | Verdict |
|--------|----------|---------|
| **A. Stub function** | `function :commit { … }` mirroring siblings | **CHOSEN** — generic across function-aware highlighters, matches existing idiom, zero new deps, thin. |
| B. Highlighter config | Register `:commit` via z-sy-h regexp/`ZSH_HIGHLIGHT_*` or a custom highlighter | Rejected — plugin-specific, load-order fragile, doesn't help fast-syntax-highlighting users, adds shell complexity (violates I/VIII). |
| C. Move buffer-rewrite into a function | Replace the widget with a function | Rejected — a normal function cannot edit the ZLE `BUFFER`; that is exactly why `:commit` is a widget. |

**Rationale**: Option A fixes the visible symptom for any highlighter that resolves functions (the
common case, covered by both major zsh highlighters) without coupling shap to a specific plugin's
internals. It also makes the command surface uniform (User Story 2).

**Alternatives considered**: See table. B and C both increase shell-layer complexity for narrower or
broken results.

## R3. Function + widget interaction (correctness)

**Decision**: Keep the `accept-line` widget as the single owner of bare-`:commit` behavior; the
function only handles fall-through.

**Findings**:
- On Enter with `BUFFER == ':commit'` (or `': commit'`), `_shap_accept_line` runs first (it is bound
  to `accept-line`), rewrites the buffer to the generated `git commit …` line, and `return 0`s
  **without** calling `.accept-line`. The `:commit` function is therefore never invoked in the normal
  path — behavior is unchanged (FR-002, FR-007).
- The function body only runs for fall-through: `:commit <args>` (the widget matches only the exact
  bare buffers), or contexts where the widget isn't installed. Today those produce "command not
  found". The function will instead print actionable guidance and **never** call git (FR-002, VI, VII).

**Rationale**: This preserves the existing, tested behavior exactly while closing the highlighting gap
and improving the misuse message. No second executable commit path is created (FR-007).

## R4. Testing strategy

**Decision**: No new Rust tests; manual verification via `quickstart.md`.

**Findings**:
- The Rust side (`shap commit --prefill-shell-buffer`) is unchanged and already covered by
  `crates/shap-cli/tests/commit.rs`.
- The project has no zsh test harness, and highlighting is rendered by third-party plugins. Consistent
  with the 004 plan's decision not to unit-test dialoguer rendering, the highlighting result and the
  trivial guidance branch are verified manually.

**Rationale**: The added logic is a one-branch guidance echo; Constitution IV does not require tests
for trivial declarative shell glue, and no harness exists to host them.

## Resolved unknowns

All Technical Context items are concrete; no `NEEDS CLARIFICATION` remained.
