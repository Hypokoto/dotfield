use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

use amane::Service;

use crate::config::{ConfigService, ShellManifest};

static NIGHT_LIGHT_ACTIVE: AtomicBool = AtomicBool::new(false);
static NIGHT_LIGHT_TEMP: AtomicU32 = AtomicU32::new(4000);

pub struct NightLightService {
    pub enabled: bool,
    pub night_temp: u32,
    pub day_temp: u32,
    pub mode: String,
}

impl Service for NightLightService {
    fn new() -> Self {
        let manifest = ConfigService::read().manifest.clone();
        let enabled = manifest.night_light.enabled;
        let night_temp = manifest.night_light.night_temp;
        let day_temp = manifest.night_light.day_temp;
        let mode = manifest.night_light.mode.clone();

        NIGHT_LIGHT_ACTIVE.store(enabled, Ordering::Relaxed);
        NIGHT_LIGHT_TEMP.store(night_temp, Ordering::Relaxed);

        if enabled {
            spawn_wlsunset(&manifest);
        }

        Self {
            enabled,
            night_temp,
            day_temp,
            mode,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(60)
    }
}

impl NightLightService {
    pub fn is_active() -> bool {
        NIGHT_LIGHT_ACTIVE.load(Ordering::Relaxed)
    }

    pub fn current_temp() -> u32 {
        NIGHT_LIGHT_TEMP.load(Ordering::Relaxed)
    }

    pub fn sync_with_manifest() {
        let manifest = ConfigService::read().manifest.clone();
        let enabled = manifest.night_light.enabled;
        let temp = manifest.night_light.night_temp;

        NIGHT_LIGHT_ACTIVE.store(enabled, Ordering::Relaxed);
        NIGHT_LIGHT_TEMP.store(temp, Ordering::Relaxed);

        if enabled {
            kill_wlsunset();
            spawn_wlsunset(&manifest);
        } else {
            kill_wlsunset();
        }
    }

    pub fn toggle() {
        let _ = ConfigService::update_and_save(|m| {
            m.night_light.enabled = !m.night_light.enabled;
        });
        Self::sync_with_manifest();
        amane::request_frame();
    }

    pub fn set_temperature(temp: u32) {
        let clamped = temp.clamp(1000, 10000);
        let _ = ConfigService::update_and_save(|m| {
            m.night_light.night_temp = clamped;
        });
        Self::sync_with_manifest();
        amane::request_frame();
    }

    pub fn set_mode(mode: &str) {
        let mode_str = mode.to_string();
        let _ = ConfigService::update_and_save(|m| {
            m.night_light.mode = mode_str;
        });
        Self::sync_with_manifest();
        amane::request_frame();
    }
}

fn kill_wlsunset() {
    let _ = Command::new("pkill").args(["-x", "wlsunset"]).status();
}

fn spawn_wlsunset(manifest: &ShellManifest) {
    let night_temp = manifest.night_light.night_temp.to_string();
    let day_temp = manifest.night_light.day_temp.to_string();

    let mut cmd = Command::new("wlsunset");

    if manifest.night_light.mode == "schedule" {
        cmd.args(["-t", &night_temp, "-T", &day_temp]);
        if let (Some(lat), Some(lon)) = (
            manifest.night_light.latitude,
            manifest.night_light.longitude,
        ) {
            let lat_str = format!("{lat:.4}");
            let lon_str = format!("{lon:.4}");
            cmd.args(["-l", &lat_str, "-L", &lon_str]);
        }
    } else {
        // In manual mode, lock temperature to target
        cmd.args(["-t", &night_temp, "-T", &night_temp]);
    }

    let _ = cmd.spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_night_light_service_initialization() {
        let manifest = ShellManifest::default();
        assert!(!manifest.night_light.enabled);
        assert_eq!(manifest.night_light.night_temp, 4000);
        assert_eq!(manifest.night_light.day_temp, 6500);
        assert_eq!(manifest.night_light.mode, "manual");
    }

    #[test]
    fn test_night_light_temp_clamping() {
        let clamped_low = 500_u32.clamp(1000, 10000);
        assert_eq!(clamped_low, 1000);
        let clamped_high = 15000_u32.clamp(1000, 10000);
        assert_eq!(clamped_high, 10000);
    }
}
