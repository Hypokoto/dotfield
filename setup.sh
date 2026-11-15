#!/usr/bin/env bash
# Dotfield Environment Setup & Deployment Script
# Idempotent bootstrap for Sway, Niri, Amane Shell, and desktop tools
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

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
HARDEN_FLATPAK=false
HARDEN_NFT=false
USE_SYMLINKS=true
declare -A LAYER_STATUS=()
declare -A LAYER_DETAILS=()

show_help() {
    cat <<EOF
Dotfield Setup & Deployment Installer

Usage: ./setup.sh [SUBCOMMAND] [OPTIONS]

Subcommands:
  sway                   Deploy configuration specifically for Sway compositor
  niri                   Deploy configuration specifically for Niri + Amane desktop
  all                    Deploy configurations for both Sway and Niri (default)
  hardening              Deploy security hardening suite (AppArmor, Flatpak, NFTables)

Options:
  --sway                 Target Sway compositor
  --niri                 Target Niri compositor
  --all                  Target both Sway and Niri (default)
  --hardening            Enable security hardening suite deployment
  --apparmor             Deploy & activate AppArmor daemon confinement profile
  --flatpak              Apply baseline restrictive Flatpak permission overrides
  --nft                  Deploy & activate NFTables workstation stateful filter
  -f, --force            Force replacement of existing configs (backed up to .bak.<timestamp>)
  --copy                 Deploy files as copies instead of Stow-style symlinks
  --prefix <dir>         Directory to install binaries into (default: ~/.local/bin)
  --build                Force compilation of Rust crate (amane)
  --no-build             Skip compiling Rust crate with cargo
  -d, --dry-run          Preview installation steps without executing changes
  -h, --help             Display this help message and exit

Examples:
  ./setup.sh sway
  ./setup.sh --niri
  ./setup.sh hardening
  ./setup.sh --hardening --apparmor --nft
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
        --copy)
            USE_SYMLINKS=false
            shift
            ;;
        --prefix)
            if [[ $# -lt 2 || -z "${2:-}" ]]; then
                err "Option --prefix requires a non-empty directory argument."
                exit 1
            fi
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
check_companion "wezterm" "WebGPU terminal emulator"
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
    check_companion "bwrap" "Bubblewrap unprivileged sandbox"
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
        # Check if already authenticated non-interactively
        if sudo -n true 2>/dev/null; then
            sudo "${cmd[@]}"
            return $?
        fi
        # If interactive TTY, prompt user with explicit prompt without hiding stderr
        if [[ -t 0 ]]; then
            if sudo -p "[dotfield:sudo] Enter password for system privilege (%u): " "${cmd[@]}"; then
                return 0
            fi
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
        LAYER_STATUS["apparmor"]="FAIL"
        LAYER_DETAILS["apparmor"]="Profile not found: ${profile_src}"
        err "AppArmor profile not found at ${profile_src}"
        return 1
    fi

    # Pre-flight check: kernel LSM enablement
    local lsm_path="/sys/module/apparmor/parameters/enabled"
    if [[ ! -r "${lsm_path}" ]] || [[ "$(< "${lsm_path}")" != "Y" ]]; then
        LAYER_STATUS["apparmor"]="FAIL"
        LAYER_DETAILS["apparmor"]="Kernel LSM 'apparmor' parameter is not 'Y' (disabled at boot)"
        err "AppArmor is disabled in the kernel (check /sys/module/apparmor/parameters/enabled)."
        info "Add 'lsm=landlock,lockdown,yama,apparmor,bpf' to your kernel boot parameters."
        return 1
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] install -m 0644 ${profile_src} ${stage_dest}"
        info "[dry-run] sudo install -m 0644 -D ${profile_src} ${profile_dest}"
        info "[dry-run] sudo apparmor_parser -r ${profile_dest}"
        LAYER_STATUS["apparmor"]="DRY_RUN"
        LAYER_DETAILS["apparmor"]="Dry-run preview (not applied)"
        return 0
    fi

    mkdir -p "$(dirname "${stage_dest}")"
    cp "${profile_src}" "${stage_dest}"
    info "Staged AppArmor profile to ${stage_dest}"

    if ! command -v apparmor_parser >/dev/null 2>&1; then
        LAYER_STATUS["apparmor"]="FAIL"
        LAYER_DETAILS["apparmor"]="apparmor_parser binary not installed"
        err "apparmor_parser is not installed on this system."
        return 1
    fi

    if ! run_sudo install -m 0644 -D "${profile_src}" "${profile_dest}"; then
        LAYER_STATUS["apparmor"]="FAIL"
        LAYER_DETAILS["apparmor"]="Failed to install profile into /etc/apparmor.d (sudo failed)"
        err "Could not install AppArmor profile into /etc/apparmor.d."
        return 1
    fi

    if ! run_sudo apparmor_parser -r "${profile_dest}"; then
        LAYER_STATUS["apparmor"]="FAIL"
        LAYER_DETAILS["apparmor"]="apparmor_parser failed to parse/load profile"
        err "apparmor_parser failed to load ${profile_dest}."
        return 1
    fi

    # Verification: check kernel security profiles directly via securityfs or sudo
    local active_mode=""
    for pdir in /sys/kernel/security/apparmor/policy/profiles/dotfield-daemon.*; do
        if [[ -d "${pdir}" && -f "${pdir}/name" ]]; then
            if [[ "$(< "${pdir}/name")" == "dotfield-daemon" ]]; then
                active_mode="$(< "${pdir}/mode")"
                break
            fi
        fi
    done

    if [[ -n "${active_mode}" ]] || \
       run_sudo grep -q "dotfield-daemon" /sys/kernel/security/apparmor/profiles 2>/dev/null || \
       (command -v aa-status >/dev/null 2>&1 && run_sudo aa-status 2>/dev/null | grep -q "dotfield-daemon"); then
        success "AppArmor profile loaded and kernel confinement verified (dotfield-daemon in ${active_mode:-complain} mode)"
        LAYER_STATUS["apparmor"]="PASS"
        LAYER_DETAILS["apparmor"]="Active kernel profile (dotfield-daemon) in ${active_mode:-complain} mode"
        return 0
    else
        LAYER_STATUS["apparmor"]="FAIL"
        LAYER_DETAILS["apparmor"]="apparmor_parser ran but profile missing from /sys/kernel/security/apparmor/profiles"
        err "AppArmor profile dotfield-daemon not found in active kernel profiles."
        return 1
    fi
}

