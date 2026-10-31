#!/usr/bin/env bash
# Dotfield Environment Setup & Deployment Script
# Idempotent bootstrap for Dotfield CLI, GUI utilities, and desktop ricing engine

set -euo pipefail

# ANSI color styling (Controlled Chaos palette)
RED='\033[0;31m'
GREEN='\033[0;32m'
AMBER='\033[0;33m'
BLUE='\033[0;34m'
STEEL='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}${BOLD}[dotfield:info]${NC} $*"
}

step() {
    echo -e "${AMBER}${BOLD}==>${NC} ${BOLD}$*${NC}"
}

success() {
    echo -e "${GREEN}${BOLD}✔${NC} $*"
}

warn() {
    echo -e "${AMBER}${BOLD}!${NC} $*"
}

err() {
    echo -e "${RED}${BOLD}✘${NC} $*" >&2
}

# Resolve script and repository directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

# Default parameters
PREFIX="${HOME}/.local/bin"
PROFILE="default"
DO_BUILD=true
DO_SYSTEMD=false
DO_DEPLOY=true
DRY_RUN=false

show_help() {
    cat <<EOF
Dotfield Setup & Deployment Installer

Usage: ./setup.sh [OPTIONS]

Options:
  -p, --profile <name>   Initial profile to compile and apply (default: 'default')
                         Available: default, minimal, hardening, battery
  --prefix <dir>         Directory to install binaries into (default: ~/.local/bin)
  --no-build             Skip compiling Rust crates with cargo
  --skip-deploy          Do not run 'dotfield apply' after installation
  --systemd              Install and enable dotfieldd systemd user service
  -d, --dry-run          Preview installation steps without executing changes
  -h, --help             Display this help message and exit

Examples:
  ./setup.sh
  ./setup.sh --profile minimal
  ./setup.sh --prefix /usr/local/bin --systemd
EOF
}

# Parse command line flags
while [[ $# -gt 0 ]]; do
    case "$1" in
        -p|--profile)
            PROFILE="$2"
            shift 2
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --no-build)
            DO_BUILD=false
            shift
            ;;
        --skip-deploy)
            DO_DEPLOY=false
            shift
            ;;
        --systemd)
            DO_SYSTEMD=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            err "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

echo -e "${BOLD}======================================================${NC}"
echo -e "${BOLD}       Dotfield Desktop Suite Installer               ${NC}"
echo -e "${STEEL}    Controlled Chaos Wayland Ricing Engine            ${NC}"
echo -e "${BOLD}======================================================${NC}\n"

# 1. Pre-flight system checks
step "Performing pre-flight checks..."

if [[ "$(uname -s)" != "Linux" ]]; then
    err "Dotfield targets Linux Wayland compositors (Sway/Niri). Operating system is not Linux."
    exit 1
fi
success "Operating system: Linux"

if ! command -v cargo >/dev/null 2>&1; then
    err "Rust toolchain ('cargo') is not installed or not in PATH."
    err "Install Rust via rustup: https://rustup.rs/"
    exit 1
fi
success "Rust toolchain: $(cargo --version)"

# Check optional desktop companions
check_companion() {
    local cmd="$1"
    local desc="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✔${NC} ${cmd} (${desc}) found"
    else
        echo -e "  ${STEEL}○${NC} ${cmd} (${desc}) not found (optional/runtime companion)"
    fi
}

echo -e "\n${BOLD}Inspecting desktop tools:${NC}"
check_companion "sway" "Sway Wayland compositor"
check_companion "niri" "Niri scrollable tiling compositor"
check_companion "foot" "Fast Wayland terminal"
check_companion "kitty" "GPU-accelerated terminal"
check_companion "alacritty" "OpenGL terminal emulator"
check_companion "ghostty" "Modern GPU-accelerated terminal"
check_companion "fuzzel" "Wayland application launcher"
check_companion "rofi" "Window switcher and launcher"
check_companion "mako" "Lightweight notification daemon"
check_companion "swayosd-client" "On-screen display daemon/client"
check_companion "playerctl" "Media player controller"
check_companion "i3status-rs" "Status bar engine"
check_companion "swaylock" "Screen locker"
check_companion "swayidle" "Idle management daemon"
check_companion "grim" "Screenshot grabber"
check_companion "slurp" "Screen region selector"
check_companion "wl-copy" "Wayland clipboard utility"
echo ""

# 2. Build binaries
if [[ "${DO_BUILD}" == true ]]; then
    step "Building Dotfield workspace binaries in release mode..."
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] cargo build --release"
    else
        cargo build --release
        success "Built dotfield (CLI), dotfield-gui (GUI), and dotfieldd (Daemon)"
    fi
fi

