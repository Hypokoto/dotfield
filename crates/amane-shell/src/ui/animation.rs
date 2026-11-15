use std::time::{Duration, Instant};

use amane::{Rectangle, request_frame};

/// Restrained 150ms modal transition duration adhering to Dotfield 120-180ms discipline.
pub const DEFAULT_MODAL_ANIMATION_DURATION: Duration = Duration::from_millis(150);

/// Represents the instantaneous progress of a modal summon animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalTransition {
    /// Normalized linear progress in [0.0, 1.0].
    pub progress: f32,
    /// Eased progress in [0.0, 1.0] using cubic ease-out.
    pub eased: f32,
}

impl ModalTransition {
    /// Computes the modal transition progress given an optional start timestamp.
    ///
    /// When `started_at` is `None` or elapsed time exceeds `duration`, the animation is
    /// considered at rest ($t=1.0$), and `request_frame()` is NOT called, parking the
    /// Wayland surface at 0 Hz.
    ///
    /// While $t < 1.0$, `request_frame()` is called to request consecutive compositor
    /// frame callbacks until arrival.
    pub fn compute(started_at: Option<Instant>, duration: Duration) -> Self {
        let Some(started) = started_at else {
            return Self {
                progress: 1.0,
                eased: 1.0,
            };
        };

        if duration.is_zero() {
            return Self {
                progress: 1.0,
                eased: 1.0,
            };
        }

        let elapsed = started.elapsed();
        if elapsed >= duration {
            Self {
                progress: 1.0,
                eased: 1.0,
            }
        } else {
            let progress = (elapsed.as_secs_f32() / duration.as_secs_f32()).clamp(0.0, 1.0);
            let eased = cubic_ease_out(progress);
            // Actively tick frame callback while transition is running
            request_frame();
            Self { progress, eased }
        }
    }

    /// Returns `true` if the transition is actively moving.
    #[inline]
    pub fn is_animating(&self) -> bool {
        self.progress < 1.0
    }

    /// Transforms a container rectangle with subtle scale, vertical slide, and opacity easing.
    /// When at rest ($t \ge 1.0$), passes through the rectangle unmodified for zero-overhead rendering.
    pub fn apply(&self, rect: Rectangle) -> Rectangle {
        if self.progress >= 1.0 {
            return rect;
        }

        let scale = 0.96 + 0.04 * self.eased;
        let y_offset = (1.0 - self.eased) * -6.0;
        let opacity = 0.2 + 0.8 * self.eased;

        rect.scale(scale).translate(0.0, y_offset).opacity(opacity)
    }
}

/// Standard cubic ease-out curve: $1 - (1 - t)^3$.
/// Fast initial acceleration with gentle deceleration into rest.
pub fn cubic_ease_out(t: f32) -> f32 {
    let clamped = t.clamp(0.0, 1.0);
    let remaining = 1.0 - clamped;
    1.0 - remaining * remaining * remaining
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_ease_out_boundaries() {
        assert_eq!(cubic_ease_out(0.0), 0.0);
        assert_eq!(cubic_ease_out(1.0), 1.0);
        assert_eq!(cubic_ease_out(-0.5), 0.0);
        assert_eq!(cubic_ease_out(1.5), 1.0);
    }

    #[test]
    fn test_cubic_ease_out_monotonicity() {
        let mut prev = 0.0;
        for step in 0..=100 {
            let t = step as f32 / 100.0;
            let val = cubic_ease_out(t);
            assert!(
                val >= prev,
                "easing must be monotonic: at t={}, val={} < prev={}",
                t,
                val,
                prev
            );
            prev = val;
        }
    }

    #[test]
    fn test_cubic_ease_out_curvature() {
        // At t = 0.5, cubic ease out should have traveled more than 50%
        let mid = cubic_ease_out(0.5);
        assert!(
            mid > 0.5,
            "ease-out should have greater progress in first half"
        );
        assert_eq!(mid, 0.875);
    }

    #[test]
    fn test_modal_transition_none_parks_at_rest() {
        let transition = ModalTransition::compute(None, DEFAULT_MODAL_ANIMATION_DURATION);
        assert_eq!(transition.progress, 1.0);
        assert_eq!(transition.eased, 1.0);
        assert!(!transition.is_animating());
    }

    #[test]
    fn test_modal_transition_elapsed_parks_at_rest() {
        let started = Instant::now() - Duration::from_millis(300);
        let transition = ModalTransition::compute(Some(started), Duration::from_millis(150));
        assert_eq!(transition.progress, 1.0);
        assert_eq!(transition.eased, 1.0);
        assert!(!transition.is_animating());
    }

    #[test]
    fn test_modal_transition_active_ticks() {
        let started = Instant::now();
        let transition = ModalTransition::compute(Some(started), Duration::from_secs(10));
        assert!(transition.progress < 1.0);
        assert!(transition.is_animating());
    }
}
