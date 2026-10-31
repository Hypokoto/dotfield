use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileMeta {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompositorConfig {
    pub engine: String,
    pub scaling: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppsConfig {
    pub terminal: String,
    pub launcher: String,
    pub editor: Option<String>,
    pub multiplexer: Option<String>,
    pub notifications: Option<String>,
    pub bar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeConfig {
    pub palette: String,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
pub struct Profile {
    pub meta: ProfileMeta,
    pub compositor: CompositorConfig,
    pub apps: AppsConfig,
    pub theme: ThemeConfig,
    pub bindings: HashMap<String, String>,
    #[serde(default)]
    pub security: SecurityConfig,
}

impl Profile {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
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