deploy_flatpak() {
    step "Applying Flatpak permission overrides..."
    local overrides_src="${SCRIPT_DIR}/security/flatpak/global"
    local stage_dest="${HOME}/.config/dotfield/security/flatpak/global"
    local global_override_dest="${HOME}/.local/share/flatpak/overrides/global"

    if [[ ! -f "${overrides_src}" ]]; then
        LAYER_STATUS["flatpak"]="FAIL"
        LAYER_DETAILS["flatpak"]="Flatpak overrides source not found: ${overrides_src}"
        err "Flatpak overrides file not found at ${overrides_src}"
        return 1
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] stage overrides to ${stage_dest} and ${global_override_dest}"
        info "[dry-run] flatpak override --user --socket=wayland --nosocket=x11 --nosocket=fallback-x11 --nosocket=ssh-auth --nosocket=gpg-agent --nofilesystem=home --nofilesystem=host --nofilesystem=host-os --nofilesystem=host-etc"
        LAYER_STATUS["flatpak"]="DRY_RUN"
        LAYER_DETAILS["flatpak"]="Dry-run preview (not applied)"
        return 0
    fi

    if ! command -v flatpak >/dev/null 2>&1; then
        LAYER_STATUS["flatpak"]="FAIL"
        LAYER_DETAILS["flatpak"]="flatpak binary not installed"
        err "Flatpak CLI is not installed on this system."
        return 1
    fi

    mkdir -p "$(dirname "${stage_dest}")" "${HOME}/.local/share/flatpak/overrides"
    if [[ -f "${global_override_dest}" ]]; then
        local ts
        ts="$(date +%Y%m%d%H%M%S)"
        cp "${global_override_dest}" "${global_override_dest}.bak.${ts}"
        info "Backed up existing Flatpak global override to ${global_override_dest}.bak.${ts}"
    fi
    cp "${overrides_src}" "${stage_dest}"
    cp "${overrides_src}" "${global_override_dest}"
    info "Deployed native Flatpak global override: ${global_override_dest}"

    # Also register via flatpak CLI to ensure user database synchronization
    if flatpak override --user \
        --socket=wayland \
        --nosocket=x11 \
        --nosocket=fallback-x11 \
        --nosocket=ssh-auth \
        --nosocket=gpg-agent \
        --nofilesystem=home \
        --nofilesystem=host \
        --nofilesystem=host-os \
        --nofilesystem=host-etc 2>&1; then
        success "Applied baseline Flatpak user overrides (Wayland-enforced, host/home restricted)"
        info "Note: X11-only Flatpaks require per-app 'flatpak override --socket=x11 <app-id>'"
        LAYER_STATUS["flatpak"]="PASS"
        LAYER_DETAILS["flatpak"]="Applied to ${global_override_dest} and flatpak user database"
        return 0
    else
        LAYER_STATUS["flatpak"]="FAIL"
        LAYER_DETAILS["flatpak"]="flatpak override command failed"
        err "Failed to apply Flatpak overrides."
        return 1
    fi
}

