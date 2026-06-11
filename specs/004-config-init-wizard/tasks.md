---

description: "Task list for Config Init Wizard & Interactive Config Editing"
---

# Tasks: Config Init Wizard & Interactive Config Editing

**Input**: Design documents from `/specs/004-config-init-wizard/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli-commands.md, quickstart.md

**Tests**: Included. Constitution IV (Tests for Meaningful Logic) and plan.md mandate unit tests for
the pure builder/serializer and `assert_cmd` tests for the non-interactive fallback. Interactive
prompt rendering is verified manually via quickstart (dialoguer is trusted).

**Organization**: Grouped by user story (US1 wizard P1, US2 editor P2, US3 non-interactive guardrail
P3). Stories are independently testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on incomplete tasks)
- **[Story]**: US1 / US2 / US3 — story phase tasks only
- All paths are repo-relative and exact.

## Path Conventions

Rust cargo workspace. Core logic in `crates/shap-core/src/`; CLI wiring in `crates/shap-cli/src/`;
CLI integration tests in `crates/shap-cli/tests/`. Unit tests live inline (`#[cfg(test)] mod tests`)
in the module they cover, per existing convention.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the new module and register it so later phases compile.

- [x] T001 Create `crates/shap-core/src/config_wizard.rs` (empty module with `//!` doc) and register it as `pub mod config_wizard;` in `crates/shap-core/src/lib.rs` (after `config`)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The shared validated-write path and error type used by BOTH the wizard (US1) and the
editor (US2). Must complete before US1/US2.

**⚠️ CRITICAL**: No user story write logic can land until this phase is complete.

- [x] T002 Add `Error::ConfigWriteFailed { path: PathBuf, source: std::io::Error }` with a miette diagnostic (`code(shap::config::write)`, `help` naming the attempted path + next step) in `crates/shap-core/src/error.rs`
- [x] T003 Add `Config::write(&self, path: &Path) -> Result<()>` in `crates/shap-core/src/config.rs`: call `self.validate()` first, serialize with `toml::to_string_pretty`, then atomic write (`create_dir_all(parent)` + temp file in same dir + `fs::rename`) mirroring `ActiveState::save` in `state.rs:45-56`; map IO errors to `Error::ConfigWriteFailed`. Relax the module-level "shap never rewrites the user's config" doc comment to note user-initiated writes.
- [x] T004 [P] Unit test in `crates/shap-core/src/config.rs` tests: round-trip a `Config` containing an agent with `extra` passthrough keys through `Config::write` → `Config::load`, asserting the passthrough survives and the result validates (FR-008, SC-003)
- [x] T005 [P] Unit test in `crates/shap-core/src/config.rs` tests: `Config::write` on an invalid `Config` (e.g. `default_agent` not in `agents`) returns the validation `Error` and writes no file (FR-004/FR-007, SC-004)

**Checkpoint**: Validated atomic config writing exists and is unit-tested.

---

## Phase 3: User Story 1 - Guided first-run setup (Priority: P1) 🎯 MVP

**Goal**: When a command needs config, none exists, and stdin is a TTY, offer a wizard that writes a
valid `config.toml` and continues.

**Independent Test**: Remove config, run `shap status` in a terminal, answer prompts → valid config
written, `shap doctor` passes (quickstart Scenario A).

### Implementation for User Story 1

