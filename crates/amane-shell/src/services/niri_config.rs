//! Niri Compositor Configuration Generator & Validator Service.
//! Safely persists user preferences to `compositor/niri/conf.d/99-amane-settings.kdl`
//! and validates with `niri validate` before reloading or reverting on failure.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use amane::Service;

#[derive(Debug, Clone, PartialEq)]
pub struct NiriSettingsState {
    // Workspaces
    pub gaps: f32,
    pub default_column_width: f32, // proportion (0.33, 0.5, 0.66, 1.0)
    pub center_focused_column: String, // "never", "always", "on-overflow"

    // Animations
    pub animations_enabled: bool,
    pub animation_duration_ms: u32,
    pub animation_curve: String, // "ease-out-cubic", "linear", "ease-out-quad"

    // Window Rules
    pub border_width: f32,
    pub active_border_color: String,
    pub inactive_border_color: String,
    pub corner_radius: f32,
    pub active_window_opacity: f32,

    // Input & Peripherals
    pub natural_scroll: bool,
    pub tap_to_click: bool,
    pub dwt: bool, // disable-while-typing
    pub repeat_delay: u32,
    pub repeat_rate: u32,
    pub mouse_accel_speed: f64, // -1.0 to 1.0

    // Status
    pub last_status: Option<String>,
}

impl Default for NiriSettingsState {
    fn default() -> Self {
        Self {
            gaps: 8.0,
            default_column_width: 0.5,
            center_focused_column: "never".to_string(),
            animations_enabled: true,
            animation_duration_ms: 110,
            animation_curve: "ease-out-cubic".to_string(),
            border_width: 2.0,
            active_border_color: "#04C5F0".to_string(),
            inactive_border_color: "#251B4A".to_string(),
            corner_radius: 8.0,
            active_window_opacity: 0.82,
            natural_scroll: true,
            tap_to_click: true,
            dwt: true,
            repeat_delay: 200,
            repeat_rate: 50,
            mouse_accel_speed: 0.0,
            last_status: None,
        }
    }
}

pub struct NiriConfigService {
    pub state: NiriSettingsState,
}

impl Service for NiriConfigService {
    fn new() -> Self {
        let mut svc = Self {
            state: NiriSettingsState::default(),
        };
        svc.load_from_kdl();
        svc
    }

    fn interval() -> Duration {
        Duration::from_secs(60)
    }

    fn update(&mut self) -> bool {
        false
    }
}

