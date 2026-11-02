#!/usr/bin/env bash
# Dotfield Environment Setup & Deployment Script
# Idempotent bootstrap for Sway, Niri, Amane Shell, and desktop tools
# Controlled Chaos Theme — Void Black · Crimson · Industrial Amber · Steel

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
export PATH="${PREFIX}:/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:${PATH:-}"
DO_BUILD=""
DRY_RUN=false
FORCE_DEPLOY=false
TARGET="all"

show_help() {
    cat <<EOF
Dotfield Setup & Deployment Installer

Usage: ./setup.sh [SUBCOMMAND] [OPTIONS]

Subcommands:
  sway                   Deploy configuration specifically for Sway compositor
  niri                   Deploy configuration specifically for Niri + Amane desktop
  all                    Deploy configurations for both Sway and Niri (default)

Options:
  --sway                 Target Sway compositor
  --niri                 Target Niri compositor
  --all                  Target both Sway and Niri (default)
  -f, --force            Force replacement of existing configs (backed up to .bak.<timestamp>)
  --prefix <dir>         Directory to install binaries into (default: ~/.local/bin)
  --build                Force compilation of Rust crate (amane)
  --no-build             Skip compiling Rust crate with cargo
  -d, --dry-run          Preview installation steps without executing changes
  -h, --help             Display this help message and exit

Examples:
  ./setup.sh sway
  ./setup.sh --niri
  ./setup.sh --force --sway
  ./setup.sh --prefix /usr/local/bin
EOF
}

# Parse positional subcommands and options
while [[ $# -gt 0 ]]; do
    case "$1" in
        sway)
            TARGET="sway"
            shift
            ;;
        niri)
            TARGET="niri"
            shift
            ;;
        all)
            TARGET="all"
            shift
            ;;
        --sway)
            TARGET="sway"
            shift
            ;;
        --niri)
            TARGET="niri"
            shift
            ;;
        --all)
            TARGET="all"
            shift
            ;;
        -f|--force)
            FORCE_DEPLOY=true
            shift
            ;;
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --build)
            DO_BUILD=true
            shift
            ;;
        --no-build)
            DO_BUILD=false
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
            err "Unknown option or subcommand: $1"
            show_help
            exit 1
            ;;
    esac
done

# If DO_BUILD was not explicitly specified:
# Build by default for 'niri' and 'all'; skip build for 'sway' since Sway does not use amane
if [[ -z "${DO_BUILD}" ]]; then
    if [[ "${TARGET}" == "sway" ]]; then
        DO_BUILD=false
    else
        DO_BUILD=true
    fi
fi

echo -e "${BOLD}======================================================${NC}"
echo -e "${BOLD}       Dotfield Desktop Suite Installer               ${NC}"
if [[ "${TARGET}" == "sway" ]]; then
    echo -e "${AMBER}       Target: Sway Wayland Compositor (Fuzzel Native)${NC}"
elif [[ "${TARGET}" == "niri" ]]; then
    echo -e "${AMBER}       Target: Niri Wayland Compositor + Amane Shell  ${NC}"
else
    echo -e "${AMBER}       Target: Universal Setup (Sway & Niri)          ${NC}"
fi
echo -e "${BOLD}======================================================${NC}\n"

# 1. Pre-flight system checks
step "Performing pre-flight checks..."

if [[ "$(uname -s)" != "Linux" ]]; then
    err "Dotfield targets Linux Wayland compositors (Niri/Sway). Operating system is not Linux."
    exit 1
fi
success "Operating system: Linux"

if [[ "${DO_BUILD}" == true ]]; then
    if ! command -v cargo >/dev/null 2>&1; then
        err "Rust toolchain ('cargo') is not installed or not in PATH."
        err "Install Rust via rustup: https://rustup.rs/ or run with --no-build"
        exit 1
    fi
    success "Rust toolchain: $(cargo --version)"
fi

# Check optional desktop companions
check_companion() {
    local cmd="$1"
    local desc="$2"
    if command -v "$cmd" >/dev/null 2>&1 || [[ -x "/usr/bin/$cmd" ]] || [[ -x "/usr/sbin/$cmd" ]] || [[ -x "/bin/$cmd" ]] || [[ -x "${PREFIX}/$cmd" ]]; then
        echo -e "  ${GREEN}✔${NC} ${cmd} (${desc}) found"
    else
        echo -e "  ${STEEL}○${NC} ${cmd} (${desc}) not found (optional/runtime companion)"
    fi
}

