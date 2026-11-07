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
ENABLE_HARDENING=false
HARDEN_APPARMOR=false
HARDEN_FIREJAIL=false
HARDEN_FLATPAK=false
HARDEN_NFT=false

show_help() {
    cat <<EOF
Dotfield Setup & Deployment Installer

Usage: ./setup.sh [SUBCOMMAND] [OPTIONS]

Subcommands:
  sway                   Deploy configuration specifically for Sway compositor
  niri                   Deploy configuration specifically for Niri + Amane desktop
  all                    Deploy configurations for both Sway and Niri (default)
  hardening              Deploy security hardening suite (AppArmor, Firejail, Flatpak, NFTables)

Options:
  --sway                 Target Sway compositor
  --niri                 Target Niri compositor
  --all                  Target both Sway and Niri (default)
  --hardening            Enable security hardening suite deployment
  --apparmor             Deploy & activate AppArmor daemon confinement profile
  --firejail             Deploy Firejail browser sandbox profile & isolation dirs
  --flatpak              Apply baseline restrictive Flatpak permission overrides
  --nft                  Deploy & activate NFTables workstation stateful filter
  -f, --force            Force replacement of existing configs (backed up to .bak.<timestamp>)
  --prefix <dir>         Directory to install binaries into (default: ~/.local/bin)
  --build                Force compilation of Rust crate (amane)
  --no-build             Skip compiling Rust crate with cargo
  -d, --dry-run          Preview installation steps without executing changes
  -h, --help             Display this help message and exit

Examples:
  ./setup.sh sway
  ./setup.sh --niri
  ./setup.sh hardening
  ./setup.sh --hardening --apparmor --firejail
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
        hardening)
            TARGET="hardening"
            ENABLE_HARDENING=true
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
        --hardening)
            ENABLE_HARDENING=true
            shift
            ;;
        --apparmor)
            ENABLE_HARDENING=true
            HARDEN_APPARMOR=true
            shift
            ;;
        --firejail)
            ENABLE_HARDENING=true
            HARDEN_FIREJAIL=true
            shift
            ;;
        --flatpak)
            ENABLE_HARDENING=true
            HARDEN_FLATPAK=true
            shift
            ;;
        --nft)
            ENABLE_HARDENING=true
            HARDEN_NFT=true
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
# Skip build for 'sway' and standalone 'hardening'; build by default for 'niri' and 'all'
if [[ -z "${DO_BUILD}" ]]; then
    if [[ "${TARGET}" == "sway" || "${TARGET}" == "hardening" ]]; then
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
elif [[ "${TARGET}" == "hardening" ]]; then
    echo -e "${AMBER}       Target: Security Hardening Suite Deployment    ${NC}"
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
check_companion "carapace" "Multi-shell structured completion engine"
if [[ "${ENABLE_HARDENING}" == true ]]; then
    check_companion "firejail" "Firejail security sandbox"
    check_companion "flatpak" "Flatpak package manager & permissions"
    check_companion "apparmor_parser" "AppArmor profile parser/loader"
    check_companion "nft" "NFTables packet filtering tool"
fi
echo ""

# Sudo execution helper with fallback support
run_sudo() {
    local cmd=("$@")
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] sudo ${cmd[*]}"
        return 0
    fi
    if command -v sudo >/dev/null 2>&1; then
        if sudo -v 2>/dev/null || sudo -p "[dotfield:sudo] Enter password for system hardening (%u): " true 2>/dev/null; then
            sudo "${cmd[@]}"
            return $?
        fi
    fi
    return 1
}

deploy_apparmor() {
    step "Deploying AppArmor profile for background daemon..."
    local profile_src="${SCRIPT_DIR}/security/apparmor/dotfield-daemon"
    local profile_dest="/etc/apparmor.d/dotfield-daemon"
    local stage_dest="${HOME}/.config/dotfield/security/apparmor/dotfield-daemon"

    if [[ ! -f "${profile_src}" ]]; then
        warn "AppArmor profile not found at ${profile_src}"
        return 0
    fi

    mkdir -p "$(dirname "${stage_dest}")"
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] install -m 0644 ${profile_src} ${stage_dest}"
        info "[dry-run] sudo install -m 0644 -D ${profile_src} ${profile_dest}"
        info "[dry-run] sudo apparmor_parser -r ${profile_dest}"
        return 0
    fi

    cp "${profile_src}" "${stage_dest}"
    info "Staged AppArmor profile to ${stage_dest}"

    if ! command -v apparmor_parser >/dev/null 2>&1; then
        warn "apparmor_parser is not installed on this system. Staged profile locally."
        return 0
    fi

    if run_sudo install -m 0644 -D "${profile_src}" "${profile_dest}" && run_sudo apparmor_parser -r "${profile_dest}"; then
        success "AppArmor profile loaded: ${profile_dest}"
    else
        warn "Could not load AppArmor profile into /etc/apparmor.d with sudo."
        info "To activate manually: sudo cp ${stage_dest} ${profile_dest} && sudo apparmor_parser -r ${profile_dest}"
    fi
}

