use std::path::Path;
use std::process::Command;

use crate::error::{DotfieldError, Result};
use crate::profile::Profile;

pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
}

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
            if let Some(ref p) = profile.security.nftables_ruleset {
                if !Path::new(p).exists() {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!("Configured NFTables ruleset does not exist: '{p}'"),
                    });
                }
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
        Command::new("which")
            .arg(binary)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
}
