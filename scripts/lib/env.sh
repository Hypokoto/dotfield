#!/usr/bin/env bash
# Dotfield Environment Detection & Helper Utilities
# Shared helper library for dotfield scripts

is_sway() {
    [[ -n "${SWAYSOCK:-}" ]] || [[ "${XDG_CURRENT_DESKTOP:-}" =~ [Ss]way ]] || pgrep -x sway >/dev/null 2>&1
}

is_niri() {
    [[ -n "${NIRI_SOCKET:-}" ]] || [[ "${XDG_CURRENT_DESKTOP:-}" =~ [Nn]iri ]] || pgrep -x niri >/dev/null 2>&1
}

get_amane_bin() {
    command -v amane 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/amane" ]] && echo "${HOME}/.local/bin/amane"
    } || command -v dotfield-shell 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/dotfield-shell" ]] && echo "${HOME}/.local/bin/dotfield-shell"
    } || true
}

is_amane_running() {
    pgrep -x amane >/dev/null 2>&1 || pgrep -x dotfield-shell >/dev/null 2>&1
}
