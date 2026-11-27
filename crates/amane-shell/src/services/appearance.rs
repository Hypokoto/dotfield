use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use amane::Service;

use crate::config::{ConfigService, ShellManifest};
use crate::services::palettes::{
    PaletteSpec, default_palette_for_mode, find_palette, find_palette_or_alias,
    resolve_manifest_palette,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiceProfile {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub shell_style: &'static str,
    pub accent: &'static str,
    pub font: &'static str,
    pub gtk_theme: Option<&'static str>,
    pub icon_theme: Option<&'static str>,
    pub cursor_theme: Option<&'static str>,
}

pub const PROFILES: &[RiceProfile] = &[
    RiceProfile {
        id: "cyber-synthwave",
        name: "Cyber Synthwave",
        description: "PALETTE.md OLED void with neon blue & cyan-teal",
        shell_style: "cyber-synthwave",
        accent: "#04C5F0",
        font: "IBM Plex Sans",
        gtk_theme: Some("Adwaita-dark"),
        icon_theme: Some("Papirus-Dark"),
        cursor_theme: Some("Bibata-Modern-Classic"),
    },
    RiceProfile {
        id: "win11-dark",
        name: "Win11 Dark",
        description: "Acrylic dark desktop with crisp blue accents",
        shell_style: "win11-dark",
        accent: "#0078D4",
        font: "Inter",
        gtk_theme: Some("Adwaita-dark"),
        icon_theme: Some("Papirus-Dark"),
        cursor_theme: Some("Bibata-Modern-Classic"),
    },
    RiceProfile {
        id: "nordic-slate",
        name: "Nordic Slate",
        description: "Calm arctic frost aesthetic with muted polar slate",
        shell_style: "nordic-slate",
        accent: "#88C0D0",
        font: "Inter",
        gtk_theme: Some("Adwaita-dark"),
        icon_theme: Some("Papirus-Dark"),
        cursor_theme: Some("Bibata-Modern-Classic"),
    },
    RiceProfile {
        id: "minimal",
        name: "Minimalist Focus",
        description: "Zero distraction monochrome layout for pure focus",
        shell_style: "minimal",
        accent: "#A0A0A0",
        font: "JetBrainsMono Nerd Font",
        gtk_theme: Some("Adwaita-dark"),
        icon_theme: Some("Papirus-Dark"),
        cursor_theme: Some("Bibata-Modern-Classic"),
    },
    RiceProfile {
        id: "obsidian-mint",
        name: "Obsidian Mint",
        description: "Deep obsidian canvas with vibrant emerald mint",
        shell_style: "obsidian-mint",
        accent: "#50FA7B",
        font: "JetBrainsMono Nerd Font",
        gtk_theme: Some("Adwaita-dark"),
        icon_theme: Some("Papirus-Dark"),
        cursor_theme: Some("Bibata-Modern-Classic"),
    },
    RiceProfile {
        id: "playful-clay",
        name: "Playful Clay",
        description: "Warm pastel tactile clay with soft golden curves",
        shell_style: "playful-clay",
        accent: "#FFB86C",
        font: "Inter",
        gtk_theme: Some("Adwaita"),
        icon_theme: Some("Papirus"),
        cursor_theme: Some("Bibata-Modern-Classic"),
    },
];

pub struct AppearanceService {
    pub gtk_themes: Vec<String>,
    pub current_gtk_theme: String,
    pub icon_themes: Vec<String>,
    pub current_icon_theme: String,
    pub cursor_themes: Vec<String>,
    pub current_cursor_theme: String,
    pub cursor_size: u32,
    pub font_families: Vec<String>,
    pub current_font: String,
    pub wallpapers: Vec<PathBuf>,
    pub current_wallpaper: Option<PathBuf>,
    pub selected_palette_family: String,
    pub status_message: Option<String>,
}

impl Service for AppearanceService {
    fn new() -> Self {
        let mut svc = Self {
            gtk_themes: Vec::new(),
            current_gtk_theme: "Adwaita-dark".to_string(),
            icon_themes: Vec::new(),
            current_icon_theme: "Papirus-Dark".to_string(),
            cursor_themes: Vec::new(),
            current_cursor_theme: "Bibata-Modern-Classic".to_string(),
            cursor_size: 24,
            font_families: Vec::new(),
            current_font: "JetBrainsMono Nerd Font".to_string(),
            wallpapers: Vec::new(),
            current_wallpaper: None,
            selected_palette_family: "All".to_string(),
            status_message: None,
        };
        svc.refresh_all();
        svc
    }

    fn interval() -> Duration {
        Duration::from_secs(30)
    }

    fn update(&mut self) -> bool {
        self.refresh_all();
        false
    }
}

impl AppearanceService {
    pub fn refresh_all(&mut self) {
        self.gtk_themes = scan_gtk_themes();
        self.icon_themes = scan_icon_themes();
        self.cursor_themes = scan_cursor_themes();
        self.font_families = scan_font_families();
        self.wallpapers = scan_wallpapers();

        // Read current gsettings
        if let Ok(theme) = read_gsetting("gtk-theme") {
            self.current_gtk_theme = theme;
        }
        if let Ok(icon) = read_gsetting("icon-theme") {
            self.current_icon_theme = icon;
        }
        if let Ok(cursor) = read_gsetting("cursor-theme") {
            self.current_cursor_theme = cursor;
        }
        if let Ok(font) = read_gsetting("font-name") {
            self.current_font = font;
        }
        if let Ok(size_str) = read_gsetting("cursor-size")
            && let Ok(sz) = size_str.parse::<u32>()
        {
            self.cursor_size = sz;
        }

        let manifest = ConfigService::read().manifest.clone();
        if let Some(ref wp) = manifest.theme.wallpaper {
            self.current_wallpaper = Some(PathBuf::from(wp));
        }
    }

    pub fn apply_gtk_theme(theme: &str) {
        let _ = write_gsetting("gtk-theme", theme);
        {
            let mut svc = AppearanceService::write();
            svc.current_gtk_theme = theme.to_string();
            svc.sync_settings_ini();
            svc.status_message = Some(format!("GTK Theme: {theme}"));
        }
        amane::request_frame();
    }

    pub fn apply_icon_theme(icon: &str) {
        let _ = write_gsetting("icon-theme", icon);
        {
            let mut svc = AppearanceService::write();
            svc.current_icon_theme = icon.to_string();
            svc.sync_settings_ini();
            svc.status_message = Some(format!("Icon Theme: {icon}"));
        }
        amane::request_frame();
    }

    pub fn apply_cursor_theme(cursor: &str) {
        let _ = write_gsetting("cursor-theme", cursor);
        {
            let mut svc = AppearanceService::write();
            svc.current_cursor_theme = cursor.to_string();
            svc.sync_settings_ini();
            svc.status_message = Some(format!("Cursor Theme: {cursor}"));
        }
        amane::request_frame();
    }

    pub fn apply_cursor_size(size: u32) {
        let _ = write_gsetting("cursor-size", &size.to_string());
        {
            let mut svc = AppearanceService::write();
            svc.cursor_size = size;
            svc.sync_settings_ini();
            svc.status_message = Some(format!("Cursor Size: {size}px"));
        }
        amane::request_frame();
    }

    pub fn apply_font(font: &str) {
        let _ = write_gsetting("font-name", font);
        {
            let mut svc = AppearanceService::write();
            svc.current_font = font.to_string();
            svc.sync_settings_ini();
            svc.status_message = Some(format!("Font: {font}"));
        }
        let _ = ConfigService::update_and_save(|m| {
            m.theme.font_family = font.to_string();
        });
        amane::request_frame();
    }

    pub fn apply_wallpaper(path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        // Invoke awww directly
        let _ = Command::new("awww").args(["img", &path_str]).status();

        let _ = ConfigService::update_and_save(|m| {
            m.theme.wallpaper = Some(path_str.clone());
        });

        {
            let mut svc = AppearanceService::write();
            svc.current_wallpaper = Some(path.to_path_buf());
            svc.status_message = Some(format!(
                "Wallpaper: {}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&path_str)
            ));
        }
        amane::request_frame();
    }

    pub fn apply_rice_profile(profile: &RiceProfile) {
        let mut updated = None;
        let _ = ConfigService::update_and_save(|m| {
            transition_manifest_rice_profile(m, profile);
            updated = Some(m.clone());
        });

        // Apply GTK / icon / cursor / font gsettings
        if let Some(gtk) = profile.gtk_theme {
            let _ = write_gsetting("gtk-theme", gtk);
        }
        if let Some(icon) = profile.icon_theme {
            let _ = write_gsetting("icon-theme", icon);
        }
        if let Some(cur) = profile.cursor_theme {
            let _ = write_gsetting("cursor-theme", cur);
        }
        let _ = write_gsetting("font-name", profile.font);

        {
            let mut svc = AppearanceService::write();
            if let Some(gtk) = profile.gtk_theme {
                svc.current_gtk_theme = gtk.to_string();
            }
            if let Some(icon) = profile.icon_theme {
                svc.current_icon_theme = icon.to_string();
            }
            if let Some(cur) = profile.cursor_theme {
                svc.current_cursor_theme = cur.to_string();
            }
            svc.current_font = profile.font.to_string();
            svc.status_message = Some(format!("Loaded profile '{}'", profile.name));
        }

        if let Some(manifest) = updated {
            Self::reconcile_theme(&manifest);
        }
    }

    pub fn apply_palette(spec: &PaletteSpec) {
        let mut updated = None;
        let _ = ConfigService::update_and_save(|m| {
            transition_manifest_palette(m, spec);
            updated = Some(m.clone());
        });

        {
            let mut svc = AppearanceService::write();
            svc.status_message = Some(format!(
                "Applied {} ({})",
                spec.name,
                if spec.is_dark { "Dark" } else { "Light" }
            ));
        }

        if let Some(manifest) = updated {
            Self::reconcile_theme(&manifest);
        }
    }

    pub fn apply_accent(hex: &str) {
        let mut updated = None;
        let _ = ConfigService::update_and_save(|m| {
            transition_manifest_accent(m, hex);
            updated = Some(m.clone());
        });

        {
            let mut svc = AppearanceService::write();
            svc.status_message = Some(format!("Accent updated to {hex}"));
        }

        if let Some(manifest) = updated {
            Self::reconcile_theme(&manifest);
        }
    }

    pub fn toggle_sync_apps() {
        let mut updated = None;
        let mut now_enabled = false;
        let _ = ConfigService::update_and_save(|m| {
            now_enabled = transition_manifest_toggle_sync(m);
            updated = Some(m.clone());
        });

        {
            let mut svc = AppearanceService::write();
            svc.status_message = Some(if now_enabled {
                "Synced theme to terminals, apps & GTK".to_string()
            } else {
                "App theme sync paused (Amane only)".to_string()
            });
        }

        if let Some(manifest) = updated {
            Self::reconcile_theme(&manifest);
        }
    }

    pub fn toggle_color_mode(dark_mode: bool) {
        let current_style = ConfigService::read().manifest.theme.style.clone();
        let cur = find_palette_or_alias(&current_style, !dark_mode);
        let target = if cur.is_dark == dark_mode {
            cur
        } else {
            find_palette(cur.counterpart).unwrap_or_else(|| default_palette_for_mode(dark_mode))
        };

        Self::apply_palette(target);
    }

    /// Reconcile system/app theme state (gsettings, GTK settings.ini, external app templates, frame redraw)
    /// from an authoritative ShellManifest snapshot.
    pub fn reconcile_theme(manifest: &ShellManifest) {
        let resolved = resolve_manifest_palette(manifest);
        let _ = write_gsetting(
            "color-scheme",
            if resolved.is_dark {
                "prefer-dark"
            } else {
                "prefer-light"
            },
        );

        {
            let mut svc = AppearanceService::write();
            if !manifest.theme.font_family.is_empty() {
                svc.current_font = manifest.theme.font_family.clone();
            }
            svc.sync_settings_ini();
        }

        if manifest.theme.sync_apps {
            crate::services::theme_sync::sync_manifest(manifest);
        }

        amane::request_frame();
    }

    fn sync_settings_ini(&self) {
        let prefer_dark = if ConfigService::read().manifest.theme.dark_mode {
            1
        } else {
            0
        };
        let content = format!(
            "[Settings]\ngtk-theme-name={}\ngtk-icon-theme-name={}\ngtk-cursor-theme-name={}\ngtk-cursor-theme-size={}\ngtk-font-name={}\ngtk-application-prefer-dark-theme={}\n",
            self.current_gtk_theme,
            self.current_icon_theme,
            self.current_cursor_theme,
            self.cursor_size,
            self.current_font,
            prefer_dark,
        );

        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            for sub in &["gtk-3.0", "gtk-4.0"] {
                let dir = home_path.join(".config").join(sub);
                let _ = fs::create_dir_all(&dir);
                let _ = fs::write(dir.join("settings.ini"), &content);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// SCANNING HELPERS
// ---------------------------------------------------------------------------

fn scan_gtk_themes() -> Vec<String> {
    let mut themes = Vec::new();
    let dirs = [
        PathBuf::from("/usr/share/themes"),
        dirs_home(".local/share/themes"),
        dirs_home(".themes"),
    ];

    for dir in &dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && (path.join("gtk-3.0").exists() || path.join("gtk-4.0").exists())
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && !themes.contains(&name.to_string())
                {
                    themes.push(name.to_string());
                }
            }
        }
    }
    themes.sort();
    if themes.is_empty() {
        themes.push("Adwaita-dark".to_string());
        themes.push("Adwaita".to_string());
    }
    themes
}

fn scan_icon_themes() -> Vec<String> {
    let mut icons = Vec::new();
    let dirs = [
        PathBuf::from("/usr/share/icons"),
        dirs_home(".local/share/icons"),
        dirs_home(".icons"),
    ];

    for dir in &dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && path.join("index.theme").exists()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && !icons.contains(&name.to_string())
                {
                    icons.push(name.to_string());
                }
            }
        }
    }
    icons.sort();
    if icons.is_empty() {
        icons.push("Papirus-Dark".to_string());
        icons.push("hicolor".to_string());
    }
    icons
}

