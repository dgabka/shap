# Feature Specification: Config Init Wizard & Interactive Config Editing

**Feature Branch**: `004-config-init-wizard`

**Created**: 2026-06-09

**Status**: Draft

**Input**: User description: "init wizard on first run, now it just says that user needs to create config. instead run an interactive wizard with some prompts to create basic config. also add a config command to interactively change the config with prompts as well"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Guided first-run setup (Priority: P1)

A new user installs the tool and runs a command that needs configuration. Today they hit an error
that prints a TOML snippet and tells them to hand-create the file. Instead, the tool detects that no
config exists and offers an interactive setup wizard that asks a short series of plain-language
questions (which agent, its launch command, which models, sensible defaults) and writes a valid
config file for them. When the wizard finishes, the user is told exactly how to proceed (or the
original command continues).

**Why this priority**: This is the core of the request and the single biggest reduction in
time-to-first-success. A new user reaching a working configuration without reading reference docs or
editing TOML by hand is the MVP. It is independently valuable even if interactive editing (Story 2)
is never built.

**Independent Test**: Remove any existing config, run the tool in an interactive terminal, answer
the wizard prompts, and confirm a valid config file is written and the tool then works (e.g.
`shap doctor` passes).

**Acceptance Scenarios**:

1. **Given** no config file exists and the user runs a command that needs config in an interactive
   terminal, **When** the wizard offers setup and the user accepts and answers the prompts, **Then**
   a valid config file is written at the standard location and the tool reports success.
2. **Given** the user is partway through the wizard, **When** they cancel (e.g. Ctrl-C or decline),
   **Then** no partial or invalid config file is left behind and the tool exits cleanly with
   guidance on how to set up later.
3. **Given** a config file already exists, **When** the user runs a normal command, **Then** the
   wizard does not trigger and behavior is unchanged.
4. **Given** the wizard has gathered all answers, **When** it writes the file, **Then** the result
   passes the same validation the tool applies on load (a valid `default_agent`, at least one agent
   with a non-empty model list whose `default_model` is a member).

---

### User Story 2 - Interactive config editing (Priority: P2)

An existing user wants to change a setting — switch the default agent, add a model, change the
picker, toggle streaming — without remembering the TOML structure or field names. They run the
config command and are walked through prompts to view and change the relevant values, then the
updated config is written back and re-validated.

**Why this priority**: High convenience value and a natural extension of Story 1, but the tool is
already usable for configured users via hand-editing, so it ranks below first-run setup.

**Independent Test**: With an existing valid config, run the config command, change one setting
through the prompts, confirm the file reflects the change and still validates.

**Acceptance Scenarios**:

1. **Given** a valid config exists, **When** the user runs the interactive config command and
   changes the default agent to another configured agent, **Then** the file is updated and the new
   default takes effect on the next command.
2. **Given** the user makes a change that would produce an invalid config, **When** they confirm,
   **Then** the change is rejected with a clear reason and the existing valid config is preserved.
3. **Given** the user runs the interactive config command and makes no changes (or cancels),
   **Then** the existing config file is left byte-for-byte unchanged.
4. **Given** an agent has opaque agent-specific passthrough keys, **When** the user edits an
   unrelated setting and saves, **Then** those passthrough keys are preserved in the written file.

---

### User Story 3 - Safe fallback for non-interactive contexts (Priority: P3)

The tool is also invoked from scripts, pipes, and the shell prompt hook where no human can answer
prompts. In those contexts the wizard must never block waiting for input; it falls back to the
current behavior — a clear, actionable message describing how to create the config — so automated
callers fail predictably.

**Why this priority**: Protects existing automated and shell-integration use from regressions. Lower
priority because it is a guardrail rather than new user-facing value, but it must ship with Story 1
to avoid breaking non-interactive callers.

**Independent Test**: Run a config-requiring command with no config and stdin not attached to a
terminal (e.g. piped/redirected); confirm the tool prints setup guidance and exits non-zero without
hanging.

**Acceptance Scenarios**:

1. **Given** no config and a non-interactive invocation, **When** a command needs config, **Then**
   the tool prints setup instructions and exits with a non-zero status without prompting.
2. **Given** the shell prompt-segment path (which reads cached state only), **When** no config
   exists, **Then** the wizard is not triggered and the hook stays cheap and silent.

---

### Edge Cases

- **Existing-but-broken config**: A config file exists but fails to parse or validate. The
  first-run wizard (for *missing* config) does not trigger; the user sees the existing parse/
  validation diagnostic. Repairing a broken config is handled through the interactive config command
  (Story 2), not the first-run wizard.
- **Config directory not writable / disk error**: Writing the file fails. The tool reports the
  write error and the path it attempted, and leaves no partial file.
- **Cancellation mid-wizard**: Declining, EOF on input, or interrupt leaves the system in its prior
  state (no file created, or existing file untouched).
- **Custom config path**: A `--config <path>` / `SHAP_CONFIG` override is in effect. The wizard
  targets that path, and the "exists?" check applies to it.
- **Concurrent edits**: The interactive config command writes the file as a whole; the last writer
  wins. The feature does not add file locking.
- **Agent command not on PATH**: The wizard/editor may accept a launch command that is not currently
  installed. It records the value and points the user to the existing validation/doctor step rather
  than blocking entry.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When a command requires configuration and no config file exists at the resolved path,
  in an interactive terminal the tool MUST offer an interactive setup wizard instead of only printing
  manual-creation instructions.
