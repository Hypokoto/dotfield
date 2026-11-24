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

                    let (new_state, alert_opt) = evaluate_battery_alert(state, percent, charging);
                    state = new_state;

                    if let Some(alert) = alert_opt {
                        send_battery_notification(
                            percent,
                            alert.urgency,
                            alert.category,
                            alert.title,
                            alert.body,
                        );
                    }
                }
            })
            .expect("failed to spawn battery monitor thread");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BatteryAlertMessage {
    pub urgency: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub body: &'static str,
}

/// Pure state machine evaluation for battery alert notifications.
pub fn evaluate_battery_alert(
    current_state: AlertState,
    percent: u8,
    charging: bool,
) -> (AlertState, Option<BatteryAlertMessage>) {
    if charging {
        return (AlertState::Normal, None);
    }

    match current_state {
        AlertState::Normal => {
            if percent <= 10 {
                (
                    AlertState::CriticalWarned,
                    Some(BatteryAlertMessage {
                        urgency: "critical",
                        category: "battery-critical",
                        title: "Battery Critical",
                        body: "Plug in power immediately to prevent shutdown!",
                    }),
                )
            } else if percent <= 20 {
                (
                    AlertState::LowWarned,
                    Some(BatteryAlertMessage {
                        urgency: "normal",
                        category: "battery-low",
                        title: "Battery Low",
                        body: "Connect power adapter soon.",
                    }),
                )
            } else {
                (AlertState::Normal, None)
            }
        }
        AlertState::LowWarned => {
            if percent <= 10 {
                (
                    AlertState::CriticalWarned,
                    Some(BatteryAlertMessage {
                        urgency: "critical",
                        category: "battery-critical",
                        title: "Battery Critical",
                        body: "Plug in power immediately to prevent shutdown!",
                    }),
                )
            } else {
                (AlertState::LowWarned, None)
            }
        }
        AlertState::CriticalWarned => (AlertState::CriticalWarned, None),
    }
}

fn send_battery_notification(
    percent: u8,
    urgency_str: &str,
    _category: &str,
    title: &str,
    body: &str,
) {
    let full_title = format!("{title} ({percent}%)");
    let urgency = match urgency_str {
        "critical" => amane::Urgency::Critical,
        "low" => amane::Urgency::Low,
        _ => amane::Urgency::Normal,
    };

    if amane::Notifications::read().running() {
        amane::Notifications::notify("battery-alert", full_title, body, urgency);
    } else {
        let icon = if urgency_str == "critical" {
            "battery-caution"
        } else {
            "battery-low"
        };
        let _ = Command::new("notify-send")
            .args([
                "-a",
                "battery-alert",
                "-u",
                urgency_str,
                "-i",
                icon,
                &full_title,
                body,
            ])
            .spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_state_transitions_normal_discharging() {
        // High battery: normal state, no alert
        let (state, alert) = evaluate_battery_alert(AlertState::Normal, 80, false);
        assert_eq!(state, AlertState::Normal);
        assert!(alert.is_none());

        // Boundary just above low warning (21%)
        let (state, alert) = evaluate_battery_alert(AlertState::Normal, 21, false);
        assert_eq!(state, AlertState::Normal);
        assert!(alert.is_none());

        // Low threshold trigger (20%)
        let (state, alert) = evaluate_battery_alert(AlertState::Normal, 20, false);
        assert_eq!(state, AlertState::LowWarned);
        let alert = alert.expect("should trigger low battery alert");
        assert_eq!(alert.urgency, "normal");
        assert_eq!(alert.title, "Battery Low");

        // Subsequent check in low range (18%) shouldn't re-trigger spam
        let (state, alert) = evaluate_battery_alert(AlertState::LowWarned, 18, false);
        assert_eq!(state, AlertState::LowWarned);
        assert!(alert.is_none());
    }

    #[test]
    fn test_alert_state_transitions_critical_discharging() {
        // Drop from LowWarned to critical (10%)
        let (state, alert) = evaluate_battery_alert(AlertState::LowWarned, 10, false);
        assert_eq!(state, AlertState::CriticalWarned);
        let alert = alert.expect("should trigger critical battery alert");
        assert_eq!(alert.urgency, "critical");
        assert_eq!(alert.title, "Battery Critical");

        // Further drop while CriticalWarned (5%) shouldn't re-trigger spam
        let (state, alert) = evaluate_battery_alert(AlertState::CriticalWarned, 5, false);
        assert_eq!(state, AlertState::CriticalWarned);
        assert!(alert.is_none());

        // Direct drop from Normal to critical (e.g. waking at 8%)
        let (state, alert) = evaluate_battery_alert(AlertState::Normal, 8, false);
        assert_eq!(state, AlertState::CriticalWarned);
        assert_eq!(alert.unwrap().urgency, "critical");
    }

    #[test]
    fn test_alert_state_charging_resets() {
        // Charging plugged in while LowWarned resets state to Normal without alert
        let (state, alert) = evaluate_battery_alert(AlertState::LowWarned, 15, true);
        assert_eq!(state, AlertState::Normal);
        assert!(alert.is_none());

        // Charging plugged in while CriticalWarned resets state to Normal without alert
        let (state, alert) = evaluate_battery_alert(AlertState::CriticalWarned, 8, true);
        assert_eq!(state, AlertState::Normal);
        assert!(alert.is_none());
    }
}
