# Chatterm shell integration for Bash.
# This wrapper preserves the user's normal Bash configuration, then adds
# non-printing prompt markers used to build command threads.

if [[ -f "$HOME/.bashrc" && "$HOME/.bashrc" != "${BASH_SOURCE[0]}" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.bashrc"
fi

__chatterm_active=0

__chatterm_marker() {
  printf '\033]777;%s\007' "$1"
}

__chatterm_prompt_command() {
  local exit_code=$?
  if [[ $__chatterm_active -eq 1 ]]; then
    __chatterm_marker "D;${exit_code}"
  fi
  __chatterm_active=1
  __chatterm_marker "A;${PWD}"
}

if declare -p PROMPT_COMMAND 2>/dev/null | grep -q 'declare -a'; then
  PROMPT_COMMAND=(__chatterm_prompt_command "${PROMPT_COMMAND[@]}")
elif [[ -n "${PROMPT_COMMAND:-}" ]]; then
  PROMPT_COMMAND="__chatterm_prompt_command;${PROMPT_COMMAND}"
else
  PROMPT_COMMAND="__chatterm_prompt_command"
fi

PS1="${PS1:-\\u@\\h:\\w\\$ }\\[\\033]777;B\\007\\]"
