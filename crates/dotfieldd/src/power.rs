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

    let mut found_battery = false;
    let mut battery_discharging = false;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let sub_path = entry.path();

        // 1. Check AC / Mains adapter
        let type_file = sub_path.join("type");
        let is_mains = if type_file.exists() {
            fs::read_to_string(&type_file)
                .map(|t| t.trim().eq_ignore_ascii_case("Mains"))
                .unwrap_or(false)
        } else {
            file_name.starts_with("AC") || file_name.starts_with("ADP")
        };

        if is_mains {
            let online_file = sub_path.join("online");
            if let Ok(content) = fs::read_to_string(&online_file) {
                if content.trim() == "1" {
                    return PowerSource::Ac;
                }
            }
        }

        // 2. Track Battery status
        if file_name.starts_with("BAT") {
            found_battery = true;
            let status_file = sub_path.join("status");
            if let Ok(status) = fs::read_to_string(&status_file) {
                let s = status.trim().to_lowercase();
                if s == "discharging" {
                    battery_discharging = true;
                }
            }
        }
    }

    if battery_discharging {
        PowerSource::Battery
    } else if found_battery {
        // Battery present but not discharging (likely plugged in / AC)
        PowerSource::Ac
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
        fs::write(bat_dir.join("status"), "Discharging\n").unwrap();

        assert_eq!(detect_power_source(dir.path()), PowerSource::Battery);
    }

    #[test]
    fn test_detect_unknown_when_empty() {
        let dir = tempdir().unwrap();
        assert_eq!(detect_power_source(dir.path()), PowerSource::Unknown);
    }
}
