#!/usr/bin/env bash
# Dotfield Wallpaper & Background Controller
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel
# Compositor aware:
#   - Niri: Animated GIF engine / Wallpaper via awww (defaults to W4.png)
#   - Sway: Low-latency tear-free canvas via swaybg (defaults to W1.jpeg)

set -euo pipefail

TARGET="${1:-}"
WP_DIR="${HOME}/.config/dotfield/background/Wallpapers"
GIF_DIR="${HOME}/.config/dotfield/background/Gifs"
CACHE_DIR="${HOME}/.cache/dotfield"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

is_target_niri() {
    [[ "${TARGET}" == "--niri" ]] || is_niri
}

cache_wallpaper() {
    local wp="$1"
    if [[ -n "${wp}" && -f "${wp}" ]]; then
        mkdir -p "${CACHE_DIR}" 2>/dev/null || true
        echo "${wp}" > "${CACHE_DIR}/current_wallpaper" 2>/dev/null || true
        if is_target_niri; then
            echo "${wp}" > "${CACHE_DIR}/current_niri_wallpaper" 2>/dev/null || true
        fi
    fi
}

get_wallpaper_list() {
    local search_dirs=(
        "${WP_DIR}/Imgs"
        "${WP_DIR}"
        "${HOME}/dev/projects/dotfield/Wallpapers/Imgs"
        "${HOME}/dev/projects/dotfield/Wallpapers"
    )
    declare -A seen=()
    local list=()
    for dir in "${search_dirs[@]}"; do
        [[ -d "${dir}" ]] || continue
        for img in "${dir}"/*.{png,jpg,jpeg,webp}; do
            [[ -f "${img}" ]] || continue
            local base
            base="$(basename "${img}")"
            if [[ -z "${seen["${base}"]:-}" ]]; then
                seen["${base}"]=1
                list+=("${img}")
            fi
        done
    done
    if [[ ${#list[@]} -gt 0 ]]; then
        printf "%s\n" "${list[@]}" | sort -V
    fi
}

get_next_or_prev() {
    local direction="$1" # "next" or "prev"
    local mapfile_out=()
    while IFS= read -r line; do
        [[ -n "${line}" ]] && mapfile_out+=("${line}")
    done < <(get_wallpaper_list)

    local total="${#mapfile_out[@]}"
    if [[ ${total} -eq 0 ]]; then
        return 1
    fi

    # Read current cached wallpaper
    local current_wp=""
    if is_target_niri && [[ -f "${CACHE_DIR}/current_niri_wallpaper" ]]; then
        current_wp="$(< "${CACHE_DIR}/current_niri_wallpaper")"
    elif [[ -f "${CACHE_DIR}/current_wallpaper" ]]; then
        current_wp="$(< "${CACHE_DIR}/current_wallpaper")"
    fi

    local current_base=""
    [[ -n "${current_wp}" ]] && current_base="$(basename "${current_wp}")"

    local idx=-1
    for i in "${!mapfile_out[@]}"; do
        if [[ "$(basename "${mapfile_out[$i]}")" == "${current_base}" ]]; then
            idx="${i}"
            break
        fi
    done

    local next_idx=0
    if [[ ${idx} -ge 0 ]]; then
        if [[ "${direction}" == "next" ]]; then
            next_idx=$(( (idx + 1) % total ))
        else
            next_idx=$(( (idx - 1 + total) % total ))
        fi
    fi

    echo "${mapfile_out[${next_idx}]}"
}

find_target_file() {
    local target="$1"
    if [[ -f "${target}" ]]; then
        echo "${target}"
        return 0
    fi
    local paths=(
        "${WP_DIR}/Imgs/${target}"
        "${WP_DIR}/${target}"
        "${WP_DIR}/Gifs/${target}"
        "${GIF_DIR}/${target}"
        "${HOME}/dev/projects/dotfield/Wallpapers/Imgs/${target}"
        "${HOME}/dev/projects/dotfield/Wallpapers/Gifs/${target}"
        "${HOME}/dev/projects/dotfield/Wallpapers/${target}"
    )
    for p in "${paths[@]}"; do
        if [[ -f "${p}" ]]; then
            echo "${p}"
            return 0
        fi
    done
    return 1
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
    local names=("W4.png" "W1.jpeg" "W2.png" "W3.png")
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

# 1. Niri Compositor Flow: awww-daemon + awww img (defaults to W4.png)
if is_target_niri; then
    pkill -x swaybg 2>/dev/null || true

    if ! command -v awww >/dev/null 2>&1; then
        echo "awww is not installed. Please install awww." >&2
        exit 1
    fi

    # Ensure awww-daemon is active with ultra-low latency socket check
    runtime_dir="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    wayland_disp="${WAYLAND_DISPLAY:-wayland-0}"
    sock="${runtime_dir}/${wayland_disp}-awww-daemon.sock"

    if ! pgrep -x awww-daemon >/dev/null 2>&1; then
        setsid awww-daemon </dev/null >/dev/null 2>&1 &
        for _ in {1..50}; do
            if [[ -S "${sock}" ]] || awww query >/dev/null 2>&1; then
                break
            fi
            sleep 0.01
        done
    elif [[ ! -S "${sock}" ]]; then
        for _ in {1..30}; do
            if [[ -S "${sock}" ]] || awww query >/dev/null 2>&1; then
                break
            fi
            sleep 0.01
        done
    fi

    NIRI_WALLPAPER=""
    if [[ "${TARGET}" == "next" || "${TARGET}" == "--next" ]]; then
        NIRI_WALLPAPER="$(get_next_or_prev next || true)"
    elif [[ "${TARGET}" == "prev" || "${TARGET}" == "--prev" ]]; then
        NIRI_WALLPAPER="$(get_next_or_prev prev || true)"
    elif [[ -n "${TARGET}" && "${TARGET}" != "--niri" ]]; then
        NIRI_WALLPAPER="$(find_target_file "${TARGET}" || find_gif "${TARGET}" || true)"
    fi

    if [[ -z "${NIRI_WALLPAPER}" ]]; then
        if [[ -f "${CACHE_DIR}/current_niri_wallpaper" ]] && [[ -f "$(< "${CACHE_DIR}/current_niri_wallpaper")" ]]; then
            NIRI_WALLPAPER="$(< "${CACHE_DIR}/current_niri_wallpaper")"
        else
            NIRI_WALLPAPER="$(find_img || find_gif "L12.gif" || true)"
        fi
    fi

    if [[ -n "${NIRI_WALLPAPER}" && -f "${NIRI_WALLPAPER}" ]]; then
        # Skip redundant re-decoding if not cycling and all connected outputs already display this wallpaper
        if [[ "${TARGET}" != "next" && "${TARGET}" != "--next" && "${TARGET}" != "prev" && "${TARGET}" != "--prev" ]]; then
            if awww query 2>/dev/null | grep -q "currently displaying:" && \
               ! (awww query 2>/dev/null | grep "currently displaying:" | grep -v -F "${NIRI_WALLPAPER}" >/dev/null 2>&1); then
                exit 0
            fi
        fi

        cache_wallpaper "${NIRI_WALLPAPER}"
        awww img "${NIRI_WALLPAPER}" --resize crop --filter Bilinear --transition-type none

        if [[ "${TARGET}" == "next" || "${TARGET}" == "--next" || "${TARGET}" == "prev" || "${TARGET}" == "--prev" ]]; then
            safe_notify "dotfield" "low" "Wallpaper" "$(basename "${NIRI_WALLPAPER}")" "preferences-desktop-wallpaper" "wallpaper"
        fi
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

SWAY_WALLPAPER=""
if [[ "${TARGET}" == "next" || "${TARGET}" == "--next" ]]; then
    SWAY_WALLPAPER="$(get_next_or_prev next || true)"
elif [[ "${TARGET}" == "prev" || "${TARGET}" == "--prev" ]]; then
    SWAY_WALLPAPER="$(get_next_or_prev prev || true)"
elif [[ -n "${TARGET}" && -f "${TARGET}" ]]; then
    SWAY_WALLPAPER="${TARGET}"
elif [[ -n "${TARGET}" && "${TARGET}" =~ ^#?[0-9a-fA-F]{6}$ ]]; then
    COLOR="${TARGET}"
    if [[ "${COLOR}" != \#* ]]; then
        COLOR="#${COLOR}"
    fi
    setsid swaybg -c "${COLOR}" </dev/null >/dev/null 2>&1 &
    exit 0
elif [[ -n "${TARGET}" && "${TARGET}" != "--sway" ]]; then
    SWAY_WALLPAPER="$(find_target_file "${TARGET}" || true)"
fi

if [[ -z "${SWAY_WALLPAPER}" ]]; then
    if [[ -f "${CACHE_DIR}/current_wallpaper" ]] && [[ -f "$(< "${CACHE_DIR}/current_wallpaper")" ]]; then
        SWAY_WALLPAPER="$(< "${CACHE_DIR}/current_wallpaper")"
    else
        SWAY_WALLPAPER="$(find_img || true)"
    fi
fi

if [[ -n "${SWAY_WALLPAPER}" && -f "${SWAY_WALLPAPER}" ]]; then
    cache_wallpaper "${SWAY_WALLPAPER}"
    setsid swaybg -i "${SWAY_WALLPAPER}" -m fill </dev/null >/dev/null 2>&1 &
    if [[ "${TARGET}" == "next" || "${TARGET}" == "--next" || "${TARGET}" == "prev" || "${TARGET}" == "--prev" ]]; then
        safe_notify "dotfield" "low" "Wallpaper" "$(basename "${SWAY_WALLPAPER}")" "preferences-desktop-wallpaper" "wallpaper"
    fi
else
    # Fallback to Void Black canvas
    setsid swaybg -c "#000000" </dev/null >/dev/null 2>&1 &
fi
