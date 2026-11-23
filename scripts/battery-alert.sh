#!/usr/bin/env bash
# Dotfield Battery Monitoring Utility
# Checks system battery status and dispatches Controlled Chaos notifications to Mako
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

# Battery Percentage Thresholds
LOW_THRESHOLD=20
CRITICAL_THRESHOLD=10
EMERGENCY_THRESHOLD=5

RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
STATE_FILE="${RUNTIME_DIR}/dotfield-battery.state"

get_battery_status() {
    local base_path="${1:-/sys/class/power_supply}"
    for b in "${base_path}"/BAT*; do
        if [[ -d "$b" ]]; then
            local capacity
            local status
            capacity=$(cat "$b/capacity" 2>/dev/null || echo "100")
            status=$(cat "$b/status" 2>/dev/null || echo "Unknown")
            echo "$capacity $status"
            return 0
        fi
    done
    return 1
}

main() {
    local sysfs_path="${1:-/sys/class/power_supply}"
    if ! get_battery_status "${sysfs_path}" >/dev/null 2>&1; then
        # No battery present on system (desktop PC)
        exit 0
    fi

    read -r capacity status <<<"$(get_battery_status "${sysfs_path}")"

    if [[ ! "${capacity}" =~ ^[0-9]+$ ]]; then
        capacity=100
    fi

    local prev_state="none"
    if [[ -f "${STATE_FILE}" ]]; then
        prev_state="$(cat "${STATE_FILE}" 2>/dev/null || echo "none")"
    fi

    # Reset debounce state when connected to AC power
    if [[ "$status" == "Charging" || "$status" == "Full" ]]; then
        if [[ "${prev_state}" != "none" && "${prev_state}" != "charging" ]]; then
            echo "charging" > "${STATE_FILE}" 2>/dev/null || true
            safe_notify "battery-alert" "normal" "Power Connected" "Battery is ${status,,} (${capacity}%)." "battery-charging" "battery-charging"
        else
            echo "charging" > "${STATE_FILE}" 2>/dev/null || true
        fi
        exit 0
    fi

    # Discharging state machine
    if (( capacity <= EMERGENCY_THRESHOLD )); then
        if [[ "${prev_state}" != "emergency" ]]; then
            echo "emergency" > "${STATE_FILE}" 2>/dev/null || true
            safe_notify "battery-alert" "critical" "Battery Emergency (${capacity}%)" "System will shut down imminently! Plug in AC power NOW!" "battery-caution" "battery-emergency"
        fi
    elif (( capacity <= CRITICAL_THRESHOLD )); then
        if [[ "${prev_state}" != "critical" && "${prev_state}" != "emergency" ]]; then
            echo "critical" > "${STATE_FILE}" 2>/dev/null || true
            safe_notify "battery-alert" "critical" "Battery Critical (${capacity}%)" "Connect power adapter immediately to prevent shutdown!" "battery-caution" "battery-critical"
        fi
    elif (( capacity <= LOW_THRESHOLD )); then
        if [[ "${prev_state}" == "none" || "${prev_state}" == "charging" ]]; then
            echo "low" > "${STATE_FILE}" 2>/dev/null || true
            safe_notify "battery-alert" "normal" "Battery Low (${capacity}%)" "Battery level is low. Please connect power adapter." "battery-low" "battery-low"
        fi
    else
        # Battery above low threshold
        echo "none" > "${STATE_FILE}" 2>/dev/null || true
    fi
}

main "$@"
