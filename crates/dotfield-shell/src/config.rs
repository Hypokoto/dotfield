use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub font_family: String,
    pub font_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteConfig {
    pub background: String,
    pub surface: String,
    pub overlay: String,
    pub muted: String,
    pub accent: String,
    pub highlight: String,
    pub foreground: String,
    #[serde(default = "default_border")]
    pub border: String,
    #[serde(default = "default_hover")]
    pub hover: String,
    #[serde(default = "default_steel")]
    pub steel: String,
}

fn default_border() -> String {
    "#34343D".to_string()
}

fn default_hover() -> String {
    "#FFD166".to_string()
}

fn default_steel() -> String {
    "#8B949E".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarSettings {
    pub height: f64,
    pub position: String,
    pub show_workspaces: bool,
    pub show_title: bool,
    pub show_telemetry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsdSettings {
    pub enabled: bool,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsSettings {
    pub enabled: bool,
    pub timeout_ms: u64,
    pub max_visible: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherSettings {
    pub enabled: bool,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCenterSettings {
    pub enabled: bool,
    pub show_mpris: bool,
    pub show_network: bool,
    pub show_bluetooth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalSettings {
    #[serde(default = "default_terminal_name")]
    pub default: String,
}

fn default_terminal_name() -> String {
    "foot".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellManifest {
    pub theme: ThemeConfig,
    pub palette: PaletteConfig,
    pub bar: BarSettings,
    pub osd: OsdSettings,
    pub notifications: NotificationsSettings,
    pub launcher: LauncherSettings,
    pub control_center: ControlCenterSettings,
    #[serde(default)]
    pub terminal: TerminalSettings,
}

impl Default for ShellManifest {
    fn default() -> Self {
        Self {
            theme: ThemeConfig {
                font_family: "JetBrainsMono Nerd Font".to_string(),
                font_size: 10.0,
            },
            palette: PaletteConfig {
                background: "#0A0A0C".to_string(),
                surface: "#121216".to_string(),
                overlay: "#1C1C23".to_string(),
                muted: "#A1A1AA".to_string(),
                accent: "#E63946".to_string(),
                highlight: "#FFB000".to_string(),
                foreground: "#E8E6E3".to_string(),
                border: "#34343D".to_string(),
                hover: "#FFD166".to_string(),
                steel: "#8B949E".to_string(),
            },
            bar: BarSettings {
                height: 32.0,
                position: "top".to_string(),
                show_workspaces: true,
                show_title: true,
                show_telemetry: true,
            },
            osd: OsdSettings {
                enabled: true,
                timeout_ms: 1200,
            },
            notifications: NotificationsSettings {
                enabled: true,
                timeout_ms: 5000,
                max_visible: 5,
            },
            launcher: LauncherSettings {
                enabled: true,
                max_results: 8,
            },
            control_center: ControlCenterSettings {
                enabled: true,
                show_mpris: true,
                show_network: true,
                show_bluetooth: true,
            },
            terminal: TerminalSettings {
                default: "foot".to_string(),
            },
        }
    }
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
