#!/usr/bin/env bash
# Dotfield Run-or-Raise (RoR) Application Switcher & Spawner
# Focuses an already open application window or launches a new instance.
# Supports multi-instance cycling across workspaces in Niri and Sway compositors.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

TARGET_APP="${1:-}"
shift || true
CUSTOM_CMD=("$@")

if [[ -z "${TARGET_APP}" ]]; then
    echo "Usage: $0 <app_name_or_alias> [command_args...]" >&2
    exit 1
fi

# Define alias mappings (pattern, default executable)
PATTERN=""
SPAWN_CMD=()

case "${TARGET_APP}" in
    browser|web)
        PATTERN="zen-alpha|zen|firefox|google-chrome|chromium|brave-browser"
        if command -v zen-browser >/dev/null 2>&1; then
            SPAWN_CMD=("zen-browser")
        elif command -v firefox >/dev/null 2>&1; then
            SPAWN_CMD=("firefox")
        elif command -v chromium >/dev/null 2>&1; then
            SPAWN_CMD=("chromium")
        else
            SPAWN_CMD=("xdg-open" "https://duckduckgo.com")
        fi
        ;;
    files|file-manager|explorer)
        PATTERN="yazi|thunar|nautilus|nemo|dolphin"
        if command -v ghostty >/dev/null 2>&1 && command -v yazi >/dev/null 2>&1; then
            SPAWN_CMD=("ghostty" "-e" "yazi")
        elif command -v thunar >/dev/null 2>&1; then
            SPAWN_CMD=("thunar")
        elif command -v yazi >/dev/null 2>&1; then
            SPAWN_CMD=("foot" "-e" "yazi")
        else
            SPAWN_CMD=("xdg-open" "${HOME}")
        fi
        ;;
    terminal|term)
        PATTERN="com\.mitchellh\.ghostty|ghostty|foot|kitty|Alacritty"
        SPAWN_CMD=("${SCRIPT_DIR}/terminal-select.sh")
        ;;
    editor|code|neovim)
        PATTERN="nvim|Neovim|code|vscodium"
        if command -v ghostty >/dev/null 2>&1 && command -v nvim >/dev/null 2>&1; then
            SPAWN_CMD=("ghostty" "-e" "nvim")
        elif command -v code >/dev/null 2>&1; then
            SPAWN_CMD=("code")
        else
            SPAWN_CMD=("ghostty" "-e" "nvim")
        fi
        ;;
    *)
        PATTERN="${TARGET_APP}"
        if [[ ${#CUSTOM_CMD[@]} -gt 0 ]]; then
            SPAWN_CMD=("${CUSTOM_CMD[@]}")
        else
            SPAWN_CMD=("${TARGET_APP}")
        fi
        ;;
esac

# If user provided explicit custom command arguments, override default spawn command
if [[ ${#CUSTOM_CMD[@]} -gt 0 ]]; then
    SPAWN_CMD=("${CUSTOM_CMD[@]}")
fi

# 1. Niri Compositor Flow
if is_niri && command -v niri >/dev/null 2>&1; then
    WINDOWS_JSON="$(niri msg --json windows 2>/dev/null || true)"

    if [[ -n "${WINDOWS_JSON}" && "${WINDOWS_JSON}" != "[]" ]]; then
        # Find all matching window IDs
        MATCH_IDS=()
        if command -v jq >/dev/null 2>&1; then
            while IFS= read -r id; do
                [[ -n "${id}" ]] && MATCH_IDS+=("${id}")
            done < <(echo "${WINDOWS_JSON}" | jq -r --arg p "${PATTERN}" '.[] | select((.app_id // "" | test($p; "i")) or (.title // "" | test($p; "i"))) | .id' 2>/dev/null || true)
        else
            # Fallback parsing
            while IFS= read -r id; do
                [[ -n "${id}" ]] && MATCH_IDS+=("${id}")
            done < <(echo "${WINDOWS_JSON}" | grep -B2 -E "\"app_id\":.*(${PATTERN})" | grep -o '"id":[0-9]*' | cut -d':' -f2 || true)
        fi

        TOTAL_MATCHES="${#MATCH_IDS[@]}"
        if [[ ${TOTAL_MATCHES} -gt 0 ]]; then
            # Check currently focused window id
            FOCUSED_ID=""
            FOCUSED_JSON="$(niri msg --json focused-window 2>/dev/null || true)"
            if [[ -n "${FOCUSED_JSON}" && "${FOCUSED_JSON}" != "null" ]]; then
                if command -v jq >/dev/null 2>&1; then
                    FOCUSED_ID="$(echo "${FOCUSED_JSON}" | jq -r '.id // empty' 2>/dev/null || true)"
                else
                    FOCUSED_ID="$(echo "${FOCUSED_JSON}" | grep -o '"id":[0-9]*' | head -n 1 | cut -d':' -f2 || true)"
                fi
            fi

            # Determine next target: if current focused window is in match list, cycle to next
            TARGET_ID="${MATCH_IDS[0]}"
            if [[ -n "${FOCUSED_ID}" && ${TOTAL_MATCHES} -gt 1 ]]; then
                for i in "${!MATCH_IDS[@]}"; do
                    if [[ "${MATCH_IDS[$i]}" == "${FOCUSED_ID}" ]]; then
                        NEXT_INDEX=$(( (i + 1) % TOTAL_MATCHES ))
                        TARGET_ID="${MATCH_IDS[${NEXT_INDEX}]}"
                        break
                    fi
                done
            fi

            # Focus target window in Niri
            niri msg action focus-window --id "${TARGET_ID}" >/dev/null 2>&1 || true
            exit 0
        fi
    fi

    # No instance running: spawn new instance
    if [[ ${#SPAWN_CMD[@]} -gt 0 ]]; then
        setsid "${SPAWN_CMD[@]}" </dev/null >/dev/null 2>&1 &
        exit 0
    fi
fi

# 2. Sway Compositor Flow
if is_sway && command -v swaymsg >/dev/null 2>&1; then
    # Try focusing matching window
    if swaymsg "[app_id=\"(?i)${PATTERN}\"] focus" >/dev/null 2>&1 || swaymsg "[class=\"(?i)${PATTERN}\"] focus" >/dev/null 2>&1; then
        exit 0
    fi
    if [[ ${#SPAWN_CMD[@]} -gt 0 ]]; then
        setsid "${SPAWN_CMD[@]}" </dev/null >/dev/null 2>&1 &
        exit 0
    fi
fi

# 3. Fallback Spawn
if [[ ${#SPAWN_CMD[@]} -gt 0 ]]; then
    setsid "${SPAWN_CMD[@]}" </dev/null >/dev/null 2>&1 &
fi
