---

description: "Task list for Shell-Native Interface for Coding Agents (shap)"
---

# Tasks: Shell-Native Interface for Coding Agents

**Input**: Design documents from `/specs/001-shell-agent-acp/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Test tasks are INCLUDED. Constitution v1.0.0 Principle IV requires unit tests for meaningful
logic, branching, edge cases, and error handling; plan.md and research.md (D12) enumerate the test
areas and snapshot targets. Trivial data definitions are not separately tested.

**Organization**: Tasks are grouped by user story (US1–US6, priority order) so each story is an
independently testable increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: User story the task serves (US1…US6); Setup/Foundational/Polish have no story label
- Exact file paths are included in each description

## Path Conventions

Cargo workspace (per plan.md): binary `crates/shap-cli/`, libraries `crates/shap-core/` and
`crates/shap-agent/`, shell helpers `crates/shap-shell/`, Zsh integration `shell/zsh/`,
end-to-end tests `tests/integration/`.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Workspace skeleton and tooling.

- [x] T001 Create Cargo workspace manifest and four crate skeletons (`Cargo.toml`, `crates/shap-cli/{Cargo.toml,src/main.rs}`, `crates/shap-core/{Cargo.toml,src/lib.rs}`, `crates/shap-agent/{Cargo.toml,src/lib.rs}`, `crates/shap-shell/{Cargo.toml,src/lib.rs}`)
- [x] T002 Declare shared dependencies in `[workspace.dependencies]` in `Cargo.toml` per plan.md (tokio, tokio-util, futures, async-trait, clap, clap_complete, serde, serde_json, toml, schemars, anyhow, thiserror, miette, tracing, tracing-subscriber, which, dialoguer, indicatif, console, anstream, anstyle, terminal_size, ignore, globset, dunce, shell-words, agent-client-protocol, agent-client-protocol-tokio; dev: assert_cmd, predicates, insta, tempfile, test-case, agent-client-protocol-test)
- [x] T003 [P] Add `rustfmt.toml` and crate-level clippy config; confirm `cargo clippy --all-targets --all-features -- -D warnings` is clean on the skeleton
- [x] T004 [P] Add CI workflow `.github/workflows/ci.yml` running fmt check, clippy `-D warnings`, `cargo nextest run --all-features`, `cargo audit`, `cargo deny check`
- [x] T005 [P] Add `deny.toml` for cargo-deny (advisories, licenses, bans)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Config, state, paths, errors, CLI skeleton, and the agent abstraction — every story depends on these.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T006 Implement path resolution (XDG dirs, `SHAP_CONFIG`/`SHAP_DATA_DIR` overrides, `~`/`$XDG_*` expansion) in `crates/shap-core/src/paths.rs`
- [x] T007 [P] Define the domain error enum with `thiserror` + `miette` diagnostics (actionable messages) in `crates/shap-core/src/error.rs`
- [x] T008 [P] Define the `AgentClient` trait and `AgentRequest`/`AgentResponse`/`AgentStream`/`SessionOptions`/`SessionId` types in `crates/shap-agent/src/client.rs`
- [x] T009 Implement `Config`/`Agent`/`UiOptions`/`HistoryOptions`/`FileOptions` types, TOML loader, and validation in `crates/shap-core/src/config.rs` (depends on T006, T007)
- [x] T010 Implement `ActiveState` type with atomic JSON read/write (temp file + rename) and config cross-check/repair in `crates/shap-core/src/state.rs` (depends on T006, T007)
- [x] T011 Implement clap CLI definitions (subcommands `send`, `agent`, `model`, `reasoning`, `new`, `status`, `commit`, `run`, `read`, `doctor`; global `--cwd`/`--config`), dispatch, and exit-code mapping (0/1/2) in `crates/shap-cli/src/cli.rs` and `crates/shap-cli/src/main.rs` (depends on T007)
- [x] T012 Initialize `tracing` subscriber with env-filter in `crates/shap-cli/src/main.rs` (depends on T011)
- [x] T013 [P] Wire `crates/shap-core/src/lib.rs` module exports (paths, error, config, state) (depends on T006, T007, T009, T010)
- [x] T014 [P] Unit tests for config validation — `default_agent` membership, `default_model ∈ models`, picker enum, positive byte limits, missing-file path — in `crates/shap-core/src/config.rs` (depends on T009)
- [x] T015 [P] Unit tests for state atomic read/write and missing-file → all-null in `crates/shap-core/src/state.rs` (depends on T010)
- [x] T016 [P] Unit tests for path resolution and env overrides in `crates/shap-core/src/paths.rs` (depends on T006)

**Checkpoint**: Foundation ready — user stories can begin.

---

## Phase 3: User Story 1 - Chat with a coding agent from the shell (Priority: P1) 🎯 MVP

**Goal**: A user selects/uses a configured agent and gets a response to `: <prompt>` in the terminal, streamed or loader-then-final.

**Independent Test**: With one agent configured, run `shap send "hello"` and confirm the reply appears; toggle `[ui].stream` and confirm both modes work; with no agent, confirm setup instructions print.

### Tests for User Story 1

- [x] T017 [P] [US1] Integration test: `shap send "hello"` returns agent output using a mock ACP agent (`agent-client-protocol-test`) in `tests/integration/send.rs`
- [x] T018 [P] [US1] Unit tests: `@file` expansion (resolve relative to cwd, reject binary, enforce `max_file_bytes`, honor gitignore, leave unresolved `@refs` visible) in `crates/shap-core/src/files.rs`
- [x] T019 [P] [US1] Unit test: base prompt composition in `crates/shap-core/src/prompt.rs`

### Implementation for User Story 1

- [x] T020 [P] [US1] Implement the agent registry (configured agent → launchable process spec) in `crates/shap-agent/src/registry.rs` (depends on T008, T009)
- [x] T021 [US1] Implement the ACP wrapper over tokio child-process stdio, implementing `AgentClient` (start session, send, stream) in `crates/shap-agent/src/acp.rs` (depends on T008, T020)
- [~] T022 [P] [US1] Implement session-id <-> ACP session mapping in `crates/shap-agent/src/session.rs` (depends on T008) -- **N/A**: under the one-shot per-invocation model the ACP session lives only inside a single `connect_with` scope, so there is no persistent agent-side session id to map. Revisit if/when resume (FR-014) lands.
- [x] T023 [P] [US1] Implement the JSONL session store (create file, append `session_started`/`user_prompt`/`agent_response`/`error`, tolerate corrupt trailing lines) in `crates/shap-core/src/session.rs` (depends on T006, T007)
- [x] T024 [P] [US1] Implement `@file` detection/resolution/guards in `crates/shap-core/src/files.rs` (depends on T009)
- [x] T025 [P] [US1] Implement base prompt composition (prompt + attachments) in `crates/shap-core/src/prompt.rs` (depends on T024)
- [x] T026 [P] [US1] Implement output rendering — streamed and spinner-then-final (`indicatif`/`anstream`) — in `crates/shap-shell/src/render.rs` (depends on T007)
- [x] T027 [US1] Implement the `send` command handler wiring registry + ACP + session store + `@file` + rendering in `crates/shap-core/src/commands.rs` (depends on T021, T023, T024, T025, T026, T010)
- [x] T028 [US1] Implement no-agent-configured setup instructions and missing-agent error (suggests `shap doctor`) in `crates/shap-core/src/commands.rs` / `crates/shap-core/src/error.rs` (depends on T027)
- [x] T029 [US1] Add the Zsh `: <prompt>` mapping and `shell/zsh/shap.zsh` source skeleton (cwd forwarding, minimal error display) (depends on T011)

**Checkpoint**: US1 is a usable MVP — chat with one agent from the shell.

---

## Phase 4: User Story 2 - Select agent, model, and reasoning effort (Priority: P2)

**Goal**: Switch agent/model/reasoning via colon commands with pickers when a value is omitted; show the selection in the prompt segment.

**Independent Test**: `shap agent` opens a picker of configured agents; `shap model` offers only the active agent's models; `shap reasoning`/`:effort` behave identically; the prompt segment reflects selections and can be toggled off.

### Tests for User Story 2

- [x] T030 [P] [US2] Integration tests: `shap model` rejects a model invalid for the active agent; switching agent resets the model to the new agent's default — in `tests/integration/selection.rs`
- [x] T031 [P] [US2] Unit test: picker resolution priority (fzf → skim → builtin) in `crates/shap-core/src/picker.rs`

### Implementation for User Story 2

- [x] T032 [P] [US2] Implement picker resolution + selection (`which` detection, external fzf/skim, `dialoguer` fallback, non-interactive guidance) in `crates/shap-core/src/picker.rs` (depends on T007)
- [x] T033 [US2] Implement the `agent` handler (set or picker; reset `active_model` to default when invalid for the new agent) in `crates/shap-core/src/commands.rs` (depends on T032, T009, T010)
- [x] T034 [US2] Implement the `model` handler (offer/set only the active agent's models) in `crates/shap-core/src/commands.rs` (depends on T032, T010)
- [x] T035 [US2] Implement the `reasoning` handler with `:effort` as an exact alias in `crates/shap-core/src/commands.rs` (depends on T032, T010)
- [x] T036 [P] [US2] Implement the prompt-segment string builder reading `state.json` directly (cheap, no subprocess) in `crates/shap-shell/src/prompt.rs` (depends on T010)
- [x] T037 [US2] Add Zsh `:agent`/`:model`/`:reasoning`/`:effort` mappings and the optional prompt-segment hook (gated by `show_prompt_segment`) in `shell/zsh/shap.zsh` (depends on T033, T034, T035, T036)

**Checkpoint**: US1 + US2 work independently — selectable agents/models/reasoning with prompt segment.

---

## Phase 5: User Story 3 - Persistent conversations and session control (Priority: P2)

**Goal**: Follow-ups continue the same session; `:new` starts a fresh one preserving selections; `:status` shows current state; config/sessions persist across restarts.

**Independent Test**: Send two related prompts and confirm continuity; run `:new` and confirm a fresh session with unchanged agent/model/reasoning; run `:status`.

### Tests for User Story 3

- [x] T038 [P] [US3] Integration test: follow-up prompt continues the same session and `:new` preserves agent/model/reasoning — in `tests/integration/sessions.rs`
- [x] T039 [P] [US3] Snapshot test: `shap status` output via `insta` in `tests/integration/status.rs`

### Implementation for User Story 3

- [x] T040 [US3] Implement active-session continuity in the send path (reuse `active_session_id`, lazy-create on first prompt) in `crates/shap-core/src/commands.rs` (depends on T027, T023)
- [x] T041 [US3] Implement the `new` handler (create session, set `active_session_id`, preserve selections) in `crates/shap-core/src/commands.rs` (depends on T023, T010)
- [x] T042 [US3] Implement the `status` handler (human output and `--json`: agent/model/reasoning/session id) in `crates/shap-core/src/commands.rs` (depends on T010)
- [x] T043 [US3] Add Zsh `:new`/`:status` mappings in `shell/zsh/shap.zsh` (depends on T041, T042)

**Checkpoint**: US1–US3 work independently — full session lifecycle.

---

## Phase 6: User Story 4 - Feed command output to the agent (Priority: P3)

**Goal**: Capture command output with `:run` and include it in a prompt with `:read`; support pipe mode.

**Independent Test**: `shap run -- <failing cmd>` captures output; `shap read "fix it"` includes it; `<cmd> | shap read "…"` works; empty capture gives a clear message.

### Tests for User Story 4

- [x] T044 [P] [US4] Integration tests: `:run` captures output, `:read` includes it, and pipe mode works — in `tests/integration/capture.rs`
- [x] T045 [P] [US4] Snapshot test: `:read` prompt-payload composition (user prompt + command + exit code + output) via `insta` in `crates/shap-core/src/prompt.rs`

### Implementation for User Story 4

- [x] T046 [P] [US4] Implement the captured-output store (write/read, truncate to `max_output_bytes` with a flag, command/exit-code/timestamp metadata) in `crates/shap-core/src/output_capture.rs` (depends on T006, T009)
- [x] T047 [US4] Implement the `run` handler (split argv via `shell-words`, spawn under tokio, stream stdout+stderr live, capture combined output and exit code) in `crates/shap-core/src/commands.rs` (depends on T046)
- [x] T048 [US4] Implement the `read` handler (load latest capture or read stdin in pipe mode, compose payload, send like `send`) in `crates/shap-core/src/commands.rs` (depends on T046, T025, T027)
- [x] T049 [US4] Add Zsh `:run`/`:read` mappings in `shell/zsh/shap.zsh` (depends on T047, T048)

**Checkpoint**: US1–US4 work independently — command output flows to the agent.

---

## Phase 7: User Story 5 - Generate a Git commit message (Priority: P3)

**Goal**: `:commit` reads the Git diff, gets a message from the agent, and prefills the shell buffer with a `git commit` command — never executing it.

**Independent Test**: With changes present, `shap commit --prefill-shell-buffer` prints a `git commit -am "…"` line and runs nothing; the Zsh widget inserts it into the buffer for manual Enter.

### Tests for User Story 5

- [x] T050 [P] [US5] Snapshot + behavior tests: `:commit` prompt composition (prefers staged diff, includes branch/status) and assertion that `git commit` is never executed — in `tests/integration/commit.rs`

### Implementation for User Story 5

- [x] T051 [P] [US5] Implement Git CLI helpers (`status --short`, `diff --staged`, `diff`, `branch --show-current`) in `crates/shap-core/src/git.rs` (depends on T007)
- [x] T052 [US5] Implement the `commit --prefill-shell-buffer` handler (choose staged-else-unstaged diff, build agent prompt, get message, print `git commit -am "<message>"`; handle not-a-repo and nothing-to-commit) in `crates/shap-core/src/commands.rs` (depends on T051, T027, T025)
- [x] T053 [US5] Implement the Zsh ZLE widget that inserts the generated command into the buffer (never executes) and bind `:commit` to it in `shell/zsh/shap.zsh` (depends on T052)

**Checkpoint**: US1–US5 work independently — commit-message assist with user control preserved.

---

## Phase 8: User Story 6 - Diagnostics and graceful degradation (Priority: P3)

**Goal**: `:doctor` validates the setup; errors are actionable; agent unavailability degrades gracefully.

**Independent Test**: Point config at a missing agent command, send a prompt → clear error suggesting `:doctor`; run `:doctor` → it reports the problem.

### Tests for User Story 6

- [x] T054 [P] [US6] Snapshot tests: `shap doctor` report and the missing-agent diagnostic via `insta` in `tests/integration/doctor.rs`

### Implementation for User Story 6

- [x] T055 [US6] Implement doctor checks (config exists/parses, agent commands on PATH, selected agent/model validity, picker presence if configured, git available, session dir writable, agent process can launch, shell integration installed) in `crates/shap-core/src/doctor.rs` (depends on T009, T010, T020, T032, T051)
- [x] T056 [US6] Implement the `doctor` handler (human report + `--json`, exit 0/1) in `crates/shap-core/src/commands.rs` (depends on T055)
- [x] T057 [US6] Implement graceful degradation: agent-unavailable mid-stream produces a readable error recorded as an `error` session record in `crates/shap-core/src/commands.rs` and `crates/shap-agent/src/acp.rs` (depends on T027, T021)
- [x] T058 [US6] Add the Zsh `:doctor` mapping in `shell/zsh/shap.zsh` (depends on T056)

**Checkpoint**: All six user stories independently functional.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Docs, completions, packaging, and final quality gates.

- [x] T059 [P] Write `docs/config.md` (config reference + generated JSON schema)
- [x] T060 [P] Write `docs/shell-integration.md` (install, prompt segment, `:commit` widget)
- [x] T061 [P] Write `docs/agents.md` (configuring ACP agents, agent-specific passthrough)
- [x] T062 [P] Export the config JSON schema via `schemars` (e.g. `shap config --schema`) in `crates/shap-core/src/config.rs`
- [x] T063 [P] Generate shell completions via `clap_complete` (`shap completions <shell>`) in `crates/shap-cli/src/cli.rs`
- [x] T064 Configure `cargo-dist` release metadata (targets: macOS arm64/x86_64, Linux x86_64/arm64) in `Cargo.toml` and add `.github/workflows/release.yml`
- [x] T065 [P] Run the quickstart.md acceptance smoke checklist end-to-end and fix gaps
- [x] T066 Final quality gate: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo nextest run --all-features`, `cargo audit`, `cargo deny check` all green

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup — BLOCKS all user stories.
- **User Stories (Phases 3–8)**: All depend on Foundational. After that they can proceed in priority
  order (P1 → P2 → P2 → P3 → P3 → P3) or in parallel by different developers.
