# Chatterm shell integration for Zsh.

__chatterm_user_zdotdir="${CHATTERM_USER_ZDOTDIR:-$HOME}"
if [[ -f "$__chatterm_user_zdotdir/.zshrc" && "$__chatterm_user_zdotdir/.zshrc" != "$0" ]]; then
  source "$__chatterm_user_zdotdir/.zshrc"
fi

autoload -Uz add-zsh-hook
typeset -g __chatterm_active=0

__chatterm_marker() {
  printf '\033]777;%s\007' "$1"
}

__chatterm_preexec() {
  __chatterm_marker "C;$1"
  __chatterm_active=1
}

__chatterm_precmd() {
  local exit_code=$?
  if (( __chatterm_active )); then
    __chatterm_marker "D;${exit_code}"
    __chatterm_active=0
  fi
  __chatterm_marker "A;${PWD}"
}

add-zsh-hook preexec __chatterm_preexec
add-zsh-hook precmd __chatterm_precmd
PROMPT="${PROMPT:-%n@%m:%~%# }%{$'\033]777;B\007'%}"
