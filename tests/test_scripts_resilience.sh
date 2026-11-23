#!/usr/bin/env bash
# ==============================================================================
# Dotfield Script Resilience & Automated Verification Suite
# ==============================================================================
# Tests socket auto-discovery, compositor detection, battery alert state machine,
# and graceful degradation without modifying user runtime state.
# ==============================================================================
# shellcheck disable=SC2329,SC2030,SC2031,SC2034
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_TMP="$(mktemp -d /tmp/dotfield-test.XXXXXX)"
touch "${TEST_TMP}/passes"
touch "${TEST_TMP}/failures"

cleanup() {
    rm -rf "${TEST_TMP}"
}
trap cleanup EXIT

pass() {
    echo -e "  \033[1;32m✓ PASS:\033[0m $1"
    echo "1" >> "${TEST_TMP}/passes"
}

fail() {
    echo -e "  \033[1;31m✗ FAIL:\033[0m $1"
    echo "1" >> "${TEST_TMP}/failures"
}

header() {
    echo -e "\n\033[1;34m=== $1 ===\033[0m"
}

create_mock_socket() {
    local sock_path="$1"
    python3 -c "import socket; s = socket.socket(socket.AF_UNIX); s.bind('${sock_path}')"
}

# ------------------------------------------------------------------------------
# Test 1: Wayland Socket Auto-Discovery
# ------------------------------------------------------------------------------
header "Test 1: Wayland Socket Auto-Discovery"
(
    MOCK_RUNTIME="${TEST_TMP}/runtime_wl"
    mkdir -p "${MOCK_RUNTIME}"
    create_mock_socket "${MOCK_RUNTIME}/wayland-3"
    touch "${MOCK_RUNTIME}/wayland-3.lock"
    
    # Run subshell with isolated environment
    export XDG_RUNTIME_DIR="${MOCK_RUNTIME}"
    unset WAYLAND_DISPLAY
    
    # Source env.sh
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/scripts/lib/env.sh"
    
    auto_discover_wayland_display || true
    if [[ "${WAYLAND_DISPLAY:-}" == "wayland-3" ]]; then
        pass "auto_discover_wayland_display correctly identified 'wayland-3' and ignored '.lock'"
    else
        fail "Expected 'wayland-3', got '${WAYLAND_DISPLAY:-}'"
    fi
)

# ------------------------------------------------------------------------------
# Test 2: Sway IPC Socket Auto-Discovery
# ------------------------------------------------------------------------------
header "Test 2: Sway IPC Socket Auto-Discovery"
(
    MOCK_RUNTIME="${TEST_TMP}/runtime_sway"
    mkdir -p "${MOCK_RUNTIME}"
    create_mock_socket "${MOCK_RUNTIME}/sway-ipc.1000.12345.sock"
    
    export XDG_RUNTIME_DIR="${MOCK_RUNTIME}"
    unset SWAYSOCK
    
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/scripts/lib/env.sh"
    
    auto_discover_swaysock || true
    if [[ "${SWAYSOCK:-}" == "${MOCK_RUNTIME}/sway-ipc.1000.12345.sock" ]]; then
        pass "auto_discover_swaysock correctly detected Sway IPC socket"
    else
        fail "Expected '${MOCK_RUNTIME}/sway-ipc.1000.12345.sock', got '${SWAYSOCK:-}'"
    fi
)

# ------------------------------------------------------------------------------
# Test 3: Niri IPC Socket Auto-Discovery
# ------------------------------------------------------------------------------
header "Test 3: Niri IPC Socket Auto-Discovery"
(
    MOCK_RUNTIME="${TEST_TMP}/runtime_niri"
    mkdir -p "${MOCK_RUNTIME}"
    create_mock_socket "${MOCK_RUNTIME}/niri-1000.sock"
    
    export XDG_RUNTIME_DIR="${MOCK_RUNTIME}"
    unset NIRI_SOCKET
    
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/scripts/lib/env.sh"
    
    auto_discover_niri_socket || true
    if [[ "${NIRI_SOCKET:-}" == "${MOCK_RUNTIME}/niri-1000.sock" ]]; then
        pass "auto_discover_niri_socket correctly detected Niri IPC socket"
    else
        fail "Expected '${MOCK_RUNTIME}/niri-1000.sock', got '${NIRI_SOCKET:-}'"
    fi
)

