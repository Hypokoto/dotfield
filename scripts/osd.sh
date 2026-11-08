#!/usr/bin/env bash
# Dotfield Hardware & Media OSD Controller
# Supports SwayOSD, Avizo, and graceful fallback via wpctl / pamixer / brightnessctl / notify-send

set -euo pipefail

ACTION="${1:-}"

if [[ -z "${ACTION}" ]]; then
    echo "Usage: $0 {volume_up|volume_down|volume_mute|mic_mute|brightness_up|brightness_down|media_play_pause|media_next|media_prev}" >&2
    exit 1
fi

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

# If amane (or legacy dotfield-shell) is active, forward directly to its IPC to prevent duplicate notifications
SHELL_BIN=$(get_amane_bin)

# In Sway, strictly bypass amane and use swayosd/mako
if ! is_sway && [[ -n "${SHELL_BIN}" ]] && is_amane_running; then
    case "${ACTION}" in
        volume_up)
            "${SHELL_BIN}" --osd volume-up 5
            exit 0
            ;;
        volume_down)
            "${SHELL_BIN}" --osd volume-down 5
            exit 0
            ;;
        volume_mute)
            "${SHELL_BIN}" --osd volume-mute
            exit 0
            ;;
        brightness_up)
            "${SHELL_BIN}" --osd brightness-up 5
            exit 0
            ;;
        brightness_down)
            "${SHELL_BIN}" --osd brightness-down 5
            exit 0
            ;;
    esac
fi

# 1. Volume Up
handle_volume_up() {
    if has_cmd swayosd-client && swayosd-client --output-volume raise 2>/dev/null; then
        return 0
    elif has_cmd volumectl && volumectl -u up 2>/dev/null; then
        return 0
    elif has_cmd wpctl; then
        wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 5%+
        show_volume_notify
    elif has_cmd pamixer; then
        pamixer -i 5
        show_volume_notify
    elif has_cmd amixer; then
        amixer set Master 5%+ >/dev/null
        show_volume_notify
    fi
}

# 2. Volume Down
handle_volume_down() {
    if has_cmd swayosd-client && swayosd-client --output-volume lower 2>/dev/null; then
        return 0
    elif has_cmd volumectl && volumectl -u down 2>/dev/null; then
        return 0
    elif has_cmd wpctl; then
        wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
        show_volume_notify
    elif has_cmd pamixer; then
        pamixer -d 5
        show_volume_notify
    elif has_cmd amixer; then
        amixer set Master 5%- >/dev/null
        show_volume_notify
    fi
}

# 3. Output Mute Toggle
handle_volume_mute() {
    if has_cmd swayosd-client && swayosd-client --output-volume mute-toggle 2>/dev/null; then
        return 0
    elif has_cmd volumectl && volumectl toggle-mute 2>/dev/null; then
        return 0
    elif has_cmd wpctl; then
        wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
        show_volume_notify
    elif has_cmd pamixer; then
        pamixer -t
        show_volume_notify
    elif has_cmd amixer; then
        amixer set Master toggle >/dev/null
        show_volume_notify
    fi
}

# 4. Input Mic Mute Toggle
handle_mic_mute() {
    if has_cmd swayosd-client && swayosd-client --input-volume mute-toggle 2>/dev/null; then
        return 0
    elif has_cmd volumectl && volumectl -m toggle-mute 2>/dev/null; then
        return 0
    elif has_cmd wpctl; then
        wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle
        if has_cmd notify-send; then
            notify-send -a dotfield -c osd -r 9993 -h "string:x-canonical-private-synchronous:mic" -t 1200 "Microphone" "Toggled Mute"
        fi
    elif has_cmd pamixer; then
        pamixer --default-source -t
        if has_cmd notify-send; then
            notify-send -a dotfield -c osd -r 9993 -h "string:x-canonical-private-synchronous:mic" -t 1200 "Microphone" "Toggled Mute"
        fi
    fi
}

# 5. Brightness Up
handle_brightness_up() {
    if has_cmd swayosd-client && swayosd-client --brightness raise 2>/dev/null; then
        return 0
    elif has_cmd lightctl && lightctl up 2>/dev/null; then
        return 0
    elif has_cmd brightnessctl; then
        brightnessctl set +5% >/dev/null
        show_brightness_notify
    fi
}

# 6. Brightness Down
handle_brightness_down() {
    if has_cmd swayosd-client && swayosd-client --brightness lower 2>/dev/null; then
        return 0
    elif has_cmd lightctl && lightctl down 2>/dev/null; then
        return 0
    elif has_cmd brightnessctl; then
        brightnessctl set 5%- >/dev/null
        show_brightness_notify
    fi
}

# 7. Media Controls
handle_media() {
    local cmd="$1"
    if has_cmd playerctl; then
        playerctl "$cmd"
    fi
}

# Fallback visual notifications via notify-send / mako (synchronous in-place replacement)
show_volume_notify() {
    if ! has_cmd notify-send; then
        return 0
    fi
    local vol=""
    local muted="false"
    if has_cmd wpctl; then
        local status
        status=$(wpctl get-volume @DEFAULT_AUDIO_SINK@ 2>/dev/null || true)
        if [[ "${status}" =~ ([0-9]+\.[0-9]+) ]]; then
            vol=$(awk "BEGIN {print int(${BASH_REMATCH[1]} * 100)}")
        fi
        if [[ "${status}" == *"[MUTED]"* ]]; then
            muted="true"
        fi
    elif has_cmd pamixer; then
        vol=$(pamixer --get-volume 2>/dev/null || echo "50")
        if pamixer --get-mute >/dev/null 2>&1; then
            muted="true"
        fi
    fi

    if [[ -n "${vol}" ]]; then
        if [[ "${muted}" == "true" ]]; then
            notify-send -a dotfield -c volume -r 9991 -h "int:value:${vol}" -h "string:x-canonical-private-synchronous:volume" -t 1200 "Volume Muted" "Current level: ${vol}%"
        else
            notify-send -a dotfield -c volume -r 9991 -h "int:value:${vol}" -h "string:x-canonical-private-synchronous:volume" -t 1200 "Volume: ${vol}%" ""
        fi
    fi
}

show_brightness_notify() {
    if ! has_cmd notify-send || ! has_cmd brightnessctl; then
        return 0
    fi
    local max
    local curr
    max=$(brightnessctl max 2>/dev/null || echo "100")
    curr=$(brightnessctl get 2>/dev/null || echo "50")
    local pct
    pct=$(awk "BEGIN {print int((${curr} / ${max}) * 100)}")
    notify-send -a dotfield -c brightness -r 9992 -h "int:value:${pct}" -h "string:x-canonical-private-synchronous:brightness" -t 1200 "Brightness: ${pct}%" ""
}

case "${ACTION}" in
    volume_up)
        handle_volume_up
        ;;
    volume_down)
        handle_volume_down
        ;;
    volume_mute)
        handle_volume_mute
        ;;
    mic_mute)
        handle_mic_mute
        ;;
    brightness_up)
        handle_brightness_up
        ;;
    brightness_down)
        handle_brightness_down
        ;;
    media_play_pause)
        handle_media play-pause
        ;;
    media_next)
        handle_media next
        ;;
    media_prev)
        handle_media previous
        ;;
    *)
        echo "Unknown action: ${ACTION}" >&2
        exit 1
        ;;
esac
