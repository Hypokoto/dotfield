use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProfileMeta {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CompositorConfig {
    pub engine: String,
    pub scaling: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppsConfig {
    pub terminal: String,
    pub launcher: String,
    pub editor: Option<String>,
    pub multiplexer: Option<String>,
    pub notifications: Option<String>,
    pub bar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeConfig {
    pub palette: String,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    pub firejail: Option<bool>,
    pub firejail_profile: Option<String>,
    pub flatpak_overrides: Option<bool>,
    pub nftables: Option<bool>,
    pub nftables_ruleset: Option<String>,
    pub apparmor: Option<bool>,
    pub apparmor_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub meta: ProfileMeta,
    pub compositor: CompositorConfig,
    pub apps: AppsConfig,
    pub theme: ThemeConfig,
    pub bindings: BTreeMap<String, String>,
    #[serde(default)]
    pub security: SecurityConfig,
}

impl Profile {
    pub fn profile_search_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Ok(dir) = std::env::var("DOTFIELD_PROFILES_DIR") {
            let p = PathBuf::from(dir);
            if p.exists() {
                dirs.push(p);
            }
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let user_profiles = PathBuf::from(&home).join(".config/dotfield/profiles");
        if user_profiles.exists() {
            dirs.push(user_profiles);
        }

        let sys_profiles = PathBuf::from("/usr/share/dotfield/profiles");
        if sys_profiles.exists() {
            dirs.push(sys_profiles);
        }

        let local_profiles = PathBuf::from("profiles");
        if local_profiles.exists() {
            dirs.push(local_profiles);
        }

        dirs
    }

    pub fn default_profiles_dir() -> PathBuf {
        Self::profile_search_dirs()
            .into_iter()
            .next()
            .unwrap_or_else(|| PathBuf::from("profiles"))
    }

    pub fn resolve_profile_path<P: AsRef<Path>>(name_or_path: P) -> PathBuf {
        let p = name_or_path.as_ref();
        if p.exists() {
            return p.to_path_buf();
        }

        // Only search profile directories for bare names (e.g. "default", "battery.toml")
        // or direct children of profiles/ (e.g. "profiles/default.toml").
        // Never substitute missing nested paths (e.g. "profiles/team/default.toml") or arbitrary paths.
        let is_bare = p.components().count() <= 1;
        let is_direct_profiles_child = p.parent() == Some(Path::new("profiles"))
            || p.parent() == Some(Path::new("./profiles"));

        if is_bare || is_direct_profiles_child {
            let file_name = if p.extension().is_some() {
                p.to_path_buf()
            } else {
                PathBuf::from(format!("{}.toml", p.display()))
            };

            for dir in Self::profile_search_dirs() {
                let candidate = dir.join(&file_name);
                if candidate.exists() {
                    return candidate;
                }

                if let Some(base) = file_name.file_name() {
                    let candidate = dir.join(base);
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }

        p.to_path_buf()
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let resolved = Self::resolve_profile_path(path);
        let content = std::fs::read_to_string(resolved)?;
        content.parse()
    }

    pub fn to_string_pretty(&self) -> Result<String> {
        let s = toml::to_string_pretty(self)?;
        Ok(s)
    }
}

impl std::str::FromStr for Profile {
    type Err = crate::error::DotfieldError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let profile: Profile = toml::from_str(s)?;
        Ok(profile)
    }
}