- **Polish (Phase 9)**: Depends on the desired user stories being complete.

### User Story Dependencies

- **US1 (P1)**: After Foundational. No dependency on other stories.
- **US2 (P2)**: After Foundational. Independent; shares `commands.rs`/`shap.zsh` with US1 (sequence edits to those files).
- **US3 (P2)**: After Foundational. Extends the US1 send path for continuity (T040 depends on T027).
- **US4 (P3)**: After Foundational. `read` reuses the send path (T048 depends on T027/T025).
- **US5 (P3)**: After Foundational. `commit` reuses send + prompt composition (T052 depends on T027/T025).
- **US6 (P3)**: After Foundational. `doctor` inspects registry/git/picker (T055 depends on T020/T032/T051), so it is most complete once those exist.

### Within Each User Story

- Tests are written alongside and MUST pass before the story is considered done (Constitution IV).
- Library modules before the `commands.rs` handler that wires them.
- Command handler before the Zsh mapping that calls it.

### Shared-file note

`crates/shap-core/src/commands.rs` and `shell/zsh/shap.zsh` are touched by multiple stories. Tasks
editing them are intentionally **not** marked `[P]` across stories — serialize edits to these two files.

### Parallel Opportunities

- Setup: T003, T004, T005 in parallel.
- Foundational: T007 and T008 in parallel; then T009/T010 (after T006/T007); tests T014/T015/T016 in parallel.
- US1: T017/T018/T019 (tests) in parallel; T020/T022/T023/T024/T026 in parallel (different files) before T027.
- US2: T030/T031 in parallel; T032 and T036 in parallel before the handlers.
- Across stories: once Foundational is done, US1/US2/US4/US5/US6 library modules can largely proceed in
  parallel; serialize only the shared `commands.rs`/`shap.zsh` edits.

