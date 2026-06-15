# Contract: Colon-Command Surface (Zsh Integration)

Scope: the recognition + behavior contract for shap colon commands in `shell/zsh/shap.zsh` after this
feature. Only `:commit` changes; all other rows are stated to lock in "no regression".

## Recognition contract (type-time highlighting)

When the integration is sourced, each typed word resolves as follows (this is what a function-aware
highlighter colors):

| Typed word | Resolves as | Highlighter result |
|------------|-------------|--------------------|
| `:agent` `:model` `:reasoning` `:effort` `:new` `:status` `:doctor` `:run` `:read` | function | recognized (unchanged) |
| `:commit` | **function** (new) | **recognized** (was: unknown/red) |
| `:` (bare) | zsh `:` builtin | recognized (unchanged) |
| `: <text>` | `:` builtin + accept-line widget | recognized (unchanged) |

**Contract**: No documented colon command resolves to "unknown" when the integration is active.

## Behavior contract: `:commit`

### B1 — Bare `:commit` / `: commit` + Enter  (UNCHANGED)

- **Given** the integration is active
- **When** the user submits a line whose buffer is exactly `:commit` or `: commit`
- **Then** the `accept-line` widget runs `shap commit --prefill-shell-buffer`, and:
  - on success: the buffer is replaced with the single generated `git commit …` line for review;
  - on failure: a ZLE message reports the error and the buffer is cleared;
  - **in no case is git executed automatically.**
- The `:commit` function is NOT invoked in this path.

### B2 — `:commit <args>` + Enter  (NEW, improved)

- **Given** the integration is active
- **When** the user submits `:commit` with trailing arguments (widget does not match)
- **Then** the `:commit` function runs, prints actionable guidance (how to use `:commit`), exits
  non-zero, and does **not** execute git.
- (Prior behavior was a bare "command not found".)

### B3 — Highlighting only / no submit

- **Given** a function-aware highlighter is enabled
- **When** the user types `:commit` without submitting
- **Then** `:commit` is rendered in the recognized-command style (not the unknown/error style).

### B4 — No highlighter / integration inactive

- **Given** no command-word highlighter, or the integration is not sourced
- **Then** there is no behavior change versus before this feature.

## Invariants

- The commit is never executed automatically (Constitution VII).
- Exactly one mechanism owns the buffer-rewrite (the widget); the function never duplicates it.
- The bare `:` builtin and `: <text>` chat path are untouched.

## Out of scope

- Bespoke per-command colors or per-theme styling.
- Non-zsh shells (only a zsh integration ships today).
- Any change to the Rust `commit` subcommand (already implemented and tested).
