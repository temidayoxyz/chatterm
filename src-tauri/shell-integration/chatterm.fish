# Chatterm shell integration for Fish.

function __chatterm_marker
    printf '\e]777;%s\a' "$argv[1]"
end

function __chatterm_preexec --on-event fish_preexec
    __chatterm_marker "C;$argv"
end

function __chatterm_postexec --on-event fish_postexec
    set -l exit_code $status
    __chatterm_marker "D;$exit_code"
    __chatterm_marker "A;$PWD"
end

__chatterm_marker "A;$PWD"
