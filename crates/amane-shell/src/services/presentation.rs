//! Zero-Disruption Presentation Shield Service
//!
//! Manages Presentation Mode state (active toggle, start timestamp for elapsed HUD).
//! Readers query state via lock-free `ArcSwap` without blocking UI or rendering threads.
//! When active, an ultra-low overhead 1-second background ticker requests frame redraws
//! to advance the stopwatch HUD capsule. When inactive, frame requests return to 0 Hz.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PresentationState {
    pub active: bool,
    pub start_time: Option<Instant>,
}

pub struct PresentationService {
    state: ArcSwap<PresentationState>,
}

static INSTANCE: LazyLock<PresentationService> = LazyLock::new(PresentationService::new);
static TICKER_PAIR: LazyLock<(std::sync::Mutex<bool>, std::sync::Condvar)> =
    LazyLock::new(|| (std::sync::Mutex::new(false), std::sync::Condvar::new()));
static TICKER_SPAWNED: AtomicBool = AtomicBool::new(false);

fn ensure_ticker() {
    if TICKER_SPAWNED.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::Builder::new()
        .name("amane-presentation-ticker".into())
        .spawn(|| {
            let (lock, cvar) = &*TICKER_PAIR;
            let mut active_guard = lock.lock().unwrap();
            loop {
                while !*active_guard {
                    active_guard = cvar.wait(active_guard).unwrap();
                }
                let (guard, _timeout_res) = cvar
                    .wait_timeout(active_guard, Duration::from_secs(1))
                    .unwrap();
                active_guard = guard;
                if *active_guard {
                    amane::request_frame();
                }
            }
        })
        .ok();
}

impl PresentationService {
    pub fn new() -> Self {
        Self {
            state: ArcSwap::from_pointee(PresentationState::default()),
        }
    }

    pub fn global() -> &'static PresentationService {
        &INSTANCE
    }

    pub fn is_active(&self) -> bool {
        self.state.load().active
    }

    pub fn elapsed(&self) -> Option<Duration> {
        let state = self.state.load();
        if state.active {
            state.start_time.map(|start| start.elapsed())
        } else {
            None
        }
    }

    pub fn elapsed_formatted(&self) -> Option<String> {
        self.elapsed().map(|dur| {
            let total_secs = dur.as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}", minutes, seconds)
        })
    }

    /// Toggle presentation mode on/off. Returns the new active state.
    pub fn toggle(&self) -> bool {
        let current = self.state.load();
        let new_active = !current.active;
        self.set_active(new_active);
        new_active
    }

    pub fn set_active(&self, active: bool) {
        let new_state = PresentationState {
            active,
            start_time: if active { Some(Instant::now()) } else { None },
        };
        self.state.store(Arc::new(new_state));
        ensure_ticker();
        {
            let (lock, cvar) = &*TICKER_PAIR;
            let mut guard = lock.lock().unwrap();
            *guard = active;
            cvar.notify_one();
        }
        amane::request_frame();
    }
}

impl Default for PresentationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presentation_toggle_and_state() {
        let service = PresentationService::new();
        assert!(!service.is_active());
        assert!(service.elapsed().is_none());
        assert!(service.elapsed_formatted().is_none());

        let active = service.toggle();
        assert!(active);
        assert!(service.is_active());
        assert!(service.elapsed().is_some());
        assert!(service.elapsed_formatted().is_some());

        let formatted = service.elapsed_formatted().unwrap();
        assert!(formatted.starts_with("00:"));

        let active = service.toggle();
        assert!(!active);
        assert!(!service.is_active());
        assert!(service.elapsed().is_none());
    }

    #[test]
    fn test_set_active_explicit() {
        let service = PresentationService::new();
        service.set_active(true);
        assert!(service.is_active());
        assert!(service.elapsed().is_some());

        service.set_active(false);
        assert!(!service.is_active());
        assert!(service.elapsed().is_none());
    }

    #[test]
    fn test_elapsed_formatting_calculation() {
        let dur = Duration::from_secs(125);
        let minutes = dur.as_secs() / 60;
        let seconds = dur.as_secs() % 60;
        assert_eq!(format!("{:02}:{:02}", minutes, seconds), "02:05");
    }
}
