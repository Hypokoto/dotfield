use std::time::Instant;

use amane::{Rectangle, request_frame};

/// Suzuha-inspired motion tokens: critically damped spatial motion (zero overshoot).
pub const MOTION_SPATIAL_CURVE: [f32; 4] = [0.2, 0.0, 0.0, 1.0];
pub const MOTION_EFFECTS_CURVE: [f32; 4] = [0.34, 0.8, 0.34, 1.0];

pub const PANEL_STIFFNESS: f32 = 500.0;
pub const PANEL_OPEN_DAMPING: f32 = 44.72; // 2 * sqrt(500) -> critically damped
pub const PANEL_CLOSE_DAMPING: f32 = 50.0; // Slightly overdamped crisp tuck

const SPRING_MAX_STEP: f32 = 1.0 / 30.0;
const SPRING_SETTLED_POS: f32 = 0.002;
const SPRING_SETTLED_VEL: f32 = 0.005;

/// Evaluates a 1D normalized cubic-bezier easing curve `[x1, y1, x2, y2]` at `t` in `[0.0, 1.0]`.
pub fn cubic_bezier(curve: [f32; 4], t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 || t >= 1.0 {
        return t;
    }

    let [x1, y1, x2, y2] = curve;
    let bezier = |p1: f32, p2: f32, u: f32| -> f32 {
        let inv = 1.0 - u;
        3.0 * inv * inv * u * p1 + 3.0 * inv * u * u * p2 + u * u * u
    };

    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    for _ in 0..24 {
        let mid = (lo + hi) * 0.5;
        if bezier(x1, x2, mid) < t {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    bezier(y1, y2, (lo + hi) * 0.5).clamp(0.0, 1.0)
}

/// Closed-form damped harmonic oscillator preserving position and velocity across mid-flight redirects.
#[derive(Debug, Clone, Copy)]
pub struct PanelSpring {
    value: f32,
    velocity: f32,
    target: f32,
    stiffness: f32,
    damping: f32,
    last_tick: Option<Instant>,
}

impl PanelSpring {
    pub fn new(initial: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            value: initial,
            velocity: 0.0,
            target: initial,
            stiffness,
            damping,
            last_tick: None,
        }
    }

    /// Redirects the spring toward `target` with `damping`, keeping current displacement & velocity.
    pub fn set_target(&mut self, target: f32, damping: f32) {
        if (self.target - target).abs() > 1e-4 {
            self.target = target;
            self.damping = damping;
            if self.last_tick.is_none() {
                self.last_tick = Some(Instant::now());
            }
        }
    }

    #[inline]
    pub fn value(&self) -> f32 {
        self.value
    }

    #[inline]
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    #[inline]
    pub fn is_moving(&self) -> bool {
        (self.target - self.value).abs() > SPRING_SETTLED_POS
            || self.velocity.abs() > SPRING_SETTLED_VEL
    }

    /// Advances the spring using real elapsed time and returns its current value (`[0.0, 1.0]`).
    pub fn advance(&mut self) -> f32 {
        let now = Instant::now();
        let dt = self
            .last_tick
            .map(|prev| now.duration_since(prev).as_secs_f32().min(SPRING_MAX_STEP))
            .unwrap_or(1.0 / 60.0);
        self.last_tick = Some(now);
        self.step(dt)
    }

    /// Deterministic single-step advancement for physics integration and unit tests.
    pub fn step(&mut self, dt: f32) -> f32 {
        if dt <= 0.0 {
            return self.value;
        }
        let dt = dt.min(SPRING_MAX_STEP);
        let disp = self.value - self.target;
        let omega = self.stiffness.max(1.0).sqrt();
        let gamma = self.damping * 0.5;
        let discriminant = gamma * gamma - self.stiffness;

        let (next_disp, next_vel) = if discriminant.abs() < 1e-3 {
            // Critically damped
            let c1 = disp;
            let c2 = self.velocity + omega * disp;
            let decay = (-omega * dt).exp();
            (
                (c1 + c2 * dt) * decay,
                (c2 - omega * (c1 + c2 * dt)) * decay,
            )
        } else if discriminant < 0.0 {
            // Underdamped
            let omega_d = (-discriminant).sqrt();
            let c1 = disp;
            let c2 = (self.velocity + gamma * disp) / omega_d;
            let decay = (-gamma * dt).exp();
            let cos_w = (omega_d * dt).cos();
            let sin_w = (omega_d * dt).sin();
            (
                decay * (c1 * cos_w + c2 * sin_w),
                decay
                    * ((-gamma * c1 + omega_d * c2) * cos_w + (-gamma * c2 - omega_d * c1) * sin_w),
            )
        } else {
            // Overdamped
            let root = discriminant.sqrt();
            let r1 = -gamma + root;
            let r2 = -gamma - root;
            let c1 = (self.velocity - r2 * disp) / (2.0 * root);
            let c2 = disp - c1;
            let e1 = (r1 * dt).exp();
            let e2 = (r2 * dt).exp();
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        };

        self.value = (self.target + next_disp).clamp(0.0, 1.0);
        self.velocity = next_vel;

        if !self.is_moving() {
            self.value = self.target;
            self.velocity = 0.0;
            self.last_tick = None;
        }

        self.value
    }
}

/// Resolved frame state for an animated overlay/panel supporting smooth open AND close.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanelMotion {
    /// Spring spatial displacement in `[0.0, 1.0]` (`0.0` closed, `1.0` open).
    pub progress: f32,
    /// Faster visual opacity curve (`0.0..=1.0`) avoiding muddy semi-transparent fades.
    pub alpha: f32,
    /// True while open or still retracting back to `0.0`.
    pub visible: bool,
    /// True while another compositor frame callback is needed (`0 Hz` when false).
    pub animating: bool,
}

