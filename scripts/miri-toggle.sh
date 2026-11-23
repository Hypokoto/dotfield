#!/usr/bin/env bash
# Dotfield Miri Layout Engine Mode Switcher & OSD Notifier
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=scripts/lib/env.sh
source "${SCRIPT_DIR}/lib/env.sh"

ACTION="${1:-cycle}"

case "${ACTION}" in
    master)
        miri action set-focused-workspace-mode master 2>/dev/null || true
        safe_notify "dotfield-miri" "normal" "Miri: Master Mode" "Master-Stack active (50% master left, stack right)" "view-grid"
        ;;
    scroll)
        miri action set-focused-workspace-mode scroll 2>/dev/null || true
        safe_notify "dotfield-miri" "normal" "Miri: Scroll Mode" "Scroll mode active (infinite horizontal ribbon)" "view-list"
        ;;
    cycle|*)
        miri action cycle-focused-workspace-mode 2>/dev/null || true
        safe_notify "dotfield-miri" "normal" "Miri: Mode Toggled" "Switched workspace tiling between Scroll and Master" "view-grid"
        ;;
esac
