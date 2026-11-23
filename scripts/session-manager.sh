#!/usr/bin/env bash
# Dotfield Niri Session Manager
# Snapshots and restores open windows and workspace layouts across reboots.
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

CACHE_DIR="${XDG_CACHE_HOME:-$HOME/.cache}/dotfield"
SESSION_FILE="${CACHE_DIR}/session.json"
ACTION="${1:-save}"

mkdir -p "${CACHE_DIR}" 2>/dev/null || true

# Helper to map app-id to command
resolve_app_command() {
    local app_id="$1"
    case "${app_id}" in
        com.mitchellh.ghostty|ghostty)
            echo "ghostty"
            ;;
        foot)
            echo "foot"
            ;;
        kitty)
            echo "kitty"
            ;;
        zen-alpha|zen)
            echo "zen-browser"
            ;;
        firefox)
            echo "firefox"
            ;;
        chromium)
            echo "chromium"
            ;;
        thunar)
            echo "thunar"
            ;;
        code)
            echo "code"
            ;;
        pavucontrol)
            echo "pavucontrol"
            ;;
        btop)
            echo "ghostty -e btop"
            ;;
        yazi)
            echo "ghostty -e yazi"
            ;;
        scratchpad-term)
            echo "" # Do not restore scratchpad automatically
            ;;
        *)
            if command -v "${app_id}" >/dev/null 2>&1; then
                echo "${app_id}"
            else
                echo ""
            fi
            ;;
    esac
}

case "${ACTION}" in
    save)
        if ! command -v niri >/dev/null 2>&1; then
            echo "Niri is not available" >&2
            exit 1
        fi

        WINDOWS_RAW="$(niri msg --json windows 2>/dev/null || true)"
        if [[ -z "${WINDOWS_RAW}" || "${WINDOWS_RAW}" == "[]" ]]; then
            echo "[]" > "${SESSION_FILE}"
            safe_notify "dotfield" "low" "Session Manager" "Session is empty (0 windows)" "document-save" "session"
            exit 0
        fi

        # Filter out scratchpad and save snapshot
        if command -v jq >/dev/null 2>&1; then
            SNAPSHOT="$(echo "${WINDOWS_RAW}" | jq '[.[] | select(.app_id != "scratchpad-term" and .app_id != "floating-term") | {id: .id, app_id: .app_id, title: .title, workspace_id: .workspace_id, is_floating: .is_floating}]')"
            echo "${SNAPSHOT}" > "${SESSION_FILE}"
            COUNT="$(echo "${SNAPSHOT}" | jq 'length')"
        else
            echo "${WINDOWS_RAW}" > "${SESSION_FILE}"
            COUNT="several"
        fi

        safe_notify "dotfield" "low" "Session Manager" "Saved ${COUNT} windows to session" "document-save" "session"
        echo "Saved session (${COUNT} windows) to ${SESSION_FILE}"
        ;;

    restore)
        if [[ ! -f "${SESSION_FILE}" ]]; then
            safe_notify "dotfield" "normal" "Session Manager" "No saved session found" "dialog-information" "session"
            exit 0
        fi

        if ! command -v jq >/dev/null 2>&1; then
            echo "jq is required for session restore" >&2
            exit 1
        fi

        TOTAL="$(jq 'length' "${SESSION_FILE}" 2>/dev/null || echo "0")"
        if [[ "${TOTAL}" -eq 0 ]]; then
            safe_notify "dotfield" "low" "Session Manager" "Saved session is empty" "dialog-information" "session"
            exit 0
        fi

        # Read currently running windows to prevent duplicate launches
        CURRENT_APPS=""
        if command -v niri >/dev/null 2>&1; then
            CURRENT_APPS="$(niri msg --json windows 2>/dev/null | jq -r '.[].app_id' || true)"
        fi

        RESTORED=0
        while IFS= read -r item; do
            app_id="$(echo "${item}" | jq -r '.app_id // empty')"
            ws_id="$(echo "${item}" | jq -r '.workspace_id // empty')"

            [[ -z "${app_id}" ]] && continue

            # If already running, skip re-spawning
            if echo "${CURRENT_APPS}" | grep -qx "${app_id}"; then
                continue
            fi

            CMD="$(resolve_app_command "${app_id}")"
            if [[ -n "${CMD}" ]]; then
                # Focus workspace if specified before spawning
                if [[ -n "${ws_id}" ]] && command -v niri >/dev/null 2>&1; then
                    niri msg action focus-workspace "${ws_id}" >/dev/null 2>&1 || true
                fi
                # shellcheck disable=SC2086
                setsid ${CMD} </dev/null >/dev/null 2>&1 &
                RESTORED=$((RESTORED + 1))
                sleep 0.2
            fi
        done < <(jq -c '.[]' "${SESSION_FILE}")

        safe_notify "dotfield" "low" "Session Manager" "Restored session (${RESTORED} applications)" "system-run" "session"
        echo "Restored ${RESTORED} applications from session"
        ;;

    clear)
        rm -f "${SESSION_FILE}"
        safe_notify "dotfield" "low" "Session Manager" "Cleared saved session" "edit-delete" "session"
        echo "Cleared ${SESSION_FILE}"
        ;;

    *)
        echo "Usage: $0 {save|restore|clear}" >&2
        exit 1
        ;;
esac