# 3. Install binaries to PREFIX
step "Installing binaries to ${PREFIX}..."
if [[ "${DRY_RUN}" == true ]]; then
    info "[dry-run] mkdir -p ${PREFIX}"
    info "[dry-run] install -m 0755 target/release/{dotfield,dotfield-gui,dotfieldd} ${PREFIX}/"
else
    mkdir -p "${PREFIX}"
    install -m 0755 "${SCRIPT_DIR}/target/release/dotfield" "${PREFIX}/dotfield"
    install -m 0755 "${SCRIPT_DIR}/target/release/dotfield-gui" "${PREFIX}/dotfield-gui"
    install -m 0755 "${SCRIPT_DIR}/target/release/dotfieldd" "${PREFIX}/dotfieldd"
    success "Installed binaries to ${PREFIX}"
fi

# Ensure PREFIX is in PATH
if [[ ":$PATH:" != *":${PREFIX}:"* ]]; then
    warn "${PREFIX} is not currently in your PATH!"
    warn "Add 'export PATH=\"${PREFIX}:\$PATH\"' to your ~/.zshrc or ~/.bashrc"
fi

# 4. Deploy helper scripts
step "Deploying helper scripts to ~/.config/dotfield/scripts..."
SCRIPTS_DEST="${HOME}/.config/dotfield/scripts"
if [[ "${DRY_RUN}" == true ]]; then
    info "[dry-run] mkdir -p ${SCRIPTS_DEST}"
    info "[dry-run] install -m 0755 scripts/*.sh ${SCRIPTS_DEST}/"
else
    mkdir -p "${SCRIPTS_DEST}"
    install -m 0755 "${SCRIPT_DIR}/scripts/screenshot.sh" "${SCRIPTS_DEST}/screenshot.sh"
    install -m 0755 "${SCRIPT_DIR}/scripts/wallpaper.sh" "${SCRIPTS_DEST}/wallpaper.sh"
    install -m 0755 "${SCRIPT_DIR}/scripts/osd.sh" "${SCRIPTS_DEST}/osd.sh"
    install -m 0755 "${SCRIPT_DIR}/scripts/lock.sh" "${SCRIPTS_DEST}/lock.sh"
    success "Helper scripts deployed to ${SCRIPTS_DEST}"
fi

# 5. Install Desktop Applications entries
step "Installing Wayland desktop application shortcuts..."
APPS_DIR="${HOME}/.local/share/applications"
if [[ "${DRY_RUN}" == true ]]; then
    info "[dry-run] mkdir -p ${APPS_DIR}"
    info "[dry-run] generate dotfield-switcher.desktop & dotfield-hud.desktop"
else
    mkdir -p "${APPS_DIR}"

    cat > "${APPS_DIR}/dotfield-switcher.desktop" <<EOF
[Desktop Entry]
Name=Dotfield Profile Switcher
Comment=Switch Dotfield Rice Profiles
Exec=${PREFIX}/dotfield-gui --switcher
Icon=preferences-desktop-theme
Terminal=false
Type=Application
Categories=Settings;DesktopSettings;
EOF

    cat > "${APPS_DIR}/dotfield-hud.desktop" <<EOF
[Desktop Entry]
Name=Dotfield Keybind HUD
Comment=View Wayland Desktop Keybindings Cheat-Sheet
Exec=${PREFIX}/dotfield-gui --hud
Icon=input-keyboard
Terminal=false
Type=Application
Categories=Utility;
EOF

    success "Desktop shortcuts installed in ${APPS_DIR}"
fi

