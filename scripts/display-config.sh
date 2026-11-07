#!/usr/bin/env bash
# Dotfield Interactive Display Configurator
# Controlled Chaos display management via Fuzzel, Sway/wlr-randr, and Kanshi
# Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=lib/env.sh
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

# If running under Niri with amane active, summon the native Wayland Display Configurator
if ! is_sway; then
    SHELL_BIN=$(get_amane_bin)

    if [[ -n "${SHELL_BIN}" ]] && is_amane_running; then
        "${SHELL_BIN}" --display-config
        exit 0
    fi
fi

# Under Sway (or standalone fallback), present the Fuzzel dmenu interface
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

detect_external() {
    local ext=""
    if is_sway && command -v swaymsg >/dev/null 2>&1; then
        ext=$(swaymsg -t get_outputs 2>/dev/null | jq -r '.[].name' 2>/dev/null | grep -v 'eDP-1' | head -n 1 || true)
    fi
    if [[ -z "${ext:-}" ]] && is_niri && command -v niri >/dev/null 2>&1; then
        ext=$(niri msg -j outputs 2>/dev/null | jq -r 'keys[]' 2>/dev/null | grep -v 'eDP-1' | head -n 1 || true)
    fi
    if [[ -z "${ext:-}" ]] && command -v wlr-randr >/dev/null 2>&1; then
        ext=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | awk '{print $1}' | grep -v 'eDP-1' | head -n 1 || true)
    fi
    echo "${ext:-}"
}

apply_mode() {
    local output="$1"
    local mode="$2"
    if is_sway && command -v swaymsg >/dev/null 2>&1; then
        swaymsg output "$output" mode "$mode" >/dev/null 2>&1 || true
    fi
    if is_niri && command -v niri >/dev/null 2>&1; then
        niri msg output "$output" mode "${mode%Hz}" >/dev/null 2>&1 || true
    fi
    if command -v wlr-randr >/dev/null 2>&1; then
        wlr-randr --output "$output" --mode "$mode" >/dev/null 2>&1 || true
    fi
}

case "$CHOICE" in
    *"144Hz"*)
        apply_mode "eDP-1" "1920x1080@144.003Hz"
        if command -v kanshictl >/dev/null 2>&1; then
            kanshictl switch docked-dual 2>/dev/null || kanshictl switch undocked 2>/dev/null || true
        fi
        notify "eDP-1 set to 1920x1080 @ 144Hz (High Performance)"
        ;;
    *"60Hz"*)
        apply_mode "eDP-1" "1920x1080@60.001Hz"
        if command -v kanshictl >/dev/null 2>&1; then
            kanshictl switch docked-dual-60hz 2>/dev/null || kanshictl switch undocked-60hz 2>/dev/null || true
        fi
        notify "eDP-1 set to 1920x1080 @ 60Hz (Power Saving)"
        ;;
    *"Extend Right"*)
        EXT=$(detect_external)
        if [[ -n "${EXT:-}" ]]; then
            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output eDP-1 enable pos 0 0; output $EXT enable pos 1920 0" >/dev/null 2>&1 || true
            fi
            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output eDP-1 on >/dev/null 2>&1 || true
                niri msg output eDP-1 position set 0 0 >/dev/null 2>&1 || true
                niri msg output "$EXT" on >/dev/null 2>&1 || true
                niri msg output "$EXT" position set 1920 0 >/dev/null 2>&1 || true
            fi
            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch docked-dual 2>/dev/null || kanshictl switch docked-dp 2>/dev/null || true
            fi
            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output eDP-1 --pos 0,0 --output "$EXT" --pos 1920,0 >/dev/null 2>&1 || true
            fi
            notify "Extended display to $EXT on the right"
        else
            notify "No external display detected"
        fi
        ;;
    *"Mirror"*)
        EXT=$(detect_external)
        if [[ -n "${EXT:-}" ]]; then
            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output eDP-1 enable pos 0 0; output $EXT enable pos 0 0" >/dev/null 2>&1 || true
            fi
            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output eDP-1 on >/dev/null 2>&1 || true
                niri msg output eDP-1 position set 0 0 >/dev/null 2>&1 || true
                niri msg output "$EXT" on >/dev/null 2>&1 || true
                niri msg output "$EXT" position set 0 0 >/dev/null 2>&1 || true
            fi
            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch docked-mirror-hdmi 2>/dev/null || kanshictl switch docked-mirror-dp 2>/dev/null || true
            fi
            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output eDP-1 --pos 0,0 --output "$EXT" --pos 0,0 >/dev/null 2>&1 || true
            fi
            notify "Mirrored display with $EXT"
        else
            notify "No external display detected"
        fi
        ;;
    *"Laptop Display Only"*)
        EXT=$(detect_external)
        if [[ -n "${EXT:-}" ]]; then
            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output eDP-1 enable; output $EXT disable" >/dev/null 2>&1 || true
            fi
            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output eDP-1 on >/dev/null 2>&1 || true
                niri msg output "$EXT" off >/dev/null 2>&1 || true
            fi
            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch undocked 2>/dev/null || true
            fi
            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output eDP-1 --on --output "$EXT" --off >/dev/null 2>&1 || true
            fi
            notify "External display $EXT turned off"
        else
            notify "No external display detected"
        fi
        ;;
    *"External Display Only"*)
        EXT=$(detect_external)
        if [[ -n "${EXT:-}" ]]; then
            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output $EXT enable pos 0 0; output eDP-1 disable" >/dev/null 2>&1 || true
            fi
            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output "$EXT" on >/dev/null 2>&1 || true
                niri msg output "$EXT" position set 0 0 >/dev/null 2>&1 || true
                niri msg output eDP-1 off >/dev/null 2>&1 || true
            fi
            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch clamshell-hdmi 2>/dev/null || kanshictl switch clamshell-dp 2>/dev/null || true
            fi
            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output "$EXT" --on --pos 0,0 --output eDP-1 --off >/dev/null 2>&1 || true
            fi
            notify "Switched to external display $EXT (Clamshell Mode)"
        else
            notify "No external display detected"
        fi
        ;;
    *"Reload Kanshi"*)
        if command -v kanshictl >/dev/null 2>&1 && kanshictl reload >/dev/null 2>&1; then
            notify "Reloaded Kanshi display profiles"
        elif pgrep -x kanshi >/dev/null 2>&1; then
            pkill -HUP kanshi
            notify "Reloaded Kanshi display daemon"
        else
            setsid kanshi </dev/null >/dev/null 2>&1 &
            notify "Started Kanshi display daemon"
        fi
        ;;
esac
