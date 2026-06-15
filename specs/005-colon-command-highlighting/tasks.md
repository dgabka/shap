---

description: "Task list for Colon-Command Syntax Highlighting"
---

# Tasks: Colon-Command Syntax Highlighting

**Input**: Design documents from `/specs/005-colon-command-highlighting/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/colon-commands.md, quickstart.md

**Tests**: No automated tests requested. This is a shell-only change; the unchanged Rust
`commit --prefill-shell-buffer` path stays covered by `crates/shap-cli/tests/commit.rs`, and the new
shell behavior is verified manually via `quickstart.md` (no zsh test harness in the project).

**Organization**: Tasks grouped by user story. Note US1 and US2 both touch the single file
`shell/zsh/shap.zsh`, so they are sequential (not parallel) — US2 verifies uniformity of the US1 edit.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: US1 / US2

## Path Conventions

Shell integration: `shell/zsh/shap.zsh`. Docs: `docs/shell-integration.md`. Rust CLI under `crates/`
is **not** modified by this feature.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Get a working `shap` binary + a highlighter-enabled zsh for manual verification.

- [X] T001 Build the CLI so `shap` is available: run `cargo build`, then `export SHAP_BIN="$(pwd)/target/debug/shap"` (per quickstart.md). No code change.
- [X] T002 In a test zsh, `source shell/zsh/shap.zsh` and source a command-word highlighter (zsh-syntax-highlighting or fast-syntax-highlighting); confirm `:agent` highlights as recognized and `:commit` currently highlights red (reproduce the defect baseline).

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Confirm the unchanged dependency the fix relies on.

**⚠️ CRITICAL**: Verify before editing so the function stub delegates to an existing, working command.

- [X] T003 Confirm `shap commit --prefill-shell-buffer` exists and is unchanged (`crates/shap-cli/src/cli.rs:75`, handler `crates/shap-cli/src/app.rs:258`, tests `crates/shap-cli/tests/commit.rs`). No edit — this path stays as-is.

**Checkpoint**: Baseline reproduced, CLI dependency confirmed — implementation can begin.

---

## Phase 3: User Story 1 - `:commit` is recognized as a valid command (Priority: P1) 🎯 MVP

**Goal**: Make `:commit` resolve as a command word so highlighters render it valid, while the
`accept-line` widget keeps owning the buffer-rewrite and nothing ever auto-commits.

**Independent Test**: With integration + highlighter active, `:commit` is no longer red; pressing Enter
on bare `:commit` still only prefills the `git commit …` line (never executes), per quickstart.md steps
1, 3, 4.

### Implementation for User Story 1

- [X] T004 [US1] In `shell/zsh/shap.zsh`, add a thin `function :commit { … }` beside the other colon functions (after `:read`, before the accept-line widget block). Body handles only fall-through/misuse (e.g. `:commit <args>`): print one actionable line telling the user to type `:commit` and press Enter; return non-zero; NEVER invoke git (FR-002, FR-007, Constitution VII). Reference contract B2 in `contracts/colon-commands.md`.
- [X] T005 [US1] In `shell/zsh/shap.zsh`, update the `# `:commit` is handled by the accept-line widget…` comment (around line 36 and the widget header) to describe the new split: function makes the word resolvable for highlighting; the `accept-line` widget still owns the bare-`:commit` buffer rewrite and intercepts before the function runs.
- [X] T006 [US1] Verify the `accept-line` widget is unchanged and still intercepts exact `:commit`/`: commit` before `.accept-line` (so the function is not invoked in the normal path); confirm via quickstart.md steps 3–4 that Enter on `:commit` prefills the commit line and never auto-commits.
- [X] T007 [US1] Verify highlighting + misuse guidance: quickstart.md step 1 (`:commit` renders recognized, not red) and step 5 (`:commit something` prints guidance, runs no git).

**Checkpoint**: `:commit` highlights as valid and behaves exactly as before (review-only). MVP complete.

---

## Phase 4: User Story 2 - All shap colon commands highlight consistently (Priority: P2)

**Goal**: Confirm every documented colon command renders uniformly as recognized (no command left in
the unknown/red style).

**Independent Test**: With integration + highlighter active, type each colon command and confirm none
renders as unknown (quickstart.md step 2).

### Implementation for User Story 2

- [X] T008 [US2] Verify uniform recognition for `:agent`, `:model`, `:reasoning`, `:effort`, `:new`, `:status`, `:doctor`, `:run`, `:read`, `:commit` (quickstart.md step 2). No code change expected — if any command is still red, fix its definition in `shell/zsh/shap.zsh`.
- [X] T009 [US2] Verify no regression on the `:` paths: bare `:` builtin and `: <text>` chat behave as before (quickstart.md steps 6–7; FR-004, SC-004).

**Checkpoint**: All colon commands consistent; `:` builtin and chat path unaffected.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Documentation and final validation.

- [X] T010 [P] Update `docs/shell-integration.md`: note `:commit` is now a function (so it highlights as valid) while the widget still owns the buffer rewrite; clarify `:commit <args>` prints guidance and never commits. Update the "handled by widget" wording in the `## :commit widget` and Commands sections.
- [X] T011 Run the full `quickstart.md` checklist end-to-end (build → highlight → behavior → misuse → no-regression) and confirm all "Done when" criteria (SC-001..SC-004) pass.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: After Setup — confirms the unchanged CLI dependency. Blocks implementation.
- **User Story 1 (Phase 3)**: After Foundational. The core edit + MVP.
- **User Story 2 (Phase 4)**: After US1 — it verifies uniformity of the same `shap.zsh` edit; not independent of US1's file change.
- **Polish (Phase 5)**: After US1 (docs can be drafted in parallel with US2 verification).

### Within / across stories

- T004 → T005 → (T006, T007): edit the function, then the comment, then verify (same file → sequential).
- US2 (T008, T009) depends on US1's edit being in place.
- T010 (docs) is [P] — different file (`docs/shell-integration.md`) — can proceed once T004/T005 land.
- T011 is the final gate; depends on all prior tasks.

### Parallel Opportunities

- Limited: the feature centers on one file. T010 (docs) can run parallel to US2 verification.
- All edits to `shell/zsh/shap.zsh` (T004–T007, plus any T008 fix) are sequential — same file.

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → reproduce the red `:commit` baseline.
2. Phase 2 Foundational → confirm `commit --prefill-shell-buffer` unchanged.
3. Phase 3 US1 → add `:commit` function + comment update; verify highlight + unchanged behavior.
4. **STOP and VALIDATE**: `:commit` no longer red; Enter still prefills, never commits. Ship-able.

### Incremental Delivery

1. US1 → highlight fix + behavior preserved (MVP).
2. US2 → confirm all colon commands uniform, no `:`-path regression.
3. Polish → docs + full quickstart validation.

---

## Notes

- [P] = different file, no dependency. Here essentially only the docs task.
- No automated tests added (no shell harness; Rust path unchanged and already covered).
- Constitution VII (never auto-commit) and VIII (thin shell) are acceptance gates, not optional.
- Commit after US1 (MVP) and after Polish.