fn scan_cursor_themes() -> Vec<String> {
    let mut cursors = Vec::new();
    let dirs = [
        PathBuf::from("/usr/share/icons"),
        dirs_home(".local/share/icons"),
        dirs_home(".icons"),
    ];

    for dir in &dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && path.join("cursors").exists()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && !cursors.contains(&name.to_string())
                {
                    cursors.push(name.to_string());
                }
            }
        }
    }
    cursors.sort();
    if cursors.is_empty() {
        cursors.push("Bibata-Modern-Classic".to_string());
        cursors.push("Adwaita".to_string());
    }
    cursors
}

fn scan_font_families() -> Vec<String> {
    let mut fonts = Vec::new();
    if let Ok(output) = Command::new("fc-list").args([":", "family"]).output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            // Split comma separated aliases
            for fam in line.split(',') {
                let trimmed = fam.trim();
                if !trimmed.is_empty() && !fonts.contains(&trimmed.to_string()) {
                    fonts.push(trimmed.to_string());
                }
            }
        }
    }

    if fonts.is_empty() {
        fonts.push("JetBrainsMono Nerd Font".to_string());
        fonts.push("Inter".to_string());
        fonts.push("Roboto".to_string());
        fonts.push("Cantarell".to_string());
    }
    fonts.sort();
    fonts
}