echo -e "\n${BOLD}Inspecting desktop tools:${NC}"
if [[ "${TARGET}" == "all" || "${TARGET}" == "sway" ]]; then
    check_companion "sway" "Sway Wayland compositor"
    check_companion "swaybg" "Sway wallpaper daemon"
fi
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    check_companion "niri" "Niri scrollable tiling compositor"
fi
check_companion "fuzzel" "Wayland application launcher"
check_companion "foot" "Fast Wayland terminal"
check_companion "kitty" "GPU-accelerated terminal"
check_companion "alacritty" "OpenGL terminal emulator"
check_companion "ghostty" "Modern GPU-accelerated terminal"
check_companion "kanshi" "Wayland dynamic display auto-configurator"
check_companion "tmux" "Terminal multiplexer"
check_companion "zellij" "Modern terminal workspace multiplexer"
check_companion "wlogout" "Wayland power and session logout menu"
check_companion "mako" "Lightweight notification daemon"
check_companion "swayosd-client" "On-screen display daemon/client"
check_companion "playerctl" "Media player controller"
check_companion "swaylock" "Screen locker"
check_companion "swayidle" "Idle management daemon"
check_companion "grim" "Screenshot grabber"
check_companion "slurp" "Screen region selector"
check_companion "wl-copy" "Wayland clipboard utility"
check_companion "zsh" "Z shell"
check_companion "starship" "Cross-shell prompt"
echo ""