impl NiriConfigService {
    fn conf_d_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dotfield_conf = PathBuf::from(&home)
            .join("dev/projects/dotfield/compositor/niri/conf.d/99-amane-settings.kdl");
        if dotfield_conf.parent().is_some_and(|p| p.exists()) {
            return dotfield_conf;
        }
        PathBuf::from(&home).join(".config/niri/conf.d/99-amane-settings.kdl")
    }

    fn master_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dotfield_master =
            PathBuf::from(&home).join("dev/projects/dotfield/compositor/niri/config.kdl");
        if dotfield_master.exists() {
            return dotfield_master;
        }
        PathBuf::from(&home).join(".config/niri/config.kdl")
    }

    pub fn load_from_kdl(&mut self) {
        let path = Self::conf_d_path();
        if !path.exists() {
            return;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            self.parse_kdl_content(&content);
        }
    }

    pub fn parse_kdl_content(&mut self, content: &str) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(val) = trimmed.strip_prefix("gaps ") {
                if let Ok(g) = val.parse::<f32>() {
                    self.state.gaps = g;
                }
            } else if let Some(val) = trimmed
                .strip_prefix("default-column-width { proportion ")
                .and_then(|s| s.strip_suffix("; }"))
            {
                if let Ok(p) = val.parse::<f32>() {
                    self.state.default_column_width = p;
                }
            } else if let Some(val) = trimmed.strip_prefix("geometry-corner-radius ") {
                if let Ok(r) = val.parse::<f32>() {
                    self.state.corner_radius = r;
                }
            } else if let Some(val) = trimmed.strip_prefix("repeat-delay ") {
                if let Ok(d) = val.parse::<u32>() {
                    self.state.repeat_delay = d;
                }
            } else if let Some(val) = trimmed.strip_prefix("repeat-rate ") {
                if let Ok(r) = val.parse::<u32>() {
                    self.state.repeat_rate = r;
                }
            } else if let Some(val) = trimmed.strip_prefix("active-color ") {
                let cleaned = val.trim_matches('"');
                self.state.active_border_color = cleaned.to_string();
            }
        }
    }

    pub fn generate_kdl(state: &NiriSettingsState) -> String {
        let mut out = String::new();
        out.push_str("// Dotfield Amane Generated Dynamic Compositor Configuration\n");
        out.push_str("// Auto-generated by amane-settings UI with rollback safety\n\n");

        // Layout & Workspaces
        out.push_str("layout {\n");
        out.push_str(&format!("    gaps {}\n", state.gaps));
        out.push_str(&format!(
            "    center-focused-column \"{}\"\n",
            state.center_focused_column
        ));
        out.push_str(&format!(
            "    default-column-width {{ proportion {}; }}\n",
            state.default_column_width
        ));
        out.push_str("    border {\n");
        out.push_str(&format!("        width {}\n", state.border_width));
        out.push_str(&format!(
            "        active-color \"{}\"\n",
            state.active_border_color
        ));
        out.push_str(&format!(
            "        inactive-color \"{}\"\n",
            state.inactive_border_color
        ));
        out.push_str("    }\n");
        out.push_str("}\n\n");

        // Window Rules (Geometry Corner Radius & Opacity)
        out.push_str("window-rule {\n");
        out.push_str(&format!(
            "    geometry-corner-radius {}\n",
            state.corner_radius
        ));
        out.push_str("    clip-to-geometry true\n");
        out.push_str("}\n\n");

        // Animations
        if state.animations_enabled {
            out.push_str("animations {\n");
            out.push_str("    workspace-switch {\n");
            out.push_str(&format!(
                "        duration-ms {}\n",
                state.animation_duration_ms
            ));
            out.push_str(&format!("        curve \"{}\"\n", state.animation_curve));
            out.push_str("    }\n");
            out.push_str("    horizontal-view-movement {\n");
            out.push_str(&format!(
                "        duration-ms {}\n",
                state.animation_duration_ms
            ));
            out.push_str(&format!("        curve \"{}\"\n", state.animation_curve));
            out.push_str("    }\n");
            out.push_str("    window-open {\n");
            out.push_str(&format!(
                "        duration-ms {}\n",
                state.animation_duration_ms
            ));
            out.push_str(&format!("        curve \"{}\"\n", state.animation_curve));
            out.push_str("    }\n");
            out.push_str("    window-close {\n");
            out.push_str(&format!(
                "        duration-ms {}\n",
                (state.animation_duration_ms as f32 * 0.9) as u32
            ));
            out.push_str(&format!("        curve \"{}\"\n", state.animation_curve));
            out.push_str("    }\n");
            out.push_str("}\n\n");
        } else {
            out.push_str("animations {\n");
            out.push_str("    off\n");
            out.push_str("}\n\n");
        }

        // Input & Touchpad
        out.push_str("input {\n");
        out.push_str("    keyboard {\n");
        out.push_str(&format!("        repeat-delay {}\n", state.repeat_delay));
        out.push_str(&format!("        repeat-rate {}\n", state.repeat_rate));
        out.push_str("    }\n");
        out.push_str("    touchpad {\n");
        if state.tap_to_click {
            out.push_str("        tap\n");
        }
        if state.natural_scroll {
            out.push_str("        natural-scroll\n");
        }
        if state.dwt {
            out.push_str("        dwt\n");
        }
        out.push_str("    }\n");
        out.push_str("}\n");

        out
    }

    pub fn update_and_validate<F>(mutate: F) -> Result<(), String>
    where
        F: FnOnce(&mut NiriSettingsState),
    {
        Self::apply_and_validate(mutate)
    }

    pub fn apply_and_validate<F>(mutate: F) -> Result<(), String>
    where
        F: FnOnce(&mut NiriSettingsState),
    {
        let target_path = Self::conf_d_path();
        let backup_path = target_path.with_extension("kdl.bak");
        let master_path = Self::master_config_path();

        let old_state = {
            let svc = NiriConfigService::read();
            svc.state.clone()
        };

        let mut new_state = old_state.clone();
        mutate(&mut new_state);

        // Ensure parent dir exists
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Create backup of current 99-amane-settings.kdl if it exists
        if target_path.exists() {
            let _ = fs::copy(&target_path, &backup_path);
        }

        let rendered = Self::generate_kdl(&new_state);
        if let Err(e) = fs::write(&target_path, &rendered) {
            return Err(format!("Failed to write dynamic config: {e}"));
        }

        // Validate using `niri validate -c <master_path>`
        let validate_status = Command::new("niri")
            .args(["validate", "-c", &master_path.to_string_lossy()])
            .output();

        match validate_status {
            Ok(output) if output.status.success() => {
                // Success! Update in-memory state
                {
                    let mut svc = NiriConfigService::write();
                    svc.state = new_state;
                    svc.state.last_status = Some("Niri config validated and applied".to_string());
                }
                // Reload live Niri session via IPC
                let _ = Command::new("niri")
                    .args(["msg", "action", "load-config-file"])
                    .status();

                // Clean backup
                if backup_path.exists() {
                    let _ = fs::remove_file(&backup_path);
                }
                amane::request_frame();
                Ok(())
            }
            Ok(output) => {
                // Validation failed! Rollback immediately
                let err_msg = String::from_utf8_lossy(&output.stderr).to_string();
                if backup_path.exists() {
                    let _ = fs::copy(&backup_path, &target_path);
                    let _ = fs::remove_file(&backup_path);
                } else if target_path.exists() {
                    let _ = fs::remove_file(&target_path);
                }
                {
                    let mut svc = NiriConfigService::write();
                    svc.state = old_state;
                    svc.state.last_status =
                        Some(format!("Validation failed, rolled back: {err_msg}"));
                }
                Err(format!("Validation failed: {err_msg}"))
            }
            Err(_e) => {
                // niri binary not runnable or not in PATH
                // In non-niri environments (or headless tests), accept valid KDL
                {
                    let mut svc = NiriConfigService::write();
                    svc.state = new_state;
                    svc.state.last_status =
                        Some("Applied without active niri validation".to_string());
                }
                if backup_path.exists() {
                    let _ = fs::remove_file(&backup_path);
                }
                amane::request_frame();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdl_generation_and_defaults() {
        let state = NiriSettingsState::default();
        let rendered = NiriConfigService::generate_kdl(&state);
        assert!(rendered.contains("gaps 8"));
        assert!(rendered.contains("default-column-width { proportion 0.5; }"));
        assert!(rendered.contains("geometry-corner-radius 8"));
        assert!(rendered.contains("repeat-delay 200"));
        assert!(rendered.contains("natural-scroll"));
        assert!(rendered.contains("workspace-switch"));
    }

    #[test]
    fn test_kdl_parsing_values() {
        let content = r##"
            layout {
                gaps 16
                default-column-width { proportion 0.66667; }
                border {
                    active-color "#FF007F"
                }
            }
            window-rule {
                geometry-corner-radius 14
            }
            input {
                keyboard {
                    repeat-delay 180
                    repeat-rate 60
                }
            }
        "##;
        let mut svc = NiriConfigService {
            state: NiriSettingsState::default(),
        };
        svc.parse_kdl_content(content);
        assert_eq!(svc.state.gaps, 16.0);
        assert!((svc.state.default_column_width - 0.66667).abs() < 0.001);
        assert_eq!(svc.state.corner_radius, 14.0);
        assert_eq!(svc.state.repeat_delay, 180);
        assert_eq!(svc.state.repeat_rate, 60);
        assert_eq!(svc.state.active_border_color, "#FF007F");
    }

    #[test]
    fn test_animations_disabled_kdl() {
        let state = NiriSettingsState {
            animations_enabled: false,
            ..Default::default()
        };
        let rendered = NiriConfigService::generate_kdl(&state);
        assert!(rendered.contains("animations {\n    off\n}"));
    }
}
