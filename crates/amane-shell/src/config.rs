use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn default_dark_mode() -> bool {
    true
}

fn default_sync_apps() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub font_family: String,
    pub font_size: f64,
    pub style: String,
    pub wallpaper: Option<String>,
    #[serde(default = "default_dark_mode")]
    pub dark_mode: bool,
    #[serde(default = "default_sync_apps")]
    pub sync_apps: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            font_family: "IBM Plex Sans".to_string(),
            font_size: 11.5,
            style: "cyber-synthwave".to_string(),
            wallpaper: None,
            dark_mode: true,
            sync_apps: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            background: "#000000".to_string(),
            surface: "#140e29".to_string(),
            overlay: "#251b4a".to_string(),
            muted: "#ABA5DB".to_string(),
            accent: "#04C5F0".to_string(),
            highlight: "#FF7F41".to_string(),
            foreground: "#EAF8FA".to_string(),
            border: "#342766".to_string(),
            hover: "#97DCE3".to_string(),
            steel: "#ffbe74".to_string(),
            success: "#7cd699".to_string(),
            danger_critical: "#CE4F48".to_string(),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NightLightSettings {
    pub enabled: bool,
    pub mode: String,
    pub night_temp: u32,
    pub day_temp: u32,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Default for NightLightSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "manual".to_string(),
            night_temp: 4000,
            day_temp: 6500,
            latitude: None,
            longitude: None,
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
    pub night_light: NightLightSettings,
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

    pub fn save_to_path_atomic(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(self).or_else(|_| toml::to_string(self))?;

        let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
        std::fs::write(&tmp_path, serialized.as_bytes())?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }
}

/// Unified, reactive shell configuration service.
/// Windows and widgets reading ConfigService::read() automatically track
/// changes and redraw when ConfigService::write() is updated on reload.
#[derive(Debug, Clone, Default)]
pub struct ConfigService {
    pub manifest: ShellManifest,
}

impl ConfigService {
    /// Mutates the manifest in memory, atomically saves it to ~/.config/dotfield/shell.toml,
    /// and requests a compositor frame redraw.
    pub fn update_and_save<F>(mutate: F) -> Result<()>
    where
        F: FnOnce(&mut ShellManifest),
    {
        use amane::Service;
        let manifest = {
            let mut svc = Self::write();
            mutate(&mut svc.manifest);
            svc.manifest.clone()
        };
        manifest.save_to_path_atomic(&ShellManifest::default_path())?;
        amane::request_frame();
        Ok(())
    }
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
        assert_eq!(manifest.theme.font_size, 11.5);
        assert_eq!(manifest.theme.font_family, "IBM Plex Sans");
        assert_eq!(manifest.theme.style, "cyber-synthwave");
        assert_eq!(manifest.palette.accent, "#04C5F0");
        assert_eq!(manifest.palette.muted, "#ABA5DB");
        assert_eq!(manifest.palette.border, "#342766");
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
        assert_eq!(manifest.theme.font_family, "IBM Plex Sans");
        assert_eq!(manifest.theme.style, "cyber-synthwave");
        assert_eq!(manifest.palette.accent, "#04C5F0");
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

    #[test]
    fn test_atomic_save_and_reload() {
        let dir = std::env::temp_dir().join(format!("amane_test_{}", std::process::id()));
        let path = dir.join("test_shell.toml");

        let mut manifest = ShellManifest::default();
        manifest.bar.position = "top".to_string();
        manifest.bar.height = 54.0;
        manifest.theme.style = "minimal".to_string();

        manifest
            .save_to_path_atomic(&path)
            .expect("failed to save manifest atomically");

        let loaded = ShellManifest::load_from_path(&path).expect("failed to reload saved manifest");
        assert_eq!(loaded.bar.position, "top");
        assert_eq!(loaded.bar.height, 54.0);
        assert_eq!(loaded.theme.style, "minimal");

        // Cleanup
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }
}