impl PanelMotion {
    /// Applies centered modal spring scale, micro-slide, and opacity.
    pub fn apply_centered_modal(&self, rect: Rectangle) -> Rectangle {
        if !self.animating && self.progress >= 1.0 {
            return rect;
        }
        let scale = 0.95 + 0.05 * self.progress;
        let y_offset = (1.0 - self.progress) * -8.0;
        let opacity = (0.15 + 0.85 * self.alpha).clamp(0.0, 1.0);
        rect.scale(scale).translate(0.0, y_offset).opacity(opacity)
    }
}

/// Evaluates and advances a persistent named panel spring outside `Service` state.
///
/// Parks cleanly at 0 Hz (does not call `request_frame()`) once settled at `0.0` or `1.0`.
pub fn panel_motion(id: &'static str, open: bool) -> PanelMotion {
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static PANEL_SPRINGS: RefCell<HashMap<&'static str, PanelSpring>> =
            RefCell::new(HashMap::new());
    }

    PANEL_SPRINGS.with(|cell| {
        let mut map = cell.borrow_mut();
        let spring = map
            .entry(id)
            .or_insert_with(|| PanelSpring::new(0.0, PANEL_STIFFNESS, PANEL_CLOSE_DAMPING));

        let (target, damping) = if open {
            (1.0, PANEL_OPEN_DAMPING)
        } else {
            (0.0, PANEL_CLOSE_DAMPING)
        };
        spring.set_target(target, damping);

        let animating = spring.is_moving();
        let progress = if animating {
            let v = spring.advance();
            if spring.is_moving() {
                request_frame();
            }
            v
        } else {
            spring.value()
        };

        let alpha = cubic_bezier(MOTION_EFFECTS_CURVE, (progress * 1.15).clamp(0.0, 1.0));
        let still_animating = spring.is_moving();

        PanelMotion {
            progress,
            alpha,
            visible: open || progress > SPRING_SETTLED_POS || still_animating,
            animating: still_animating,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cubic_bezier_spatial_endpoints_and_monotonicity() {
        assert_eq!(cubic_bezier(MOTION_SPATIAL_CURVE, 0.0), 0.0);
        assert_eq!(cubic_bezier(MOTION_SPATIAL_CURVE, 1.0), 1.0);
        let mut prev = 0.0;
        for step in 0..=100 {
            let v = cubic_bezier(MOTION_SPATIAL_CURVE, step as f32 / 100.0);
            assert!(v >= prev && v <= 1.0);
            prev = v;
        }
    }

    #[test]
    fn test_panel_spring_critically_damped_no_bounce_and_settles() {
        let mut s = PanelSpring::new(0.0, PANEL_STIFFNESS, PANEL_OPEN_DAMPING);
        s.set_target(1.0, PANEL_OPEN_DAMPING);
        let mut prev = 0.0;
        for _ in 0..40 {
            let v = s.step(1.0 / 60.0);
            assert!(
                v <= 1.0,
                "critical panel spring must never bounce past 1.0 (got {v})"
            );
            assert!(v >= prev - 1e-4);
            prev = v;
        }
        assert!(!s.is_moving());
        assert_eq!(s.value(), 1.0);
    }

    #[test]
    fn test_panel_spring_mid_flight_reversal_preserves_continuity() {
        let mut s = PanelSpring::new(0.0, PANEL_STIFFNESS, PANEL_OPEN_DAMPING);
        s.set_target(1.0, PANEL_OPEN_DAMPING);
        for _ in 0..4 {
            s.step(1.0 / 60.0);
        }
        let mid = s.value();
        assert!(mid > 0.05 && mid < 0.95);
        s.set_target(0.0, PANEL_CLOSE_DAMPING);
        assert_eq!(s.value(), mid);
        assert!(s.is_moving());
        for _ in 0..40 {
            s.step(1.0 / 60.0);
        }
        assert!(!s.is_moving());
        assert_eq!(s.value(), 0.0);
    }
}
