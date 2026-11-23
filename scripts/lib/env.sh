#!/usr/bin/env bash
# Dotfield Environment Detection & Helper Utilities
# Shared helper library for dotfield scripts
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Resolve WAYLAND_DISPLAY if unset or stale
auto_discover_wayland_display() {
    if [[ -n "${WAYLAND_DISPLAY:-}" && -S "${XDG_RUNTIME_DIR}/${WAYLAND_DISPLAY}" ]]; then
        return 0
    fi
    for sock in "${XDG_RUNTIME_DIR}"/wayland-[0-9]*; do
        if [[ -S "${sock}" && "${sock}" != *.lock ]]; then
            WAYLAND_DISPLAY="$(basename "${sock}")"
            export WAYLAND_DISPLAY
            return 0
        fi
    done
    return 1
}
auto_discover_wayland_display || true

# 2. Sway Compositor Detection & Socket Auto-Discovery
auto_discover_swaysock() {
    if [[ -n "${SWAYSOCK:-}" && -S "${SWAYSOCK}" ]]; then
        return 0
    fi
    for sock in "${XDG_RUNTIME_DIR}"/sway-ipc.*.sock; do
        if [[ -S "${sock}" ]]; then
            export SWAYSOCK="${sock}"
            return 0
        fi
    done
    return 1
}

is_sway() {
    if [[ -n "${SWAYSOCK:-}" && -S "${SWAYSOCK}" ]]; then
        return 0
    fi
    if [[ "${XDG_CURRENT_DESKTOP:-}" =~ [Ss]way ]] || pgrep -x sway >/dev/null 2>&1; then
        auto_discover_swaysock || true
        return 0
    fi
    return 1
}

is_target_sway() {
    local target="${1:-}"
    if [[ "${target}" == "sway" || "${target}" == "--sway" ]]; then
        return 0
    elif [[ -n "${target}" && "${target}" != "auto" && "${target}" != "--auto" ]]; then
        return 1
    fi
    is_sway
}

# 3. Niri Compositor Detection & Socket Auto-Discovery
auto_discover_niri_socket() {
    if [[ -n "${NIRI_SOCKET:-}" && -S "${NIRI_SOCKET}" ]]; then
        return 0
    fi
    for sock in "${XDG_RUNTIME_DIR}"/niri-*.sock "${XDG_RUNTIME_DIR}"/niri.sock; do
        if [[ -S "${sock}" ]]; then
            export NIRI_SOCKET="${sock}"
            return 0
        fi
    done
    return 1
}

is_niri() {
    if [[ -n "${NIRI_SOCKET:-}" && -S "${NIRI_SOCKET}" ]]; then
        return 0
    fi
    if [[ "${XDG_CURRENT_DESKTOP:-}" =~ [Nn]iri ]] || pgrep -x niri >/dev/null 2>&1; then
        auto_discover_niri_socket || true
        return 0
    fi
    return 1
}

is_target_niri() {
    local target="${1:-}"
    if [[ "${target}" == "niri" || "${target}" == "--niri" ]]; then
        return 0
    elif [[ -n "${target}" && "${target}" != "auto" && "${target}" != "--auto" ]]; then
        return 1
    fi
    is_niri
}

# 4. Amane Desktop Shell Detection
get_amane_msg_bin() {
    command -v amane-msg 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/amane-msg" ]] && echo "${HOME}/.local/bin/amane-msg"
    } || true
}

get_amane_bin() {
    command -v amane 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/amane" ]] && echo "${HOME}/.local/bin/amane"
    } || command -v dotfield-shell 2>/dev/null || {
        [[ -x "${HOME}/.local/bin/dotfield-shell" ]] && echo "${HOME}/.local/bin/dotfield-shell"
    } || true
}

is_amane_running() {
    pgrep -x amane >/dev/null 2>&1 || pgrep -x dotfield-shell >/dev/null 2>&1
}

# 5. Safe Notification Dispatcher (no hard crash if notification daemon is absent)
safe_notify() {
    local app_name="${1:-dotfield}"
    local urgency="${2:-normal}"
    local title="${3:-Notification}"
    local body="${4:-}"
    local icon="${5:-}"
    local category="${6:-}"

    if has_cmd notify-send; then
        local args=("-a" "${app_name}" "-u" "${urgency}")
        [[ -n "${icon}" ]] && args+=("-i" "${icon}")
        [[ -n "${category}" ]] && args+=("-c" "${category}")
        notify-send "${args[@]}" "${title}" "${body}" 2>/dev/null || true
    fi
}