- [x] T006 [P] [US1] Define `WizardDraft` struct and a small static `PRESETS` list (e.g. `codex`→`codex-acp`, `claude`→`claude-agent-acp` with starter models) in `crates/shap-core/src/config_wizard.rs`
- [x] T007 [US1] Implement pure `WizardDraft::into_config(self) -> Config` (single-agent config, `default_agent = agent_name`, empty `extra`, default `ui`/`history`/`files`) in `crates/shap-core/src/config_wizard.rs` (depends on T006)
- [x] T008 [P] [US1] Unit test in `crates/shap-core/src/config_wizard.rs` tests: `into_config` output passes `Config::validate()` for both a preset draft and a custom draft (SC-003)
- [x] T009 [US1] Implement `run_wizard() -> Result<Option<Config>>` prompt flow with dialoguer (`Confirm` set-up-now → `Select` preset/custom → `Input` name/command for custom → `Input` models (non-empty re-prompt) → `Select` default model → `Confirm` accept UI defaults → summary `Confirm`); cancel/Esc/decline ⇒ `Ok(None)`, leaving nothing behind, in `crates/shap-core/src/config_wizard.rs` (depends on T006, T007)
- [x] T010 [US1] Wire the first-run hook in `Context::load` (`crates/shap-cli/src/app.rs:26`): when `Config::load` returns `Error::ConfigNotFound` and `std::io::stdin().is_terminal()`, call `run_wizard()`; on `Some(config)` → `config.write(paths.config())` then re-load and proceed; on `None` → print the `ConfigNotFound` setup guidance and return non-zero (FR-001..005)
- [x] T011 [US1] Manual verification per quickstart Scenario A (accept path writes valid config + command continues; cancel path leaves no partial file) — interactive, not `assert_cmd`-testable

**Checkpoint**: First-run wizard fully functional and independently demonstrable (MVP).

---

## Phase 4: User Story 2 - Interactive config editing (Priority: P2)

**Goal**: `shap config edit` (and bare `shap config` on a TTY) lets an existing user change common
settings through prompts, re-validates, and writes back preserving passthrough keys.

**Independent Test**: With a valid config, run `shap config edit`, change a setting, save → file
updated, still validates, passthrough keys intact (quickstart Scenario C).

### Implementation for User Story 2

- [x] T012 [US2] Extend the `Config` subcommand in `crates/shap-cli/src/cli.rs` to a group: `path` (default), `schema` (keep `--schema` working), and `edit`; bare `Config` keeps a way to reach path/editor per the contract (FR-012)
- [x] T013 [US2] Update dispatch in `crates/shap-cli/src/main.rs` for the new `Config` subcommand shape (depends on T012)
- [x] T014 [US2] Implement `run_editor(config: Config) -> Result<Option<Config>>` (top-level `Select` of edit actions from data-model `EditAction`, mutate an in-memory clone, Save/Cancel) in `crates/shap-core/src/config_wizard.rs`
- [x] T015 [US2] Implement the `config` handler in `crates/shap-cli/src/app.rs`: `path`/`schema` preserve current outputs; bare+TTY and `edit` load config → `run_editor` → on `Some` `config.write(...)` (validate-first), on `None` leave file unchanged; `edit` with non-TTY ⇒ "requires a terminal" error (FR-006..009, FR-012) (depends on T012, T014)
- [x] T016 [P] [US2] Unit test in `crates/shap-core/src/config_wizard.rs` tests: applying an edit action to a `Config` with agent `extra` keys, then `write`→`load`, preserves the passthrough and rejects an edit that fails validation (FR-007/FR-008)

**Checkpoint**: Interactive editor works; US1 still works.

---

## Phase 5: User Story 3 - Safe fallback for non-interactive contexts (Priority: P3)

**Goal**: No prompting when stdin is not a TTY; missing-config falls back to today's printed guidance
+ non-zero exit; the prompt-segment path never triggers a wizard or write.

**Independent Test**: Piped stdin + no config → `ConfigNotFound` diagnostic, non-zero, no hang, no
file written (quickstart Scenario B).

### Implementation for User Story 3

- [x] T017 [US3] Confirm the `Context::load` hook (T010) propagates `Error::ConfigNotFound` unchanged when `stdin` is not a TTY — no prompt, no write — and add an explanatory comment in `crates/shap-cli/src/app.rs`
- [x] T018 [P] [US3] `assert_cmd` test in `crates/shap-cli/tests/config_wizard.rs`: with `SHAP_CONFIG` pointing at a missing path and piped (non-TTY) stdin, `shap send "hi"` exits non-zero, prints the missing-config help, and writes no file (FR-010, SC-005)
- [x] T019 [P] [US3] `assert_cmd` test in `crates/shap-cli/tests/config_wizard.rs`: `shap config` (non-TTY) prints the resolved path and `shap config --schema` prints the JSON schema — back-compat (FR-012, INV-5)
- [x] T020 [P] [US3] `assert_cmd` test in `crates/shap-cli/tests/config_wizard.rs`: `shap prompt-segment` with no config stays silent and exits 0 (never triggers the wizard) (FR-011)