deploy_nftables() {
    step "Deploying NFTables workstation stateful packet filter..."
    local nft_src="${SCRIPT_DIR}/security/nft/workstation.nft"
    local nft_dest="/etc/nftables.d/dotfield.nft"
    local stage_dest="${HOME}/.config/dotfield/security/nft/workstation.nft"

    if [[ ! -f "${nft_src}" ]]; then
        LAYER_STATUS["nft"]="FAIL"
        LAYER_DETAILS["nft"]="Ruleset source not found: ${nft_src}"
        err "NFTables ruleset not found at ${nft_src}"
        return 1
    fi

    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] install -m 0644 ${nft_src} ${stage_dest}"
        info "[dry-run] sudo install -m 0644 -D ${nft_src} ${nft_dest}"
        info "[dry-run] sudo nft -c -f ${nft_src}"
        info "[dry-run] sudo nft -f ${nft_dest}"
        LAYER_STATUS["nft"]="DRY_RUN"
        LAYER_DETAILS["nft"]="Dry-run preview (not applied)"
        return 0
    fi

    mkdir -p "$(dirname "${stage_dest}")"
    cp "${nft_src}" "${stage_dest}"
    info "Staged NFTables ruleset to ${stage_dest}"

    if ! command -v nft >/dev/null 2>&1; then
        LAYER_STATUS["nft"]="FAIL"
        LAYER_DETAILS["nft"]="nft binary not installed"
        err "nft command (nftables) is not installed on this system."
        return 1
    fi

    # Pre-flight syntax validation
    step "Validating NFTables ruleset syntax with pre-flight check..."
    if ! run_sudo nft -c -f "${nft_src}"; then
        LAYER_STATUS["nft"]="FAIL"
        LAYER_DETAILS["nft"]="nft -c -f syntax check failed"
        err "NFTables ruleset syntax validation failed."
        return 1
    fi

    if ! run_sudo install -m 0644 -D "${nft_src}" "${nft_dest}"; then
        LAYER_STATUS["nft"]="FAIL"
        LAYER_DETAILS["nft"]="Failed to install ruleset into /etc/nftables.d (sudo failed)"
        err "Could not install ruleset to ${nft_dest}."
        return 1
    fi

    # Ensure persistence across reboots via /etc/nftables.conf include
    local nft_main_conf="/etc/nftables.conf"
    local include_stmt='include "/etc/nftables.d/*.nft"'
    if [[ -f "${nft_main_conf}" ]]; then
        if ! grep -qF 'include "/etc/nftables.d/' "${nft_main_conf}" 2>/dev/null; then
            step "Hooking persistence include into ${nft_main_conf}..."
            local ts
            ts="$(date +%Y%m%d%H%M%S)"
            run_sudo cp "${nft_main_conf}" "${nft_main_conf}.bak.${ts}"
            echo -e "\n# Dotfield workstation modular ruleset include\n${include_stmt}" | run_sudo tee -a "${nft_main_conf}" >/dev/null
            info "Appended include statement to ${nft_main_conf} (backup created at ${nft_main_conf}.bak.${ts})"
        fi
    else
        step "Creating initial ${nft_main_conf} with persistence include..."
        echo -e "#!/usr/sbin/nft -f\nflush ruleset\n${include_stmt}" | run_sudo tee "${nft_main_conf}" >/dev/null
        run_sudo chmod 0644 "${nft_main_conf}"
    fi

    # Enable systemd nftables service if present
    run_sudo systemctl enable nftables.service 2>/dev/null || true

    if ! run_sudo nft -f "${nft_dest}"; then
        LAYER_STATUS["nft"]="FAIL"
        LAYER_DETAILS["nft"]="nft -f failed to load ruleset into kernel"
        err "Failed to load NFTables ruleset."
        return 1
    fi

    # Verification: check that table inet dotfield is active in kernel and persistence hook exists
    if run_sudo nft list table inet dotfield >/dev/null 2>&1; then
        local persisted=false
        if [[ -f "${nft_main_conf}" ]] && grep -qF 'include "/etc/nftables.d/' "${nft_main_conf}" 2>/dev/null; then
            persisted=true
        fi
        if [[ "${persisted}" == true ]]; then
            success "NFTables stateful filter active in kernel and verified persistent across reboot"
            LAYER_STATUS["nft"]="PASS"
            LAYER_DETAILS["nft"]="Active in kernel (table inet dotfield) & persisted via ${nft_main_conf}"
            return 0
        else
            warn "NFTables ruleset active in kernel, but persistence include missing from ${nft_main_conf}"
            LAYER_STATUS["nft"]="WARN"
            LAYER_DETAILS["nft"]="Active in kernel (table inet dotfield), but unpersisted (missing from ${nft_main_conf})"
            return 0
        fi
    else
        LAYER_STATUS["nft"]="FAIL"
        LAYER_DETAILS["nft"]="Table inet dotfield not found in active ruleset after load"
        err "NFTables ruleset verification failed: table inet dotfield not found."
        return 1
    fi
}

