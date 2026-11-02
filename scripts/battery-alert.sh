#!/usr/bin/env bash
# Dotfield Battery Monitoring Utility
# Checks system battery status and dispatches Controlled Chaos notifications to Mako

set -euo pipefail

# Low and critical percentage thresholds
LOW_THRESHOLD=20
CRITICAL_THRESHOLD=10

get_battery_status() {
    for b in /sys/class/power_supply/BAT*; do
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
    if ! get_battery_status >/dev/null 2>&1; then
        # No battery present on system (desktop PC)
        exit 0
    fi

    read -r capacity status <<<"$(get_battery_status)"

    if [[ "$status" == "Charging" || "$status" == "Full" ]]; then
        exit 0
    fi

    if (( capacity <= CRITICAL_THRESHOLD )); then
        notify-send \
            -a "battery-alert" \
            -u critical \
            -c "battery-critical" \
            -i "battery-caution" \
            "Battery Critical (${capacity}%)" \
            "Connect power adapter immediately to prevent shutdown!"
    elif (( capacity <= LOW_THRESHOLD )); then
        notify-send \
            -a "battery-alert" \
            -u normal \
            -c "battery-low" \
            -i "battery-low" \
            "Battery Low (${capacity}%)" \
            "Battery level is low. Please connect power adapter."
    fi
}

main "$@"
