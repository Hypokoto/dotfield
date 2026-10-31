#!/usr/bin/env bash
# Dotfield Wallpaper & Background Controller
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

TARGET="${1:-}"
WP_DIR="${HOME}/.config/dotfield/background/Wallpapers"

# Kill existing wallpaper process
pkill -x swaybg 2>/dev/null || true

if ! command -v swaybg >/dev/null 2>&1; then
    exit 0
fi

# 1. If explicit image path provided and exists
if [[ -n "${TARGET}" && -f "${TARGET}" ]]; then
    setsid swaybg -i "${TARGET}" -m fill </dev/null >/dev/null 2>&1 &
    exit 0
fi

# 2. If explicit hex color provided
if [[ -n "${TARGET}" && "${TARGET}" =~ ^#?[0-9a-fA-F]{6}$ ]]; then
    COLOR="${TARGET}"
    if [[ "${COLOR}" != \#* ]]; then
        COLOR="#${COLOR}"
    fi
    setsid swaybg -c "${COLOR}" </dev/null >/dev/null 2>&1 &
    exit 0
fi

# 3. Default: load high-resolution Controlled Chaos wallpaper
if [[ -f "${WP_DIR}/W1.jpeg" ]]; then
    setsid swaybg -i "${WP_DIR}/W1.jpeg" -m fill </dev/null >/dev/null 2>&1 &
elif [[ -f "${WP_DIR}/W2.png" ]]; then
    setsid swaybg -i "${WP_DIR}/W2.png" -m fill </dev/null >/dev/null 2>&1 &
elif [[ -f "${WP_DIR}/W3.png" ]]; then
    setsid swaybg -i "${WP_DIR}/W3.png" -m fill </dev/null >/dev/null 2>&1 &
else
    # Fallback to Void Black solid canvas
    setsid swaybg -c "#0A0A0C" </dev/null >/dev/null 2>&1 &
fi