deploy_firejail() {
    step "Deploying Firejail sandbox profile and isolation storage..."
    local profile_src="${SCRIPT_DIR}/security/firejail/browser.profile"
    local profile_dest="${HOME}/.config/firejail/browser.profile"
    local sandbox_dir="${HOME}/.local/share/dotfield-sandbox"

    if [[ ! -f "${profile_src}" ]]; then
        warn "Firejail profile not found at ${profile_src}"
        return 0
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] mkdir -p ~/.config/firejail ${sandbox_dir}/downloads ${sandbox_dir}/browser-profile"
        info "[dry-run] install -m 0644 ${profile_src} ${profile_dest}"
        return 0
    fi

    mkdir -p "${HOME}/.config/firejail" "${sandbox_dir}/downloads" "${sandbox_dir}/browser-profile"
    install -m 0644 "${profile_src}" "${profile_dest}"
    success "Firejail browser profile deployed: ${profile_dest}"
    success "Isolated sandbox storage initialized at ${sandbox_dir}"
}

deploy_flatpak() {
    step "Applying Flatpak permission overrides..."
    local overrides_src="${SCRIPT_DIR}/security/flatpak/overrides.toml"
    local stage_dest="${HOME}/.config/dotfield/security/flatpak/overrides.toml"

    if [[ ! -f "${overrides_src}" ]]; then
        warn "Flatpak overrides file not found at ${overrides_src}"
        return 0
    fi

    mkdir -p "$(dirname "${stage_dest}")"
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] stage overrides to ${stage_dest}"
        info "[dry-run] flatpak override --user --socket=wayland --nosocket=x11 --nosocket=fallback-x11 --nosocket=pulseaudio --nosocket=ssh-auth --nosocket=gpg-agent --nofilesystem=home --nofilesystem=host --nofilesystem=host-os --nofilesystem=host-etc"
        return 0
    fi

    cp "${overrides_src}" "${stage_dest}"
    info "Staged Flatpak declarative overrides: ${stage_dest}"

    if command -v flatpak >/dev/null 2>&1; then
        flatpak override --user \
            --socket=wayland \
            --nosocket=x11 \
            --nosocket=fallback-x11 \
            --nosocket=pulseaudio \
            --nosocket=ssh-auth \
            --nosocket=gpg-agent \
            --nofilesystem=home \
            --nofilesystem=host \
            --nofilesystem=host-os \
            --nofilesystem=host-etc \
            >/dev/null 2>&1 || true
        success "Applied baseline Flatpak user overrides (Wayland-enforced, x11/host/home restricted)"
    else
        warn "Flatpak CLI is not installed. Staged declarative overrides configuration only."
    fi
}

deploy_nftables() {
    step "Deploying NFTables workstation stateful packet filter..."
    local nft_src="${SCRIPT_DIR}/security/nft/workstation.nft"
    local nft_dest="/etc/nftables.d/dotfield.nft"
    local stage_dest="${HOME}/.config/dotfield/security/nft/workstation.nft"

    if [[ ! -f "${nft_src}" ]]; then
        warn "NFTables ruleset not found at ${nft_src}"
        return 0
    fi

    mkdir -p "$(dirname "${stage_dest}")"
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] install -m 0644 ${nft_src} ${stage_dest}"
        info "[dry-run] sudo install -m 0644 -D ${nft_src} ${nft_dest}"
        info "[dry-run] sudo nft -f ${nft_dest}"
        return 0
    fi

    cp "${nft_src}" "${stage_dest}"
    info "Staged NFTables ruleset to ${stage_dest}"

    if ! command -v nft >/dev/null 2>&1; then
        warn "nft command (nftables) is not installed on this system. Staged ruleset locally."
        return 0
    fi

    if run_sudo install -m 0644 -D "${nft_src}" "${nft_dest}" && run_sudo nft -f "${nft_dest}"; then
        success "NFTables workstation stateful ruleset loaded: ${nft_dest}"
    else
        warn "Could not load NFTables ruleset into /etc/nftables.d with sudo."
        info "To activate manually: sudo cp ${stage_dest} ${nft_dest} && sudo nft -f ${nft_dest}"
    fi
}

