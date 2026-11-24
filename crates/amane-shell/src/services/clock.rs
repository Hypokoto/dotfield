use std::thread;
use std::time::Duration;

use amane::Service;
use chrono::{Local, Timelike};

pub struct ClockService {
    formatted: String,
    time_formatted: String,
    date_formatted: String,
    last_minute_epoch: i64,
}

impl Service for ClockService {
    fn new() -> Self {
        let now = Local::now();
        Self {
            formatted: now.format("%a %b %d  %H:%M").to_string(),
            time_formatted: now.format("%H:%M").to_string(),
            date_formatted: now.format("%d/%m/%Y").to_string(),
            last_minute_epoch: now.timestamp() / 60,
        }
    }

    fn listen() {
        loop {
            let now = Local::now();
            let second = now.second();
            let subsec_millis = now.timestamp_subsec_millis();
            let millis_until_next_minute =
                ((60 - second) as u64 * 1000).saturating_sub(subsec_millis as u64);
            // Add a small 20ms margin to guarantee crossing into the next minute cleanly
            let sleep_duration = Duration::from_millis(millis_until_next_minute + 20);

            thread::sleep(sleep_duration);

            let mut service = Self::write();
            if !service.update() {
                service.quiet();
            }
        }
    }

    fn update(&mut self) -> bool {
        self.update_with(Local::now())
    }
}

impl ClockService {
    /// Pure update logic parameterized by datetime for deterministic testing.
    pub fn update_with<Tz: chrono::TimeZone>(&mut self, now: chrono::DateTime<Tz>) -> bool
    where
        Tz::Offset: std::fmt::Display,
    {
        let minute_epoch = now.timestamp() / 60;
        if minute_epoch != self.last_minute_epoch {
            self.last_minute_epoch = minute_epoch;
            self.formatted = now.format("%a %b %d  %H:%M").to_string();
            self.time_formatted = now.format("%H:%M").to_string();
            self.date_formatted = now.format("%d/%m/%Y").to_string();
            true
        } else {
            false
        }
    }

    pub fn formatted(&self) -> &str {
        &self.formatted
    }

    pub fn time(&self) -> &str {
        &self.time_formatted
    }

    pub fn date(&self) -> &str {
        &self.date_formatted
    }

    pub fn last_minute(&self) -> u32 {
        (self.last_minute_epoch.rem_euclid(60)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_clock_service_initialization() {
        let clock = ClockService::new();
        assert!(!clock.formatted().is_empty());
        assert!(clock.last_minute() <= 59);
    }

    #[test]
    fn test_clock_service_update_idempotency() {
        let mut clock = ClockService::new();
        let changed = clock.update();
        assert!(!changed);
    }

    #[test]
    fn test_clock_service_hour_and_day_transition_with_identical_minute() {
        let mut clock = ClockService::new();
        let t1 = Local.with_ymd_and_hms(2026, 3, 15, 10, 30, 0).unwrap();
        assert!(clock.update_with(t1));
        assert_eq!(clock.time(), "10:30");
        assert_eq!(clock.date(), "15/03/2026");

        // Same minute one hour later (e.g. system wake)
        let t2 = Local.with_ymd_and_hms(2026, 3, 15, 11, 30, 0).unwrap();
        assert!(clock.update_with(t2));
        assert_eq!(clock.time(), "11:30");

        // Same minute next day
        let t3 = Local.with_ymd_and_hms(2026, 3, 16, 11, 30, 0).unwrap();
        assert!(clock.update_with(t3));
        assert_eq!(clock.time(), "11:30");
        assert_eq!(clock.date(), "16/03/2026");

        // Same timestamp should not trigger update
        assert!(!clock.update_with(t3));
    }
}
