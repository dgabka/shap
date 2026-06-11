# Phase 0 Research: Config Init Wizard & Interactive Config Editing

All Technical Context items resolved; no NEEDS CLARIFICATION remain. Decisions below are constrained
by the constitution (KISS/YAGNI/Minimize Dependencies) and the existing codebase patterns.

## D1 — Prompt library

**Decision**: Use `dialoguer` 0.11 (already a workspace dependency) for all prompts: `Input` (agent
name, command, model list, byte limits if surfaced), `Select` (default model, default agent, picker),
`MultiSelect`/repeated `Input` (model list), `Confirm` (accept defaults, confirm write).

**Rationale**: `picker.rs::builtin_select` already uses `dialoguer::Select`; reusing it adds zero
dependencies (Constitution IX) and matches an established in-repo pattern (Constitution III/X). It
handles cancellation (returns `Err`/`None` on Esc/Ctrl-C) which maps directly to FR-005's
"leave nothing behind".

**Alternatives considered**: `inquire` (new dependency, rejected — duplicates dialoguer); hand-rolled
stdin reads (more code, worse UX, reinvents what dialoguer does); driving `fzf` like the picker
(fzf is a filter, not a form — wrong tool for free-text/confirm prompts).

## D2 — Command surface for interactive editing

**Decision**: Expand the existing `Config` subcommand into an explicit subcommand group:
`shap config path` (default, prints resolved path — preserves today's no-flag output),
`shap config schema` (or keep `--schema`), and `shap config edit` (new interactive editor).
`shap config` with no args opens the interactive editor when stdin is a TTY, otherwise prints the
path (backward-compatible, non-interactive default).

**Rationale**: FR-012 requires the current non-interactive outputs (path, schema) stay reachable for
scripts/docs. A nested subcommand keeps each behavior named and discoverable while letting the
no-arg/TTY case become the new interactive default. Exact spelling (`edit` subcommand vs `--edit`
flag) is finalized in the contract; the constraint is: scripts that call `shap config` (path) and
`shap config --schema` keep working.

**Alternatives considered**: A separate top-level `shap init` command for the wizard — rejected
because the wizard is auto-offered on first run (FR-001) and a manual entry point can simply be
`config edit` against a missing file; adding `init` is extra surface (YAGNI). Making bare
`shap config` always interactive — rejected: breaks non-interactive callers (FR-010/FR-012).

## D3 — First-run trigger point

**Decision**: Hook the wizard at the single config-load chokepoint `Context::load`
(`crates/shap-cli/src/app.rs:26`, line 32 `Config::load(...)`). When `Config::load` returns
`Error::ConfigNotFound` **and** `std::io::stdin().is_terminal()`, offer the wizard; on completion,
write the file and re-load. Otherwise propagate `ConfigNotFound` unchanged (today's behavior).

**Rationale**: Every config-requiring command funnels through `Context::load`, so one hook covers
`send`, `agent`, `model`, `reasoning`, `new`, `status`, `commit`, `run`, `read` without touching each
handler (KISS/DRY). `doctor` calls `Config::load` directly and intentionally treats a missing config
as a failing check — it is left as-is (no wizard offer mid-diagnostic). `PromptSegment` never loads
config, so it is structurally immune (FR-011).

**Alternatives considered**: A persisted "first run" flag — rejected (YAGNI; presence of the config
file is the signal, per spec Assumptions). Per-command hooks — rejected (duplication, Constitution X).

## D4 — Interactivity detection

**Decision**: `std::io::stdin().is_terminal()` (the exact check already used in `picker.rs` and
`app.rs:202`). Non-TTY ⇒ no prompt; fall back to `Error::ConfigNotFound`'s existing diagnostic.

**Rationale**: Consistent with the existing `builtin_select` guard (`Error::NonInteractivePicker`),
so behavior is uniform across the tool. Covers pipes, redirects, CI, and the shell prompt hook
(FR-010/FR-011). No new abstraction needed.

**Alternatives considered**: Checking stdout/stderr TTY too — unnecessary; prompts read stdin, and
dialoguer renders to the terminal; stdin is the correct gate. A `--no-input`/`--yes` flag —
deferred (YAGNI); not required by any FR.

## D5 — Writing the config (serialization + atomicity)

**Decision**: Serialize the in-memory `Config` with `toml::to_string_pretty` (the type already
derives `Serialize`, and `#[serde(flatten)] extra: toml::Table` means passthrough keys round-trip
automatically). Write atomically: temp file in the same directory + `std::fs::rename`, creating parent
dirs with `create_dir_all` — copied from `ActiveState::save` (`state.rs:45-56`). Always call
`Config::validate()` on the constructed `Config` **before** writing; never write on validation failure
or user cancel.

**Rationale**: Reuses a proven in-repo pattern (Constitution III/X); atomicity guarantees FR-005's
"no partial file". `#[serde(flatten)]` on `Agent.extra` is the key enabler for FR-008 — unsurfaced
per-agent keys survive a load→edit→serialize round-trip with no special handling.

**Caveats / scope**: `toml::to_string_pretty` does **not** preserve comments or the user's original
key ordering/formatting (it re-emits canonical TOML). This is acceptable per the spec (the editor
surfaces structured fields; passthrough *values* are preserved, formatting/comments are not). This
limitation is documented in the contract and in `config.md`. A format-preserving editor
(`toml_edit`) is rejected as a new dependency and added complexity not justified by current needs
(Constitution II/IX) — revisit only if comment loss becomes a real complaint.

**Alternatives considered**: `toml_edit` crate for comment/format preservation — rejected (new
dependency, more complex API; YAGNI for v1). Writing in place without temp+rename — rejected
(risks partial files on crash, violates FR-005).

## D6 — Agent presets

**Decision**: Offer a tiny static preset list in the wizard drawn from the documented example agents
(e.g. `codex` → `codex-acp` and `claude` → `claude-agent-acp`), plus a "custom" option that prompts
for an arbitrary name + command. Presets pre-fill the command and a starter model list, all editable.
The preset list is a small hard-coded constant, not a config-driven registry.

**Rationale**: Presets make SC-002 (basic config in ~5 prompts / under 2 min) achievable for the
common case while "custom" keeps the wizard general. A hard-coded list is the lean choice
(Constitution II) — no plugin/registry infrastructure. Validation (`shap doctor`) remains the place
that checks whether the chosen command is actually installed (spec Dependencies); the wizard accepts
a not-yet-installed command and points the user to `shap doctor`.

**Alternatives considered**: No presets, always custom — rejected (slower first run, hurts SC-002).
Auto-detecting installed ACP agents on PATH — rejected (speculative, YAGNI; `which`-probing a guessed
list is fragile and out of scope).

## D7 — Cancellation & error semantics

**Decision**: dialoguer's `interact_opt()` / `Err` on Esc/Ctrl-C returns "no selection"; map that to a
clean exit (FR-005) — for the first-run wizard, print the same setup guidance as the non-interactive
fallback and exit non-zero without writing. For the editor, leave the existing file byte-for-byte
untouched (FR-009). Write failures (permission/disk) surface as an actionable `Error` naming the
attempted path (FR-014) — add a `ConfigWriteFailed { path, source }` variant in `error.rs` following
the existing diagnostic style (`code`, `help`).

**Rationale**: Aligns with Constitution VI (Fail Clearly) and VII (Keep User Control); reuses the
established `Error`/`miette` diagnostic conventions in `error.rs`.

**Alternatives considered**: Treating cancel as success with an empty config — rejected (would write
invalid config, violates FR-005). Panicking on write error — rejected (FR-014, constitution).

## Summary of resolved unknowns

| Item | Resolution |
|------|------------|
| Prompt library | `dialoguer` (already vendored) |
| Command shape | `config` subcommand group: `path`/`schema`/`edit`; bare+TTY ⇒ edit |
| First-run hook | `Context::load` on `ConfigNotFound` + TTY |
| Interactivity gate | `std::io::stdin().is_terminal()` |
| Write strategy | `toml::to_string_pretty` + atomic temp+rename + validate-first |
| Passthrough preservation | automatic via existing `#[serde(flatten)] extra` |
| Presets | small static list + custom |
| Cancel/error | clean no-write exit; `ConfigWriteFailed` diagnostic |
| New dependencies | none |