# 6. Deploy base compositor & launcher configs (without overwriting user custom configs)
step "Setting up configuration directory skeletons in ~/.config..."
ensure_config_link_or_dir() {
    local src="$1"
    local dest="$2"

    if [[ ! -e "${dest}" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            info "[dry-run] cp -r ${src} ${dest}"
        else
            mkdir -p "$(dirname "${dest}")"
            cp -r "${src}" "${dest}"
            success "Staged base config: ${dest}"
        fi
    else
        info "Config already exists: ${dest} (preserving)"
    fi
}

ensure_config_link_or_dir "${SCRIPT_DIR}/compositor/sway" "${HOME}/.config/sway"
ensure_config_link_or_dir "${SCRIPT_DIR}/compositor/niri" "${HOME}/.config/niri"
ensure_config_link_or_dir "${SCRIPT_DIR}/launcher/fuzzel" "${HOME}/.config/fuzzel"
ensure_config_link_or_dir "${SCRIPT_DIR}/launcher/rofi" "${HOME}/.config/rofi"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/foot" "${HOME}/.config/foot"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/kitty" "${HOME}/.config/kitty"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/alacritty" "${HOME}/.config/alacritty"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/ghostty" "${HOME}/.config/ghostty"
ensure_config_link_or_dir "${SCRIPT_DIR}/mako" "${HOME}/.config/mako"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swayosd" "${HOME}/.config/swayosd"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/avizo" "${HOME}/.config/avizo"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swaylock" "${HOME}/.config/swaylock"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swayidle" "${HOME}/.config/swayidle"
ensure_config_link_or_dir "${SCRIPT_DIR}/i3-rust" "${HOME}/.config/i3status-rust"
ensure_config_link_or_dir "${SCRIPT_DIR}/profiles" "${HOME}/.config/dotfield/profiles"

# Ensure modular drop-in directories exist
if [[ "${DRY_RUN}" != true ]]; then
    mkdir -p "${HOME}/.config/sway/config.d"
    mkdir -p "${HOME}/.config/niri/config.kdl.d"
fi

# Preflight diagnostics: check if existing Sway/Niri configs include Dotfield drop-in fragments
check_include() {
    local cfg="$1"
    local pattern="$2"
    local hint="$3"
    if [[ -f "${cfg}" ]] && ! grep -Eq "${pattern}" "${cfg}"; then
        warn "${cfg} does not include Dotfield fragments. Add: ${hint}"
    fi
}
check_include "${HOME}/.config/sway/config" 'include .*\.config/sway/config\.d/' 'include ~/.config/sway/config.d/*.conf'
check_include "${HOME}/.config/niri/config.kdl" 'include "config\.kdl\.d/00-generated-bindings\.kdl"' 'include "config.kdl.d/00-generated-bindings.kdl"'

# 7. Install systemd user service (optional or requested)
SYSTEMD_USER_DIR="${HOME}/.config/systemd/user"
if [[ "${DO_SYSTEMD}" == true ]]; then
    step "Installing dotfieldd systemd user service..."
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] mkdir -p ${SYSTEMD_USER_DIR}"
        info "[dry-run] install dotfieldd.service and enable via systemctl --user"
    else
        mkdir -p "${SYSTEMD_USER_DIR}"
        cat > "${SYSTEMD_USER_DIR}/dotfieldd.service" <<EOF
[Unit]
Description=Dotfield Hardware Event & Profile Switching Daemon
Documentation=https://github.com/hypokoto/dotfield
PartOf=graphical-session.target
After=graphical-session.target

[Service]
Type=simple
ExecStart=${PREFIX}/dotfieldd --daemon
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=graphical-session.target
EOF
        if command -v systemctl >/dev/null 2>&1; then
            systemctl --user daemon-reload
            systemctl --user enable --now dotfieldd.service || warn "Could not start dotfieldd systemd unit (session might not be active)"
            success "dotfieldd service enabled"
        fi
    fi
fi

# 8. Compile and apply initial generation
if [[ "${DO_DEPLOY}" == true ]]; then
    PROFILE_FILE="${SCRIPT_DIR}/profiles/${PROFILE}.toml"
    if [[ ! -f "${PROFILE_FILE}" ]]; then
        err "Profile file not found: ${PROFILE_FILE}"
        err "Available profiles in profiles/:"
        ls -1 "${SCRIPT_DIR}/profiles"
        exit 1
    fi

    step "Validating and applying initial profile '${PROFILE}'..."
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] ${PREFIX}/dotfield check ${PROFILE_FILE}"
        info "[dry-run] ${PREFIX}/dotfield apply ${PROFILE_FILE}"
    else
        "${PREFIX}/dotfield" check "${PROFILE_FILE}"
        "${PREFIX}/dotfield" apply "${PROFILE_FILE}"
        success "Applied profile '${PROFILE}'"
    fi
fi

# 9. Summary & Quick Guide
echo -e "\n${GREEN}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}       Dotfield Setup Completed Successfully!         ${NC}"
echo -e "${GREEN}${BOLD}======================================================${NC}\n"

echo -e "${BOLD}Quick Command Reference:${NC}"
echo -e "  ${AMBER}dotfield status${NC}           Show active generation and target fragments"
echo -e "  ${AMBER}dotfield list${NC}             List all available profiles"
echo -e "  ${AMBER}dotfield apply <profile>${NC}  Deploy an alternate profile atomically"
echo -e "  ${AMBER}dotfield rollback${NC}         Instantly revert to previous generation"
echo -e "  ${AMBER}dotfield-gui --hud${NC}        Open keybind cheat-sheet HUD"
echo -e "  ${AMBER}dotfield-gui --switcher${NC}   Open visual profile switcher"
echo ""
echo -e "${BOLD}Wayland Keybindings (when compositor is running):${NC}"
echo -e "  ${STEEL}Mod4+Return${NC}   Terminal (Foot)"
echo -e "  ${STEEL}Mod4+Space${NC}    Application Launcher (Fuzzel)"
echo -e "  ${STEEL}Mod4+/${NC}        Keybind Cheat-Sheet HUD"
echo -e "  ${STEEL}Print${NC}         Area Screenshot"
echo ""
