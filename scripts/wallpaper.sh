#!/usr/bin/env bash
# Dotfield Wallpaper & Background Controller
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel
# Compositor aware:
#   - Niri: Animated GIF engine via awww (defaults to L12.gif)
#   - Sway: Low-latency tear-free canvas via swaybg (defaults to W1.jpeg)

set -euo pipefail

TARGET="${1:-}"
WP_DIR="${HOME}/.config/dotfield/background/Wallpapers"
GIF_DIR="${HOME}/.config/dotfield/background/Gifs"

is_niri() {
    [[ -n "${NIRI_SOCKET:-}" ]] || pgrep -x niri >/dev/null 2>&1 || [[ "${TARGET}" == "--niri" ]]
}

find_gif() {
    local name="${1:-L12.gif}"
    local paths=(
        "${GIF_DIR}/${name}"
        "${WP_DIR}/Gifs/${name}"
        "${HOME}/dev/projects/dotfield/Wallpapers/Gifs/${name}"
    )
    for p in "${paths[@]}"; do
        if [[ -f "${p}" ]]; then
            echo "${p}"
            return 0
        fi
    done
    return 1
}

find_img() {
    local names=("W1.jpeg" "W2.png" "W3.png")
    for name in "${names[@]}"; do
        for p in "${WP_DIR}/Imgs/${name}" "${WP_DIR}/${name}" "${HOME}/dev/projects/dotfield/Wallpapers/Imgs/${name}"; do
            if [[ -f "${p}" ]]; then
                echo "${p}"
                return 0
            fi
        done
    done
    return 1
}

# 1. Niri Compositor Flow: awww-daemon + awww img (L12.gif animated background)
if is_niri; then
    pkill -x swaybg 2>/dev/null || true

    if ! command -v awww >/dev/null 2>&1; then
        echo "awww is not installed. Please install awww." >&2
        exit 1
    fi

    # Ensure awww-daemon is active
    if ! pgrep -x awww-daemon >/dev/null 2>&1; then
        setsid awww-daemon </dev/null >/dev/null 2>&1 &
        for _ in {1..20}; do
            if awww query >/dev/null 2>&1; then
                break
            fi
            sleep 0.1
        done
    fi

    NIRI_WALLPAPER=""
    if [[ -n "${TARGET}" && "${TARGET}" != "--niri" && -f "${TARGET}" ]]; then
        NIRI_WALLPAPER="${TARGET}"
    elif [[ -n "${TARGET}" && "${TARGET}" != "--niri" ]]; then
        NIRI_WALLPAPER=$(find_gif "${TARGET}" || true)
    fi

    if [[ -z "${NIRI_WALLPAPER}" ]]; then
        NIRI_WALLPAPER=$(find_gif "L12.gif" || true)
    fi

    if [[ -n "${NIRI_WALLPAPER}" && -f "${NIRI_WALLPAPER}" ]]; then
        setsid awww img "${NIRI_WALLPAPER}" --resize crop --transition-type none </dev/null >/dev/null 2>&1 &
    else
        setsid awww clear 0A0A0C </dev/null >/dev/null 2>&1 &
    fi
    exit 0
fi

# 2. Sway Compositor Flow: swaybg (zero-animation low-latency static wallpaper)
pkill -x awww-daemon 2>/dev/null || true
pkill -x swaybg 2>/dev/null || true

if ! command -v swaybg >/dev/null 2>&1; then
    exit 0
fi

# Explicit image path provided
if [[ -n "${TARGET}" && -f "${TARGET}" ]]; then
    setsid swaybg -i "${TARGET}" -m fill </dev/null >/dev/null 2>&1 &
    exit 0
fi

# Explicit hex color provided
if [[ -n "${TARGET}" && "${TARGET}" =~ ^#?[0-9a-fA-F]{6}$ ]]; then
    COLOR="${TARGET}"
    if [[ "${COLOR}" != \#* ]]; then
        COLOR="#${COLOR}"
    fi
    setsid swaybg -c "${COLOR}" </dev/null >/dev/null 2>&1 &
    exit 0
fi

# Default high-resolution Controlled Chaos image
IMAGE=$(find_img || true)
if [[ -n "${IMAGE}" && -f "${IMAGE}" ]]; then
    setsid swaybg -i "${IMAGE}" -m fill </dev/null >/dev/null 2>&1 &
else
    # Fallback to Void Black canvas
    setsid swaybg -c "#0A0A0C" </dev/null >/dev/null 2>&1 &
fi
