# Phase 1 Data Model: Colon-Command Syntax Highlighting

This feature introduces **no data entities, schemas, or persisted state**. The relevant "model" is the
resolution and behavior state of the `:commit` word in the shell. Captured here for clarity.

## Entity: Shap colon command (conceptual)

A `:`-prefixed word typed at the zsh prompt. Two recognition mechanisms exist in `shap.zsh`:

| Attribute | Values |
|-----------|--------|
| Recognition mechanism | `function` (resolvable command word) · `accept-line widget` (buffer match at Enter) · `:` builtin |
| Highlighter result | recognized (function/builtin/command color) · unknown (red) |
| Executes git | never (Constitution VII) |

Before this feature, `:commit` is the only colon command whose recognition mechanism is *widget-only*,
so its highlighter result is **unknown (red)**. After, `:commit` also has a function → **recognized**.

## State model: `:commit` resolution + behavior

```
Type-time (highlighter scans first word):
  BEFORE: `:commit` → no function/alias/builtin → UNKNOWN (red)
  AFTER : `:commit` → function `:commit` exists  → RECOGNIZED (valid)

Enter-time (BUFFER evaluated):
  BUFFER == ":commit" | ": commit"
        → accept-line widget intercepts (matches first)
        → runs `shap commit --prefill-shell-buffer`
        → success: BUFFER := generated `git commit …` line (review only; NOT executed)
        → failure: ZLE message shown; BUFFER cleared
        → `:commit` function is NEVER invoked here   [unchanged behavior]

  BUFFER == ":commit <args>"  (widget does not match; falls through to .accept-line)
        → `:commit` function runs
        → prints actionable guidance ("type `:commit` then Enter to prefill the commit line")
        → returns non-zero; does NOT execute git        [improves prior "command not found"]
```

## Validation / invariants

- INV-1: The commit is never executed automatically by either the widget or the function (VII).
- INV-2: The `accept-line` widget remains the sole owner of the bare-`:commit` buffer rewrite; the
  function does not duplicate or trigger that path (FR-007).
- INV-3: Bare `:` and `: <text>` resolution/behavior are unaffected (FR-004).
- INV-4: No state is read or written; resolution is purely in-shell at type/Enter time.
