#!/usr/bin/env bash
# Dotfield Dependency & Package Installer
# Installs system packages, toolchains, and desktop dependencies for Sway & Niri
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
TARGET="all"
AUTO_YES=false
RUN_SETUP=true
DO_BUILD=""
DRY_RUN=false
FORCE_DEPLOY=false

show_help() {
    cat <<EOF
Dotfield Dependency & Package Installer

Usage: ./install.sh [SUBCOMMAND] [OPTIONS]

Subcommands:
  sway                   Install dependencies for Sway compositor setup
  niri                   Install dependencies for Niri + Amane desktop setup
  all                    Install dependencies for both compositors (default)

Options:
  --sway                 Install dependencies for Sway
  --niri                 Install dependencies for Niri + Amane
  --all                  Install dependencies for both compositors (default)
  --hardening            Forward hardening suite configuration to setup.sh
  --apparmor             Forward AppArmor profile setup to setup.sh
  --firejail             Forward Firejail sandbox setup to setup.sh
  --flatpak              Forward Flatpak overrides setup to setup.sh
  --nft                  Forward NFTables firewall setup to setup.sh
  -y, --yes, --noconfirm Automatically answer yes to package manager prompts
  --no-setup             Skip running setup.sh after dependency installation
  --no-build             Skip compiling amane desktop shell binary
  -f, --force            Force replacement of existing configs (forwarded to setup.sh)
  --prefix <dir>         Directory to install binaries into (default: ~/.local/bin)
  -d, --dry-run          Preview installation commands without executing them
  -h, --help             Display this help message and exit

Examples:
  ./install.sh sway
  ./install.sh --niri -y
  ./install.sh --all --hardening
  ./install.sh --all --dry-run
EOF
}

HARDENING_FLAGS=()

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
        hardening)
            TARGET="all"
            HARDENING_FLAGS+=("--hardening")
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
        --hardening|--apparmor|--firejail|--flatpak|--nft)
            HARDENING_FLAGS+=("$1")
            shift
            ;;
        -y|--yes|--noconfirm)
            AUTO_YES=true
            shift
            ;;
        --no-setup)
            RUN_SETUP=false
            shift
            ;;
        --no-build)
            DO_BUILD=false
            shift
            ;;
        --build)
            DO_BUILD=true
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

if [[ -z "${DO_BUILD}" ]]; then
    if [[ "${TARGET}" == "sway" ]]; then
        DO_BUILD=false
    else
        DO_BUILD=true
    fi
fi

echo -e "${BOLD}======================================================${NC}"
echo -e "${BOLD}       Dotfield Dependency & Package Installer        ${NC}"
if [[ "${TARGET}" == "sway" ]]; then
    echo -e "${AMBER}       Scope: Sway Wayland Compositor (Fuzzel Native) ${NC}"
elif [[ "${TARGET}" == "niri" ]]; then
    echo -e "${AMBER}       Scope: Niri Wayland Compositor + Amane Shell   ${NC}"
else
    echo -e "${AMBER}       Scope: Universal Setup (Sway & Niri)           ${NC}"
fi
echo -e "${BOLD}======================================================${NC}\n"

# 1. Detect Distribution & Package Manager
step "Detecting Linux distribution and package manager..."

PKG_MANAGER=""
AUR_HELPER=""

if [[ -f /etc/os-release ]]; then
    # shellcheck source=/dev/null
    . /etc/os-release
    DISTRO_ID="${ID:-unknown}"
    DISTRO_LIKE="${ID_LIKE:-}"
else
    DISTRO_ID="unknown"
    DISTRO_LIKE=""
fi

if command -v pacman >/dev/null 2>&1; then
    PKG_MANAGER="pacman"
    if command -v yay >/dev/null 2>&1; then
        AUR_HELPER="yay"
    elif command -v paru >/dev/null 2>&1; then
        AUR_HELPER="paru"
    fi
elif command -v apt-get >/dev/null 2>&1; then
    PKG_MANAGER="apt"
elif command -v dnf >/dev/null 2>&1; then
    PKG_MANAGER="dnf"
elif command -v zypper >/dev/null 2>&1; then
    PKG_MANAGER="zypper"
fi

if [[ -n "${PKG_MANAGER}" ]]; then
    success "Detected OS: ${DISTRO_ID} (package manager: ${PKG_MANAGER}${AUR_HELPER:+, AUR helper: ${AUR_HELPER}})"