- **FR-002**: The wizard MUST collect, at minimum, the information needed for a valid config: at
  least one agent (a name, its launch command, its list of models, and the default model among
  them) and the default agent.
- **FR-003**: The wizard MUST apply sensible defaults and allow the user to accept them with minimal
  input, so a basic working config can be produced in a few prompts.
- **FR-004**: The wizard MUST write a config file that passes the tool's existing load-time
  validation, at the standard resolved location (honoring `--config` / `SHAP_CONFIG` overrides).
- **FR-005**: If the user cancels or the wizard cannot complete, the tool MUST NOT write a partial or
  invalid config file and MUST exit cleanly with guidance.
- **FR-006**: The tool MUST provide an interactive config command that lets an existing user view and
  change configuration values through prompts, without hand-editing TOML.
- **FR-007**: The interactive config command MUST re-validate the result before writing and MUST
  reject changes that would produce an invalid config, preserving the prior valid file.
- **FR-008**: The interactive config command MUST preserve agent-specific opaque passthrough keys and
  any fields it does not surface in its prompts when writing the file back.
- **FR-009**: If the user makes no changes or cancels the interactive config command, the existing
  config file MUST be left unchanged.
- **FR-010**: In non-interactive contexts (no attached terminal / not a TTY), the tool MUST NOT
  prompt; for a missing config it MUST fall back to printing actionable setup instructions and exit
  non-zero, preserving today's behavior for scripts, pipes, and the prompt hook.
- **FR-011**: The cheap shell prompt-segment path (cached-state-only) MUST NOT trigger the wizard or
  any config write.
- **FR-012**: Existing non-interactive `config` outputs MUST remain available (the resolved config
  path and the generated JSON schema) so current scripts and docs are not broken.
- **FR-013**: All wizard/editor prompts MUST be in plain language understandable without reading the
  config reference; field jargon SHOULD be explained inline.
- **FR-014**: Any failure to write the config (permissions, disk, invalid target) MUST surface as an
  actionable diagnostic naming the attempted path, never a panic.

### Key Entities *(include if data involved)*

- **Config file**: The single user-editable TOML the tool reads. The wizard creates it; the
  interactive config command rewrites it from gathered values while preserving unsurfaced/passthrough
  content. Must always satisfy load-time validation when written.
- **Agent entry**: A named agent with a launch command, a non-empty model list, and a default model
  that is a member of that list; may carry opaque passthrough keys.
- **Wizard session**: The transient question-and-answer flow that gathers answers and produces a
  config; holds no persisted state of its own and leaves nothing behind if abandoned.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A new user with no prior config can reach a valid, working configuration entirely
  through prompts, without opening an editor or reading the config reference.
- **SC-002**: A new user can complete first-run setup in under 2 minutes and in roughly five prompts
  or fewer for the simplest single-agent setup.
- **SC-003**: 100% of configs produced by the wizard pass the tool's existing load-time validation
  (no wizard output is ever rejected on next load).
- **SC-004**: Cancelling the wizard or the interactive editor at any point never leaves a partial or
  invalid config file, and never corrupts an existing one (verified across cancel/EOF/interrupt).
- **SC-005**: Non-interactive invocations with no config behave exactly as before (printed guidance,
  non-zero exit, no hang) — zero regressions for scripts and the shell prompt hook.
- **SC-006**: An existing user can change a common setting (default agent, add a model, picker,
  streaming toggle) through the interactive command without hand-editing TOML, and opaque
  passthrough keys survive the edit.

## Assumptions

- **First-run trigger**: "First run" means the first time a command that *needs* config is invoked
  while no config file exists. There is no separate persisted "has run before" flag; the presence of
  the config file is the signal. A missing config in an interactive terminal triggers the offer.
- **Interactivity detection**: "Interactive" is determined by stdin being attached to a terminal
  (TTY). When it is not, the tool uses the non-interactive fallback (FR-010). This keeps scripts,
  pipes, CI, and the shell hook safe.
- **Surfacing the command**: The interactive config editor is exposed through the existing `config`
  command (e.g. running it with no sub-flags becomes interactive), while its current non-interactive
  outputs (resolved path, `--schema`) remain reachable via flags. Exact flag/subcommand naming is an
  implementation detail for planning.
- **Editor scope**: The interactive editor covers the commonly changed settings (default agent;
  per-agent command/models/default model; picker; streaming; prompt segment toggle). It is not a
  full TOML editor; advanced or unknown keys are preserved untouched rather than exposed for editing.
- **"Never rewrites" relaxation**: The project previously documented that the tool never rewrites the
  config. This feature intentionally introduces tool-written config (only via explicit wizard/editor
  action, never silently), and the configuration docs will be updated to reflect that the wizard and
  interactive editor write the file.
- **Agent presets**: The wizard may offer known agent presets (e.g. the documented example agents) to
  speed setup, with a "custom" option for anything else. Whether presets are included is left to
  planning; the requirement is only that a basic config can be produced quickly.
- **Single config, no merging**: The feature operates on the one resolved config file; it does not
  introduce layered/multi-file config or environment-variable-based field overrides beyond the
  existing path override.
- **Platform**: Interactive prompts target the same macOS/Linux terminals the tool already supports.

## Dependencies

- The existing config schema, load-time validation rules, and path resolution (config path override
  via `--config` / `SHAP_CONFIG`) are reused as the source of truth for what a valid config is.
- The existing `doctor` validation and agent-command checks remain the place where "is this agent
  actually installed/working" is verified; the wizard need not re-implement that.
