#!/usr/bin/env bash
# Dotfield Screen Locking Utility
# Gracefully captures session snapshot, applies Controlled Chaos blur, and locks screen
# Supports swaylock, hyprlock, and gtklock

set -euo pipefail

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Prefer swaylock with dotfield config if available
if has_cmd swaylock; then
    CONFIG_PATH="${HOME}/.config/swaylock/config"
    [[ ! -f "${CONFIG_PATH}" ]] && CONFIG_PATH="$(dirname "$0")/../osd/swaylock/config"

    # Attempt to generate blurred backdrop if grim and an image processor are present
    BLUR_IMG=""
    if has_cmd grim && (has_cmd magick || has_cmd convert || has_cmd ffmpeg); then
        RAW_SHOT=$(mktemp /tmp/dotfield_lock_raw_XXXXXX.png)
        BLUR_SHOT=$(mktemp /tmp/dotfield_lock_blur_XXXXXX.png)

        if grim "$RAW_SHOT" 2>/dev/null; then
            if has_cmd magick; then
                magick "$RAW_SHOT" -scale 10% -blur 0x2 -resize 1000% -fill "#0A0A0C" -colorize 40% "$BLUR_SHOT" 2>/dev/null && BLUR_IMG="$BLUR_SHOT"
            elif has_cmd convert; then
                convert "$RAW_SHOT" -scale 10% -blur 0x2 -resize 1000% -fill "#0A0A0C" -colorize 40% "$BLUR_SHOT" 2>/dev/null && BLUR_IMG="$BLUR_SHOT"
            elif has_cmd ffmpeg; then
                ffmpeg -y -i "$RAW_SHOT" -vf "scale=iw/8:ih/8,boxblur=8,scale=iw*8:ih*8" "$BLUR_SHOT" >/dev/null 2>&1 && BLUR_IMG="$BLUR_SHOT"
            fi
        fi
        rm -f "$RAW_SHOT"
    fi

    # Lock execution
    LOCK_ARGS=("-f")
    if [[ -f "${CONFIG_PATH}" ]]; then
        LOCK_ARGS+=("-C" "${CONFIG_PATH}")
    fi

    if [[ -n "${BLUR_IMG}" && -f "${BLUR_IMG}" ]]; then
        swaylock "${LOCK_ARGS[@]}" -i "${BLUR_IMG}" -s fill
        # Wait a moment for swaylock to fork/detach before removing temporary file
        ( sleep 2 && rm -f "${BLUR_IMG}" ) &
    else
        swaylock "${LOCK_ARGS[@]}"
    fi

elif has_cmd hyprlock; then
    exec hyprlock
elif has_cmd gtklock; then
    exec gtklock -d
else
    echo "Error: No supported screen locker found (swaylock, hyprlock, gtklock)" >&2
    exit 1
fi
