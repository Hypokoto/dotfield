#!/usr/bin/env bash
# Dotfield Dynamic Floating Scratchpad Controller
# Toggles a floating drop-down terminal on demand across workspaces.
# Supports Niri and Sway compositors with instant state recall.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

SCRATCH_APP="scratchpad-term"
SCRATCH_WS="05 SCRATCH"

spawn_scratchpad() {
    if command -v ghostty >/dev/null 2>&1; then
        setsid ghostty --class="${SCRATCH_APP}" </dev/null >/dev/null 2>&1 &
    elif command -v foot >/dev/null 2>&1; then
        setsid foot --app-id="${SCRATCH_APP}" </dev/null >/dev/null 2>&1 &
    elif command -v kitty >/dev/null 2>&1; then
        setsid kitty --class="${SCRATCH_APP}" </dev/null >/dev/null 2>&1 &
    else
        setsid "${SCRIPT_DIR}/terminal-select.sh" </dev/null >/dev/null 2>&1 &
    fi
}

# 1. Niri Compositor Flow
if is_niri && command -v niri >/dev/null 2>&1; then
    WINDOWS_JSON="$(niri msg --json windows 2>/dev/null || true)"

    SCRATCH_ID=""
    if [[ -n "${WINDOWS_JSON}" && "${WINDOWS_JSON}" != "[]" ]]; then
        if command -v jq >/dev/null 2>&1; then
            SCRATCH_ID="$(echo "${WINDOWS_JSON}" | jq -r --arg app "${SCRATCH_APP}" '.[] | select(.app_id == $app) | .id' | head -n 1 || true)"
        else
            SCRATCH_ID="$(echo "${WINDOWS_JSON}" | grep -B2 "\"app_id\":\"${SCRATCH_APP}\"" | grep -o '"id":[0-9]*' | head -n 1 | cut -d':' -f2 || true)"
        fi
    fi

    # If scratchpad is not running, spawn it
    if [[ -z "${SCRATCH_ID}" ]]; then
        spawn_scratchpad
        exit 0
    fi

    # Check if currently focused
    FOCUSED_ID=""
    FOCUSED_JSON="$(niri msg --json focused-window 2>/dev/null || true)"
    if [[ -n "${FOCUSED_JSON}" && "${FOCUSED_JSON}" != "null" ]]; then
        if command -v jq >/dev/null 2>&1; then
            FOCUSED_ID="$(echo "${FOCUSED_JSON}" | jq -r '.id // empty' 2>/dev/null || true)"
        else
            FOCUSED_ID="$(echo "${FOCUSED_JSON}" | grep -o '"id":[0-9]*' | head -n 1 | cut -d':' -f2 || true)"
        fi
    fi

    if [[ "${FOCUSED_ID}" == "${SCRATCH_ID}" ]]; then
        # Already focused -> stash away to scratch workspace
        niri msg action move-window-to-workspace "${SCRATCH_WS}" >/dev/null 2>&1 || true
    else
        # Not focused -> summon to active workspace and focus
        ACTIVE_WS=""
        WS_JSON="$(niri msg --json workspaces 2>/dev/null || true)"
        if [[ -n "${WS_JSON}" ]] && command -v jq >/dev/null 2>&1; then
            ACTIVE_WS="$(echo "${WS_JSON}" | jq -r '.[] | select(.is_focused == true) | (.name // (.idx | tostring))' | head -n 1 || true)"
        fi

        # Focus the window
        niri msg action focus-window --id "${SCRATCH_ID}" >/dev/null 2>&1 || true

        # If we know the active workspace, move it here
        if [[ -n "${ACTIVE_WS}" ]]; then
            niri msg action move-window-to-workspace "${ACTIVE_WS}" >/dev/null 2>&1 || true
            niri msg action focus-window --id "${SCRATCH_ID}" >/dev/null 2>&1 || true
        fi
        niri msg action center-column >/dev/null 2>&1 || true
    fi
    exit 0
fi

# 2. Sway Compositor Flow
if is_sway && command -v swaymsg >/dev/null 2>&1; then
    if ! swaymsg "[app_id=\"${SCRATCH_APP}\"] scratchpad show" >/dev/null 2>&1; then
        spawn_scratchpad
    fi
    exit 0
fi

# Fallback
spawn_scratchpad
