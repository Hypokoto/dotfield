use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub font_family: String,
    pub font_size: f64,
    pub style: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            font_family: "Inter".to_string(),
            font_size: 10.0,
            style: "obsidian-mint".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteConfig {
    pub background: String,
    pub surface: String,
    pub overlay: String,
    pub muted: String,
    pub accent: String,
    pub highlight: String,
    pub foreground: String,
    pub border: String,
    pub hover: String,
    pub steel: String,
    pub success: String,
    pub danger_critical: String,
}

impl Default for PaletteConfig {
    fn default() -> Self {
        Self {
            background: "#080B0B".to_string(),
            surface: "#0E1413".to_string(),
            overlay: "#16201E".to_string(),
            muted: "#94A3A0".to_string(),
            accent: "#2DD4BF".to_string(),
            highlight: "#FBBF24".to_string(),
            foreground: "#ECFDF5".to_string(),
            border: "#23322F".to_string(),
            hover: "#10B981".to_string(),
            steel: "#71827E".to_string(),
            success: "#10B981".to_string(),
            danger_critical: "#F43F5E".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BarSettings {
    pub enabled: bool,
    pub height: f64,
    pub position: String,
    pub show_workspaces: bool,
    pub show_title: bool,
    pub show_telemetry: bool,
}

impl Default for BarSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 48.0,
            position: "bottom".to_string(),
            show_workspaces: true,
            show_title: true,
            show_telemetry: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OsdSettings {
    pub enabled: bool,
    pub timeout_ms: u64,
}

impl Default for OsdSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 1200,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NotificationsSettings {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub max_visible: usize,
}

impl Default for NotificationsSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: 5000,
            max_visible: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LauncherSettings {
    pub enabled: bool,
    pub max_results: usize,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_results: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ControlCenterSettings {
    pub enabled: bool,
    pub show_mpris: bool,
    pub show_network: bool,
    pub show_bluetooth: bool,
}

impl Default for ControlCenterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            show_mpris: true,
            show_network: true,
            show_bluetooth: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalSettings {
    pub default: String,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            default: "ghostty".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ShellManifest {
    pub theme: ThemeConfig,
    pub palette: PaletteConfig,
    pub bar: BarSettings,
    pub osd: OsdSettings,
    pub notifications: NotificationsSettings,
    pub launcher: LauncherSettings,
    pub control_center: ControlCenterSettings,
    pub terminal: TerminalSettings,
}

impl ShellManifest {
    pub fn default_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/dotfield/shell.toml")
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        let manifest: Self = toml::from_str(&raw)?;
        Ok(manifest)
    }
}

/// Unified, reactive shell configuration service.
/// Windows and widgets reading ConfigService::read() automatically track
/// changes and redraw when ConfigService::write() is updated on reload.
#[derive(Debug, Clone, Default)]
pub struct ConfigService {
    pub manifest: ShellManifest,
}

impl amane::Service for ConfigService {
    fn new() -> Self {
        let path = ShellManifest::default_path();
        let manifest = ShellManifest::load_from_path(&path).unwrap_or_else(|err| {
            eprintln!(
                "[amane:config:warn] failed to load shell manifest from {}: {}, using defaults",
                path.display(),
                err
            );
            ShellManifest::default()
        });
        Self { manifest }
    }

    fn interval() -> std::time::Duration {
        std::time::Duration::from_secs(3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_manifest() {
        let manifest = ShellManifest::default();
        assert_eq!(manifest.bar.height, 48.0);
        assert!(manifest.bar.enabled);
        assert_eq!(manifest.theme.font_size, 10.0);
        assert_eq!(manifest.theme.style, "obsidian-mint");
        assert_eq!(manifest.palette.accent, "#2DD4BF");
        assert_eq!(manifest.terminal.default, "ghostty");
    }

    #[test]
    fn test_partial_toml_deserialization() {
        let toml_str = r#"
            [bar]
            height = 28.0

            [theme]
            font_size = 12.0
        "#;
        let manifest: ShellManifest =
            toml::from_str(toml_str).expect("failed to parse partial toml");
        assert_eq!(manifest.bar.height, 28.0);
        assert!(manifest.bar.enabled);
        assert_eq!(manifest.bar.position, "bottom");
        assert_eq!(manifest.theme.font_size, 12.0);
        assert_eq!(manifest.theme.font_family, "Inter");
        assert_eq!(manifest.theme.style, "obsidian-mint");
        assert_eq!(manifest.palette.accent, "#2DD4BF");
    }

    #[test]
    fn test_empty_toml_deserialization() {
        let manifest: ShellManifest = toml::from_str("").expect("failed to parse empty toml");
        assert_eq!(manifest.bar.height, 48.0);
        assert_eq!(manifest.terminal.default, "ghostty");
    }

    #[test]
    fn test_corrupt_toml_returns_error() {
        let res: Result<ShellManifest, _> = toml::from_str("[unclosed_table");
        assert!(res.is_err());
    }
}