**Checkpoint**: Non-interactive and shell-hook paths verified unchanged; all stories independent.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T021 [P] Update `docs/config.md`: document the first-run wizard and `shap config edit`; correct the "The tool never rewrites it" statement to "the tool only writes config via the wizard/`config edit`"; note that serialization does not preserve comments/key order (research D5)
- [x] T022 [P] Run `cargo fmt --all` and `cargo clippy --workspace --all-targets` (warnings are denied via workspace lints); fix any lint findings
- [x] T023 Run `cargo nextest run --workspace` (or `cargo test --workspace`) and the quickstart Scenarios A–C end-to-end before marking the feature done

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 — no dependencies; unblocks everything (module must exist to compile).
- **Foundational (Phase 2)**: T002–T005 depend on T001. BLOCKS US1 and US2 (they call `Config::write`).
- **US1 (Phase 3)**: depends on Phase 2. Delivers the MVP.
- **US2 (Phase 4)**: depends on Phase 2. Independent of US1 (shares only the foundational writer + the
  `config_wizard` module; no behavioral dependency on the wizard).
- **US3 (Phase 5)**: T017 depends on T010 (the hook lives there). T018–T020 are independent
  `assert_cmd` tests and only need the binary to build.
- **Polish (Phase 6)**: after the targeted stories are complete.

### Within Each User Story

- Models/structs before the functions that build them (T006 → T007 → T009).
- Core function before its CLI wiring (T014 → T015; T009 → T010).
- CLI subcommand definition before dispatch/handler (T012 → T013/T015).

### Parallel Opportunities

- **Phase 2**: T004 and T005 [P] together (both are tests in `config.rs`, independent assertions) once
  T002/T003 land.
- **US1**: T006 and T008 [P]; T008 runs after T007.
- **US2**: T016 [P] alongside handler wiring once T014 exists.
- **US3**: T018, T019, T020 [P] — independent test cases in the same new test file (write together).
- **Polish**: T021 and T022 [P].
- **Cross-story**: once Phase 2 is done, US1 and US2 can be built in parallel by different developers
  (different functions in `config_wizard.rs` + different CLI handlers).

---

## Parallel Example: User Story 1

```bash
# After T007 lands, builder test and preset struct work are parallel-friendly:
Task: "T006 Define WizardDraft + PRESETS in crates/shap-core/src/config_wizard.rs"
Task: "T008 Unit test into_config validates, in crates/shap-core/src/config_wizard.rs tests"
```

## Parallel Example: User Story 3

```bash
# Three independent assert_cmd cases in one new test file:
Task: "T018 non-TTY missing-config fallback test in crates/shap-cli/tests/config_wizard.rs"
Task: "T019 config path/--schema back-compat test in crates/shap-cli/tests/config_wizard.rs"
Task: "T020 prompt-segment no-wizard test in crates/shap-cli/tests/config_wizard.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1 (T001) → Phase 2 (T002–T005) → Phase 3 (T006–T011).
2. **STOP and VALIDATE**: quickstart Scenario A — wizard writes a valid config and the command
   continues; cancel leaves no file.
3. Demo: a fresh user reaches a working config without hand-editing TOML.

### Incremental Delivery

1. Setup + Foundational → validated writer ready.
2. US1 → first-run wizard (MVP) → validate → demo.
3. US2 → interactive `config edit` → validate → demo.
4. US3 → lock down non-interactive/back-compat guarantees with tests → validate.
5. Polish → docs + fmt/clippy + full quickstart.

---

## Notes

- No new dependencies — `dialoguer`, `console`/`IsTerminal`, `toml` are already vendored (plan.md,
  research D1).
- The first-run hook has exactly one home: `Context::load` (`crates/shap-cli/src/app.rs`), the single
  config-load chokepoint (research D3).
- Passthrough preservation (FR-008) is automatic via the existing `#[serde(flatten)] extra` on
  `Agent`; the foundational round-trip test (T004) guards it.
- Interactive accept-path UX (T011) is verified manually; everything testable without a TTY is covered
  by unit + `assert_cmd` tests.
- Commit after each task or logical group; stop at any checkpoint to validate a story independently.
