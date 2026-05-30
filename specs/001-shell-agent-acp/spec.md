# Feature Specification: Shell-Native Interface for Coding Agents

**Feature Branch**: `001-shell-agent-acp`

**Created**: 2026-05-30

**Status**: Draft

**Input**: User description: "Build a shell-native interface for coding agents using ACP. A lightweight
shell integration that lets users chat with coding agents (Claude Code, Codex, and other compatible
agents) directly from the terminal using colon-prefixed commands."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Chat with a coding agent from the shell (Priority: P1)

A developer working in their terminal selects a coding agent, types a colon followed by a
natural-language request, and receives the agent's response inline in the terminal — without
opening an editor or web UI. When streaming is enabled the response appears progressively; when
disabled, a loader is shown until the final answer prints.

**Why this priority**: This is the core value of the product. Without it, nothing else matters.
A user who can only do this one thing already has a usable tool.

**Independent Test**: With one agent configured, run an agent-selection command, then send a
prompt (`: hello`) and confirm the agent's reply appears in the terminal.

**Acceptance Scenarios**:

1. **Given** an agent is configured and selected, **When** the user types `: hello`, **Then** the
   prompt is sent to the active agent and its response appears in the terminal.
2. **Given** streaming is enabled, **When** the user sends a prompt, **Then** the response renders
   progressively as it arrives.
3. **Given** streaming is disabled, **When** the user sends a prompt, **Then** a loader message is
   shown and the final response prints once complete.
4. **Given** no agent is configured, **When** the user sends a prompt, **Then** the tool prints
   clear setup instructions instead of failing silently.

---

### User Story 2 - Select agent, model, and reasoning effort (Priority: P2)

A developer switches between agents and tunes the active model and reasoning effort using simple
colon commands. When a value is omitted, an interactive picker appears. The model picker shows only
models valid for the currently selected agent. The active agent, model, and reasoning effort can be
shown as a segment in the shell prompt.

**Why this priority**: Choosing the right agent and settings is essential to the product's promise
of switching agents without leaving the shell, but a single hard-coded agent (US1) is still usable.

**Independent Test**: Run `:agent` with no argument and confirm a picker of configured agents opens;
select one; run `:model` and confirm only that agent's models are offered.

**Acceptance Scenarios**:

1. **Given** multiple agents are configured, **When** the user runs `:agent` with no argument,
   **Then** an interactive picker of configured agents opens.
2. **Given** an agent name is supplied, **When** the user runs `:agent codex`, **Then** that agent
   becomes active without a picker.
3. **Given** an agent is active, **When** the user runs `:model`, **Then** only models available for
   that agent are offered.
4. **Given** the user runs `:reasoning`, **Then** a picker of reasoning-effort levels opens; `:effort`
   behaves identically.
5. **Given** the prompt segment is enabled, **When** the active agent/model/reasoning change,
   **Then** the shell prompt reflects the current selection (e.g., `<codex/gpt-5/high>`).
6. **Given** the prompt segment is disabled in configuration, **Then** no such segment is shown.

---

### User Story 3 - Persistent conversations and session control (Priority: P2)

A developer's follow-up prompts continue the same conversation so context is preserved. They can
start a fresh conversation with `:new` (keeping their agent/model/reasoning selections), inspect the
current state with `:status`, and have configuration and sessions persist across terminal restarts.

**Why this priority**: Conversation continuity makes multi-step help useful, but a single-shot prompt
(US1) is still valuable on its own.

**Independent Test**: Send two related prompts and confirm the second is answered with awareness of
the first; run `:new` and confirm a fresh session starts while the agent/model/reasoning stay the same.

**Acceptance Scenarios**:

1. **Given** an active session, **When** the user sends a follow-up prompt, **Then** it is handled
   within the same conversation context.
2. **Given** an active session, **When** the user runs `:new`, **Then** a new session starts and the
   selected agent, model, and reasoning effort are preserved.
3. **Given** any state, **When** the user runs `:status`, **Then** the active agent, model, reasoning
   effort, and current session identifier are displayed.
4. **Given** the user has made selections, **When** the terminal is closed and reopened, **Then** the
   active configuration is restored.
5. **Given** conversations have occurred, **Then** their sessions are saved locally.

---

### User Story 4 - Feed command output to the agent (Priority: P3)

A developer captures the output of a command and includes it in a prompt so the agent can diagnose
real terminal output. They explicitly capture output with `:run <command>` and then reference it with
`:read <prompt>`; a piped invocation is also accepted.

**Why this priority**: Powerful for debugging workflows, but the core chat experience (US1–US3) stands
without it.

**Independent Test**: Run `:run` on a failing command, then `:read fix the failure` and confirm the
captured output was included in the prompt sent to the agent.

**Acceptance Scenarios**:

1. **Given** the user runs `:run pnpm test`, **Then** the command executes and its output is stored
   for later use.
2. **Given** stored command output exists, **When** the user runs `:read fix the test`, **Then** the
   stored output is sent to the agent together with the prompt.
