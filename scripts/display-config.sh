#!/usr/bin/env bash
# Dotfield Interactive Display Configurator
# Controlled Chaos display management via Fuzzel, wlr-randr, and Kanshi
# Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

# If dotfield-shell is active under Niri, summon the native Wayland Display Configurator
SHELL_BIN=$(command -v dotfield-shell 2>/dev/null || {
    [[ -x "${HOME}/.local/bin/dotfield-shell" ]] && echo "${HOME}/.local/bin/dotfield-shell"
} || {
    [[ -x "${HOME}/.cargo/bin/dotfield-shell" ]] && echo "${HOME}/.cargo/bin/dotfield-shell"
} || true)

# In Sway, strictly bypass dotfield-shell / amane and use Fuzzel
if [[ -z "${SWAYSOCK:-}" ]] && [[ -n "${SHELL_BIN}" ]] && pgrep -x dotfield-shell >/dev/null 2>&1; then
    "${SHELL_BIN}" --display-config
    exit 0
fi

MENU=$(cat << 'EOF'
󰍹  eDP-1: 1920x1080 @ 144Hz (High Performance)
󰌢  eDP-1: 1920x1080 @ 60Hz (Battery Saver)
󰍺  Dual Monitor: Extend Right
󰍺  Dual Monitor: Mirror / Clone
󰍹  Laptop Display Only (Disable External)
󰍺  External Display Only (Clamshell Mode)
󰑐  Reload Kanshi Display Profiles
EOF
)

CHOICE=$(echo "$MENU" | fuzzel --dmenu --prompt "Display ❯ " --lines 7 --width 48 || true)

if [[ -z "$CHOICE" ]]; then
    exit 0
fi

notify() {
    local msg="$1"
    if command -v notify-send >/dev/null 2>&1; then
        notify-send -a "dotfield" -u normal "Display Configurator" "$msg"
    fi
}

case "$CHOICE" in
    *"144Hz"*)
        if command -v wlr-randr >/dev/null 2>&1; then
            wlr-randr --output eDP-1 --mode 1920x1080@144.003Hz
            notify "eDP-1 set to 1920x1080 @ 144Hz (High Performance)"
        fi
        ;;
    *"60Hz"*)
        if command -v wlr-randr >/dev/null 2>&1; then
            wlr-randr --output eDP-1 --mode 1920x1080@60.001Hz
            notify "eDP-1 set to 1920x1080 @ 60Hz (Power Saving)"
        fi
        ;;
    *"Extend Right"*)
        EXT=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}')
        if [[ -n "${EXT:-}" ]]; then
            wlr-randr --output eDP-1 --pos 0,0 --output "$EXT" --pos 1920,0
            notify "Extended display to $EXT on the right"
        else
            notify "No external display detected"
        fi
        ;;
    *"Mirror"*)
        EXT=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}')
        if [[ -n "${EXT:-}" ]]; then
            wlr-randr --output eDP-1 --pos 0,0 --output "$EXT" --pos 0,0
            notify "Mirrored display with $EXT"
        else
            notify "No external display detected"
        fi
        ;;
    *"Laptop Display Only"*)
        EXT=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}')
        if [[ -n "${EXT:-}" ]]; then
            wlr-randr --output eDP-1 --on --output "$EXT" --off
            notify "External display $EXT turned off"
        fi
        ;;
    *"External Display Only"*)
        EXT=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}')
        if [[ -n "${EXT:-}" ]]; then
            wlr-randr --output "$EXT" --on --pos 0,0 --output eDP-1 --off
            notify "Switched to external display $EXT (Clamshell Mode)"
        else
            notify "No external display detected"
        fi
        ;;
    *"Reload Kanshi"*)
        if pgrep -x kanshi >/dev/null 2>&1; then
            pkill -HUP kanshi
            notify "Reloaded Kanshi display daemon"
        else
            setsid kanshi </dev/null >/dev/null 2>&1 &
            notify "Started Kanshi display daemon"
        fi
        ;;
esac
