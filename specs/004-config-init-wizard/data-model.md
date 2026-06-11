# Phase 1 Data Model: Config Init Wizard & Interactive Config Editing

This feature adds **no new persisted data shape** — it writes the existing `Config` type
(`crates/shap-core/src/config.rs`). The model below covers the transient in-memory structures the
wizard/editor use and the validation/write flow.

## Existing types reused (source of truth)

- **`Config`** — `default_agent: String`, `agents: BTreeMap<String, Agent>`, `ui: UiOptions`,
  `history: HistoryOptions`, `files: FileOptions`. Already derives `Serialize` (enables writing).
- **`Agent`** — `command: String`, `models: Vec<String>`, `default_model: String`,
  `extra: toml::Table` (`#[serde(flatten)]`, opaque passthrough — preserved automatically on
  round-trip).
- **`UiOptions`** — `stream: bool`, `picker: Picker (Fzf|Skim|Builtin)`, `show_prompt_segment: bool`.
- **`HistoryOptions`** / **`FileOptions`** — byte limits + flags (defaults applied; surfaced only if
  the editor chooses to expose them — see scope below).
- **`Config::validate()`** — the single authority for validity (see Validation rules). The wizard and
  editor MUST call it before writing.

## New transient types (no persistence)

### `WizardDraft`

A mutable, partially-built representation gathered from prompts before it becomes a `Config`.

| Field | Type | Source prompt |
|-------|------|---------------|
| `agent_name` | `String` | preset choice or custom `Input` |
| `command` | `String` | preset default or custom `Input` |
| `models` | `Vec<String>` | `Input` (comma/space list) or repeated entries; non-empty enforced |
| `default_model` | `String` | `Select` over `models` |
| `ui_*` | accept-defaults `Confirm` | optional; defaults from `UiOptions::default()` |

- **Builder**: `WizardDraft::into_config(self) -> Config` — pure, no I/O. Constructs a single-agent
  `Config` with `default_agent = agent_name`, empty `extra`, and default `ui`/`history`/`files`
  (unless surfaced). Unit-testable.
- **Lifecycle**: exists only during the prompt flow; dropped on cancel, leaving nothing on disk.

### `EditAction` (interactive editor)

The editor loads the current `Config`, presents a top-level `Select` menu, applies one change to the
in-memory `Config`, then re-prompts or saves. Conceptual actions (not necessarily a literal enum):

| Action | Effect on `Config` |
|--------|--------------------|
| Change default agent | `Select` over `agents.keys()` → set `default_agent` |
| Add model to agent | pick agent → `Input` model → push to `agents[a].models` |
| Set default model | pick agent → `Select` over its `models` → set `default_model` |
| Add agent | sub-flow like the wizard's single-agent path → insert into `agents` |
| Change picker | `Select` Fzf/Skim/Builtin → `ui.picker` |
| Toggle streaming | `Confirm` → `ui.stream` |
| Toggle prompt segment | `Confirm` → `ui.show_prompt_segment` |
| Save & exit | validate → atomic write |
| Cancel | discard in-memory changes; file untouched |

- The editor mutates a **clone** of the loaded `Config`; the original on disk is replaced only on a
  successful validated write (FR-007, FR-009).
- `agents[a].extra` is never touched by the editor, so passthrough keys survive (FR-008), and
  `toml::to_string_pretty` re-emits them from the flattened table.

## Validation rules (unchanged authority — `Config::validate()`)

Applied to the constructed/edited `Config` **before** any write:

1. `agents` non-empty (`NoAgentConfigured`).
2. `default_agent` ∈ `agents` (`UnknownDefaultAgent`).
3. each agent `models` non-empty (`AgentEmptyModels`).
4. each agent `default_model` ∈ its `models` (`DefaultModelNotInModels`).
5. `history.max_output_bytes > 0` and `files.max_file_bytes > 0` (`NonPositiveByteLimit`).
6. `picker` is a valid enum value (enforced at parse/serialize; the editor only offers valid choices).

The wizard's prompt flow is designed so a completed draft always satisfies 1–5, but `validate()` is
still called as the gate (defense in depth, FR-004/FR-007). If validation fails in the editor, the
change is rejected with the diagnostic and the prior file is preserved.

## Write flow (new)

```text
build/edit Config (in memory)
        │
        ▼
Config::validate()  ──fail──▶  show diagnostic, do NOT write (preserve existing file)
        │ ok
        ▼
toml::to_string_pretty(&config)
        │
        ▼
atomic write:  create_dir_all(parent)
               write temp file in same dir
               fs::rename(temp, config_path)     ──io err──▶  Error::ConfigWriteFailed { path, source }
        │ ok
        ▼
(first-run) re-load via Config::load → continue original command
(editor)   report success
```

- **Atomicity**: temp-file-in-same-dir + `rename`, identical to `ActiveState::save`
  (`state.rs:45-56`). Guarantees no partial/torn config (FR-005, SC-004).
- **Formatting note**: serialization emits canonical TOML; user comments and original key order are
  **not** preserved (documented limitation, research D5). Passthrough *values* are preserved.

## State transitions (config file lifecycle)

```text
            (no file)
               │  first command needs config
               ▼
   stdin TTY? ──no──▶ print setup instructions, exit non-zero   [FR-010]  (file stays absent)
        │ yes
        ▼
   offer wizard ──decline/cancel──▶ print guidance, exit non-zero (file stays absent)  [FR-005]
        │ accept + complete
        ▼
   validate + atomic write ──▶  (valid file exists) ──▶ continue command
               │
               │  later: `shap config edit`
               ▼
   load → edit clone → validate ──fail──▶ keep existing file   [FR-007]
               │ ok + save
               ▼
   atomic overwrite (passthrough preserved)   [FR-008]
               │  cancel / no change
               ▼
   existing file unchanged   [FR-009]
```
