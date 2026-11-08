#!/usr/bin/env bash
# Dotfield Screen Locking Utility
# Instant privacy lock with Controlled Chaos palette, dynamic wallpaper tracking, and notification suppression
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFIELD_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Do not start multiple lock sessions concurrently
if pgrep -x swaylock >/dev/null 2>&1 || pgrep -x hyprlock >/dev/null 2>&1 || pgrep -x gtklock >/dev/null 2>&1; then
    exit 0
fi

# 2. Suppress notification popups while locked (Privacy Mode)
if has_cmd makoctl; then
    makoctl mode -a locked 2>/dev/null || true
    # Background watcher: restore notifications as soon as the screen is unlocked
    (
        while pgrep -x swaylock >/dev/null 2>&1 || pgrep -x hyprlock >/dev/null 2>&1 || pgrep -x gtklock >/dev/null 2>&1; do
            sleep 0.5
        done
        makoctl mode -r locked 2>/dev/null || true
    ) &
fi

# 3. Resolve active desktop wallpaper for instant lock background
resolve_wallpaper() {
    local cache_file="${HOME}/.cache/dotfield/current_wallpaper"
    if [[ -f "${cache_file}" ]]; then
        local cached
        cached="$(cat "${cache_file}" 2>/dev/null || true)"
        if [[ -n "${cached}" && -f "${cached}" && "${cached}" =~ \.(png|jpe?g)$ ]]; then
            echo "${cached}"
            return 0
        fi
    fi

    # Fallback paths
    local candidates=(
        "${HOME}/.config/dotfield/background/Wallpapers/Imgs/W1.jpeg"
        "${DOTFIELD_ROOT}/Wallpapers/Imgs/W1.jpeg"
        "${DOTFIELD_ROOT}/Wallpapers/Imgs/W2.png"
        "${DOTFIELD_ROOT}/Wallpapers/Imgs/W3.png"
    )

    for c in "${candidates[@]}"; do
        if [[ -f "${c}" ]]; then
            echo "${c}"
            return 0
        fi
    done

    return 1
}

# 4. Lock Execution
if has_cmd swaylock; then
    CONFIG_PATH="${HOME}/.config/swaylock/config"
    [[ ! -f "${CONFIG_PATH}" ]] && CONFIG_PATH="${DOTFIELD_ROOT}/osd/swaylock/config"

    LOCK_ARGS=("-f")
    if [[ -f "${CONFIG_PATH}" ]]; then
        LOCK_ARGS+=("-C" "${CONFIG_PATH}")
    fi

    WALLPAPER="$(resolve_wallpaper || true)"
    if [[ -n "${WALLPAPER}" && -f "${WALLPAPER}" ]]; then
        exec swaylock "${LOCK_ARGS[@]}" -i "${WALLPAPER}" -s fill
    else
        exec swaylock "${LOCK_ARGS[@]}"
    fi

elif has_cmd hyprlock; then
    exec hyprlock
elif has_cmd gtklock; then
    exec gtklock -d
else
    echo "Error: No supported screen locker found (swaylock, hyprlock, gtklock)" >&2
    exit 1
fi
