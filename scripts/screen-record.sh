#!/usr/bin/env bash
# Screen recording helper managed by Dotfield
# Supports --fullscreen, --area, --stop, --toggle, --status, and optional --audio

set -euo pipefail

umask 077
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
PID_FILE="${XDG_RUNTIME_DIR}/dotfield-recording.pid"
META_FILE="${XDG_RUNTIME_DIR}/dotfield-recording.meta"
TARGET_DIR="${HOME}/Videos/Recordings"

mkdir -p "${TARGET_DIR}"

is_recording() {
    if [[ -f "${PID_FILE}" ]]; then
        local pid
        pid="$(cat "${PID_FILE}" 2>/dev/null || true)"
        if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
            return 0
        fi
        rm -f "${PID_FILE}" "${META_FILE}" 2>/dev/null || true
    fi
    return 1
}

stop_recording() {
    if ! is_recording; then
        echo "No recording in progress." >&2
        return 0
    fi

    local pid
    pid="$(cat "${PID_FILE}")"
    echo "Stopping screen recording (PID: ${pid})..."

    # Send SIGINT to wf-recorder so it can finalize and write MP4 trailer cleanly
    kill -INT "${pid}" 2>/dev/null || true

    # Wait up to 5 seconds for clean exit
    local count=0
    while kill -0 "${pid}" 2>/dev/null; do
        sleep 0.2
        count=$((count + 1))
        if [[ ${count} -ge 25 ]]; then
            echo "Forcing termination of recording process..." >&2
            kill -KILL "${pid}" 2>/dev/null || true
            break
        fi
    done

    local output_file=""
    if [[ -f "${META_FILE}" ]]; then
        output_file="$(cat "${META_FILE}" 2>/dev/null || true)"
    fi

    rm -f "${PID_FILE}" "${META_FILE}" 2>/dev/null || true

    if [[ -n "${output_file}" && -f "${output_file}" ]]; then
        local file_size
        file_size="$(du -h "${output_file}" | cut -f1)"
        echo "Recording saved: ${output_file} (${file_size})"
        if command -v notify-send >/dev/null 2>&1; then
            notify-send -u normal -i video-x-generic "Screen Recording Saved" "Saved to $(basename "${output_file}") (${file_size})"
        fi
    fi
}

start_recording() {
    local mode="${1:---area}"
    local with_audio=false

    for arg in "$@"; do
        if [[ "${arg}" == "--audio" ]]; then
            with_audio=true
        fi
    done

    if is_recording; then
        echo "A recording is already active. Stop it first." >&2
        exit 1
    fi

    if ! command -v wf-recorder >/dev/null 2>&1; then
        echo "Error: wf-recorder is required for Wayland screen recording." >&2
        if command -v notify-send >/dev/null 2>&1; then
            notify-send -u critical "Screen Recorder" "wf-recorder is not installed."
        fi
        exit 1
    fi

    local timestamp
    timestamp="$(date +'%Y-%m-%d_%H-%M-%S')"
    local output_file="${TARGET_DIR}/recording_${timestamp}.mp4"

    local extra_args=()
    if [[ "${with_audio}" == "true" ]]; then
        extra_args+=(--audio)
    fi

    local geometry=""
    if [[ "${mode}" == "--area" ]]; then
        if ! command -v slurp >/dev/null 2>&1; then
            echo "Error: slurp is required for region selection." >&2
            exit 1
        fi
        geometry="$(slurp 2>/dev/null || true)"
        if [[ -z "${geometry}" ]]; then
            echo "Selection canceled."
            exit 0
        fi
        extra_args+=(-g "${geometry}")
    fi

    echo "${output_file}" > "${META_FILE}"

    # Launch wf-recorder in background
    wf-recorder "${extra_args[@]}" -f "${output_file}" >/dev/null 2>&1 &
    local rec_pid=$!
    echo "${rec_pid}" > "${PID_FILE}"

    # Verify process started
    sleep 0.3
    if kill -0 "${rec_pid}" 2>/dev/null; then
        echo "Recording started (PID: ${rec_pid}) -> ${output_file}"
        if command -v notify-send >/dev/null 2>&1; then
            notify-send -u low -i media-record "Screen Recording Started" "Press hotkey or click REC indicator to stop"
        fi
    else
        echo "Error: wf-recorder failed to start" >&2
        rm -f "${PID_FILE}" "${META_FILE}" 2>/dev/null || true
        exit 1
    fi
}

case "${1:-}" in
    --stop)
        stop_recording
        ;;
    --toggle)
        if is_recording; then
            stop_recording
        else
            start_recording "--area" "${@:2}"
        fi
        ;;
    --status)
        if is_recording; then
            rec_pid="$(cat "${PID_FILE}")"
            echo "{\"status\":\"recording\",\"pid\":${rec_pid}}"
        else
            echo "{\"status\":\"idle\"}"
        fi
        ;;
    --fullscreen)
        start_recording "--fullscreen" "${@:2}"
        ;;
    --area)
        start_recording "--area" "${@:2}"
        ;;
    *)
        echo "Usage: $0 {--area|--fullscreen|--stop|--toggle|--status} [--audio]"
        exit 1
        ;;
esac
