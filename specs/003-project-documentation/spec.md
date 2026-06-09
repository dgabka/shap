# Feature Specification: Project Documentation

**Feature Branch**: `003-project-documentation`

**Created**: 2026-06-09

**Status**: Draft

**Input**: User description: "README, docs, how to's, guide: what it is and what it does, installation, usage, etc"

## Clarifications

### Session 2026-06-09

- Q: Which installation methods must the install guide document? → A: Cargo build from source and the Nix flake only; prebuilt release binaries are out of scope for the docs this iteration.
- Q: How self-contained should the README (front page) be? → A: Overview + links — identity, capabilities, quick install, one minimal usage example, then links into `docs/` for depth (no duplicated reference content).
- Q: Which agent should the getting-started walkthrough use as its primary worked example? → A: Claude Code as the concrete example agent, with a note that any ACP-compatible agent works.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Understand what the project is in under a minute (Priority: P1)

A developer who has never heard of the project lands on the repository's front page. Within the
first screen they learn what the tool is, the problem it solves, and whether it is relevant to
them — without reading source code or scrolling through design specs.

**Why this priority**: The front page is the single highest-traffic surface. If a newcomer cannot
tell what the project does in the first screen, nothing else in the documentation gets read. This
is the minimum viable documentation: a project with only this still gains adoption.

**Independent Test**: Show the repository front page to someone unfamiliar with the project and
confirm they can correctly state, in their own words, what the tool is and the core problem it
solves after reading only the top section.

**Acceptance Scenarios**:

1. **Given** a reader opens the repository front page, **When** they read the opening section,
   **Then** they can state what the tool is and the primary problem it solves.
2. **Given** a reader is on the front page, **When** they look for the value proposition,
   **Then** a short list of the tool's core capabilities is visible without scrolling past the
   first screen.
3. **Given** a reader wants to go deeper, **When** they reach the end of the front page,
   **Then** clearly labeled links point to installation, usage, and topic guides.

---

### User Story 2 - Install the tool from scratch (Priority: P1)

A developer decides to try the tool. They follow the installation instructions on a clean machine
and end up with a working command available in their terminal, using whichever supported method
fits their environment.

**Why this priority**: Documentation that explains the product but does not get a user to a working
install produces no adopters. Installation is the first irreversible commitment a user makes and
must succeed without guesswork.

**Independent Test**: On a clean environment, follow only the documented installation steps and
confirm the tool's command runs and reports its version or help output.

**Acceptance Scenarios**:

1. **Given** a clean environment, **When** the user follows the documented prerequisites and
   install steps for their chosen method, **Then** the tool's command is available and runs.
2. **Given** the two documented install methods (cargo build from source and the Nix flake), **When**
   the user reads the install section, **Then** each is listed with its prerequisites and the
   trade-offs between them.
3. **Given** the optional shell integration exists, **When** the user follows its setup steps,
   **Then** the integrated commands work and the docs state that the tool is fully usable without
   the integration.
4. **Given** an install step fails, **When** the user consults the docs, **Then** a troubleshooting
   note or self-check command is referenced to diagnose the problem.

---

### User Story 3 - Accomplish the core task by following usage docs (Priority: P1)

A newly-installed user follows the usage guide to perform the tool's primary task end to end —
from first-run setup through producing the expected result — without reading source code.

**Why this priority**: Installation without a successful first use still produces an abandoned tool.
The usage walkthrough converts an install into an active user and is the payoff of the whole
document set.

**Independent Test**: A user who has only installed the tool follows the usage walkthrough and
completes the primary task successfully, producing the documented expected output.

**Acceptance Scenarios**:

1. **Given** a freshly installed tool, **When** the user follows the getting-started walkthrough
   (worked through with Claude Code as the example agent), **Then** they complete first-run setup
   and perform the primary task successfully.
2. **Given** the tool exposes multiple commands, **When** the user consults the usage docs,
   **Then** each command is documented with its purpose, arguments, and at least one example.
3. **Given** the tool requires configuration, **When** the user reads the usage or configuration
   docs, **Then** required and optional settings are described with their defaults and effects.
4. **Given** a command can be invoked two equivalent ways, **When** the user reads the docs,
   **Then** both forms are shown and noted as equivalent.

---

### User Story 4 - Navigate to topic guides and how-tos (Priority: P2)

A user with a specific question (configuration, a particular integration, an environment setup)
finds a focused topic guide that answers it, reached from a discoverable index rather than by
searching source files.

**Why this priority**: Deep topic guides retain and grow existing users, but the project is already
usable for newcomers once the front page, install, and usage docs (P1) exist. Topic guides build on
that base.

**Independent Test**: From the documentation index, a user with a specific question locates the
relevant topic guide via a labeled link and finds their question answered there.

**Acceptance Scenarios**:

1. **Given** the documentation set has multiple topic guides, **When** the user opens the docs
   index, **Then** each guide is listed with a one-line description of what it covers.
2. **Given** a user has a question covered by a topic guide, **When** they follow the index link,
   **Then** they reach the guide that answers it.
3. **Given** topic guides exist for distinct concerns, **When** the user reads any one guide,
   **Then** it is self-contained for its topic and cross-links related guides where relevant.

---

### User Story 5 - Contribute and understand project status (Priority: P3)

A potential contributor or evaluator finds the project's license, supported platforms, maturity
status, and where to report issues or propose changes.

**Why this priority**: This widens the project to contributors and sets expectations, but adoption
and use (P1) and retention (P2) do not depend on it. It is valuable polish rather than a blocker.

**Independent Test**: A reader can determine the project's license, supported platforms, current
maturity, and where to file an issue from the documentation alone.