fn scan_wallpapers() -> Vec<PathBuf> {
    let mut wallpapers = Vec::new();
    let candidates = [
        dirs_home("Pictures/Wallpapers"),
        dirs_home("Pictures/wallpapers"),
        dirs_home(".local/share/wallpapers"),
        PathBuf::from("/usr/share/backgrounds"),
    ];

    let valid_exts = ["png", "jpg", "jpeg", "webp"];

    for dir in &candidates {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && valid_exts.contains(&ext.to_lowercase().as_str())
                    && !wallpapers.contains(&path)
                {
                    wallpapers.push(path);
                }
            }
        }
    }
    wallpapers.sort();
    wallpapers
}

fn dirs_home(sub: &str) -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(sub)
    } else {
        PathBuf::from("/tmp").join(sub)
    }
}

fn read_gsetting(key: &str) -> Result<String, ()> {
    let out = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", key])
        .output()
        .map_err(|_| ())?;
    if out.status.success() {
        let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
        // Strip single quotes
        let stripped = raw.trim_matches('\'').to_string();
        Ok(stripped)
    } else {
        Err(())
    }
}

fn write_gsetting(key: &str, val: &str) -> Result<(), ()> {
    let status = Command::new("gsettings")
        .args(["set", "org.gnome.desktop.interface", key, val])
        .status()
        .map_err(|_| ())?;
    if status.success() { Ok(()) } else { Err(()) }
}

