#!/usr/bin/env bash
# Dotfield Terminal Selector
# Presents an instantaneous Fuzzel menu to pick active terminal
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

# If dotfield-shell is active under Niri/Wayland, summon the native Terminal Manager overlay
SHELL_BIN=$(command -v dotfield-shell 2>/dev/null || {
    [[ -x "${HOME}/.local/bin/dotfield-shell" ]] && echo "${HOME}/.local/bin/dotfield-shell"
} || {
    [[ -x "${HOME}/.cargo/bin/dotfield-shell" ]] && echo "${HOME}/.cargo/bin/dotfield-shell"
} || true)

if [[ -n "${SHELL_BIN}" ]] && pgrep -x dotfield-shell >/dev/null 2>&1; then
    "${SHELL_BIN}" --terminal-manager
    exit 0
fi

# Available terminal definitions (Glyph, Label, Executable)
OPTIONS=(
    "󰞷  Foot       (Fast Wayland Client):foot"
    "󰄛  Kitty      (GPU Accelerated):kitty"
    "󰆍  Alacritty  (OpenGL Fast Terminal):alacritty"
    "󰊠  Ghostty    (Modern GPU Terminal):ghostty"
)

# Filter options to only those installed on the system
MENU=""
for item in "${OPTIONS[@]}"; do
    label="${item%%:*}"
    bin="${item##*:}"
    if command -v "$bin" >/dev/null 2>&1; then
        if [[ -n "$MENU" ]]; then
            MENU+=$'\n'
        fi
        MENU+="$label"
    fi
done

if [[ -z "$MENU" ]]; then
    # Fallback to foot or sh if none found
    if command -v foot >/dev/null 2>&1; then
        exec foot
    elif command -v kitty >/dev/null 2>&1; then
        exec kitty
    else
        exit 0
    fi
fi

# Show menu via Fuzzel dmenu mode with Controlled Chaos prompt
CHOICE=$(echo "$MENU" | fuzzel --dmenu --prompt "Terminal ❯ " --lines 4 --width 40 || true)

if [[ -z "$CHOICE" ]]; then
    exit 0
fi

# Match choice to binary and spawn detached
for item in "${OPTIONS[@]}"; do
    label="${item%%:*}"
    bin="${item##*:}"
    if [[ "$CHOICE" == "$label"* ]]; then
        setsid "$bin" </dev/null >/dev/null 2>&1 &
        exit 0
    fi
done
