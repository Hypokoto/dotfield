use std::thread;
use std::time::Duration;

use amane::Service;
use chrono::{Local, Timelike};

pub struct ClockService {
    formatted: String,
    last_minute: u32,
}

impl Service for ClockService {
    fn new() -> Self {
        let now = Local::now();
        Self {
            formatted: now.format("%a %b %d  %H:%M").to_string(),
            last_minute: now.minute(),
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
        let now = Local::now();
        let minute = now.minute();
        if minute != self.last_minute {
            self.last_minute = minute;
            self.formatted = now.format("%a %b %d  %H:%M").to_string();
            true
        } else {
            false
        }
    }
}

impl ClockService {
    pub fn formatted(&self) -> &str {
        &self.formatted
    }

    pub fn last_minute(&self) -> u32 {
        self.last_minute
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_service_initialization() {
        let clock = ClockService::new();
        assert!(!clock.formatted().is_empty());
        assert!(clock.last_minute() <= 59);
    }

    #[test]
    fn test_clock_service_update_idempotency() {
        let mut clock = ClockService::new();
        // Since the minute hasn't changed immediately, update should return false
        let changed = clock.update();
        assert!(!changed);
    }
}
