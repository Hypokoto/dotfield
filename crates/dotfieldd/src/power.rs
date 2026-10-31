use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSource {
    Ac,
    Battery,
    Unknown,
}

pub fn detect_power_source<P: AsRef<Path>>(sysfs_path: P) -> PowerSource {
    let path = sysfs_path.as_ref();
    if !path.exists() {
        return PowerSource::Unknown;
    }

    let entries = match fs::read_dir(path) {
        Ok(e) => e,
        Err(_) => return PowerSource::Unknown,
    };

    let mut has_ac_online = false;
    let mut has_ac_offline = false;
    let mut found_battery = false;
    let mut battery_discharging = false;
    let mut battery_charging = false;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let sub_path = entry.path();

        let supply_type = sub_path
            .join("type")
            .exists()
            .then(|| fs::read_to_string(sub_path.join("type")).ok())
            .flatten()
            .map(|t| t.trim().to_lowercase())
            .unwrap_or_else(|| {
                if file_name.starts_with("AC") || file_name.starts_with("ADP") {
                    "mains".to_string()
                } else if file_name.starts_with("BAT") || file_name.starts_with("CMB") {
                    "battery".to_string()
                } else {
                    "unknown".to_string()
                }
            });

        // 1. Check Mains / AC adapter / USB variants (USB, USB_PD, USB_C)
        if supply_type == "mains" || supply_type.starts_with("usb") {
            let online_file = sub_path.join("online");
            if let Ok(content) = fs::read_to_string(&online_file) {
                match content.trim() {
                    "1" => has_ac_online = true,
                    "0" => has_ac_offline = true,
                    _ => {}
                }
            }
        }

        // 2. Track Battery status (ignore peripheral device-scope batteries)
        let is_device_scope = fs::read_to_string(sub_path.join("scope"))
            .map(|s| s.trim().eq_ignore_ascii_case("device"))
            .unwrap_or(false);
        if supply_type == "battery" && !is_device_scope {
            found_battery = true;
            let status_file = sub_path.join("status");
            if let Ok(status) = fs::read_to_string(&status_file) {
                let s = status.trim().to_lowercase();
                if s == "discharging" {
                    battery_discharging = true;
                } else if s == "charging" {
                    battery_charging = true;
                }
            }
        }
    }

    if battery_discharging {
        PowerSource::Battery
    } else if has_ac_online || battery_charging {
        PowerSource::Ac
    } else if found_battery && has_ac_offline {
        PowerSource::Battery
    } else {
        PowerSource::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_ac_online() {
        let dir = tempdir().unwrap();
        let ac_dir = dir.path().join("ACAD");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("type"), "Mains\n").unwrap();
        fs::write(ac_dir.join("online"), "1\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Ac);
    }

    #[test]
    fn test_detect_battery_discharging() {
        let dir = tempdir().unwrap();
        let ac_dir = dir.path().join("AC");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("type"), "Mains\n").unwrap();
        fs::write(ac_dir.join("online"), "0\n").unwrap();

        let bat_dir = dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("type"), "Battery\n").unwrap();
        fs::write(bat_dir.join("status"), "Discharging\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Battery);
    }

    #[test]
    fn test_battery_discharging_takes_priority_over_ac_online() {
        let dir = tempdir().unwrap();
        let ac_dir = dir.path().join("AC");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("type"), "Mains\n").unwrap();
        fs::write(ac_dir.join("online"), "1\n").unwrap();

        let bat_dir = dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("type"), "Battery\n").unwrap();
        fs::write(bat_dir.join("status"), "Discharging\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Battery);
    }

    #[test]
    fn test_detect_battery_not_charging_when_ac_offline() {
        let dir = tempdir().unwrap();
        let ac_dir = dir.path().join("AC");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("type"), "Mains\n").unwrap();
        fs::write(ac_dir.join("online"), "0\n").unwrap();

        let bat_dir = dir.path().join("CMB0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("type"), "Battery\n").unwrap();
        fs::write(bat_dir.join("status"), "Not charging\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Battery);
    }

    #[test]
    fn test_ignore_device_scope_battery_on_desktop() {
        let dir = tempdir().unwrap();
        // Desktop machine without AC power supply node, but with wireless mouse battery
        let mouse_bat = dir.path().join("hidpp_battery_0");
        fs::create_dir_all(&mouse_bat).unwrap();
        fs::write(mouse_bat.join("type"), "Battery\n").unwrap();
        fs::write(mouse_bat.join("scope"), "Device\n").unwrap();
        fs::write(mouse_bat.join("status"), "Discharging\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Unknown);
    }

    #[test]
    fn test_detect_usb_pd_online() {
        let dir = tempdir().unwrap();
        let usb_pd_dir = dir.path().join("tcpm-source-psy-i2c-0-0022");
        fs::create_dir_all(&usb_pd_dir).unwrap();
        fs::write(usb_pd_dir.join("type"), "USB_PD\n").unwrap();
        fs::write(usb_pd_dir.join("online"), "1\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Ac);
    }

    #[test]
    fn test_battery_charging_without_adapter_signals_ac() {
        let dir = tempdir().unwrap();
        let bat_dir = dir.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("type"), "Battery\n").unwrap();
        fs::write(bat_dir.join("status"), "Charging\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Ac);
    }

    #[test]
    fn test_detect_unknown_when_empty() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_power_source(dir.path()), PowerSource::Unknown);
    }
}
