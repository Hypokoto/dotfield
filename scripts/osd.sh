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

# 1. Volume Up
handle_volume_up() {
    if has_cmd swayosd-client; then
        swayosd-client --output-volume raise
    elif has_cmd volumectl; then
        volumectl -u up
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
    if has_cmd swayosd-client; then
        swayosd-client --output-volume lower
    elif has_cmd volumectl; then
        volumectl -u down
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
    if has_cmd swayosd-client; then
        swayosd-client --output-volume mute-toggle
    elif has_cmd volumectl; then
        volumectl toggle-mute
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
    if has_cmd swayosd-client; then
        swayosd-client --input-volume mute-toggle
    elif has_cmd volumectl; then
        volumectl -m toggle-mute
    elif has_cmd wpctl; then
        wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle
        if has_cmd notify-send; then
            notify-send -a dotfield -c osd -t 1500 "Microphone" "Toggled Mute"
        fi
    elif has_cmd pamixer; then
        pamixer --default-source -t
        if has_cmd notify-send; then
            notify-send -a dotfield -c osd -t 1500 "Microphone" "Toggled Mute"
        fi
    fi
}

# 5. Brightness Up
handle_brightness_up() {
    if has_cmd swayosd-client; then
        swayosd-client --brightness raise
    elif has_cmd lightctl; then
        lightctl up
    elif has_cmd brightnessctl; then
        brightnessctl set +5% >/dev/null
        show_brightness_notify
    fi
}

# 6. Brightness Down
handle_brightness_down() {
    if has_cmd swayosd-client; then
        swayosd-client --brightness lower
    elif has_cmd lightctl; then
        lightctl down
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

# Fallback visual notifications via notify-send / mako
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
            notify-send -a dotfield -c volume -h "int:value:${vol}" -t 1500 "Volume Muted" "Current level: ${vol}%"
        else
            notify-send -a dotfield -c volume -h "int:value:${vol}" -t 1500 "Volume: ${vol}%" ""
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
    notify-send -a dotfield -c brightness -h "int:value:${pct}" -t 1500 "Brightness: ${pct}%" ""
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
