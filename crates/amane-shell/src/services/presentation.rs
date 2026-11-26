//! Zero-Disruption Presentation Shield Service
//!
//! Manages Presentation Mode state (active toggle, start timestamp for elapsed HUD).
//! Readers query state via lock-free `ArcSwap` without blocking UI or rendering threads.
//! When active, an ultra-low overhead 1-second background ticker requests frame redraws
//! to advance the stopwatch HUD capsule. When inactive, frame requests return to 0 Hz.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use amane::Service;
use arc_swap::ArcSwap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationState {
    pub active: bool,
    pub start_time: Option<Instant>,
    pub target_duration_secs: Option<u64>,
    pub completed_sprint: bool,
    pub last_drained_count: usize,
    pub last_error: Option<String>,
}

impl Default for PresentationState {
    fn default() -> Self {
        Self {
            active: false,
            start_time: None,
            target_duration_secs: Some(1500), // Default 25 min sprint preset
            completed_sprint: false,
            last_drained_count: 0,
            last_error: None,
        }
    }
}

pub struct PresentationSignal;

impl amane::Service for PresentationSignal {
    fn new() -> Self {
        Self
    }

    fn listen() {}
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
                    let instance = PresentationService::global();
                    let st = instance.state.load();
                    let mut should_complete_sprint = false;
                    let mut start_instant = None;

                    if st.active
                        && !st.completed_sprint
                        && let (Some(start), Some(target_secs)) =
                            (st.start_time, st.target_duration_secs)
                    {
                        start_instant = Some(start);
                        if start.elapsed().as_secs() >= target_secs {
                            should_complete_sprint = true;
                        }
                    }

                    if should_complete_sprint {
                        if let Some(start) = start_instant {
                            // Drop active_guard before calling complete_sprint to avoid deadlock with TICKER_PAIR lock
                            drop(active_guard);
                            instance.complete_sprint(start);
                            active_guard = lock.lock().unwrap();
                        }
                    } else {
                        let _ = PresentationSignal::write();
                    }
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

    pub fn state(&self) -> Arc<PresentationState> {
        self.state.load_full()
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

    pub fn remaining_duration(&self) -> Option<Duration> {
        let state = self.state.load();
        if state.active {
            if let (Some(start), Some(target_secs)) = (state.start_time, state.target_duration_secs)
            {
                let elapsed = start.elapsed().as_secs();
                if elapsed < target_secs {
                    Some(Duration::from_secs(target_secs - elapsed))
                } else {
                    Some(Duration::ZERO)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn remaining_formatted(&self) -> Option<String> {
        self.remaining_duration().map(|dur| {
            let total_secs = dur.as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}", minutes, seconds)
        })
    }

    pub fn progress_ratio(&self) -> f32 {
        let state = self.state.load();
        if !state.active {
            return 0.0;
        }
        match (state.start_time, state.target_duration_secs) {
            (Some(start), Some(target_secs)) if target_secs > 0 => {
                let elapsed = start.elapsed().as_secs_f32();
                (elapsed / target_secs as f32).clamp(0.0, 1.0)
            }
            _ => 0.0,
        }
    }

    pub fn target_duration_secs(&self) -> Option<u64> {
        self.state.load().target_duration_secs
    }

    pub fn set_target_duration(&self, secs: Option<u64>) {
        let current = self.state.load();
        let new_state = PresentationState {
            target_duration_secs: secs,
            ..(**current).clone()
        };
        self.state.store(Arc::new(new_state));
        let _ = PresentationSignal::write();
    }

    pub fn complete_sprint(&self, expected_start: Instant) -> Option<usize> {
        let current = self.state.load();
        if !current.active || current.start_time != Some(expected_start) {
            return None;
        }

        let drained = self.set_active(false);
        {
            let current_post = self.state.load();
            let mut post_state = (**current_post).clone();
            post_state.completed_sprint = true;
            post_state.last_drained_count = drained;
            self.state.store(Arc::new(post_state));
        }

        amane::Notifications::notify(
            "FocusShield",
            "Sprint complete!",
            "Take a break. Notifications unsuppressed.",
            amane::Urgency::Normal,
        );
        let _ = PresentationSignal::write();

        Some(drained)
    }

    /// Toggle presentation mode on/off. Returns the new active state.
    pub fn toggle(&self) -> bool {
        let (active, _) = self.toggle_with_drained();
        active
    }

    /// Toggle presentation mode on/off and return (is_now_active, drained_notification_count).
    pub fn toggle_with_drained(&self) -> (bool, usize) {
        let current = self.state.load();
        let new_active = !current.active;
        let drained = self.set_active(new_active);
        (new_active, drained)
    }

    pub fn set_active(&self, active: bool) -> usize {
        let current = self.state.load();
        let new_state = PresentationState {
            active,
            start_time: if active { Some(Instant::now()) } else { None },
            completed_sprint: false,
            last_drained_count: 0,
            last_error: None,
            target_duration_secs: current.target_duration_secs,
        };
        self.state.store(Arc::new(new_state));
        ensure_ticker();
        {
            let (lock, cvar) = &*TICKER_PAIR;
            let mut guard = lock.lock().unwrap();
            *guard = active;
            cvar.notify_one();
        }

        let drained_count = if active {
            let _ =
                crate::services::niri::NiriService::quarantine_external_outputs_for_presentation();
            0
        } else {
            crate::ui::notifications::drain_presentation_buffer().len()
        };

        let _ = PresentationSignal::write();
        drained_count
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
        let _guard = crate::ui::notifications::tests::NOTIF_TEST_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
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
        let _guard = crate::ui::notifications::tests::NOTIF_TEST_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
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

    #[test]
    fn test_presentation_drain_buffer_integration() {
        let _guard = crate::ui::notifications::tests::NOTIF_TEST_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let _ = crate::ui::notifications::drain_presentation_buffer();
        let service = PresentationService::new();
        // Buffer a test notification
        crate::ui::notifications::buffer_notification(
            crate::ui::notifications::BufferedNotification {
                id: 9999,
                app_name: "test".to_string(),
                summary: "test alert".to_string(),
                body: "presentation shield".to_string(),
                urgency: amane::Urgency::Normal,
                received: std::time::SystemTime::now(),
            },
        );
        assert!(crate::ui::notifications::presentation_buffer_len() > 0);

        // Enter presentation mode
        let (active, drained) = service.toggle_with_drained();
        assert!(active);
        assert_eq!(drained, 0);

        // Exit presentation mode — buffer must be drained
        let (active, drained) = service.toggle_with_drained();
        assert!(!active);
        assert!(drained >= 1);
        assert_eq!(crate::ui::notifications::presentation_buffer_len(), 0);
    }
}
