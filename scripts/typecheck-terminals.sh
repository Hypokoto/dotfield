#!/usr/bin/env bash
# Dotfield Terminal Configuration Typechecker & Syntax Validator
# Validates foot, kitty, alacritty, and ghostty configurations

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'
BOLD='\033[1m'

FAILED=0

step() {
    echo -e "${BOLD}==>${NC} $1"
}

pass() {
    echo -e "  ${GREEN}✔${NC} $1"
}

fail() {
    echo -e "  ${RED}✘${NC} $1"
    FAILED=1
}

step "Typechecking Foot terminal configuration..."
if command -v foot >/dev/null 2>&1; then
    if foot --check-config --config="${SCRIPT_DIR}/terminals/foot/foot.ini" -o "include=${SCRIPT_DIR}/terminals/foot/colors.ini" >/dev/null 2>&1; then
        pass "foot.ini syntax is valid"
    else
        fail "foot.ini validation failed"
    fi
else
    pass "foot not installed; skipping binary validation"
fi

step "Typechecking Kitty terminal configuration..."
if command -v kitty >/dev/null 2>&1; then
    if kitty +runpy "from kitty.config import load_config; load_config('${SCRIPT_DIR}/terminals/kitty/kitty.conf')" >/dev/null 2>&1; then
        pass "kitty.conf syntax is valid"
    else
        fail "kitty.conf validation failed"
    fi
else
    pass "kitty not installed; skipping binary validation"
fi

step "Typechecking Alacritty terminal configuration..."
if command -v alacritty >/dev/null 2>&1; then
    if alacritty --config-file "${SCRIPT_DIR}/terminals/alacritty/alacritty.toml" migrate --dry-run >/dev/null 2>&1; then
        pass "alacritty.toml syntax is valid"
    else
        fail "alacritty.toml validation failed"
    fi
else
    pass "alacritty not installed; skipping binary validation"
fi

step "Typechecking Ghostty terminal configuration..."
if command -v ghostty >/dev/null 2>&1; then
    if ghostty +validate-config --config-file="${SCRIPT_DIR}/terminals/ghostty/config" >/dev/null 2>&1; then
        pass "ghostty config syntax is valid"
    else
        fail "ghostty config validation failed"
    fi
else
    pass "ghostty not installed; skipping binary validation"
fi

if [[ "${FAILED}" -ne 0 ]]; then
    echo -e "\n${RED}${BOLD}Terminal validation failed!${NC}"
    exit 1
fi

echo -e "\n${GREEN}${BOLD}All terminal configurations typechecked successfully!${NC}"
exit 0
