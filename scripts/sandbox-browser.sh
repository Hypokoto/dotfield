#!/usr/bin/env bash
# Dotfield Unprivileged Browser Sandbox
# Pure user-namespace isolation powered by bubblewrap (bwrap)
# Zero setuid-root binaries, isolated filesystem, Wayland-native display

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${SCRIPT_DIR}/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${SCRIPT_DIR}/lib/env.sh"
elif [[ -f "${HOME}/.config/dotfield/scripts/lib/env.sh" ]]; then
    # shellcheck source=/dev/null
    source "${HOME}/.config/dotfield/scripts/lib/env.sh"
fi

if ! has_cmd bwrap; then
    echo "[dotfield:error] bubblewrap (bwrap) is not installed." >&2
    echo "Install bubblewrap (e.g. 'sudo pacman -S bubblewrap' or 'sudo apt install bubblewrap')" >&2
    exit 1
fi

# 1. Resolve browser binary
BROWSER_BIN=""
BROWSER_ARGS=()

if [[ $# -gt 0 && "${1}" != -* ]]; then
    BROWSER_BIN="$1"
    shift
    BROWSER_ARGS=("$@")
elif [[ -n "${BROWSER:-}" ]] && has_cmd "${BROWSER}"; then
    BROWSER_BIN="${BROWSER}"
    BROWSER_ARGS=("$@")
else
    for candidate in firefox zen-browser chromium brave google-chrome-stable vivaldi; do
        if has_cmd "${candidate}"; then
            BROWSER_BIN="${candidate}"
            break
        fi
    done
    BROWSER_ARGS=("$@")
fi

if [[ -z "${BROWSER_BIN}" ]] || ! has_cmd "${BROWSER_BIN}"; then
    echo "[dotfield:error] No supported web browser found on system PATH." >&2
    exit 1
fi

# 2. Setup isolated sandbox directories
SANDBOX_ROOT="${XDG_DATA_HOME:-${HOME}/.local/share}/dotfield-sandbox"
SANDBOX_HOME="${SANDBOX_ROOT}/home"
DOWNLOADS_DIR="${XDG_DOWNLOAD_DIR:-${HOME}/Downloads}"

mkdir -p "${SANDBOX_HOME}" "${DOWNLOADS_DIR}"

# 3. Resolve Wayland and audio runtime sockets
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
WAYLAND_SOCK="${WAYLAND_DISPLAY:-wayland-0}"

BWRAP_ARGS=(
    # Read-only root system binaries and system configuration
    --ro-bind /usr /usr
    --ro-bind-try /lib /lib
    --ro-bind-try /lib64 /lib64
    --ro-bind-try /bin /bin
    --ro-bind-try /sbin /sbin
    --ro-bind-try /etc /etc
    # Mask machine-id to prevent persistent host hardware fingerprinting
    --tmpfs /etc/machine-id
    --tmpfs /var/lib/dbus/machine-id

    # Virtual filesystems
    --proc /proc
    --dev /dev
    --tmpfs /tmp

    # GPU acceleration and audio device nodes
    --dev-bind-try /dev/dri /dev/dri
    --dev-bind-try /dev/snd /dev/snd

    # Isolated user home and shared downloads
    --bind "${SANDBOX_HOME}" "${HOME}"
    --bind "${DOWNLOADS_DIR}" "${HOME}/Downloads"

    # Minimal XDG runtime directory (Wayland & Audio only)
    --tmpfs "${RUNTIME_DIR}"
    --ro-bind-try "${RUNTIME_DIR}/${WAYLAND_SOCK}" "${RUNTIME_DIR}/${WAYLAND_SOCK}"
    --ro-bind-try "${RUNTIME_DIR}/pipewire-0" "${RUNTIME_DIR}/pipewire-0"
    --ro-bind-try "${RUNTIME_DIR}/pulse" "${RUNTIME_DIR}/pulse"

    # Environment variables
    --setenv HOME "${HOME}"
    --setenv USER "${USER:-$(id -un)}"
    --setenv LOGNAME "${LOGNAME:-${USER:-$(id -un)}}"
    --setenv PATH "/usr/local/bin:/usr/bin:/bin"
    --setenv XDG_RUNTIME_DIR "${RUNTIME_DIR}"
    --setenv WAYLAND_DISPLAY "${WAYLAND_SOCK}"
    --setenv MOZ_ENABLE_WAYLAND "1"
    --setenv GDK_BACKEND "wayland"
    --setenv QT_QPA_PLATFORM "wayland"

    # Unprivileged security boundaries: network permitted, process/IPC isolated
    --unshare-all
    --share-net
    --die-with-parent
    --new-session
)

exec bwrap "${BWRAP_ARGS[@]}" "$(command -v "${BROWSER_BIN}")" "${BROWSER_ARGS[@]}"
