#!/usr/bin/env bash
# Dotfield Niri & Neovim Directional Navigation Bridge
# Moves focus seamlessly between Neovim splits and Niri window columns.
# Uses RPC communication when in Neovim; falls back to Niri column focus otherwise.

set -euo pipefail

DIR="${1:-left}"
CUSTOM_MOD="${2:-}"

case "${DIR}" in
    up|down|left|right) ;;
    *)
        echo "Usage: $0 {up|down|left|right} [w|m]" >&2
        exit 1
        ;;
esac

CUSTOM_ACTION=""
if [[ "${CUSTOM_MOD}" == "w" ]]; then
    CUSTOM_ACTION="or-workspace-"
elif [[ "${CUSTOM_MOD}" == "m" ]]; then
    CUSTOM_ACTION="or-monitor-"
fi

TIMEOUT="${VIM_NIRI_NAV_TIMEOUT:-0.1s}"

get_descendant_vim_pid() {
    local pid="$1"
    if [[ ! -d "/proc/${pid}" ]]; then
        return 1
    fi

    local comm
    comm="$(cat "/proc/${pid}/comm" 2>/dev/null || true)"
    if echo "${comm}" | grep -iqE '^g?(view|n?vim?x?)(diff)?$'; then
        echo "${pid}"
        return 0
    fi

    local child
    for child in $(pgrep -P "${pid}" 2>/dev/null || true); do
        if get_descendant_vim_pid "${child}"; then
            return 0
        fi
    done
    return 1
}

with_timeout() {
    if [[ -n "${TIMEOUT}" && "${TIMEOUT}" != "0" ]] && command -v timeout >/dev/null 2>&1; then
        timeout "${TIMEOUT}" "$@"
    else
        "$@"
    fi
}

vim_remote_expr() {
    local program="$1"
    local sname="$2"
    local expr="$3"

    if [[ "${program}" == "nvim" ]] && command -v nvim >/dev/null 2>&1; then
        with_timeout nvim --headless --server "${sname}" --remote-expr "${expr}" 2>/dev/null || true
    elif [[ "${program}" == "vim" ]] && command -v vim >/dev/null 2>&1; then
        with_timeout vim --servername "${sname}" --remote-expr "${expr}" 2>/dev/null || true
    fi
}

# 1. Check if active window is running Neovim/Vim
if command -v niri >/dev/null 2>&1; then
    FOCUSED_JSON="$(niri msg --json focused-window 2>/dev/null || true)"
    if [[ -n "${FOCUSED_JSON}" && "${FOCUSED_JSON}" != "null" ]]; then
        FOCUSED_PID=""
        if command -v jq >/dev/null 2>&1; then
            FOCUSED_PID="$(echo "${FOCUSED_JSON}" | jq -r '.pid // empty' 2>/dev/null || true)"
        fi

        if [[ -n "${FOCUSED_PID}" ]]; then
            VIM_PID="$(get_descendant_vim_pid "${FOCUSED_PID}" || true)"
            if [[ -n "${VIM_PID}" ]]; then
                SERVERNAME_FILE="${XDG_RUNTIME_DIR:-/tmp}/vim-niri-nav.${VIM_PID}.servername"
                if [[ -f "${SERVERNAME_FILE}" ]]; then
                    read -r PROG SNAME < "${SERVERNAME_FILE}" || true
                    if [[ -n "${SNAME:-}" ]]; then
                        NAV_RESULT="$(vim_remote_expr "${PROG}" "${SNAME}" "v:lua.require('configs.niri_nav').navigate('${DIR}')")"
                        if [[ "${NAV_RESULT}" == "true" ]]; then
                            exit 0
                        fi
                    fi
                fi
            fi
        fi
    fi

    # 2. Outside Vim or at edge: Dispatch Niri navigation
    case "${DIR}" in
        up|down)
            niri msg action "focus-window-${CUSTOM_ACTION}${DIR}" >/dev/null 2>&1 || true
            ;;
        left|right)
            niri msg action "focus-column-${DIR}" >/dev/null 2>&1 || true
            ;;
    esac
    exit 0
fi

# Fallback for non-Niri environments
exit 0