# 2. Build amane binary (if enabled)
if [[ "${DO_BUILD}" == true ]]; then
    step "Building amane binary in release mode..."
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] cargo build --release -p amane-shell"
    else
        cargo build --release -p amane-shell
        success "Built amane (Amane Desktop Shell for Niri)"
    fi

    # 3. Install binary to PREFIX
    step "Installing binary to ${PREFIX}..."
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] mkdir -p ${PREFIX}"
        info "[dry-run] install -m 0755 target/release/amane ${PREFIX}/"
        info "[dry-run] ln -sf amane ${PREFIX}/dotfield-shell"
    else
        mkdir -p "${PREFIX}"
        install -m 0755 "${SCRIPT_DIR}/target/release/amane" "${PREFIX}/amane"
        ln -sf amane "${PREFIX}/dotfield-shell"
        success "Installed amane (and dotfield-shell symlink) to ${PREFIX}"
    fi
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
    for s in "${SCRIPT_DIR}/scripts"/*.sh; do
        if [[ -f "${s}" ]]; then
            install -m 0755 "${s}" "${SCRIPTS_DEST}/$(basename "${s}")"
        fi
    done
    ln -sf "${SCRIPTS_DEST}/terminal-select.sh" "${PREFIX}/terminal-select"
    ln -sf "${SCRIPTS_DEST}/display-config.sh" "${PREFIX}/display-config"
    ln -sf "${SCRIPTS_DEST}/typecheck-terminals.sh" "${PREFIX}/typecheck-terminals"
    ln -sf "${SCRIPTS_DEST}/lock.sh" "${PREFIX}/dotfield-lock" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/screenshot.sh" "${PREFIX}/dotfield-screenshot" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/screen-record.sh" "${PREFIX}/dotfield-record" 2>/dev/null || true
    success "Helper scripts deployed to ${SCRIPTS_DEST}"
fi

# 5. Deploy base configurations
step "Setting up configuration directory skeletons in ~/.config..."
ensure_config_link_or_dir() {
    local src="$1"
    local dest="$2"

    # 1. Clean broken or dead symlinks
    if [[ -L "${dest}" && ! -e "${dest}" ]]; then
        info "Cleaning dangling broken symlink: ${dest}"
        if [[ "${DRY_RUN}" != true ]]; then
            rm -f "${dest}"
        fi
    fi

    # 2. Check if destination does not exist
    if [[ ! -e "${dest}" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            info "[dry-run] cp -r ${src} ${dest}"
        else
            mkdir -p "$(dirname "${dest}")"
            cp -r "${src}" "${dest}"
            success "Staged base config: ${dest}"
        fi
    elif [[ "${FORCE_DEPLOY}" == true ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            info "[dry-run] backup ${dest} and replace with ${src}"
        else
            local ts
            ts=$(date +%s)
            mv "${dest}" "${dest}.bak.${ts}"
            mkdir -p "$(dirname "${dest}")"
            cp -r "${src}" "${dest}"
            success "Replaced base config (backed up to ${dest}.bak.${ts}): ${dest}"
        fi
    else
        info "Config already exists: ${dest} (updating modular contents)"
        # Sync newly added modular subdirectories (e.g. conf.d)
        if [[ -d "${src}" ]]; then
            cp -rn "${src}/." "${dest}/" 2>/dev/null || true
        fi
    fi
}

# Compositor Configurations
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    ensure_config_link_or_dir "${SCRIPT_DIR}/compositor/niri" "${HOME}/.config/niri"
fi

if [[ "${TARGET}" == "all" || "${TARGET}" == "sway" ]]; then
    ensure_config_link_or_dir "${SCRIPT_DIR}/compositor/sway" "${HOME}/.config/sway"
fi

# Common Desktop Configurations
ensure_config_link_or_dir "${SCRIPT_DIR}/launcher/fuzzel" "${HOME}/.config/fuzzel"
ensure_config_link_or_dir "${SCRIPT_DIR}/kanshi" "${HOME}/.config/kanshi"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/foot" "${HOME}/.config/foot"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/kitty" "${HOME}/.config/kitty"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/alacritty" "${HOME}/.config/alacritty"
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/ghostty" "${HOME}/.config/ghostty"
ensure_config_link_or_dir "${SCRIPT_DIR}/mako" "${HOME}/.config/mako"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swayosd" "${HOME}/.config/swayosd"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/avizo" "${HOME}/.config/avizo"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swaylock" "${HOME}/.config/swaylock"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swayidle" "${HOME}/.config/swayidle"
ensure_config_link_or_dir "${SCRIPT_DIR}/ly" "${HOME}/.config/dotfield/ly"
ensure_config_link_or_dir "${SCRIPT_DIR}/nvim" "${HOME}/.config/nvim"
ensure_config_link_or_dir "${SCRIPT_DIR}/multiplexer/tmux" "${HOME}/.config/tmux"
ensure_config_link_or_dir "${SCRIPT_DIR}/multiplexer/zellij" "${HOME}/.config/zellij"
ensure_config_link_or_dir "${SCRIPT_DIR}/wlogout" "${HOME}/.config/wlogout"
ensure_config_link_or_dir "${SCRIPT_DIR}/fastfetch" "${HOME}/.config/fastfetch"
ensure_config_link_or_dir "${SCRIPT_DIR}/shell/starship/starship.toml" "${HOME}/.config/starship.toml"
ensure_config_link_or_dir "${SCRIPT_DIR}/shell/zsh/.zshrc" "${HOME}/.zshrc"

# Deploy systemd user service for amane (Niri)
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    mkdir -p "${HOME}/.config/systemd/user" 2>/dev/null || true
    ensure_config_link_or_dir "${SCRIPT_DIR}/systemd/user/amane.service" "${HOME}/.config/systemd/user/amane.service"
    if [[ ! -e "${HOME}/.config/systemd/user/dotfield-shell.service" || -L "${HOME}/.config/systemd/user/dotfield-shell.service" ]]; then
        if [[ "${DRY_RUN}" != true ]]; then
            ln -sf amane.service "${HOME}/.config/systemd/user/dotfield-shell.service" 2>/dev/null || true
        fi
    fi
fi

# Ensure ~/.tmux.conf points cleanly to ~/.config/tmux/tmux.conf
if [[ ! -e "${HOME}/.tmux.conf" || -L "${HOME}/.tmux.conf" ]]; then
    if [[ "${DRY_RUN}" != true ]]; then
        ln -sfn "${HOME}/.config/tmux/tmux.conf" "${HOME}/.tmux.conf"
    fi
fi

# Pre-compile zshrc if zsh is available
if [[ "${DRY_RUN}" != true ]] && command -v zsh >/dev/null 2>&1; then
    zsh -c "zcompile ${HOME}/.zshrc 2>/dev/null" || true
fi

if [[ -d "${SCRIPT_DIR}/Wallpapers" ]]; then
    ensure_config_link_or_dir "${SCRIPT_DIR}/Wallpapers" "${HOME}/.config/dotfield/background/Wallpapers"
fi
if [[ -d "${SCRIPT_DIR}/Wallpapers/Gifs" ]]; then
    ensure_config_link_or_dir "${SCRIPT_DIR}/Wallpapers/Gifs" "${HOME}/.config/dotfield/background/Gifs"
fi

# 6. Reload running desktop components
step "Reloading running desktop components..."
if [[ "${DRY_RUN}" != true ]]; then
    if [[ -x "${HOME}/.config/dotfield/scripts/wallpaper.sh" ]]; then
        "${HOME}/.config/dotfield/scripts/wallpaper.sh" >/dev/null 2>&1 || true
    fi

    if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
        if command -v niri >/dev/null 2>&1 && pgrep -x niri >/dev/null 2>&1; then
            niri msg action load-config-file >/dev/null 2>&1 || true
            info "Reloaded Niri configuration"
        fi
        if systemctl --user is-active --quiet amane.service 2>/dev/null; then
            systemctl --user restart amane.service 2>/dev/null || true
            info "Restarted amane.service"
        fi
    fi

    if [[ "${TARGET}" == "all" || "${TARGET}" == "sway" ]]; then
        if command -v swaymsg >/dev/null 2>&1 && pgrep -x sway >/dev/null 2>&1; then
            SWAYSOCK=$(ls /run/user/$(id -u)/sway-ipc.*.sock 2>/dev/null | head -n 1) swaymsg reload >/dev/null 2>&1 || true
            info "Reloaded Sway configuration"
        fi
    fi

    if command -v makoctl >/dev/null 2>&1 && pgrep -x mako >/dev/null 2>&1; then
        makoctl reload >/dev/null 2>&1 || true
    fi
fi

# 7. Summary & Quick Guide
echo -e "\n${GREEN}${BOLD}======================================================${NC}"
echo -e "${GREEN}${BOLD}       Dotfield Setup Completed Successfully!         ${NC}"
echo -e "${GREEN}${BOLD}======================================================${NC}\n"

if [[ "${TARGET}" == "all" || "${TARGET}" == "sway" ]]; then
    echo -e "${AMBER}${BOLD}Sway Desktop Keybindings (Zero Bar · Fuzzel Native):${NC}"
    echo -e "  ${STEEL}Mod+Return${NC}         Launch default terminal"
    echo -e "  ${STEEL}Mod+Shift+Return${NC}   Terminal Selector (Fuzzel dmenu)"
    echo -e "  ${STEEL}Mod+Space${NC}          Application Launcher (Fuzzel)"
    echo -e "  ${STEEL}Mod+Shift+D${NC}        Display Configurator (Fuzzel / wlr-randr)"
    echo -e "  ${STEEL}Mod+Shift+R${NC}        Screen Recorder (wf-recorder)"
    echo -e "  ${STEEL}Mod+Shift+E${NC}        Session Logout Menu (wlogout)"
    echo -e "  ${STEEL}Mod+Q${NC}              Kill focused window"
    echo -e "  ${STEEL}Mod+Ctrl+R${NC}         Reload Sway configuration"
    echo -e "  ${STEEL}Mod+Escape${NC}         Lock screen (swaylock)"
    echo -e "  ${STEEL}Print / Shift+Print${NC} Screenshot (Full / Area)"
    echo ""
fi

if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    echo -e "${AMBER}${BOLD}Niri + Amane Desktop Keybindings:${NC}"
    echo -e "  ${STEEL}Mod+Return${NC}         Launch default terminal"
    echo -e "  ${STEEL}Mod+Shift+Return${NC}   Terminal Manager overlay"
    echo -e "  ${STEEL}Mod+Space${NC}          Application Launcher overlay"
    echo -e "  ${STEEL}Mod+Shift+Space${NC}    Control Center & Quick Settings"
    echo -e "  ${STEEL}Mod+Shift+C${NC}        Interactive Monthly Calendar"
    echo -e "  ${STEEL}Mod+Shift+R${NC}        Screen Recorder overlay"
    echo -e "  ${STEEL}Mod+Shift+D${NC}        Display Configurator overlay"
    echo -e "  ${STEEL}Mod+Shift+E${NC}        Powermenu overlay"
    echo -e "  ${STEEL}Mod+Shift+N/A/B/T${NC}  Network / Audio / Bluetooth / Tray overlays"
    echo -e "  ${STEEL}Mod+Q${NC}              Close focused window"
    echo -e "  ${STEEL}Mod+Escape${NC}         Lock screen"
    echo -e "  ${STEEL}Print / Shift+Print${NC} Screenshot (Full / Area)"
    echo ""
fi