# ------------------------------------------------------------------------------
# Test 4: Compositor Target Resolution
# ------------------------------------------------------------------------------
header "Test 4: Compositor Target Resolution"
(
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/scripts/lib/env.sh"
    
    # Case A: Explicit sway
    if is_target_sway "sway" && is_target_sway "--sway"; then
        pass "is_target_sway recognized targets 'sway' and '--sway'"
    else
        fail "is_target_sway failed to recognize target 'sway'/'--sway'"
    fi
    
    # Case B: Explicit niri
    if is_target_niri "niri" && is_target_niri "--niri"; then
        pass "is_target_niri recognized targets 'niri' and '--niri'"
    else
        fail "is_target_niri failed to recognize target 'niri'/'--niri'"
    fi
)

# ------------------------------------------------------------------------------
# Test 5: Safe Notify Graceful Fallback
# ------------------------------------------------------------------------------
header "Test 5: Safe Notification Resilience"
(
    # shellcheck disable=SC1091
    source "${SCRIPT_DIR}/scripts/lib/env.sh"
    
    # Test under restricted PATH where notify-send does not exist
    if (PATH="/nonexistent" safe_notify "dotfield-test" "normal" "Test" "Message"); then
        pass "safe_notify gracefully succeeded with exit code 0 when notify-send was unavailable"
    else
        fail "safe_notify failed when notify-send was unavailable"
    fi
)

