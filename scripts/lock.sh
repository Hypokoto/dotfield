#!/usr/bin/env bash
# Dotfield Screen Locking Utility
# Gracefully invokes swaylock, hyprlock, or gtklock

set -euo pipefail

has_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Prefer swaylock with dotfield config if available
if has_cmd swaylock; then
    CONFIG_PATH="${HOME}/.config/swaylock/config"
    if [[ -f "${CONFIG_PATH}" ]]; then
        exec swaylock -f -C "${CONFIG_PATH}"
    else
        exec swaylock -f
    fi
elif has_cmd hyprlock; then
    exec hyprlock
elif has_cmd gtklock; then
    exec gtklock -d
else
    echo "Error: No supported screen locker found (swaylock, hyprlock, gtklock)" >&2
    exit 1
fi