fn apply_spec_colors(m: &mut ShellManifest, spec: &PaletteSpec) {
    m.theme.dark_mode = spec.is_dark;
    m.palette.background = spec.bg.to_string();
    m.palette.surface = spec.surface.to_string();
    m.palette.overlay = spec.overlay.to_string();
    m.palette.muted = spec.fg_muted.to_string();
    m.palette.accent = spec.accent.to_string();
    m.palette.highlight = spec.warning.to_string();
    m.palette.foreground = spec.fg.to_string();
    m.palette.border = spec.border.to_string();
    m.palette.hover = spec.fg_secondary.to_string();
    m.palette.steel = spec.fg_muted.to_string();
    m.palette.success = spec.success.to_string();
    m.palette.danger_critical = spec.danger.to_string();
}

pub fn transition_manifest_palette(m: &mut ShellManifest, spec: &PaletteSpec) {
    m.theme.style = spec.id.to_string();
    apply_spec_colors(m, spec);
}

pub fn transition_manifest_rice_profile(m: &mut ShellManifest, profile: &RiceProfile) {
    let base = find_palette_or_alias(profile.shell_style, m.theme.dark_mode);
    apply_spec_colors(m, base);
    m.theme.style = profile.shell_style.to_string();
    m.palette.accent = profile.accent.to_string();
    m.theme.font_family = profile.font.to_string();
}