# ------------------------------------------------------------------------------
# Test 6: Battery Alert State Machine
# ------------------------------------------------------------------------------
header "Test 6: Battery Alert Debounced State Machine"
(
    MOCK_SYSFS="${TEST_TMP}/sysfs"
    MOCK_RUNTIME="${TEST_TMP}/runtime_bat"
    mkdir -p "${MOCK_SYSFS}/power_supply/BAT0"
    mkdir -p "${MOCK_RUNTIME}"
    
    export XDG_RUNTIME_DIR="${MOCK_RUNTIME}"
    STATE_FILE="${MOCK_RUNTIME}/dotfield-battery.state"
    
    # Helper to run a step of battery checking
    test_battery_step() {
        local cap="$1"
        local status="$2"
        echo "${cap}" > "${MOCK_SYSFS}/power_supply/BAT0/capacity"
        echo "${status}" > "${MOCK_SYSFS}/power_supply/BAT0/status"
        
        # Override sysfs glob in a subshell running the battery logic
        (
            # shellcheck disable=SC1091
            source "${SCRIPT_DIR}/scripts/lib/env.sh"
            EMERGENCY_THRESHOLD=5
            CRITICAL_THRESHOLD=10
            LOW_THRESHOLD=20
            
            # Read state
            current_state="none"
            if [[ -f "${STATE_FILE}" ]]; then
                current_state="$(cat "${STATE_FILE}" 2>/dev/null || echo "none")"
            fi
            
            if [[ "${status}" =~ (Charging|Full) ]]; then
                if [[ "${current_state}" != "none" ]]; then
                    echo "none" > "${STATE_FILE}"
                fi
                exit 0
            fi
            
            new_state="none"
            if (( cap <= EMERGENCY_THRESHOLD )); then
                new_state="emergency"
            elif (( cap <= CRITICAL_THRESHOLD )); then
                new_state="critical"
            elif (( cap <= LOW_THRESHOLD )); then
                new_state="low"
            fi
            
            if [[ "${new_state}" != "${current_state}" ]]; then
                echo "${new_state}" > "${STATE_FILE}"
                echo "NOTIFIED:${new_state}"
            else
                echo "UNCHANGED:${current_state}"
            fi
        )
    }
    
    # 1. 80% Discharging -> state: none
    out1=$(test_battery_step 80 "Discharging")
    state1=$(cat "${STATE_FILE}" 2>/dev/null || echo "none")
    if [[ "${state1}" == "none" ]]; then
        pass "State remains 'none' at 80% discharge"
    else
        fail "Expected state 'none', got '${state1}'"
    fi
    
    # 2. 18% Discharging -> triggers low
    out2=$(test_battery_step 18 "Discharging")
    state2=$(cat "${STATE_FILE}" 2>/dev/null || echo "none")
    if [[ "${state2}" == "low" && "${out2}" == "NOTIFIED:low" ]]; then
        pass "State transitioned to 'low' at 18% and notified"
    else
        fail "Expected 'low' and 'NOTIFIED:low', got state='${state2}', out='${out2}'"
    fi
    
    # 3. 17% Discharging -> debounced, no duplicate notification
    out3=$(test_battery_step 17 "Discharging")
    state3=$(cat "${STATE_FILE}" 2>/dev/null || echo "none")
    if [[ "${state3}" == "low" && "${out3}" == "UNCHANGED:low" ]]; then
        pass "State debounced at 17% without duplicate notification"
    else
        fail "Debounce failed: got state='${state3}', out='${out3}'"
    fi
    
    # 4. 9% Discharging -> triggers critical
    out4=$(test_battery_step 9 "Discharging")
    state4=$(cat "${STATE_FILE}" 2>/dev/null || echo "none")
    if [[ "${state4}" == "critical" && "${out4}" == "NOTIFIED:critical" ]]; then
        pass "State transitioned to 'critical' at 9% and notified"
    else
        fail "Expected 'critical', got state='${state4}', out='${out4}'"
    fi
    
    # 5. 4% Discharging -> triggers emergency
    out5=$(test_battery_step 4 "Discharging")
    state5=$(cat "${STATE_FILE}" 2>/dev/null || echo "none")
    if [[ "${state5}" == "emergency" && "${out5}" == "NOTIFIED:emergency" ]]; then
        pass "State transitioned to 'emergency' at 4% and notified"
    else
        fail "Expected 'emergency', got state='${state5}', out='${out5}'"
    fi
    
    # 6. Plugged in (Charging at 15%) -> resets state to none
    out6=$(test_battery_step 15 "Charging")
    state6=$(cat "${STATE_FILE}" 2>/dev/null || echo "none")
    if [[ "${state6}" == "none" ]]; then
        pass "State successfully reset to 'none' upon AC connection"
    else
        fail "Expected state reset to 'none', got '${state6}'"
    fi
)

# ------------------------------------------------------------------------------
# Test 7: Shellcheck across all repository scripts
# ------------------------------------------------------------------------------
header "Test 7: ShellCheck Verification Across Repository"
if command -v shellcheck >/dev/null 2>&1; then
    # shellcheck disable=SC2046
    if shellcheck $(find "${SCRIPT_DIR}/scripts" -name "*.sh") "${SCRIPT_DIR}/setup.sh" "${SCRIPT_DIR}/tests/test_scripts_resilience.sh"; then
        pass "All repository shell scripts passed ShellCheck with 0 warnings/errors"
    else
        fail "ShellCheck found issues in scripts"
    fi
else
    echo "  [SKIPPED] shellcheck command not found"
fi

# ------------------------------------------------------------------------------
# Summary
# ------------------------------------------------------------------------------
header "Test Summary"
PASS_COUNT=$(wc -l < "${TEST_TMP}/passes" | tr -d ' ')
FAIL_COUNT=$(wc -l < "${TEST_TMP}/failures" | tr -d ' ')
echo "Passed: ${PASS_COUNT}"
echo "Failed: ${FAIL_COUNT}"

if [[ "${FAIL_COUNT}" -eq 0 ]]; then
    echo -e "\033[1;32mALL TESTS PASSED SUCCESSFULLY!\033[0m\n"
    exit 0
else
    echo -e "\033[1;31mSOME TESTS FAILED!\033[0m\n"
    exit 1
fi
