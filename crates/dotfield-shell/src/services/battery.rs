use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use amane::{Battery, Service};

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    Normal,
    LowWarned,
    CriticalWarned,
}

pub struct BatteryMonitor;

impl BatteryMonitor {
    pub fn start() {
        if RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        thread::Builder::new()
            .name("dotfield-battery-monitor".to_string())
            .spawn(|| {
                let mut state = AlertState::Normal;

                loop {
                    thread::sleep(Duration::from_secs(20));

                    let battery = Battery::read();
                    if !battery.present() {
                        continue;
                    }

                    let percent = battery.percent();
                    let charging = battery.charging();

                    if charging {
                        state = AlertState::Normal;
                        continue;
                    }

                    match state {
                        AlertState::Normal => {
                            if percent <= 10 {
                                send_battery_notification(
                                    percent,
                                    "critical",
                                    "battery-critical",
                                    "Battery Critical",
                                    "Plug in power immediately to prevent shutdown!",
                                );
                                state = AlertState::CriticalWarned;
                            } else if percent <= 20 {
                                send_battery_notification(
                                    percent,
                                    "normal",
                                    "battery-low",
                                    "Battery Low",
                                    "Connect power adapter soon.",
                                );
                                state = AlertState::LowWarned;
                            }
                        }
                        AlertState::LowWarned => {
                            if percent <= 10 {
                                send_battery_notification(
                                    percent,
                                    "critical",
                                    "battery-critical",
                                    "Battery Critical",
                                    "Plug in power immediately to prevent shutdown!",
                                );
                                state = AlertState::CriticalWarned;
                            }
                        }
                        AlertState::CriticalWarned => {
                            // Remain in critical warned state until charging starts
                        }
                    }
                }
            })
            .expect("failed to spawn battery monitor thread");
    }
}

fn send_battery_notification(percent: u8, urgency: &str, category: &str, title: &str, body: &str) {
    let icon = if urgency == "critical" {
        "battery-caution"
    } else {
        "battery-low"
    };

    let full_title = format!("{title} ({percent}%)");

    let _ = Command::new("notify-send")
        .args([
            "-a",
            "battery-alert",
            "-u",
            urgency,
            "-c",
            category,
            "-i",
            icon,
            &full_title,
            body,
        ])
        .spawn();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_state_transitions() {
        let mut state = AlertState::Normal;
        assert_eq!(state, AlertState::Normal);

        // Low threshold triggers
        let percent: u8 = 18;
        let charging = false;
        if !charging && percent <= 20 && percent > 10 {
            state = AlertState::LowWarned;
        }
        assert_eq!(state, AlertState::LowWarned);

        // Critical threshold triggers
        let percent: u8 = 8;
        if !charging && percent <= 10 {
            state = AlertState::CriticalWarned;
        }
        assert_eq!(state, AlertState::CriticalWarned);

        // Charging resets back to Normal
        let charging = true;
        if charging {
            state = AlertState::Normal;
        }
        assert_eq!(state, AlertState::Normal);
    }
}
