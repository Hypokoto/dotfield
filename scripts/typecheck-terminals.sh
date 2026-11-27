#!/usr/bin/env bash
# Dotfield Terminal & Launcher Configuration Validator
# Validates foot, kitty, ghostty, and fuzzel configurations deterministically

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'
BOLD='\033[1m'

FAILED=0
ISOLATED_HOME="$(mktemp -d)"
trap 'rm -rf "${ISOLATED_HOME}"' EXIT
mkdir -p "${ISOLATED_HOME}/.config/dotfield/themes"
ln -sfn "${SCRIPT_DIR}/terminals/foot" "${ISOLATED_HOME}/.config/foot"
ln -sfn "${SCRIPT_DIR}/terminals/ghostty" "${ISOLATED_HOME}/.config/ghostty"
ln -sfn "${SCRIPT_DIR}/terminals/kitty" "${ISOLATED_HOME}/.config/kitty"
ln -sfn "${SCRIPT_DIR}/launcher/fuzzel" "${ISOLATED_HOME}/.config/fuzzel"
for theme_file in foot.ini fuzzel.ini kitty.conf ghostty.conf ghostty-gtk.css; do
    : > "${ISOLATED_HOME}/.config/dotfield/themes/${theme_file}"
done

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
    if HOME="${ISOLATED_HOME}" foot --check-config --config="${SCRIPT_DIR}/terminals/foot/foot.ini" >/dev/null 2>&1; then
        pass "foot.ini syntax is valid"
    else
        fail "foot.ini validation failed"
    fi
else
    pass "foot not installed; skipping binary validation"
fi

step "Typechecking Kitty terminal configuration..."
if command -v kitty >/dev/null 2>&1; then
    if HOME="${ISOLATED_HOME}" kitty +runpy "from kitty.config import load_config; load_config('${SCRIPT_DIR}/terminals/kitty/kitty.conf')" >/dev/null 2>&1; then
        pass "kitty.conf syntax is valid"
    else
        fail "kitty.conf validation failed"
    fi
else
    pass "kitty not installed; skipping binary validation"
fi

step "Typechecking Ghostty terminal configuration..."
if command -v ghostty >/dev/null 2>&1; then
    if HOME="${ISOLATED_HOME}" ghostty +validate-config --config-file="${SCRIPT_DIR}/terminals/ghostty/config" >/dev/null 2>&1; then
        pass "ghostty config syntax is valid"
    else
        fail "ghostty config validation failed"
    fi
else
    pass "ghostty not installed; skipping binary validation"
fi

step "Typechecking Fuzzel launcher configuration..."
if command -v fuzzel >/dev/null 2>&1; then
    if HOME="${ISOLATED_HOME}" fuzzel --config="${SCRIPT_DIR}/launcher/fuzzel/fuzzel.ini" --check-config >/dev/null 2>&1; then
        pass "fuzzel.ini syntax is valid"
    else
        fail "fuzzel.ini validation failed"
    fi
else
    pass "fuzzel not installed; skipping binary validation"
fi

if [[ "${FAILED}" -ne 0 ]]; then
    echo -e "\n${RED}${BOLD}Configuration validation failed!${NC}"
    exit 1
fi

echo -e "\n${GREEN}${BOLD}All terminal and launcher configurations validated successfully!${NC}"
exit 0
