#!/usr/bin/env bash
# Screenshot helper managed by Dotfield
# Supports --area, --window, --full

set -euo pipefail

TARGET_DIR="${HOME}/Pictures/Screenshots"
mkdir -p "${TARGET_DIR}"

TIMESTAMP="$(date +'%Y-%m-%d_%H-%M-%S')"
OUTPUT_FILE="${TARGET_DIR}/${TIMESTAMP}.png"

MODE="${1:---area}"

case "${MODE}" in
    --area)
        GEOMETRY="$(slurp 2>/dev/null)" || exit 0
        grim -g "${GEOMETRY}" "${OUTPUT_FILE}"
        ;;
    --window)
        # Capture region of active window using slurp selector
        GEOMETRY="$(slurp 2>/dev/null)" || exit 0
        grim -g "${GEOMETRY}" "${OUTPUT_FILE}"
        ;;
    --full)
        grim "${OUTPUT_FILE}"
        ;;
    *)
        echo "Usage: $0 [--area|--window|--full]" >&2
        exit 1
        ;;
esac

if [[ -f "${OUTPUT_FILE}" ]]; then
    wl-copy < "${OUTPUT_FILE}"
    command -v notify-send >/dev/null 2>&1 && notify-send "Screenshot Captured" "Saved to ${OUTPUT_FILE} and copied to clipboard." || true
fi
