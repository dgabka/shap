#!/usr/bin/env zsh
# shap — thin Zsh integration.
#
# This layer only maps `:` commands to the `shap` binary, forwards the current
# directory, and (later) inserts a generated `git commit` line into the buffer.
# All product logic lives in `shap`; keep this file thin (Constitution VIII).
#
# Install: add to ~/.zshrc
#     source /path/to/shap/shell/zsh/shap.zsh
# Override the binary with `export SHAP_BIN=/path/to/shap` before sourcing.

# Resolve the binary once.
: ${SHAP_BIN:=shap}

if ! command -v "${SHAP_BIN}" >/dev/null 2>&1; then
  print -ru2 -- "shap: '${SHAP_BIN}' not found on PATH; the shell integration is inactive."
  return 0 2>/dev/null || exit 0
fi

# --- colon subcommands -------------------------------------------------------
# Function names may contain ':'. These never collide with the `:` builtin,
# which only matches the bare word `:` (used by the chat widget below).

function :agent     { command "${SHAP_BIN}" agent     "$@" --cwd "${PWD}" }
function :model     { command "${SHAP_BIN}" model     "$@" --cwd "${PWD}" }
function :reasoning { command "${SHAP_BIN}" reasoning "$@" --cwd "${PWD}" }
function :effort    { command "${SHAP_BIN}" reasoning "$@" --cwd "${PWD}" }  # alias
function :new       { command "${SHAP_BIN}" new            --cwd "${PWD}" }
function :status    { command "${SHAP_BIN}" status    "$@" --cwd "${PWD}" }
function :doctor    { command "${SHAP_BIN}" doctor    "$@" --cwd "${PWD}" }
function :run       { command "${SHAP_BIN}" run            --cwd "${PWD}" -- "$@" }
function :read      { command "${SHAP_BIN}" read "$*"      --cwd "${PWD}" }
# `:commit` is handled by the accept-line widget below (it must edit the buffer,
# which a normal function cannot do).

# --- accept-line widget -------------------------------------------------------
# Handles two leading-colon cases without disturbing the `:` builtin:
#   `:commit`  → replace the buffer with the generated `git commit` line for
#                review (NEVER executed automatically — Constitution VII).
#   `: <text>` → run `shap send <text>` (colon-space only, so `: ${x:=y}` and
#                other `:` builtin uses are untouched).
_shap_accept_line() {
  emulate -L zsh
  if [[ ${BUFFER} == ':commit' || ${BUFFER} == ': commit' ]]; then
    local out
    out="$(command "${SHAP_BIN}" commit --prefill-shell-buffer --cwd "${PWD}" 2>&1)"
    if [[ ${out} == git\ commit* ]]; then
      BUFFER=${out}
      CURSOR=${#BUFFER}
      zle redisplay
    else
      zle -M -- "${out:-shap: could not generate a commit message}"
      BUFFER=""
    fi
    return 0
  fi
  if [[ ${BUFFER} == ': '* ]]; then
    local prompt=${BUFFER#: }
    if [[ -n ${prompt//[[:space:]]/} ]]; then
      BUFFER="command ${SHAP_BIN} send ${(q)prompt} --cwd ${(q)PWD}"
    fi
  fi
  zle .accept-line
}
zle -N accept-line _shap_accept_line

# --- prompt segment ----------------------------------------------------------
# A precmd hook caches the segment in ${SHAP_PROMPT_INFO} once per prompt (not
# per redraw). Add ${SHAP_PROMPT_INFO} to your PROMPT or RPROMPT to show it.
# Disable with `export SHAP_PROMPT_SEGMENT=0`. The `prompt-segment` subcommand
# reads only state.json, so this stays cheap.
typeset -g SHAP_PROMPT_INFO=""

_shap_prompt_precmd() {
  if [[ ${SHAP_PROMPT_SEGMENT:-1} == 1 ]]; then
    SHAP_PROMPT_INFO="$(command "${SHAP_BIN}" prompt-segment 2>/dev/null)"
  else
    SHAP_PROMPT_INFO=""
  fi
}

autoload -Uz add-zsh-hook 2>/dev/null && add-zsh-hook precmd _shap_prompt_precmd