# 1b. Hardening Deployment
if [[ "${ENABLE_HARDENING}" == true ]]; then
    # Interactive selection if no individual flag was specified
    if [[ "${HARDEN_APPARMOR}" == false && "${HARDEN_FIREJAIL}" == false && "${HARDEN_FLATPAK}" == false && "${HARDEN_NFT}" == false ]]; then
        if [[ -t 0 && "${DRY_RUN}" != true ]]; then
            echo -e "\n${AMBER}${BOLD}Dotfield Security Hardening Suite${NC}"
            echo -e "Choose the security layers to configure and deploy:"
            echo -e "  ${BOLD}1)${NC} All security layers (AppArmor + Firejail + Flatpak + NFTables)"
            echo -e "  ${BOLD}2)${NC} AppArmor profile only (dotfield-daemon)"
            echo -e "  ${BOLD}3)${NC} Firejail profile only (browser sandbox & isolated storage)"
            echo -e "  ${BOLD}4)${NC} Flatpak overrides only (restrict host/home/x11 permissions)"
            echo -e "  ${BOLD}5)${NC} NFTables firewall only (workstation stateful packet filter)"
            read -r -p "Enter selection [1-5] (default: 1): " H_CHOICE
            case "${H_CHOICE:-1}" in
                1)
                    HARDEN_APPARMOR=true
                    HARDEN_FIREJAIL=true
                    HARDEN_FLATPAK=true
                    HARDEN_NFT=true
                    ;;
                2)
                    HARDEN_APPARMOR=true
                    ;;
                3)
                    HARDEN_FIREJAIL=true
                    ;;
                4)
                    HARDEN_FLATPAK=true
                    ;;
                5)
                    HARDEN_NFT=true
                    ;;
                *)
                    warn "Unrecognized selection '${H_CHOICE}', defaulting to all layers."
                    HARDEN_APPARMOR=true
                    HARDEN_FIREJAIL=true
                    HARDEN_FLATPAK=true
                    HARDEN_NFT=true
                    ;;
            esac
        else
            HARDEN_APPARMOR=true
            HARDEN_FIREJAIL=true
            HARDEN_FLATPAK=true
            HARDEN_NFT=true
        fi
    fi

    echo -e "\n${BOLD}======================================================${NC}"
    echo -e "${BOLD}       Deploying Security Hardening Layers            ${NC}"
    echo -e "${BOLD}======================================================${NC}\n"

    if [[ "${HARDEN_FIREJAIL}" == true ]]; then
        deploy_firejail
    fi

    if [[ "${HARDEN_FLATPAK}" == true ]]; then
        deploy_flatpak
    fi

    if [[ "${HARDEN_APPARMOR}" == true ]]; then
        deploy_apparmor
    fi

    if [[ "${HARDEN_NFT}" == true ]]; then
        deploy_nftables
    fi

    if [[ "${TARGET}" == "hardening" ]]; then
        echo -e "\n${GREEN}${BOLD}======================================================${NC}"
        echo -e "${GREEN}${BOLD}       Security Hardening Completed Successfully!     ${NC}"
        echo -e "${GREEN}${BOLD}======================================================${NC}\n"
        exit 0
    fi
fi

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
    mkdir -p "${SCRIPTS_DEST}/lib"
    for s in "${SCRIPT_DIR}/scripts"/*.sh; do
        if [[ -f "${s}" ]]; then
            install -m 0755 "${s}" "${SCRIPTS_DEST}/$(basename "${s}")"
        fi
    done
    if [[ -d "${SCRIPT_DIR}/scripts/lib" ]]; then
        for lib in "${SCRIPT_DIR}/scripts/lib"/*.sh; do
            if [[ -f "${lib}" ]]; then
                install -m 0755 "${lib}" "${SCRIPTS_DEST}/lib/$(basename "${lib}")"
            fi
        done
    fi
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
ensure_config_link_or_dir "${SCRIPT_DIR}/wireplumber" "${HOME}/.config/wireplumber"
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

# Install fzf-tab (Tab completion via fzf — no prompt-jumping, themed popup)
FZF_TAB_DIR="${HOME}/.local/share/zsh/plugins/fzf-tab"
if [[ ! -d "${FZF_TAB_DIR}/.git" ]]; then
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] git clone --depth=1 https://github.com/Aloxaf/fzf-tab ${FZF_TAB_DIR}"
    else
        info "Installing fzf-tab …"
        mkdir -p "$(dirname "${FZF_TAB_DIR}")"
        if git clone --depth=1 https://github.com/Aloxaf/fzf-tab "${FZF_TAB_DIR}" 2>/dev/null; then
            success "fzf-tab installed at ${FZF_TAB_DIR}"
        else
            warn "fzf-tab clone failed — Tab completions will fall back to standard menu"
        fi
    fi
else
    success "fzf-tab already present at ${FZF_TAB_DIR}"
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

if [[ "${ENABLE_HARDENING}" == true ]]; then
    echo -e "${AMBER}${BOLD}Active Security Hardening Layers:${NC}"
    [[ "${HARDEN_APPARMOR}" == true ]] && echo -e "  ${GREEN}✔${NC} AppArmor Daemon Confinement (/etc/apparmor.d/dotfield-daemon)"
    [[ "${HARDEN_FIREJAIL}" == true ]] && echo -e "  ${GREEN}✔${NC} Firejail Untrusted App Sandbox (~/.config/firejail/browser.profile)"
    [[ "${HARDEN_FLATPAK}" == true ]] && echo -e "  ${GREEN}✔${NC} Flatpak Declarative Permission Overrides"
    [[ "${HARDEN_NFT}" == true ]] && echo -e "  ${GREEN}✔${NC} NFTables Workstation Stateful Packet Filter"
    echo ""
fi