else
    warn "Unsupported or unknown package manager. Package installation will provide instructions."
fi

# 2. Define Package Lists per Compositor & Distro
declare -a COMMON_PKGS=()
declare -a SWAY_PKGS=()
declare -a NIRI_PKGS=()

case "${PKG_MANAGER}" in
    pacman)
        # Arch Linux / Manjaro / EndeavourOS
        COMMON_PKGS=(
            foot kitty alacritty fuzzel kanshi tmux zellij neovim zsh starship fastfetch
            mako swaylock swayidle grim slurp wl-clipboard wlr-randr wlogout
            wf-recorder eza bat ripgrep fd dust duf btop doggo xh trash-cli
            zoxide direnv ttf-jetbrains-mono-nerd papirus-icon-theme playerctl
        )
        SWAY_PKGS=(sway swaybg swaynag)
        NIRI_PKGS=(rust vulkan-headers vulkan-icd-loader libxkbcommon wayland-protocols)
        ;;
    apt)
        # Debian / Ubuntu
        COMMON_PKGS=(
            foot kitty alacritty tmux neovim zsh kanshi playerctl fastfetch
            swaylock swayidle grim slurp wl-clipboard wlogout ripgrep
            fd-find bat btop direnv
        )
        SWAY_PKGS=(sway swaybg)
        NIRI_PKGS=(cargo rustc libxkbcommon-dev libwayland-dev wayland-protocols libvulkan-dev)
        ;;
    dnf)
        # Fedora
        COMMON_PKGS=(
            foot kitty alacritty fuzzel kanshi tmux zellij neovim zsh starship fastfetch
            mako playerctl swaylock swayidle grim slurp wl-clipboard wlr-randr
            wlogout ripgrep fd-find bat eza btop zoxide direnv
        )
        SWAY_PKGS=(sway swaybg)
        NIRI_PKGS=(rust cargo vulkan-headers vulkan-loader-devel libxkbcommon-devel wayland-protocols-devel)
        ;;
    zypper)
        # openSUSE
        COMMON_PKGS=(
            kitty alacritty tmux zellij neovim zsh starship mako playerctl fastfetch
            swaylock swayidle grim slurp wl-clipboard wlogout ripgrep bat
        )
        SWAY_PKGS=(sway swaybg)
        NIRI_PKGS=(rust cargo libxkbcommon-devel wayland-protocols-devel)
        ;;
    *)
        COMMON_PKGS=()
        SWAY_PKGS=()
        NIRI_PKGS=()
        ;;
esac

# Build target package list
declare -a TARGET_PKGS=("${COMMON_PKGS[@]}")
if [[ "${TARGET}" == "all" || "${TARGET}" == "sway" ]]; then
    TARGET_PKGS+=("${SWAY_PKGS[@]}")
fi
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    TARGET_PKGS+=("${NIRI_PKGS[@]}")
fi

# 3. Check for Missing Packages
step "Auditing installed packages..."

declare -a MISSING_PKGS=()

if [[ "${PKG_MANAGER}" == "pacman" ]]; then
    for pkg in "${TARGET_PKGS[@]}"; do
        if ! pacman -Qi "${pkg}" >/dev/null 2>&1 && ! pacman -Qg "${pkg}" >/dev/null 2>&1; then
            MISSING_PKGS+=("${pkg}")
        fi
    done
elif [[ "${PKG_MANAGER}" == "apt" ]]; then
    for pkg in "${TARGET_PKGS[@]}"; do
        if ! dpkg -s "${pkg}" >/dev/null 2>&1; then
            MISSING_PKGS+=("${pkg}")
        fi
    done
elif [[ "${PKG_MANAGER}" == "dnf" ]]; then
    for pkg in "${TARGET_PKGS[@]}"; do
        if ! rpm -q "${pkg}" >/dev/null 2>&1; then
            MISSING_PKGS+=("${pkg}")
        fi
    done
fi

