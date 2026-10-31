#!/usr/bin/env bash
# Screenshot helper managed by Dotfield
# Supports --area, --window, --full

set -euo pipefail

umask 077
TARGET_DIR="${HOME}/Pictures/Screenshots"
mkdir -p "${TARGET_DIR}"

TIMESTAMP="$(date +'%Y-%m-%d_%H-%M-%S')"
OUTPUT_FILE="${TARGET_DIR}/${TIMESTAMP}.png"
INDEX=1
RESERVED=false
for _ in {1..100}; do
    if [[ -e "${OUTPUT_FILE}" ]]; then
        OUTPUT_FILE="${TARGET_DIR}/${TIMESTAMP}_${INDEX}.png"
        INDEX=$((INDEX + 1))
        continue
    fi
    if (set -o noclobber; : > "${OUTPUT_FILE}") 2>/dev/null; then
        RESERVED=true
        break
    elif [[ -e "${OUTPUT_FILE}" ]]; then
        OUTPUT_FILE="${TARGET_DIR}/${TIMESTAMP}_${INDEX}.png"
        INDEX=$((INDEX + 1))
    else
        echo "Error: Failed to reserve ${OUTPUT_FILE} (directory not writable or filesystem full)" >&2
        exit 1
    fi
done

if [[ "${RESERVED}" != "true" ]]; then
    echo "Error: Could not reserve unique screenshot filename after 100 attempts" >&2
    exit 1
fi

TIMED_OUT=false
# Ensure empty reserved file is removed if selection is canceled or capture fails
cleanup() {
    if [[ "${TIMED_OUT}" == "true" ]]; then
        return
    fi
    if [[ -f "${OUTPUT_FILE}" && ! -s "${OUTPUT_FILE}" ]]; then
        rm -f "${OUTPUT_FILE}"
    fi
}
trap cleanup EXIT

MODE="${1:---area}"

case "${MODE}" in
    --area)
        GEOMETRY="$(slurp 2>/dev/null)" || exit 0
        grim -g "${GEOMETRY}" "${OUTPUT_FILE}"
        ;;
    --window)
        # Obtain active window capture from compositor (Niri / Sway)
        if command -v niri >/dev/null 2>&1 && [[ -n "${NIRI_SOCKET:-}" ]] && niri msg action screenshot-window --path "${OUTPUT_FILE}" 2>/dev/null; then
            # Niri IPC returns before asynchronous write finishes; poll until file is written
            CAPTURED=false
            for _ in {1..40}; do
                if [[ -s "${OUTPUT_FILE}" ]]; then
                    CAPTURED=true
                    break
                fi
                sleep 0.05
            done
            if [[ "${CAPTURED}" != "true" ]]; then
                TIMED_OUT=true
                echo "Error: Timed out waiting for Niri window capture at ${OUTPUT_FILE}" >&2
                exit 1
            fi
        elif command -v swaymsg >/dev/null 2>&1 && [[ -n "${SWAYSOCK:-}" ]] && command -v jq >/dev/null 2>&1; then
            GEOMETRY="$(swaymsg -t get_tree 2>/dev/null | jq -r '.. | select(.focused? == true).rect? | select(. != null) | "\(.x),\(.y) \(.width)x\(.height)"' 2>/dev/null || true)"
            if [[ -n "${GEOMETRY:-}" && "${GEOMETRY}" != "null" ]]; then
                grim -g "${GEOMETRY}" "${OUTPUT_FILE}"
            else
                GEOMETRY="$(slurp 2>/dev/null)" || exit 0
                grim -g "${GEOMETRY}" "${OUTPUT_FILE}"
            fi
        else
            GEOMETRY="$(slurp 2>/dev/null)" || exit 0
            grim -g "${GEOMETRY}" "${OUTPUT_FILE}"
        fi
        ;;
    --full)
        grim "${OUTPUT_FILE}"
        ;;
    *)
        echo "Usage: $0 [--area|--window|--full]" >&2
        exit 1
        ;;
esac

if [[ -s "${OUTPUT_FILE}" ]]; then
    wl-copy < "${OUTPUT_FILE}" &
    if command -v notify-send >/dev/null 2>&1; then
        notify-send -a "Dotfield Screenshot" -h string:x-canonical-private-synchronous:screenshot "Screenshot Captured" "Saved to ${OUTPUT_FILE} and copied to clipboard." &
    fi
elif [[ -f "${OUTPUT_FILE}" && "${TIMED_OUT}" != "true" ]]; then
    rm -f "${OUTPUT_FILE}"
fi
