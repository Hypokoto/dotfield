#!/usr/bin/env bash
# Solid canvas background setter managed by Dotfield
# Controlled Chaos Void (#0A0A0C)

set -euo pipefail

COLOR="${1:-#0A0A0C}"

# Kill existing wallpaper process
pkill -x swaybg 2>/dev/null || true

# Launch swaybg in background with solid color
if command -v swaybg >/dev/null 2>&1; then
    nohup swaybg -c "${COLOR}" >/dev/null 2>&1 &
fi
