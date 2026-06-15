# Feature Specification: Colon-Command Syntax Highlighting

**Feature Branch**: `005-colon-command-highlighting`

**Created**: 2026-06-15

**Status**: Draft

**Input**: User description: "syntax highlights for shell commands, now `:commit` command show as invalid (red)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - `:commit` is recognized as a valid command (Priority: P1)

A user has the shap zsh integration active and uses a shell that highlights command
words as they type (highlighting an unrecognized command in red). They type `:commit`
to generate a commit message. Today the word `:commit` is rendered red — the same way
a typo or a non-existent command would be — which makes the user think the feature is
broken or misspelled, even though pressing Enter still works. After this feature, typing
`:commit` is highlighted the same way the other working shap commands are (recognized /
valid), and its behavior is unchanged: pressing Enter still replaces the line with the
generated `git commit …` command for review and never executes it automatically.

**Why this priority**: This is the reported defect. The red highlighting actively signals
"this command is wrong" to the user, undermining trust in the only colon command that is
not already highlighted correctly. Fixing it restores a consistent, non-alarming
experience for the core commit flow.

**Independent Test**: In a zsh session with the integration sourced and a command-word
highlighter enabled, type `:commit` (without pressing Enter) and confirm the word is no
longer styled as an invalid/unknown command. Then press Enter and confirm the buffer is
still replaced with the generated `git commit` line and nothing is executed automatically.

**Acceptance Scenarios**:

1. **Given** the integration is active and a command-word highlighter is enabled, **When** the user types `:commit`, **Then** `:commit` is styled as a recognized command (not the unknown-command/error style).
2. **Given** the user has typed `:commit`, **When** they press Enter, **Then** the line is replaced with the generated `git commit …` command for review and is not executed automatically.
3. **Given** the integration is active, **When** the user types the bare `:` builtin or a `: <text>` chat line, **Then** highlighting and behavior are unchanged from today.

---

### User Story 2 - All shap colon commands highlight consistently (Priority: P2)

A user types any of the shap colon commands (`:agent`, `:model`, `:reasoning`, `:effort`,
`:new`, `:status`, `:doctor`, `:run`, `:read`, `:commit`). Every one of them is highlighted
the same way — as a recognized command — so the command surface looks uniform and trustworthy.

**Why this priority**: Consistency prevents the same confusion from reappearing for any
command that is wired up through a non-function mechanism, and confirms the fix is not a
one-off special case. Lower priority because all commands except `:commit` already render
correctly today.

**Independent Test**: With the integration active and a highlighter enabled, type each
documented colon command and confirm none is styled as an invalid/unknown command.

**Acceptance Scenarios**:

1. **Given** the integration is active, **When** the user types any documented shap colon command, **Then** none of them is rendered in the invalid/unknown-command style.

---

### Edge Cases

- **No highlighter installed**: A user with no command-word highlighter sees no visual change; command behavior is identical to today.
- **`:commit` with trailing text** (e.g. `:commit foo`): Highlighting recognition of the leading `:commit` word must not change the existing execution behavior, which only special-cases the exact `:commit` / `: commit` buffers.
- **Bare `:` and `: <text>` chat line**: The zsh `:` builtin and the colon-space chat path must remain untouched — neither their highlighting nor their behavior may regress.
- **Different highlighter implementations**: Behavior should be correct for the common case where the highlighter classifies a word by whether the shell can resolve it as a runnable command.
- **Integration not active** (binary missing / not sourced): No new behavior or highlighting is introduced.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When the shap zsh integration is active, `:commit` MUST be resolvable by the shell as a recognized command word so that command-word highlighters render it as valid rather than as an unknown/invalid command.
- **FR-002**: The behavior of `:commit` (and `: commit`) MUST remain exactly as today: pressing Enter replaces the current line with the generated `git commit …` command for the user to review, and the commit is NEVER executed automatically.
- **FR-003**: All documented shap colon commands MUST be highlighted consistently as recognized commands when the integration is active.
- **FR-004**: The bare `:` zsh builtin and the `: <text>` chat path MUST retain their current highlighting and behavior with no regression.
- **FR-005**: The shell integration layer MUST remain thin — recognition is achieved without moving product logic into the shell or duplicating command behavior already owned by the `shap` binary.
- **FR-006**: When no command-word highlighter is present, or the integration is not active, there MUST be no change to command behavior.
- **FR-007**: The fix MUST NOT introduce a second, separately-executable code path for `:commit`; the existing line-rewriting mechanism remains the single source of `:commit` behavior.

### Key Entities

- **Shap colon command**: A `:`-prefixed word typed at the shell prompt (e.g. `:commit`, `:agent`) that the integration maps to the `shap` binary or to a buffer-editing action.
- **Command-word highlighter**: The user's shell highlighting mechanism that classifies the first word of a command line as recognized or unknown and colors it accordingly.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a shell session with the integration active and a command-word highlighter enabled, `:commit` is no longer displayed in the invalid/unknown-command style (0 occurrences of the red/error styling for `:commit`).
- **SC-002**: 100% of the documented shap colon commands render in the recognized-command style under the same conditions.
- **SC-003**: `:commit` continues to produce the generated `git commit` line for review on Enter, and in 100% of cases performs no automatic commit.
- **SC-004**: The bare `:` builtin and the `: <text>` chat path show no change in behavior or highlighting compared to before the change (0 regressions).

## Assumptions

- The reported "red" rendering comes from a zsh command-word highlighter (e.g. zsh-syntax-highlighting / fast-syntax-highlighting) that flags words the shell cannot resolve as a runnable command. The other colon commands appear valid because they are defined as shell functions, whereas `:commit` is currently handled only by the accept-line widget with no corresponding resolvable command word.
- Zsh is the only shell integration in scope (the project currently ships only a zsh integration).
- "Recognized command" styling means the highlighter's normal command color, not necessarily a bespoke color; matching the other colon commands is sufficient.
- The existing constitutional rules apply: the shell layer stays thin (Constitution VIII) and `:commit` never auto-executes a commit (Constitution VII).
- Supporting the highlighter's command-vs-unknown classification covers the common highlighters; pixel-exact styling per third-party theme is out of scope.