if [[ ${#MISSING_PKGS[@]} -eq 0 ]]; then
    success "All target system packages are already installed!"
else
    info "Found ${#MISSING_PKGS[@]} missing packages: ${MISSING_PKGS[*]}"

    # Install packages
    step "Installing missing packages via ${PKG_MANAGER}..."
    SUDO_CMD=""
    if [[ $EUID -ne 0 ]]; then
        SUDO_CMD="sudo"
    fi

    YES_FLAG=""
    if [[ "${AUTO_YES}" == true ]]; then
        case "${PKG_MANAGER}" in
            pacman) YES_FLAG="--noconfirm" ;;
            apt)    YES_FLAG="-y" ;;
            dnf)    YES_FLAG="-y" ;;
            zypper) YES_FLAG="-y" ;;
        esac
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        case "${PKG_MANAGER}" in
            pacman) info "[dry-run] ${SUDO_CMD} pacman -S --needed ${YES_FLAG} ${MISSING_PKGS[*]}" ;;
            apt)    info "[dry-run] ${SUDO_CMD} apt-get install ${YES_FLAG} ${MISSING_PKGS[*]}" ;;
            dnf)    info "[dry-run] ${SUDO_CMD} dnf install ${YES_FLAG} ${MISSING_PKGS[*]}" ;;
            zypper) info "[dry-run] ${SUDO_CMD} zypper install ${YES_FLAG} ${MISSING_PKGS[*]}" ;;
            *)      info "[dry-run] install ${MISSING_PKGS[*]}" ;;
        esac
    else
        case "${PKG_MANAGER}" in
            pacman)
                ${SUDO_CMD} pacman -S --needed ${YES_FLAG} "${MISSING_PKGS[@]}"
                ;;
            apt)
                ${SUDO_CMD} apt-get update
                ${SUDO_CMD} apt-get install ${YES_FLAG} "${MISSING_PKGS[@]}"
                ;;
            dnf)
                ${SUDO_CMD} dnf install ${YES_FLAG} "${MISSING_PKGS[@]}"
                ;;
            zypper)
                ${SUDO_CMD} zypper install ${YES_FLAG} "${MISSING_PKGS[@]}"
                ;;
        esac
        success "Installed packages successfully."
    fi
fi

# 4. Check Rust toolchain (required for Niri / Amane desktop shell)
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    step "Checking Rust development toolchain for amane..."
    if ! command -v cargo >/dev/null 2>&1; then
        warn "Rust toolchain ('cargo') not found."
        if [[ "${DRY_RUN}" == true ]]; then
            info "[dry-run] curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
        else
            info "Installing Rust toolchain via rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
            # shellcheck source=/dev/null
            [[ -f "${HOME}/.cargo/env" ]] && source "${HOME}/.cargo/env"
            success "Rust toolchain installed: $(cargo --version)"
        fi
    else
        success "Rust toolchain available: $(cargo --version)"
    fi
fi

# 5. Check Niri availability
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    if ! command -v niri >/dev/null 2>&1; then
        warn "Niri compositor binary was not found in PATH."
        if [[ -n "${AUR_HELPER}" ]]; then
            info "You can install Niri via ${AUR_HELPER}: '${AUR_HELPER} -S niri'"
        else
            info "You can install Niri via cargo: 'cargo install --locked niri'"
        fi
    else
        success "Niri compositor: $(command -v niri)"
    fi
fi

# 6. Execute Dotfield Setup Script
if [[ "${RUN_SETUP}" == true ]]; then
    step "Invoking setup.sh for configuration deployment..."
    SETUP_ARGS=("$TARGET")

    if [[ "${FORCE_DEPLOY}" == true ]]; then
        SETUP_ARGS+=("--force")
    fi
    if [[ "${DO_BUILD}" == true ]]; then
        SETUP_ARGS+=("--build")
    else
        SETUP_ARGS+=("--no-build")
    fi
    if [[ "${DRY_RUN}" == true ]]; then
        SETUP_ARGS+=("--dry-run")
    fi
    if [[ -n "${PREFIX}" ]]; then
        SETUP_ARGS+=("--prefix" "${PREFIX}")
    fi
    if [[ ${#HARDENING_FLAGS[@]} -gt 0 ]]; then
        SETUP_ARGS+=("${HARDENING_FLAGS[@]}")
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] ./setup.sh ${SETUP_ARGS[*]}"
    else
        "${SCRIPT_DIR}/setup.sh" "${SETUP_ARGS[@]}"
    fi
else
    success "Package installation complete. Run './setup.sh ${TARGET}' to deploy configurations."
fi