# 1b. Hardening Deployment
if [[ "${ENABLE_HARDENING}" == true ]]; then
    # Interactive selection if no individual flag was specified
    if [[ "${HARDEN_APPARMOR}" == false && "${HARDEN_FLATPAK}" == false && "${HARDEN_NFT}" == false ]]; then
        if [[ -t 0 && "${DRY_RUN}" != true ]]; then
            echo -e "\n${AMBER}${BOLD}Dotfield Security Hardening Suite${NC}"
            echo -e "Choose the security layers to configure and deploy:"
            echo -e "  ${BOLD}1)${NC} All security layers (AppArmor + Flatpak + NFTables)"
            echo -e "  ${BOLD}2)${NC} AppArmor profile only (amane daemon confinement)"
            echo -e "  ${BOLD}3)${NC} Flatpak overrides only (restrict host/home/x11 permissions)"
            echo -e "  ${BOLD}4)${NC} NFTables firewall only (workstation stateful packet filter)"
            echo -e "  ${BOLD}5)${NC} Abort without applying (Default)"
            read -r -p "Enter selection [1-5] (default: 5 - safe exit): " H_CHOICE
            case "${H_CHOICE:-5}" in
                1)
                    HARDEN_APPARMOR=true
                    HARDEN_FLATPAK=true
                    HARDEN_NFT=true
                    ;;
                2)
                    HARDEN_APPARMOR=true
                    ;;
                3)
                    HARDEN_FLATPAK=true
                    ;;
                4)
                    HARDEN_NFT=true
                    ;;
                *)
                    info "Safe abort: No security hardening layers selected."
                    exit 0
                    ;;
            esac
        else
            err "Non-interactive hardening requires explicit layer flags (e.g. --apparmor, --flatpak, --nft)."
            err "Refusing to apply default firewall or sandbox rules without explicit flags."
            exit 1
        fi
    fi

    echo -e "\n${BOLD}======================================================${NC}"
    echo -e "${BOLD}       Deploying Security Hardening Layers            ${NC}"
    echo -e "${BOLD}======================================================${NC}\n"

    hardening_has_failures=false

    if [[ "${HARDEN_FLATPAK}" == true ]]; then
        deploy_flatpak || hardening_has_failures=true
    fi

    if [[ "${HARDEN_APPARMOR}" == true ]]; then
        deploy_apparmor || hardening_has_failures=true
    fi

    if [[ "${HARDEN_NFT}" == true ]]; then
        deploy_nftables || hardening_has_failures=true
    fi

    echo -e "\n${BOLD}Security Hardening Verification Summary:${NC}"
    for layer in "${!LAYER_STATUS[@]}"; do
        layer_status="${LAYER_STATUS[$layer]}"
        layer_detail="${LAYER_DETAILS[$layer]:-}"
        if [[ "${layer_status}" == "PASS" ]]; then
            echo -e "  ${GREEN}✔ PASS${NC} [${layer}] ${layer_detail}"
        elif [[ "${layer_status}" == "DRY_RUN" ]]; then
            echo -e "  ${BLUE}○ PREVIEW${NC} [${layer}] ${layer_detail}"
        elif [[ "${layer_status}" == "WARN" ]]; then
            echo -e "  ${AMBER}! WARN${NC} [${layer}] ${layer_detail}"
        else
            echo -e "  ${RED}✘ FAIL${NC} [${layer}] ${layer_detail}"
        fi
    done

    if [[ "${hardening_has_failures}" == true ]]; then
        err "One or more security hardening layers failed verification."
        if [[ "${TARGET}" == "hardening" ]]; then
            exit 1
        fi
    else
        success "All requested security hardening layers verified successfully."
        if [[ "${TARGET}" == "hardening" ]]; then
            exit 0
        fi
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
        if [[ -f "${SCRIPT_DIR}/target/release/amane-msg" ]]; then
            install -m 0755 "${SCRIPT_DIR}/target/release/amane-msg" "${PREFIX}/amane-msg"
        fi
        if [[ -d "${HOME}/.cargo/bin" ]]; then
            ln -sf "${PREFIX}/amane" "${HOME}/.cargo/bin/amane"
            ln -sf "${PREFIX}/amane-msg" "${HOME}/.cargo/bin/amane-msg"
        fi
        success "Installed amane and amane-msg to ${PREFIX} (and ~/.cargo/bin symlinks)"
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
    info "[dry-run] mkdir -p ${SCRIPTS_DEST} ${PREFIX}"
    info "[dry-run] install -m 0755 scripts/* ${SCRIPTS_DEST}/"
    info "[dry-run] ln -sf ${SCRIPTS_DEST}/* ${PREFIX}/"
