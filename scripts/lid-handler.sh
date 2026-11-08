#!/usr/bin/env bash
# Dotfield Hardware Lid Switch Handler
# Handles laptop lid close/open events for Niri and Sway compositors
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

ACTION="${1:-close}"

notify() {
    local msg="$1"
    if command -v notify-send >/dev/null 2>&1; then
        notify-send -a "dotfield" -u normal "Display Manager" "$msg"
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
    # DRM sysfs check for connected external displays
    if [[ -z "${ext:-}" ]]; then
        for p in /sys/class/drm/*/status; do
            if [[ -f "$p" ]] && grep -q '^connected$' "$p" 2>/dev/null; then
                local name
                name=$(basename "$(dirname "$p")")
                name="${name#*-}" # strip card0- / card1-
                if [[ "$name" != "eDP-1" ]] && [[ "$name" != "Writeback-1" ]]; then
                    ext="$name"
                    break
                fi
            fi
        done
    fi
    echo "${ext:-}"
}

EXT=$(detect_external)

case "$ACTION" in
    close)
        if [[ -n "${EXT:-}" ]]; then
            # Clamshell mode: laptop lid is closed while connected to external monitor
            PROFILE="clamshell-hdmi"
            if [[ "$EXT" == DP* ]]; then
                PROFILE="clamshell-dp"
            fi

            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch "$PROFILE" 2>/dev/null || true
            fi

            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output "$EXT" on >/dev/null 2>&1 || true
                niri msg output "$EXT" position set 0 0 >/dev/null 2>&1 || true
                niri msg output eDP-1 off >/dev/null 2>&1 || true
            fi

            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output $EXT enable pos 0 0; output eDP-1 disable" >/dev/null 2>&1 || true
            fi

            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output "$EXT" --on --pos 0,0 --output eDP-1 --off >/dev/null 2>&1 || true
            fi

            if [[ -x "${SCRIPT_DIR}/wallpaper.sh" ]]; then
                "${SCRIPT_DIR}/wallpaper.sh" >/dev/null 2>&1 &
            elif [[ -x "${HOME}/.config/dotfield/scripts/wallpaper.sh" ]]; then
                "${HOME}/.config/dotfield/scripts/wallpaper.sh" >/dev/null 2>&1 &
            fi

            notify "Clamshell Mode: eDP-1 off, ${EXT} active."
        else
            # Undocked: lid closed without external monitor -> lock and suspend
            if [[ -x "${SCRIPT_DIR}/lock.sh" ]]; then
                "${SCRIPT_DIR}/lock.sh" >/dev/null 2>&1 &
            elif [[ -x "${HOME}/.config/dotfield/scripts/lock.sh" ]]; then
                "${HOME}/.config/dotfield/scripts/lock.sh" >/dev/null 2>&1 &
            fi
            systemctl suspend || true
        fi
        ;;

    open)
        if [[ -n "${EXT:-}" ]]; then
            # Reopened lid while connected to external monitor -> restore dual-display
            PROFILE="docked-dual"
            if [[ "$EXT" == DP* ]]; then
                PROFILE="docked-dp"
            fi

            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch "$PROFILE" 2>/dev/null || true
            fi

            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output eDP-1 on >/dev/null 2>&1 || true
                niri msg output eDP-1 position set 0 0 >/dev/null 2>&1 || true
                niri msg output "$EXT" on >/dev/null 2>&1 || true
                niri msg output "$EXT" position set 1920 0 >/dev/null 2>&1 || true
            fi

            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output eDP-1 enable pos 0 0; output $EXT enable pos 1920 0" >/dev/null 2>&1 || true
            fi

            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output eDP-1 --on --pos 0,0 --output "$EXT" --on --pos 1920,0 >/dev/null 2>&1 || true
            fi

            if [[ -x "${SCRIPT_DIR}/wallpaper.sh" ]]; then
                "${SCRIPT_DIR}/wallpaper.sh" >/dev/null 2>&1 &
            elif [[ -x "${HOME}/.config/dotfield/scripts/wallpaper.sh" ]]; then
                "${HOME}/.config/dotfield/scripts/wallpaper.sh" >/dev/null 2>&1 &
            fi

            notify "Dual Display Mode: eDP-1 and ${EXT} active."
        else
            # Reopened lid undocked -> ensure internal display is on
            if command -v kanshictl >/dev/null 2>&1; then
                kanshictl switch undocked 2>/dev/null || true
            fi

            if is_niri && command -v niri >/dev/null 2>&1; then
                niri msg output eDP-1 on >/dev/null 2>&1 || true
            fi

            if is_sway && command -v swaymsg >/dev/null 2>&1; then
                swaymsg "output eDP-1 enable" >/dev/null 2>&1 || true
            fi

            if command -v wlr-randr >/dev/null 2>&1; then
                wlr-randr --output eDP-1 --on >/dev/null 2>&1 || true
            fi

            if [[ -x "${SCRIPT_DIR}/wallpaper.sh" ]]; then
                "${SCRIPT_DIR}/wallpaper.sh" >/dev/null 2>&1 &
            elif [[ -x "${HOME}/.config/dotfield/scripts/wallpaper.sh" ]]; then
                "${HOME}/.config/dotfield/scripts/wallpaper.sh" >/dev/null 2>&1 &
            fi
        fi
        ;;

    *)
        echo "Usage: $0 [close|open]" >&2
        exit 1
        ;;
esac
