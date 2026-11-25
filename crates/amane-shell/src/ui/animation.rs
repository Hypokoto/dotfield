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

/// Analytical Damped Harmonic Oscillator representing physical Apple / iOS spring physics.
///
/// Models the standard closed-form spring equation:
/// $$\omega_n = \frac{2\pi}{\text{response}}$$
/// $$\omega_d = \omega_n \sqrt{|1 - \zeta^2|}$$
/// Underdamped ($\zeta < 1$):
/// $$x(t) = 1.0 - e^{-\zeta \omega_n t} \left( \cos(\omega_d t) + \frac{\zeta}{\sqrt{1 - \zeta^2}} \sin(\omega_d t) \right)$$
///
/// Zero CPU overhead: Once settled within $\epsilon = 0.001$, `request_frame()` stops being
/// invoked, parking Wayland surface rendering at 0 Hz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringTransition {
    /// Progress value from 0.0 to ~1.0 (may subtly overshoot 1.0 when underdamped).
    pub value: f32,
    /// Whether the spring has settled within rest threshold.
    pub is_settled: bool,
}

impl SpringTransition {
    /// Instantaneous rest state ($x=1.0$, settled).
    pub const AT_REST: Self = Self {
        value: 1.0,
        is_settled: true,
    };

    /// Apple HIG Snappy spring preset (`response = 0.26s`, `damping_ratio = 0.85`).
    /// Ideal for tactile control switches, squircle buttons, and compact bento toggles.
    pub fn snappy(started_at: Option<Instant>) -> Self {
        Self::compute(started_at, 0.26, 0.85)
    }

    /// Apple HIG Smooth spring preset (`response = 0.35s`, `damping_ratio = 0.82`).
    /// Ideal for modal sheets, control center panels, and bento card expansions.
    pub fn smooth(started_at: Option<Instant>) -> Self {
        Self::compute(started_at, 0.35, 0.82)
    }

    /// Apple HIG Bouncy / Playful spring preset (`response = 0.40s`, `damping_ratio = 0.68`).
    /// Has intentional tasteful overshoot for interactive notifications and dynamic island pills.
    pub fn bouncy(started_at: Option<Instant>) -> Self {
        Self::compute(started_at, 0.40, 0.68)
    }

    /// Computes the closed-form damped harmonic oscillator displacement at elapsed time.
    ///
    /// - `response`: Duration (in seconds) of one undamped cycle ($T_n = 2\pi / \omega_n$).
    /// - `damping_ratio`: Damping factor $\zeta$. 1.0 = critically damped, <1.0 = underdamped.
    pub fn compute(started_at: Option<Instant>, response: f32, damping_ratio: f32) -> Self {
        let Some(started) = started_at else {
            return Self::AT_REST;
        };

        let t = started.elapsed().as_secs_f32();
        if t <= 0.0 {
            request_frame();
            return Self {
                value: 0.0,
                is_settled: false,
            };
        }

        let resp = response.max(0.01);
        let zeta = damping_ratio.max(0.01);
        let omega_n = (2.0 * std::f32::consts::PI) / resp;

        // Settling time heuristic: when envelope e^(-zeta * omega_n * t) < 0.002
        let decay = (-zeta * omega_n * t).exp();
        if decay < 0.002 {
            return Self::AT_REST;
        }

        let value = if (zeta - 1.0).abs() < 1e-4 {
            // Critically damped (zeta = 1)
            1.0 - (1.0 + omega_n * t) * decay
        } else if zeta < 1.0 {
            // Underdamped (oscillatory with decay envelope)
            let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
            let sin_term = (zeta / (1.0 - zeta * zeta).sqrt()) * (omega_d * t).sin();
            let cos_term = (omega_d * t).cos();
            1.0 - decay * (cos_term + sin_term)
        } else {
            // Overdamped (zeta > 1)
            let alpha = omega_n * (zeta * zeta - 1.0).sqrt();
            let c1 = 0.5 * (1.0 + (zeta * omega_n) / alpha);
            let c2 = 0.5 * (-1.0 + (zeta * omega_n) / alpha);
            1.0 - (c1 * ((-zeta * omega_n + alpha) * t).exp()
                + c2 * ((-zeta * omega_n - alpha) * t).exp())
        };

        // If the deviation from 1.0 is negligible and envelope is small, park
        if (value - 1.0).abs() < 0.001 && decay < 0.01 {
            Self::AT_REST
        } else {
            // Actively tick frame callback until settled
            request_frame();
            Self {
                value,
                is_settled: false,
            }
        }
    }

    /// Returns `true` if the spring is still active.
    #[inline]
    pub fn is_animating(&self) -> bool {
        !self.is_settled
    }

    /// Transforms a modal container rectangle with iOS-grade spring scale, subtle vertical glide, and opacity.
    pub fn apply_modal(&self, rect: Rectangle) -> Rectangle {
        if self.is_settled {
            return rect;
        }

        let scale = 0.94 + 0.06 * self.value;
        let y_offset = (1.0 - self.value) * -8.0;
        let opacity = (0.2 + 0.8 * self.value).clamp(0.0, 1.0);

        rect.scale(scale).translate(0.0, y_offset).opacity(opacity)
    }

    /// Transforms an interactive control with tactile spring press depression (scale down to 0.96).
    pub fn apply_press(&self, rect: Rectangle, is_pressed: bool) -> Rectangle {
        let target_scale = if is_pressed { 0.95 } else { 1.0 };
        let current_scale = 1.0 - (1.0 - target_scale) * self.value;
        rect.scale(current_scale)
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

    #[test]
    fn test_spring_transition_none_parks_at_rest() {
        let spring = SpringTransition::smooth(None);
        assert_eq!(spring.value, 1.0);
        assert!(spring.is_settled);
        assert!(!spring.is_animating());
    }

    #[test]
    fn test_spring_transition_presets() {
        let snappy = SpringTransition::snappy(None);
        assert_eq!(snappy, SpringTransition::AT_REST);
        let smooth = SpringTransition::smooth(None);
        assert_eq!(smooth, SpringTransition::AT_REST);
        let bouncy = SpringTransition::bouncy(None);
        assert_eq!(bouncy, SpringTransition::AT_REST);
    }

    #[test]
    fn test_spring_transition_elapsed_parks() {
        let started = Instant::now() - Duration::from_secs(2);
        let spring = SpringTransition::smooth(Some(started));
        assert_eq!(spring.value, 1.0);
        assert!(spring.is_settled);
        assert!(!spring.is_animating());
    }

    #[test]
    fn test_spring_transition_active_ticks() {
        let started = Instant::now();
        let spring = SpringTransition::smooth(Some(started));
        assert!(spring.is_animating());
    }

    #[test]
    fn test_spring_apply_modal_at_rest_is_noop() {
        let spring = SpringTransition::AT_REST;
        let rect = Rectangle::new().width(100.0).height(100.0);
        let _applied = spring.apply_modal(rect);
    }
}
