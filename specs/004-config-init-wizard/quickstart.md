# Quickstart: Config Init Wizard & Interactive Config Editing

Build + manual verification of the two flows. Uses isolated config/data paths so your real config is
untouched. No automated harness beyond the unit/`assert_cmd` tests added with the feature.

## Build

```sh
cargo build
cargo nextest run -p shap-core config        # wizard builder / serializer / validate round-trip
cargo test -p shap-cli                        # assert_cmd: non-interactive fallback
```

## Scratch environment

```sh
export SHAP_CONFIG=/tmp/shap-wizard/config.toml
export SHAP_DATA_DIR=/tmp/shap-wizard/data
rm -rf /tmp/shap-wizard && mkdir -p /tmp/shap-wizard/data
BIN=./target/debug/shap
```

## Scenario A — First-run wizard (US1, FR-001..005)

Interactive terminal, no config yet:

```sh
$BIN status            # triggers the wizard offer
```

Expected:

1. Prompt: "No config found. Set one up now?" → accept.
2. Pick a preset (e.g. `claude`) or `custom`.
3. Enter / confirm models, pick a default model, accept UI defaults.
4. Confirm the shown summary → file is written.
5. The original `status` command then runs.

Verify:

```sh
test -f "$SHAP_CONFIG" && echo "config written"
$BIN doctor            # the written config validates
$BIN config path       # prints $SHAP_CONFIG
```

**Cancel check** (FR-005, SC-004): remove the file, run `$BIN status`, accept the offer, then press
Ctrl-C / Esc mid-wizard. Expect: no file created, setup guidance printed, non-zero exit.

```sh
rm -f "$SHAP_CONFIG"
$BIN status            # accept, then cancel partway
test -f "$SHAP_CONFIG" || echo "no partial file (correct)"
```

## Scenario B — Non-interactive fallback (US3, FR-010/011, SC-005)

Stdin not a TTY → no prompt, today's behavior:

```sh
rm -f "$SHAP_CONFIG"
echo "" | $BIN send "hello"      # piped stdin → not a TTY
echo "exit=$?"                   # expect non-zero, printed setup instructions, no hang
$BIN prompt-segment              # must stay silent/cheap, never wizard
```

Expect: the `ConfigNotFound` diagnostic, non-zero exit, no prompt, no file created.

## Scenario C — Interactive editor (US2, FR-006..009)

Start from a valid config (from Scenario A). Add a passthrough key by hand first to prove
preservation (FR-008):

```sh
printf '\n[agents.claude]\napi_key_env = "ANTHROPIC_API_KEY"\n' >> "$SHAP_CONFIG"  # if not present
$BIN config edit
```

In the editor: change the default agent (or add a model), then Save.

Verify:

```sh
grep api_key_env "$SHAP_CONFIG" && echo "passthrough preserved"   # FR-008
$BIN doctor                                                        # still valid
```

**Invalid-change check** (FR-007): try to set a `default_model` not in an agent's `models` (if the
editor allows constructing it) or remove all models — expect rejection with a diagnostic and the
existing file unchanged.

**No-op check** (FR-009): run `$BIN config edit` and cancel immediately; confirm the file's mtime /
contents are unchanged.

## Backward-compatibility checks (FR-012, INV-5)

```sh
$BIN config            # non-TTY (e.g. piped) → prints path, NOT interactive
$BIN config --schema   # prints JSON schema unchanged
```

## Acceptance mapping

| Check | Spec |
|-------|------|
| Wizard writes valid config, command continues | US1 AS1/AS4, SC-001/003 |
| Cancel leaves no partial file | US1 AS2, FR-005, SC-004 |
| Existing config ⇒ no wizard | US1 AS3 |
| Piped stdin ⇒ instructions + non-zero, no hang | US3 AS1, FR-010, SC-005 |
| prompt-segment never triggers wizard | US3 AS2, FR-011 |
| Editor changes default agent / adds model | US2 AS1, SC-006 |
| Invalid edit rejected, file preserved | US2 AS2, FR-007 |
| No-op/cancel leaves file unchanged | US2 AS3, FR-009 |
| Passthrough keys survive edit | US2 AS4, FR-008 |
| `config` (non-TTY) / `--schema` unchanged | FR-012, INV-5 |
```