else
    mkdir -p "${PREFIX}"
    mkdir -p "${SCRIPTS_DEST}/lib"
    for s in "${SCRIPT_DIR}/scripts"/*.sh "${SCRIPT_DIR}/scripts/fw-open"; do
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
    ln -sf "${SCRIPTS_DEST}/gtk-sync.sh" "${PREFIX}/dotfield-gtk-sync" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/screenshot.sh" "${PREFIX}/dotfield-screenshot" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/screen-record.sh" "${PREFIX}/dotfield-record" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/sandbox-browser.sh" "${PREFIX}/sandbox-browser" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/sandbox-browser.sh" "${PREFIX}/dotfield-sandbox-browser" 2>/dev/null || true
    ln -sf "${SCRIPTS_DEST}/fw-open" "${PREFIX}/fw-open" 2>/dev/null || true
    success "Helper scripts deployed to ${SCRIPTS_DEST}"
fi

# 5. Deploy base configurations
step "Setting up configuration directory skeletons in ~/.config..."
ensure_config_link_or_dir() {
    local src="$1"
    local dest="$2"

    # 1. Guard against missing source configuration
    if [[ ! -e "${src}" ]]; then
        warn "Missing source configuration: ${src}"
        return 0
    fi

    # 2. Clean dangling broken symlinks
    if [[ -L "${dest}" && ! -e "${dest}" ]]; then
        if [[ "${DRY_RUN}" == true ]]; then
            info "[dry-run] rm -f broken symlink: ${dest}"
        else
            info "Cleaning dangling broken symlink: ${dest}"
            rm -f "${dest}"
        fi
    fi

    if [[ "${USE_SYMLINKS}" == true ]]; then
        # Stow-style symlink mode
        if [[ -L "${dest}" ]]; then
            local current_target
            current_target="$(readlink -f "${dest}" 2>/dev/null || true)"
            local expected_target
            expected_target="$(readlink -f "${src}" 2>/dev/null || true)"
            if [[ -n "${current_target}" && "${current_target}" == "${expected_target}" ]]; then
                info "Already synchronized: ${dest} -> ${src}"
                return 0
            fi
        fi

        if [[ "${DRY_RUN}" == true ]]; then
            if [[ -e "${dest}" || -L "${dest}" ]]; then
                info "[dry-run] backup ${dest} -> ${dest}.bak.<timestamp> and symlink -> ${src}"
            else
                info "[dry-run] mkdir -p $(dirname "${dest}") && ln -sfn ${src} ${dest}"
            fi
            return 0
        fi

        mkdir -p "$(dirname "${dest}")"
        if [[ -e "${dest}" || -L "${dest}" ]]; then
            local ts
            ts=$(date +%s)
            mv "${dest}" "${dest}.bak.${ts}"
            info "Backed up existing $(basename "${dest}") to ${dest}.bak.${ts}"
        fi
        ln -sfn "${src}" "${dest}"
        success "Linked: ${dest} -> ${src}"
    else
        # Copy mode (--copy)
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
            if [[ "${DRY_RUN}" == true ]]; then
                info "[dry-run] skipped existing config (use --force to overwrite): ${dest}"
            else
                info "Skipped existing config (use --force to overwrite): ${dest}"
            fi
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
ensure_config_link_or_dir "${SCRIPT_DIR}/terminals/wezterm" "${HOME}/.config/wezterm"
ensure_config_link_or_dir "${SCRIPT_DIR}/mako" "${HOME}/.config/mako"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swayosd" "${HOME}/.config/swayosd"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/avizo" "${HOME}/.config/avizo"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swaylock" "${HOME}/.config/swaylock"
ensure_config_link_or_dir "${SCRIPT_DIR}/osd/swayidle" "${HOME}/.config/swayidle"
ensure_config_link_or_dir "${SCRIPT_DIR}/gtk/gtk-3.0" "${HOME}/.config/gtk-3.0"
ensure_config_link_or_dir "${SCRIPT_DIR}/gtk/gtk-4.0" "${HOME}/.config/gtk-4.0"
ensure_config_link_or_dir "${SCRIPT_DIR}/ly" "${HOME}/.config/dotfield/ly"
ensure_config_link_or_dir "${SCRIPT_DIR}/nvim" "${HOME}/.config/nvim"
ensure_config_link_or_dir "${SCRIPT_DIR}/multiplexer/tmux" "${HOME}/.config/tmux"
ensure_config_link_or_dir "${SCRIPT_DIR}/multiplexer/zellij" "${HOME}/.config/zellij"
ensure_config_link_or_dir "${SCRIPT_DIR}/wlogout" "${HOME}/.config/wlogout"
ensure_config_link_or_dir "${SCRIPT_DIR}/fastfetch" "${HOME}/.config/fastfetch"
ensure_config_link_or_dir "${SCRIPT_DIR}/shell/starship/starship.toml" "${HOME}/.config/starship.toml"
ensure_config_link_or_dir "${SCRIPT_DIR}/shell/zsh/.zshrc" "${HOME}/.zshrc"

# Compile/prime Neovim NvChad base46 highlights if nvim is available
if [[ "${DRY_RUN}" != true ]] && command -v nvim >/dev/null 2>&1; then
    info "Pre-compiling Neovim NvChad base46 highlights..."
    nvim --headless -c "lua pcall(function() require('base46').load_all_highlights() end)" -c "qa" >/dev/null 2>&1 || true
fi

# Synchronize GTK interface & dark mode preferences
if [[ "${DRY_RUN}" != true && -f "${SCRIPT_DIR}/scripts/gtk-sync.sh" ]]; then
    bash "${SCRIPT_DIR}/scripts/gtk-sync.sh" || true
fi

# Deploy systemd user service and environment for amane (Niri)
if [[ "${TARGET}" == "all" || "${TARGET}" == "niri" ]]; then
    if [[ "${DRY_RUN}" != true ]]; then
        mkdir -p "${HOME}/.config/systemd/user" "${HOME}/.config/environment.d" 2>/dev/null || true
    fi
    if [[ ! -f "${HOME}/.config/environment.d/10-dotfield-path.conf" ]]; then
        if [[ "${DRY_RUN}" != true ]]; then
            # shellcheck disable=SC2016
            echo 'PATH=${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}' > "${HOME}/.config/environment.d/10-dotfield-path.conf"
        fi
    fi
    ensure_config_link_or_dir "${SCRIPT_DIR}/systemd/user/amane.service" "${HOME}/.config/systemd/user/amane.service"
    ensure_config_link_or_dir "${SCRIPT_DIR}/systemd/user/kanshi.service" "${HOME}/.config/systemd/user/kanshi.service"
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

# Detect Ly display manager and advise on system configuration sync
if [[ -d "/etc/ly" && -f "${SCRIPT_DIR}/ly/config.ini" ]]; then
    if ! cmp -s "${SCRIPT_DIR}/ly/config.ini" "/etc/ly/config.ini" 2>/dev/null; then
        info "Ly display manager detected at /etc/ly. To sync BLACKSITE theme, run: sudo cp ${SCRIPT_DIR}/ly/config.ini /etc/ly/config.ini"
    fi
fi

# Install fzf-tab (Tab completion via fzf: no prompt-jumping, themed popup)
FZF_TAB_DIR="${HOME}/.local/share/zsh/plugins/fzf-tab"
FZF_TAB_REV="d7e0234614dbe5369fdd760907d12c0e05a4dccc" # v1.3.0 pinned
if [[ ! -d "${FZF_TAB_DIR}/.git" ]]; then
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] git clone https://github.com/Aloxaf/fzf-tab ${FZF_TAB_DIR} && git checkout ${FZF_TAB_REV}"
    else
        info "Installing fzf-tab (pinned: ${FZF_TAB_REV:0:7}) …"
        mkdir -p "$(dirname "${FZF_TAB_DIR}")"
        if git clone https://github.com/Aloxaf/fzf-tab "${FZF_TAB_DIR}" 2>/dev/null && (cd "${FZF_TAB_DIR}" && git checkout -q "${FZF_TAB_REV}" 2>/dev/null); then
            success "fzf-tab installed at ${FZF_TAB_DIR} (pinned to ${FZF_TAB_REV:0:7})"
        else
            warn "fzf-tab clone/checkout failed: Tab completions will fall back to standard menu"
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
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] ln -sfn ${SCRIPT_DIR}/Wallpapers ${HOME}/.config/dotfield/background/Wallpapers"
    else
        mkdir -p "${HOME}/.config/dotfield/background"
        if [[ -d "${HOME}/.config/dotfield/background/Wallpapers" && ! -L "${HOME}/.config/dotfield/background/Wallpapers" ]]; then
            cp -rn "${SCRIPT_DIR}/Wallpapers/"* "${HOME}/.config/dotfield/background/Wallpapers/" 2>/dev/null || true
        else
            ln -sfn "${SCRIPT_DIR}/Wallpapers" "${HOME}/.config/dotfield/background/Wallpapers"
        fi
    fi
fi
if [[ -d "${SCRIPT_DIR}/Wallpapers/Gifs" ]]; then
    if [[ "${DRY_RUN}" == true ]]; then
        info "[dry-run] ln -sfn ${SCRIPT_DIR}/Wallpapers/Gifs ${HOME}/.config/dotfield/background/Gifs"
    else
        mkdir -p "${HOME}/.config/dotfield/background"
        ln -sfn "${SCRIPT_DIR}/Wallpapers/Gifs" "${HOME}/.config/dotfield/background/Gifs"
    fi
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
            SWAYSOCK="${SWAYSOCK:-$(find "/run/user/$(id -u)" -maxdepth 1 -name "sway-ipc.*.sock" 2>/dev/null | head -n 1)}" swaymsg reload >/dev/null 2>&1 || true
            info "Reloaded Sway configuration"
        fi
    fi

    if command -v makoctl >/dev/null 2>&1 && pgrep -x mako >/dev/null 2>&1; then
        makoctl reload >/dev/null 2>&1 || true
    fi
fi

# 7. Summary & Quick Guide
if [[ "${hardening_has_failures:-false}" == true ]]; then
    echo -e "\n${RED}${BOLD}======================================================${NC}"
    echo -e "${RED}${BOLD}    Dotfield Setup Finished with Hardening Failures   ${NC}"
    echo -e "${RED}${BOLD}======================================================${NC}\n"
else
    echo -e "\n${GREEN}${BOLD}======================================================${NC}"
    echo -e "${GREEN}${BOLD}       Dotfield Setup Completed Successfully!         ${NC}"
    echo -e "${GREEN}${BOLD}======================================================${NC}\n"
fi

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
    echo -e "${AMBER}${BOLD}Security Hardening Verification Status:${NC}"
    for layer in "${!LAYER_STATUS[@]}"; do
        layer_status="${LAYER_STATUS[$layer]}"
        layer_detail="${LAYER_DETAILS[$layer]:-}"
        if [[ "${layer_status}" == "PASS" ]]; then
            echo -e "  ${GREEN}✔ PASS${NC} [${layer}] ${layer_detail}"
        elif [[ "${layer_status}" == "DRY_RUN" ]]; then
            echo -e "  ${BLUE}○ PREVIEW${NC} [${layer}] ${layer_detail}"
        elif [[ "${layer_status}" == "WARN" ]]; then
            echo -e "  ${AMBER}! WARN${NC} [${layer}] ${layer_detail}"
        else
            echo -e "  ${RED}✘ FAIL${NC} [${layer}] ${layer_detail}"
        fi
    done
    echo ""
fi

if [[ "${hardening_has_failures:-false}" == true ]]; then
    err "One or more security hardening layers failed verification."
    exit 1
fi
