#!/usr/bin/env bash
# Dotfield GTK & Desktop Interface Synchronizer
# Idempotently applies Controlled Chaos theme, icons, cursor, and typography to GSettings and GTK configs
# Controlled Chaos Theme: Void Black · Crimson · Industrial Amber · Steel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOTFIELD_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# 1. Synchronize GTK configuration directories
for d in "${HOME}/.config/gtk-3.0" "${HOME}/.config/gtk-4.0"; do
    if [[ -L "${d}" && ! -e "${d}" ]]; then
        rm -f "${d}"
    fi
done
mkdir -p "${HOME}/.config/gtk-3.0" "${HOME}/.config/gtk-4.0"

if [[ -f "${DOTFIELD_ROOT}/gtk/gtk-3.0/settings.ini" ]]; then
    cp -f "${DOTFIELD_ROOT}/gtk/gtk-3.0/settings.ini" "${HOME}/.config/gtk-3.0/settings.ini"
fi
if [[ -f "${DOTFIELD_ROOT}/gtk/gtk-3.0/gtk.css" ]]; then
    cp -f "${DOTFIELD_ROOT}/gtk/gtk-3.0/gtk.css" "${HOME}/.config/gtk-3.0/gtk.css"
fi

if [[ -f "${DOTFIELD_ROOT}/gtk/gtk-4.0/settings.ini" ]]; then
    cp -f "${DOTFIELD_ROOT}/gtk/gtk-4.0/settings.ini" "${HOME}/.config/gtk-4.0/settings.ini"
fi
if [[ -f "${DOTFIELD_ROOT}/gtk/gtk-4.0/gtk.css" ]]; then
    cp -f "${DOTFIELD_ROOT}/gtk/gtk-4.0/gtk.css" "${HOME}/.config/gtk-4.0/gtk.css"
fi

# 2. Synchronize GSettings (GNOME Desktop Interface & Dark Preference)
if command -v gsettings >/dev/null 2>&1; then
    SCHEMA="org.gnome.desktop.interface"

    # Only set if schema is installed
    if gsettings list-schemas 2>/dev/null | grep -qx "${SCHEMA}"; then
        gsettings set "${SCHEMA}" color-scheme "prefer-dark" 2>/dev/null || true
        gsettings set "${SCHEMA}" gtk-theme "Adwaita-dark" 2>/dev/null || true
        gsettings set "${SCHEMA}" icon-theme "Papirus-Dark" 2>/dev/null || true
        gsettings set "${SCHEMA}" cursor-theme "Bibata-Modern-Amber" 2>/dev/null || true
        gsettings set "${SCHEMA}" cursor-size 24 2>/dev/null || true
        gsettings set "${SCHEMA}" font-name "JetBrainsMono Nerd Font 10" 2>/dev/null || true
        gsettings set "${SCHEMA}" document-font-name "JetBrainsMono Nerd Font 10" 2>/dev/null || true
        gsettings set "${SCHEMA}" monospace-font-name "JetBrainsMono Nerd Font 10" 2>/dev/null || true
        gsettings set "${SCHEMA}" font-antialiasing "rgba" 2>/dev/null || true
        gsettings set "${SCHEMA}" font-hinting "slight" 2>/dev/null || true
    fi

    # Sound theme mute
    SOUND_SCHEMA="org.gnome.desktop.sound"
    if gsettings list-schemas 2>/dev/null | grep -qx "${SOUND_SCHEMA}"; then
        gsettings set "${SOUND_SCHEMA}" event-sounds false 2>/dev/null || true
        gsettings set "${SOUND_SCHEMA}" input-feedback-sounds false 2>/dev/null || true
    fi

    # WM preferences
    WM_SCHEMA="org.gnome.desktop.wm.preferences"
    if gsettings list-schemas 2>/dev/null | grep -qx "${WM_SCHEMA}"; then
        gsettings set "${WM_SCHEMA}" button-layout "appmenu:" 2>/dev/null || true
        gsettings set "${WM_SCHEMA}" titlebar-font "JetBrainsMono Nerd Font Bold 10" 2>/dev/null || true
    fi
fi

# 3. Synchronize XCursor theme for legacy apps
mkdir -p "${HOME}/.icons/default"
cat <<'EOF' > "${HOME}/.icons/default/index.theme"
[Icon Theme]
Name=Default
Comment=Default Cursor Theme
Inherits=Bibata-Modern-Amber
EOF

echo "✔ Dotfield GTK & Interface preferences synchronized cleanly."
