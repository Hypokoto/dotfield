#!/usr/bin/env bash
# Solid canvas background setter managed by Dotfield
# Controlled Chaos Void (#0A0A0C)

set -euo pipefail

COLOR="${1:-#0A0A0C}"

# Validate hex color format (^#?[0-9a-fA-F]{6}$) before any process manipulation
if [[ ! "${COLOR}" =~ ^#?[0-9a-fA-F]{6}$ ]]; then
    echo "Error: Invalid hex color format: '${COLOR}'. Expected 6-digit hex like '#0A0A0C' or '0A0A0C'." >&2
    exit 1
fi

if [[ "${COLOR}" != \#* ]]; then
    COLOR="#${COLOR}"
fi

# Kill existing wallpaper process
pkill -x swaybg 2>/dev/null || true

# Launch swaybg in background with solid color
if command -v swaybg >/dev/null 2>&1; then
    nohup swaybg -c "${COLOR}" >/dev/null 2>&1 &
fi