pub fn transition_manifest_accent(m: &mut ShellManifest, hex: &str) {
    let base = find_palette_or_alias(&m.theme.style, m.theme.dark_mode);
    apply_spec_colors(m, base);
    m.palette.accent = hex.to_string();
}

pub fn transition_manifest_toggle_sync(m: &mut ShellManifest) -> bool {
    m.theme.sync_apps = !m.theme.sync_apps;
    m.theme.sync_apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rice_profiles_definitions() {
        assert!(PROFILES.len() >= 5);
        let cyber = PROFILES.iter().find(|p| p.id == "cyber-synthwave").unwrap();
        assert_eq!(cyber.shell_style, "cyber-synthwave");
        assert_eq!(cyber.accent, "#04C5F0");
    }

    #[test]
    fn test_explicit_manifest_state_transitions() {
        let mut m = ShellManifest::default();
        let latte = find_palette("catppuccin-latte").unwrap();
        transition_manifest_palette(&mut m, latte);
        assert!(!m.theme.dark_mode);
        assert_eq!(m.palette.background, latte.bg);

        // Switching to a RiceProfile must reset all 12 palette fields + dark_mode, not leave light-mode stale!
        let nordic = PROFILES.iter().find(|p| p.id == "nordic-slate").unwrap();
        transition_manifest_rice_profile(&mut m, nordic);
        assert!(m.theme.dark_mode);
        assert_eq!(m.theme.style, "nordic-slate");
        assert_eq!(m.palette.background, "#2e3440");
        assert_eq!(m.palette.accent, nordic.accent);

        // Selecting cyan accent (#04C5F0) keeps nordic-slate base and overrides accent cleanly
        transition_manifest_accent(&mut m, "#04C5F0");
        assert_eq!(m.theme.style, "nordic-slate");
        assert_eq!(m.palette.background, "#2e3440");
        assert_eq!(m.palette.accent, "#04C5F0");

        let resolved = resolve_manifest_palette(&m);
        assert_eq!(resolved.accent, "#04C5F0");
        assert_eq!(resolved.bg, "#2e3440");

        // Atomic sync toggle flips boolean without stale closure state
        let first = transition_manifest_toggle_sync(&mut m);
        assert_eq!(first, !ShellManifest::default().theme.sync_apps);
        let second = transition_manifest_toggle_sync(&mut m);
        assert_eq!(second, ShellManifest::default().theme.sync_apps);
    }

    #[test]
    fn test_scan_gtk_themes_fallback() {
        let themes = scan_gtk_themes();
        assert!(!themes.is_empty());
    }

    #[test]
    fn test_scan_font_families_fallback() {
        let fonts = scan_font_families();
        assert!(!fonts.is_empty());
    }
}
