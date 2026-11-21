#!/usr/bin/env bash
# Dotfield Network Diagnostic & Auto-Healing Doctor
# Checks Wi-Fi power saving, interface route metrics, DNS latency, ASPM, and roaming status
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

export PATH="/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin:${PATH:-}"

# ANSI color styling (Controlled Chaos palette)
RED='\033[0;31m'
GREEN='\033[0;32m'
AMBER='\033[0;33m'
BLUE='\033[0;34m'
STEEL='\033[0;90m'
BOLD='\033[1m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}${BOLD}[net-doctor:info]${NC} $*"
}

step() {
    echo -e "\n${AMBER}${BOLD}==>${NC} ${BOLD}$*${NC}"
}

success() {
    echo -e "  ${GREEN}${BOLD}✔${NC} $*"
}

warn() {
    echo -e "  ${AMBER}${BOLD}!${NC} $*"
}

err() {
    echo -e "  ${RED}${BOLD}✘${NC} $*"
}

detail() {
    echo -e "    ${STEEL}•${NC} $*"
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DO_FIX=false
CRITICAL_ERRORS=0
WARNINGS=0

show_help() {
    cat <<EOF
Dotfield Network Diagnostic & Auto-Healing Doctor

Usage: $(basename "$0") [OPTIONS]

Options:
  --fix          Automatically fix detected issues (disable powersave, fix metrics, add fallback DNS)
  -h, --help     Display this help message and exit

Checks performed:
  1. Default route priority and USB tethering route hijack detection
  2. Wi-Fi 802.11 power saving status
  3. MediaTek MT7922 / MT7921 PCIe ASPM sleep status
  4. Dual-band roaming, frequency, and link bitrate health
  5. DNS resolver latency & fallback availability
  6. End-to-end ping & IPv6 routing validation
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --fix)
            DO_FIX=true
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

echo -e "${BOLD}Dotfield Network Diagnostic Doctor${NC}"
echo -e "${STEEL}Auditing interface priorities, driver sleep states, and routing tables...${NC}"

# Find active Wi-Fi interface and connection
WLAN_IFACE=$(ip -o link show | awk -F': ' '$2 ~ /^wl/ {print $2}' | head -n 1 || true)
WLAN_CONN=""
if [[ -n "${WLAN_IFACE}" ]] && command -v nmcli >/dev/null 2>&1; then
    WLAN_CONN=$(nmcli -t -f DEVICE,NAME connection show --active | awk -F':' -v dev="${WLAN_IFACE}" '$1 == dev {print $2}' | head -n 1 || true)
fi

# ---------------------------------------------------------
# Check 1: Default Route Priority & Interface Metric Audit
# ---------------------------------------------------------
step "Auditing default route and metric hierarchy..."
DEFAULT_ROUTES=$(ip route show default 2>/dev/null || true)

if [[ -z "${DEFAULT_ROUTES}" ]]; then
    err "No default gateway found. System has no internet routing."
    CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
else
    PRIMARY_DEV=$(echo "${DEFAULT_ROUTES}" | head -n 1 | awk '{for(i=1;i<=NF;i++) if($i=="dev") print $(i+1)}')
    PRIMARY_METRIC=$(echo "${DEFAULT_ROUTES}" | head -n 1 | awk '{for(i=1;i<=NF;i++) if($i=="metric") print $(i+1)}')
    PRIMARY_METRIC=${PRIMARY_METRIC:-0}

    detail "Primary default interface: ${BOLD}${PRIMARY_DEV}${NC} (metric: ${PRIMARY_METRIC})"

    # Check if a non-Wi-Fi USB interface is hijacking the route over Wi-Fi
    if [[ -n "${WLAN_IFACE}" ]] && [[ "${PRIMARY_DEV}" != "${WLAN_IFACE}" ]]; then
        if [[ "${PRIMARY_DEV}" =~ ^(enp|eth|usb).*u.* ]]; then
            warn "USB interface '${PRIMARY_DEV}' has stolen the default route from Wi-Fi '${WLAN_IFACE}' (metric ${PRIMARY_METRIC})!"
            detail "If this is a phone charging or tethering, desktop traffic stalls when the phone sleeps."
            CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
        else
            info "Default route is active on '${PRIMARY_DEV}' (metric ${PRIMARY_METRIC})."
        fi
    elif [[ -n "${WLAN_IFACE}" ]] && [[ "${PRIMARY_DEV}" == "${WLAN_IFACE}" ]]; then
        success "Wi-Fi interface '${WLAN_IFACE}' holds primary default route (metric ${PRIMARY_METRIC})."
    fi
fi

# ---------------------------------------------------------
# Check 2: Wi-Fi 802.11 Power Saving
# ---------------------------------------------------------
step "Checking Wi-Fi 802.11 power saving state..."
if [[ -n "${WLAN_IFACE}" ]] && command -v iw >/dev/null 2>&1; then
    POWERSAVE_STATUS=$(iw dev "${WLAN_IFACE}" get power_save 2>/dev/null | awk '{print $NF}' || true)
    if [[ "${POWERSAVE_STATUS}" == "on" ]]; then
        err "Wi-Fi Power Save is ON for '${WLAN_IFACE}'."
        detail "Known to cause radio sleep freeze, 1.0 Mbps link collapse, and packet stalls."
        CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
    elif [[ "${POWERSAVE_STATUS}" == "off" ]]; then
        success "Wi-Fi Power Save is OFF for '${WLAN_IFACE}' (radio sleep prevented)."
    else
        warn "Could not query power save status for '${WLAN_IFACE}'."
        WARNINGS=$((WARNINGS + 1))
    fi
else
    warn "No active Wi-Fi interface found or 'iw' tool missing."
    WARNINGS=$((WARNINGS + 1))
fi

# ---------------------------------------------------------
# Check 3: MediaTek MT7922 / MT7921 ASPM Driver Status
# ---------------------------------------------------------
step "Checking kernel wireless driver and PCIe ASPM..."
if [[ -d "/sys/module/mt7921e" ]] || (command -v lsmod >/dev/null 2>&1 && lsmod | grep -q "mt7921e"); then
    detail "MediaTek MT7921/MT7922 wireless driver detected."
    if [[ -f "/sys/module/mt7921e/parameters/disable_aspm" ]]; then
        ASPM_VAL=$(cat /sys/module/mt7921e/parameters/disable_aspm 2>/dev/null || echo "unknown")
        if [[ "${ASPM_VAL}" == "Y" || "${ASPM_VAL}" == "1" ]]; then
            success "PCIe ASPM is disabled on mt7921e (prevents bus sleep freezes)."
        else
            warn "PCIe ASPM is active on mt7921e ('disable_aspm=N')."
            detail "Can cause PCIe bus sleep lockup on idle. Recommended: options mt7921e disable_aspm=1"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi
else
    detail "No MediaTek mt7921e driver loaded; skipping MT7921-specific ASPM check."
fi

# ---------------------------------------------------------
# Check 4: Wi-Fi Link Health, Frequency, and Roaming
# ---------------------------------------------------------
step "Inspecting active Wi-Fi link parameters..."
if [[ -n "${WLAN_IFACE}" ]] && command -v iw >/dev/null 2>&1; then
    LINK_INFO=$(iw dev "${WLAN_IFACE}" link 2>/dev/null || true)
    if echo "${LINK_INFO}" | grep -q "Connected to"; then
        SSID=$(echo "${LINK_INFO}" | awk -F'SSID: ' 'NF>1 {print $2}' | head -n 1)
        FREQ=$(echo "${LINK_INFO}" | awk -F'freq: ' 'NF>1 {print $2}' | head -n 1)
        SIGNAL=$(echo "${LINK_INFO}" | awk -F'signal: ' 'NF>1 {print $2}' | head -n 1)
        RX_BITRATE=$(echo "${LINK_INFO}" | awk -F'rx bitrate: ' 'NF>1 {print $2}' | head -n 1)
        TX_BITRATE=$(echo "${LINK_INFO}" | awk -F'tx bitrate: ' 'NF>1 {print $2}' | head -n 1)

        BAND="2.4 GHz"
        if (( $(echo "${FREQ} > 4000" | bc -l 2>/dev/null || echo 0) )); then
            BAND="5 GHz"
        fi

        success "Connected to '${SSID}' on ${BAND} (${FREQ} MHz)"
        detail "Signal: ${SIGNAL} | RX: ${RX_BITRATE} | TX: ${TX_BITRATE}"

        # Alert if RX bitrate is dangerously low
        if [[ "${RX_BITRATE}" =~ ^1\.0[[:space:]]MBit ]]; then
            err "RX bitrate is collapsed to 1.0 MBit/s! Packet loss imminent."
            CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
        fi
    else
        warn "Interface '${WLAN_IFACE}' is not currently associated with an access point."
        WARNINGS=$((WARNINGS + 1))
    fi
fi

# ---------------------------------------------------------
# Check 5: DNS Resolver Health & Latency Audit
# ---------------------------------------------------------
step "Auditing DNS configuration and latency..."
NAMESERVERS=()
if [[ -f "/etc/resolv.conf" ]]; then
    while read -r ns; do
        [[ -n "$ns" ]] && NAMESERVERS+=("$ns")
    done < <(awk '$1 == "nameserver" {print $2}' /etc/resolv.conf)
fi

if [[ ${#NAMESERVERS[@]} -eq 0 ]]; then
    err "No nameservers found in /etc/resolv.conf!"
    CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
else
    detail "Configured nameservers: ${NAMESERVERS[*]}"
    for ns in "${NAMESERVERS[@]}"; do
        if command -v dig >/dev/null 2>&1; then
            QUERY_OUT=$(dig @"${ns}" +time=2 +tries=1 +stats google.com 2>/dev/null || true)
            QUERY_TIME=$(echo "${QUERY_OUT}" | awk -F'Query time: ' 'NF>1 {print $2}' | head -n 1 || true)
            if [[ -n "${QUERY_TIME}" ]]; then
                MS_VAL=$(echo "${QUERY_TIME}" | awk '{print $1}')
                if (( MS_VAL > 100 )); then
                    warn "Nameserver ${ns} responded in ${QUERY_TIME} (slow)."
                    WARNINGS=$((WARNINGS + 1))
                else
                    success "Nameserver ${ns} query latency: ${QUERY_TIME}"
                fi
            else
                err "Nameserver ${ns} timed out or failed to resolve google.com!"
                CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
            fi
        fi
    done
fi

# ---------------------------------------------------------
# Check 6: End-to-End Connectivity
# ---------------------------------------------------------
step "Testing end-to-end connectivity..."
if ping -c 2 -W 2 1.1.1.1 >/dev/null 2>&1; then
    success "IPv4 internet reachability confirmed (ping 1.1.1.1 OK)."
else
    err "Cannot reach 1.1.1.1 via ICMP!"
    CRITICAL_ERRORS=$((CRITICAL_ERRORS + 1))
fi

# IPv6 routing check
if ip -6 route show default 2>/dev/null | grep -q default; then
    if ping -6 -c 2 -W 2 2606:4700:4700::1111 >/dev/null 2>&1; then
        success "IPv6 internet reachability confirmed."
    else
        warn "IPv6 default route exists but cannot ping Cloudflare IPv6 (blackholing possible)."
        WARNINGS=$((WARNINGS + 1))
    fi
else
    detail "IPv6 default route is disabled or inactive (clean fallback to IPv4)."
fi

# ---------------------------------------------------------
# Summary & Auto-Healing (--fix)
# ---------------------------------------------------------
echo ""
if [[ ${CRITICAL_ERRORS} -eq 0 && ${WARNINGS} -eq 0 ]]; then
    echo -e "${GREEN}${BOLD}✔ Network Health: ALL CHECKS PASSED.${NC}"
    exit 0
elif [[ ${CRITICAL_ERRORS} -eq 0 && ${WARNINGS} -gt 0 ]]; then
    echo -e "${GREEN}${BOLD}✔ Network Health: Core routing & Wi-Fi healthy.${NC} ${STEEL}(${WARNINGS} optimization recommendation(s))${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}✘ Detected ${CRITICAL_ERRORS} network error(s) and ${WARNINGS} warning(s).${NC}"
fi

if [[ "${DO_FIX}" == false ]]; then
    echo -e "\nRun '${BOLD}$(basename "$0") --fix${NC}' to automatically resolve connection metrics, disable Wi-Fi powersave, and inject reliable fallback DNS."
    exit 1
fi

step "Applying automated network fixes..."

if [[ -n "${WLAN_CONN}" ]] && command -v nmcli >/dev/null 2>&1; then
    info "Disabling Wi-Fi power saving on connection '${WLAN_CONN}'..."
    nmcli connection modify "${WLAN_CONN}" 802-11-wireless.powersave 2

    info "Setting priority metric 50 on connection '${WLAN_CONN}'..."
    nmcli connection modify "${WLAN_CONN}" ipv4.route-metric 50

    info "Configuring fallback DNS (1.1.1.1, 8.8.8.8) on '${WLAN_CONN}'..."
    nmcli connection modify "${WLAN_CONN}" ipv4.dns "1.1.1.1,8.8.8.8,192.168.0.1" ipv4.ignore-auto-dns no ipv6.never-default yes

    info "Reactivating Wi-Fi connection '${WLAN_CONN}'..."
    nmcli connection up "${WLAN_CONN}" >/dev/null 2>&1 || true
    success "Wi-Fi connection '${WLAN_CONN}' tuned."
fi

# Fix USB tethering connections to prevent default route hijack
if command -v nmcli >/dev/null 2>&1; then
    for conn in $(nmcli -t -f NAME,TYPE connection show | awk -F':' '$2=="ethernet" {print $1}'); do
        if [[ "${conn}" =~ (Wired|tether|usb) ]]; then
            info "Demoting potential tethering connection '${conn}' to metric 1000..."
            nmcli connection modify "${conn}" ipv4.route-metric 1000 ipv6.route-metric 1000 2>/dev/null || true
            success "Connection '${conn}' route metric demoted."
        fi
    done
fi

# Deploy system drop-ins if /etc is writable or sudo available
NM_DROPIN="${SCRIPT_DIR}/../network/NetworkManager/conf.d/default-wifi-powersave-off.conf"
MODPROBE_DROPIN="${SCRIPT_DIR}/../network/modprobe.d/mt7921e.conf"

if [[ -f "${NM_DROPIN}" ]]; then
    if sudo -n true 2>/dev/null; then
        sudo cp "${NM_DROPIN}" "/etc/NetworkManager/conf.d/default-wifi-powersave-off.conf"
        sudo systemctl reload NetworkManager 2>/dev/null || true
        success "Installed /etc/NetworkManager/conf.d/default-wifi-powersave-off.conf."
    else
        detail "To make Wi-Fi power save disable global for all networks, run:"
        detail "  sudo cp \"${NM_DROPIN}\" /etc/NetworkManager/conf.d/"
    fi
fi

if [[ -f "${MODPROBE_DROPIN}" ]] && lsmod | grep -q "mt7921e"; then
    if sudo -n true 2>/dev/null; then
        sudo cp "${MODPROBE_DROPIN}" "/etc/modprobe.d/mt7921e.conf"
        success "Installed /etc/modprobe.d/mt7921e.conf."
    else
        detail "To prevent MT7922 PCIe bus sleep freezes permanently, run:"
        detail "  sudo cp \"${MODPROBE_DROPIN}\" /etc/modprobe.d/"
    fi
fi

echo -e "\n${GREEN}${BOLD}✔ Auto-healing complete. Re-run '$(basename "$0")' to verify health.${NC}"
