use std::path::Path;

use crate::error::{DotfieldError, Result};
use crate::profile::Profile;

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Warning,
    Error,
}

pub struct Validator;

impl Validator {
    pub fn validate(profile: &Profile) -> Result<Vec<ValidationIssue>> {
        let mut issues = Vec::new();

        // 1. Check supported compositor
        match profile.compositor.engine.as_str() {
            "sway" | "niri" => {}
            other => {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!(
                        "Unsupported compositor engine: '{other}'. Expected 'sway' or 'niri'"
                    ),
                });
            }
        }

        // Check theme palette support
        if profile.theme.palette != "controlled-chaos" {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Error,
                message: format!(
                    "Unsupported theme palette: '{}'. Expected 'controlled-chaos'",
                    profile.theme.palette
                ),
            });
        }

        // 2. Check binary presence on PATH for configured apps
        let apps_to_check = [
            ("terminal", &profile.apps.terminal),
            ("launcher", &profile.apps.launcher),
        ];

        for (role, binary) in apps_to_check {
            if !Self::binary_exists(binary) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning,
                    message: format!("Configured {role} '{binary}' not found on system PATH"),
                });
            }
        }

        // 3. Check security paths if specified
        if let Some(true) = profile.security.firejail {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Security option 'firejail' is declared, but automatic sandboxing enforcement is not yet implemented by the compiler runtime".to_string(),
            });
            if let Some(ref p) = profile.security.firejail_profile {
                if !Path::new(p).exists() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!("Configured Firejail profile path does not exist: '{p}'"),
                    });
                }
            }
        }

        if let Some(true) = profile.security.nftables {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Security option 'nftables' is declared, but automatic firewall rule enforcement is not yet implemented by the compiler runtime".to_string(),
            });
            if let Some(ref p) = profile.security.nftables_ruleset {
                if !Path::new(p).exists() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!("Configured NFTables ruleset does not exist: '{p}'"),
                    });
                }
            }
        }

        if let Some(true) = profile.security.apparmor {
            issues.push(ValidationIssue {
                severity: IssueSeverity::Warning,
                message: "Security option 'apparmor' is declared, but automatic AppArmor profile loading is not yet implemented by the compiler runtime".to_string(),
            });
            if let Some(ref p) = profile.security.apparmor_profile {
                if !Path::new(p).exists() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!("Configured AppArmor profile does not exist: '{p}'"),
                    });
                }
            }
        }

        // 4. Validate binding action names against allowlist
        const KNOWN_ACTIONS: &[&str] = &[
            "terminal",
            "launcher",
            "app_menu",
            "hud",
            "close_window",
            "fullscreen",
            "lock",
            "screenshot_area",
            "screenshot_full",
        ];

        let mut seen_keys: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        for (action, key) in &profile.bindings {
            if !KNOWN_ACTIONS.contains(&action.as_str()) && !action.starts_with("exec:") {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!(
                        "Unknown keybinding action '{action}'. Expected one of: {} or explicit 'exec:<command>'",
                        KNOWN_ACTIONS.join(", ")
                    ),
                });
            }

            // Normalize keybinding (case-insensitive, Super/Mod4 -> mod)
            let mut parts: Vec<String> = key
                .split('+')
                .map(|p| {
                    let mut s = p.trim().to_lowercase();
                    if s == "super" || s == "mod4" {
                        s = "mod".to_string();
                    }
                    s
                })
                .collect();
            parts.sort();
            let normalized_key = parts.join("+");

            if let Some(existing_action) = seen_keys.get(&normalized_key) {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error,
                    message: format!(
                        "Duplicate keybinding '{key}' for action '{action}' conflicts with existing action '{existing_action}'"
                    ),
                });
            } else {
                seen_keys.insert(normalized_key, action.clone());
            }
        }

        // If any error severity issues found, return failure
        let has_errors = issues
            .iter()
            .any(|i| matches!(i.severity, IssueSeverity::Error));
        if has_errors {
            let error_messages: Vec<String> = issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Error))
                .map(|i| i.message.clone())
                .collect();
            return Err(DotfieldError::Validation(error_messages.join("; ")));
        }

        Ok(issues)
    }

    fn binary_exists(binary: &str) -> bool {
        let bin_name = binary.split_whitespace().next().unwrap_or(binary);
        let path = Path::new(bin_name);
        if path.is_absolute() || path.components().count() > 1 {
            return Self::is_executable(path);
        }
        if let Some(paths) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&paths) {
                let candidate = dir.join(bin_name);
                if Self::is_executable(&candidate) {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(unix)]
    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
        } else {
            false
        }
    }

    #[cfg(not(unix))]
    fn is_executable(path: &Path) -> bool {
        path.is_file()
    }
}
