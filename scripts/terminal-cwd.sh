#!/usr/bin/env bash
# Dotfield Terminal CWD Launcher
# Spawns a new terminal window in the working directory of the currently focused window.
# Supports Niri compositor window inspection and fallback to HOME.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

resolve_focused_cwd() {
    if ! command -v niri >/dev/null 2>&1; then
        echo "${HOME}"
        return 0
    fi

    local focused_json
    focused_json="$(niri msg --json focused-window 2>/dev/null || true)"
    if [[ -z "${focused_json}" || "${focused_json}" == "null" ]]; then
        echo "${HOME}"
        return 0
    fi

    # Extract pid using basic sed/grep or jq if present
    local pid
    if command -v jq >/dev/null 2>&1; then
        pid="$(echo "${focused_json}" | jq -r '.pid // empty' 2>/dev/null || true)"
    else
        pid="$(echo "${focused_json}" | grep -o '"pid":[0-9]*' | head -n 1 | cut -d':' -f2 || true)"
    fi

    if [[ -z "${pid}" || ! -d "/proc/${pid}" ]]; then
        echo "${HOME}"
        return 0
    fi

    # Traverse child processes to find deepest active foreground shell / editor
    local current_pid="${pid}"
    local target_pid="${pid}"
    while true; do
        local child
        child="$(pgrep -P "${current_pid}" 2>/dev/null | tail -n 1 || true)"
        if [[ -n "${child}" && -d "/proc/${child}/cwd" ]]; then
            current_pid="${child}"
            target_pid="${child}"
        else
            break
        fi
    done

    local resolved_cwd=""
    if [[ -d "/proc/${target_pid}/cwd" ]]; then
        resolved_cwd="$(readlink -f "/proc/${target_pid}/cwd" 2>/dev/null || true)"
    elif [[ -d "/proc/${pid}/cwd" ]]; then
        resolved_cwd="$(readlink -f "/proc/${pid}/cwd" 2>/dev/null || true)"
    fi

    if [[ -n "${resolved_cwd}" && -d "${resolved_cwd}" ]]; then
        echo "${resolved_cwd}"
    else
        echo "${HOME}"
    fi
}

TARGET_CWD="$(resolve_focused_cwd)"
[[ -d "${TARGET_CWD}" ]] || TARGET_CWD="${HOME}"

if [[ "${1:-}" == "--print-cwd" || "${1:-}" == "-p" ]]; then
    echo "${TARGET_CWD}"
    exit 0
fi

# Determine preferred terminal
TARGET_BIN=""
if command -v ghostty >/dev/null 2>&1; then
    TARGET_BIN="ghostty"
elif command -v foot >/dev/null 2>&1; then
    TARGET_BIN="foot"
elif command -v kitty >/dev/null 2>&1; then
    TARGET_BIN="kitty"
elif [[ -n "${TERMINAL:-}" ]] && command -v "${TERMINAL}" >/dev/null 2>&1; then
    TARGET_BIN="${TERMINAL}"
elif command -v xterm >/dev/null 2>&1; then
    TARGET_BIN="xterm"
fi

if [[ -z "${TARGET_BIN}" ]]; then
    echo "No suitable terminal emulator found" >&2
    exit 1
fi

# Launch terminal inside resolved directory
cd "${TARGET_CWD}"
case "${TARGET_BIN}" in
    ghostty)
        setsid ghostty --working-directory="${TARGET_CWD}" </dev/null >/dev/null 2>&1 &
        ;;
    foot)
        setsid foot -D "${TARGET_CWD}" </dev/null >/dev/null 2>&1 &
        ;;
    kitty)
        setsid kitty -d "${TARGET_CWD}" </dev/null >/dev/null 2>&1 &
        ;;
    *)
        setsid "${TARGET_BIN}" </dev/null >/dev/null 2>&1 &
        ;;
esac
exit 0