---

## Parallel Example: User Story 1

```bash
# Tests for US1 together:
Task: "Integration test shap send in tests/integration/send.rs"
Task: "Unit tests @file expansion in crates/shap-core/src/files.rs"
Task: "Unit test prompt composition in crates/shap-core/src/prompt.rs"

# Independent library modules for US1 together (before the send handler T027):
Task: "Agent registry in crates/shap-agent/src/registry.rs"
Task: "Session-id mapping in crates/shap-agent/src/session.rs"
Task: "JSONL session store in crates/shap-core/src/session.rs"
Task: "@file resolution in crates/shap-core/src/files.rs"
Task: "Output rendering in crates/shap-shell/src/render.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 Setup → 2. Phase 2 Foundational → 3. Phase 3 US1 → 4. **STOP and validate**: chat with one
   agent end-to-end (`shap send`, streamed + non-streamed, no-agent message). This is a demoable MVP.

### Incremental Delivery

Foundation → US1 (MVP) → US2 (selection + prompt segment) → US3 (sessions) → US4 (capture) →
US5 (commit) → US6 (doctor). Each story is testable and demoable on its own before the next.

### Parallel Team Strategy

After Foundational: assign US1 first (it defines the send path others reuse), then split US2/US4/US5/US6
across developers, coordinating edits to `commands.rs` and `shap.zsh`.

---

## Notes

- `[P]` = different files, no dependency on an incomplete task.
- `[Story]` labels (US1–US6) map tasks to spec.md user stories for traceability.
- Constitution Principle VII: `:commit` and any Git command must be prefilled for review, never executed
  by the tool (T052, T053, T050 verify this).
- Constitution Principle VIII: keep `shap.zsh` thin — mapping, prompt segment, and buffer insertion only.
- Commit after each task or logical group; stop at any checkpoint to validate a story independently.