**Acceptance Scenarios**:

1. **Given** a reader evaluates the project, **When** they consult the docs, **Then** the license
   and supported platforms are stated.
2. **Given** a reader wants to contribute or report a problem, **When** they consult the docs,
   **Then** the channel for issues and contributions is named.
3. **Given** the project is pre-1.0, **When** a reader checks status, **Then** the maturity level
   and any stability caveats are stated honestly.

---

### Edge Cases

- What does a reader see when a documented command or flag has been renamed or removed — how is the
  documentation kept from describing behavior the tool no longer has?
- How does the install guide handle a user whose environment is missing a prerequisite (e.g., the
  required toolchain or an external dependency)?
- What does a user encounter if they follow the optional integration setup but the core command is
  not on their PATH?
- How does the documentation serve a user who lands directly on a deep topic guide via a search
  engine, without passing through the front page first?
- How is documentation that references version-specific behavior handled when the version changes?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The project MUST provide a front-page document at the repository root that states
  what the tool is and the primary problem it solves within its opening section.
- **FR-002**: The front page MUST list the tool's core capabilities and link to installation,
  usage, and the topic-guide index.
- **FR-002a**: The front page MUST stay overview-scoped — identity, capabilities, a quick install,
  and a single minimal usage example — and MUST link into `docs/` for depth rather than duplicating
  the full command reference or topic-guide content inline.
- **FR-003**: The documentation MUST describe the two in-scope installation methods — building from
  source with cargo and installing via the Nix flake — each with its prerequisites and the
  trade-offs between them. Prebuilt release binaries are out of scope for this documentation
  iteration.
- **FR-004**: The documentation MUST provide a getting-started walkthrough that takes a user from a
  fresh install through first-run setup to completing the tool's primary task, using Claude Code as
  the concrete example agent and noting that any ACP-compatible agent works.
- **FR-005**: The documentation MUST document every user-facing command with its purpose, accepted
  arguments, and at least one usage example.
- **FR-006**: The documentation MUST describe required and optional configuration, including each
  setting's default and effect.
- **FR-007**: The documentation MUST describe the optional shell integration's setup and explicitly
  state that the tool is fully usable without it.
- **FR-008**: Where a command can be invoked through more than one equivalent surface, the
  documentation MUST present each form and note them as equivalent.
- **FR-009**: The documentation MUST provide a discoverable index of topic guides, each entry
  carrying a one-line description of what the guide covers.
- **FR-010**: Each topic guide MUST be self-contained for its topic and cross-link related guides
  where relevant.
- **FR-011**: The documentation MUST reference a troubleshooting path (including any built-in
  self-check command) for diagnosing failed installation or setup.
- **FR-012**: The documentation MUST state the project's license, supported platforms, current
  maturity status, and the channel for reporting issues or contributing.
- **FR-013**: Documentation examples MUST reflect the tool's actual current commands, arguments,
  and outputs at the time of writing, with no references to removed or renamed behavior.
- **FR-014**: The documentation MUST be navigable such that a reader can reach any major topic
  (install, usage, configuration, integration, troubleshooting) from the front page within a small
  number of labeled links.

### Key Entities *(include if feature involves data)*

- **Front-page document**: The repository's primary entry document. Conveys identity, value
  proposition, core capabilities, and navigation to all other documentation.
- **Installation guide**: The section or document covering prerequisites, supported install
  methods, and verification of a working install.
- **Usage / getting-started guide**: The walkthrough and command reference that takes a user from
  install to completing the primary task and beyond.
- **Topic guide**: A focused document covering one concern (e.g., configuration, an integration,
  an environment setup), reached from the documentation index.
- **Documentation index**: The catalog that lists topic guides with one-line descriptions and links.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reader unfamiliar with the project can correctly describe what the tool does after
  reading only the front page's opening section.
- **SC-002**: A new user can go from a clean environment to a working, runnable command by
  following the installation guide, with no steps that require reading source code.
- **SC-003**: A new user can complete the tool's primary task end to end by following the
  getting-started walkthrough on the first attempt.
- **SC-004**: 100% of user-facing commands have an entry documenting purpose, arguments, and at
  least one example.
- **SC-005**: Every topic guide is reachable from the documentation index, and every index entry
  links to an existing guide (no broken or missing links).
- **SC-006**: A reader can determine the project's license, supported platforms, maturity, and
  issue-reporting channel from the documentation alone.
- **SC-007**: Every command, flag, and output shown in the documentation matches the tool's actual
  behavior at publication time.

## Assumptions

- The audience is developers comfortable with a terminal; the docs assume command-line familiarity
  but not prior knowledge of this specific tool.
- Documentation is authored in Markdown and lives in the repository (root front page plus a `docs/`
  directory), consistent with the project's existing documentation.
- Several topic guides already exist (configuration, the shell integration, the environment/build
  setup, and agent setup); this feature organizes, indexes, and completes them and adds the missing
  front page and getting-started walkthrough rather than starting from nothing.
- The front page is the README rendered by the repository host, kept overview-scoped with links into
  `docs/` rather than duplicating reference content (see Clarifications).
- Documented install methods are cargo-from-source and the Nix flake; prebuilt release binaries,
  though produced by CI, are not documented this iteration (see Clarifications).
- The getting-started walkthrough features Claude Code as its example agent (see Clarifications).
- Documentation is English-only for this iteration; localization is out of scope.
- A hosted documentation website, API reference generation, and screenshots/video are out of scope
  for this iteration; the deliverable is in-repository Markdown.
- The tool remains pre-1.0, so docs state stability caveats rather than guaranteeing a frozen
  interface.