3. **Given** output is piped into the tool's read mode, **Then** the piped output is included with the
   prompt.
4. **Given** no command output has been captured, **When** the user runs `:read`, **Then** the tool
   reports clearly that there is no captured output.

---

### User Story 5 - Generate a Git commit message (Priority: P3)

A developer runs `:commit`; the tool inspects the current Git diff, asks the active agent to generate
a commit message, and prefills the shell command buffer with a ready-to-edit `git commit` command. It
never runs the commit automatically — the user reviews and executes it.

**Why this priority**: A convenient accelerator for a routine task, but not core to chatting with an
agent.

**Independent Test**: With staged/unstaged changes present, run `:commit` and confirm the shell input
buffer is prefilled with a `git commit` command that is not executed until the user presses enter.

**Acceptance Scenarios**:

1. **Given** a Git repository with changes, **When** the user runs `:commit`, **Then** the tool reads
   the diff and obtains a generated commit message from the active agent.
2. **Given** a generated message, **Then** the shell command buffer is prefilled with a `git commit`
   command containing that message.
3. **Given** the prefilled command, **Then** it is never executed automatically; the user must
   explicitly run it.
4. **Given** there are no changes to commit, **When** the user runs `:commit`, **Then** the tool
   reports that there is nothing to commit.

---

### User Story 6 - Diagnostics and graceful degradation (Priority: P3)

A developer can validate their setup with `:doctor` and receives clear, actionable errors when an
agent is missing or unavailable, so problems are easy to resolve.

**Why this priority**: Improves reliability and onboarding, but the tool can function for a correctly
configured user without it.

**Independent Test**: Point the configuration at a non-existent agent command, send a prompt, and
confirm a clear error appears suggesting `:doctor`; run `:doctor` and confirm it reports the problem.

**Acceptance Scenarios**:

1. **Given** the selected agent's command is missing, **When** the user sends a prompt, **Then** a
   clear error is shown that suggests running `:doctor`.
2. **Given** any state, **When** the user runs `:doctor`, **Then** the tool validates the installation
   and reports each configured agent's availability.
3. **Given** an agent becomes unavailable mid-use, **Then** the tool degrades gracefully with a
   readable message rather than crashing.

---

### Edge Cases

- A prompt contains a file reference (e.g., `@test/server.ts`) — the reference is passed through to
  the agent as part of the prompt text.
- The user runs a command that needs a value (e.g., `:agent`) but no interactive picker is available
  in the current environment — the tool reports how to supply the value directly.
- Captured command output is very large — output is bounded to a configured maximum size before being
  sent to the agent.
- `:commit` is run outside a Git repository — the tool reports that the directory is not a Git repo.
- The core command-line tool is invoked directly (without the shell integration) — all commands remain
  usable and produce equivalent results.
- An agent produces no response or errors during streaming — the partial state is surfaced clearly.

## Requirements *(mandatory)*

### Functional Requirements

**Core interaction**

- **FR-001**: The tool MUST let a user send a free-form natural-language prompt to the active agent by
  typing a colon followed by the prompt (`: <prompt>`).
- **FR-002**: The tool MUST display the active agent's response in the terminal.
- **FR-003**: The tool MUST support streamed output (progressive rendering) and non-streamed output
  (loader message followed by the final response), selectable via configuration.

**Selection**

- **FR-004**: The tool MUST let a user select the active agent with `:agent`, accepting an optional
  agent name; when omitted, it MUST open an interactive picker of configured agents.
- **FR-005**: The tool MUST let a user select the active model with `:model`, offering only models
  valid for the currently selected agent.
- **FR-006**: The tool MUST let a user select reasoning effort with `:reasoning`, and MUST treat
  `:effort` as an exact alias.
- **FR-007**: The tool MUST open an interactive picker whenever a command requires a value and none is
  provided.
- **FR-008**: The tool MUST optionally display the current agent, model, and reasoning effort as a
  shell prompt segment, and this display MUST be toggleable via configuration.

**Sessions and state**

- **FR-009**: The tool MUST maintain conversation context so follow-up prompts continue the same
  session.
- **FR-010**: The tool MUST let a user start a new conversation with `:new`, preserving the selected
  agent, model, and reasoning effort.
- **FR-011**: The tool MUST display current status (active agent, model, reasoning effort, session
  identifier) with `:status`.
- **FR-012**: The tool MUST persist the active configuration between shell sessions.
- **FR-013**: The tool MUST persist conversation sessions locally.
- **FR-014**: The tool's design MUST allow resuming previous sessions in a future version (resume is
  out of scope for the MVP).

**Command output capture**

- **FR-015**: The tool MUST let a user run a command and capture its output for later use with
  `:run <command>`.
- **FR-016**: The tool MUST let a user send a prompt together with previously captured command output
  using `:read <prompt>`.
- **FR-017**: The tool MUST accept command output piped into a read mode and include it with the prompt.
- **FR-018**: The tool MUST bound captured output to a configurable maximum size before sending it to
  an agent.

