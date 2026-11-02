#!/usr/bin/env bash
# Dotfield Terminal Selector & Dispatcher
# Launches configured default terminal directly or presents selector menu
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

MODE="${1:-default}"

# 1. If explicit menu requested (--select, --menu, -m), summon Terminal Manager
if [[ "$MODE" == "--select" || "$MODE" == "--menu" || "$MODE" == "-m" ]]; then
    SHELL_BIN=$(command -v amane 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/amane" ]] && echo "${HOME}/.local/bin/amane"
    } || command -v dotfield-shell 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/dotfield-shell" ]] && echo "${HOME}/.local/bin/dotfield-shell"
    } || true)

    # In Sway, we strictly avoid amane and use fuzzel
    if [[ -z "${SWAYSOCK:-}" ]] && [[ -n "${SHELL_BIN}" ]] && { pgrep -x amane >/dev/null 2>&1 || pgrep -x dotfield-shell >/dev/null 2>&1; }; then
        exec "${SHELL_BIN}" --terminal-manager
    fi

    # Fallback to Fuzzel if shell daemon is offline
    OPTIONS=(
        "󰞷  Foot       (Fast Wayland Client):foot"
        "󰄛  Kitty      (GPU Accelerated):kitty"
        "󰆍  Alacritty  (OpenGL Fast Terminal):alacritty"
        "󰊠  Ghostty    (Modern GPU Terminal):ghostty"
    )

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

    if [[ -n "$MENU" ]]; then
        CHOICE=$(echo "$MENU" | fuzzel --dmenu --prompt "Terminal ❯ " --lines 4 --width 40 || true)
        if [[ -n "$CHOICE" ]]; then
            for item in "${OPTIONS[@]}"; do
                label="${item%%:*}"
                bin="${item##*:}"
                if [[ "$CHOICE" == "$label"* ]]; then
                    setsid "$bin" </dev/null >/dev/null 2>&1 &
                    exit 0
                fi
            done
        fi
    fi
    exit 0
fi

# 2. Default behavior for Super+Return: Launch default terminal directly
TARGET_BIN=""

# Check dotfield shell manifest
SHELL_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/dotfield/shell.toml"
if [[ -f "${SHELL_CONFIG}" ]]; then
    CONF_DEFAULT=$(awk -F'=' '/\[terminal\]/{flag=1; next} /^\[/{flag=0} flag && /default/ {gsub(/[ "]/, "", $2); print $2}' "${SHELL_CONFIG}" 2>/dev/null || true)
    if [[ -n "${CONF_DEFAULT}" ]] && command -v "${CONF_DEFAULT}" >/dev/null 2>&1; then
        TARGET_BIN="${CONF_DEFAULT}"
    fi
fi

# Fallback to $TERMINAL
if [[ -z "${TARGET_BIN}" && -n "${TERMINAL:-}" ]] && command -v "${TERMINAL}" >/dev/null 2>&1; then
    TARGET_BIN="${TERMINAL}"
fi

# Fallback to preferred installed terminal
if [[ -z "${TARGET_BIN}" ]]; then
    for cand in alacritty foot kitty ghostty; do
        if command -v "$cand" >/dev/null 2>&1; then
            TARGET_BIN="$cand"
            break
        fi
    done
fi

if [[ -n "${TARGET_BIN}" ]]; then
    setsid "${TARGET_BIN}" </dev/null >/dev/null 2>&1 &
    exit 0
fi

# Ultimate fallback
if command -v xterm >/dev/null 2>&1; then
    setsid xterm </dev/null >/dev/null 2>&1 &
fi