**Git commit assistance**

- **FR-019**: The tool MUST generate a commit message from the current Git diff via the active agent
  when the user runs `:commit`.
- **FR-020**: The tool MUST prefill the shell command buffer with a ready-to-edit `git commit` command
  and MUST NOT execute it automatically.

**Configuration and agents**

- **FR-021**: The tool MUST support per-agent model lists.
- **FR-022**: The tool MUST support passing agent-specific configuration through to each agent.
- **FR-023**: The tool MUST NOT hide important differences between agents from the user.
- **FR-024**: The tool MUST store configuration in a simple, human-editable text format.

**Safety**

- **FR-025**: The tool MUST NOT execute destructive commands automatically.
- **FR-026**: The tool MUST NOT auto-commit code; any commit requires explicit user action.

**Diagnostics and resilience**

- **FR-027**: The tool MUST validate installation and report each configured agent's availability via
  `:doctor`.
- **FR-028**: When the selected agent's command is missing, the tool MUST print a clear error that
  suggests running `:doctor`.
- **FR-029**: When no agent is configured, the tool MUST print actionable setup instructions.
- **FR-030**: The tool MUST degrade gracefully and present readable, actionable errors when an agent is
  unavailable.

**Architecture and portability**

- **FR-031**: The core command-line tool MUST be usable directly, independently of the shell
  integration.
- **FR-032**: The shell integration MUST stay thin — limited to parsing colon commands, prompt
  integration, and passing context to the core tool — with core logic residing in the command-line tool.
- **FR-033**: The shell integration MUST NOT measurably slow normal shell startup or prompt rendering.
- **FR-034**: The tool MUST be structured so that additional shells can be added later (Zsh first).
- **FR-035**: The tool MUST be structured so that additional compatible agents can be added later
  without being tied to a single vendor.

### Key Entities *(include if feature involves data)*

- **Agent**: A configured coding agent the user can talk to. Has a name, a launch command, a list of
  available models, a default model, and optional agent-specific configuration.
- **Model**: A selectable model belonging to a specific agent.
- **Reasoning effort**: A selectable level controlling depth-versus-speed for the active agent.
- **Session**: A persisted conversation with an agent, identified uniquely, holding the exchange
  history needed to continue context. Created fresh by `:new`.
- **Active state**: The user's current selections (agent, model, reasoning effort, active session)
  persisted between shell sessions.
- **Configuration**: User-editable settings — default agent, per-agent definitions, UI options
  (streaming, prompt segment, picker), and history/output options.
- **Captured command output**: The stored output of a user-run command, bounded in size, available to
  include in a subsequent prompt.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can go from selecting an agent to receiving a response to a prompt entirely
  within the terminal, without opening any editor or web UI.
- **SC-002**: The model picker never offers a model that is invalid for the selected agent (zero
  invalid model selections are possible).
- **SC-003**: 100% of generated `git commit` commands require an explicit user action to run; none are
  executed automatically.
- **SC-004**: Active configuration and conversation sessions survive a terminal restart in 100% of
  cases.
- **SC-005**: When an agent is missing or unavailable, 100% of failures produce a readable message
  that names a concrete next step (e.g., run `:doctor`).
- **SC-006**: Enabling the shell integration adds no perceptible delay to prompt rendering for normal
  (non-agent) commands.
- **SC-007**: Every colon command produces the same result when the equivalent core tool command is
  invoked directly, confirming the tool is usable without the shell layer.
- **SC-008**: Starting a new conversation with `:new` preserves the selected agent, model, and
  reasoning effort 100% of the time.

## Assumptions

- **Reasoning-effort levels**: A small fixed set of levels (assumed `low`, `medium`, `high`) is
  offered; the exact set may be refined during planning and can vary per agent where an agent exposes
  different levels.
- **MVP shell**: Zsh is the only supported shell for the MVP; the design keeps shell-specific glue
  isolated so other shells can be added later.
- **MVP agents**: At least one compatible agent is supported in the MVP; the configuration model
  supports multiple agents from the start.
- **Interactive picker**: A picker is used when a required value is omitted; if an external picker tool
  is unavailable, the tool falls back to a built-in selection prompt or instructs the user to pass the
  value directly.
- **Local-only storage**: All configuration and session data is stored locally on the user's machine;
  remote sync, sharing, and cross-device features are out of scope.
- **Prompt buffer prefill**: The shell integration is able to place text into the user's command input
  buffer for review before execution (used by `:commit`).
- **File references in prompts**: Tokens such as `@path/to/file` are passed through to the agent as
  part of the prompt; the tool does not itself resolve or attach file contents in the MVP unless the
  agent requests it.
- **Out of scope for MVP**: automatic scrollback capture, Bash/Fish support, web UI, remote sync, team
  sharing, automatic code modification without confirmation, automatic Git commits, complex session
  search, plugin marketplace, and editor integrations.
