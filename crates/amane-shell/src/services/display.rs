use std::time::{Duration, Instant};

use amane::Service;
use niri_ipc::{
    ConfiguredMode, ConfiguredPosition, ModeToSet, OutputAction, PositionToSet, ScaleToSet,
    VrrToSet,
};

use crate::services::niri::NiriService;

#[derive(Debug, Clone, PartialEq)]
pub struct OutputModeInfo {
    pub width: u16,
    pub height: u16,
    pub refresh_mhz: u32,
    pub refresh_hz: f64,
    pub is_preferred: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputState {
    pub name: String,
    pub make: String,
    pub model: String,
    pub modes: Vec<OutputModeInfo>,
    pub current_mode_index: Option<usize>,
    pub width: u16,
    pub height: u16,
    pub refresh_hz: f64,
    pub scale: f64,
    pub vrr_supported: bool,
    pub vrr_enabled: bool,
    pub pos_x: i32,
    pub pos_y: i32,
    pub transform: String,
    pub is_connected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SafeModePending {
    pub output_name: String,
    pub previous_mode: (u16, u16, Option<f64>),
    pub previous_scale: f64,
    pub remaining_secs: u8,
    pub expires_at: Instant,
}

pub struct DisplayTopologyService {
    pub outputs: Vec<OutputState>,
    pub selected_output: String,
    pub safe_mode: Option<SafeModePending>,
    pub status_message: Option<String>,
}

impl Service for DisplayTopologyService {
    fn new() -> Self {
        let mut svc = Self {
            outputs: Vec::new(),
            selected_output: String::new(),
            safe_mode: None,
            status_message: None,
        };
        svc.refresh_outputs_internal();
        if let Some(first) = svc.outputs.first() {
            svc.selected_output = first.name.clone();
        }
        svc
    }

    fn interval() -> Duration {
        Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let mut changed = false;

        // 1. Check Safe Mode countdown timer
        if let Some(ref mut safe) = self.safe_mode {
            let now = Instant::now();
            if now >= safe.expires_at {
                // Timer expired -> auto-revert
                let output = safe.output_name.clone();
                let (prev_w, prev_h, prev_ref) = safe.previous_mode;
                let prev_scale = safe.previous_scale;
                self.safe_mode = None;
                self.status_message = Some("Settings reverted automatically".to_string());

                let _ = NiriService::configure_output(
                    output.clone(),
                    OutputAction::Mode {
                        mode: ModeToSet::Specific(ConfiguredMode {
                            width: prev_w,
                            height: prev_h,
                            refresh: prev_ref,
                        }),
                    },
                );
                let _ = NiriService::configure_output(
                    output,
                    OutputAction::Scale {
                        scale: ScaleToSet::Specific(prev_scale),
                    },
                );
                self.refresh_outputs_internal();
                return true;
            } else {
                let remaining = safe.expires_at.duration_since(now).as_secs() as u8;
                safe.remaining_secs = remaining.max(1);
                changed = true;
            }
        }

        // 2. Refresh outputs from Niri IPC
        let before_count = self.outputs.len();
        self.refresh_outputs_internal();
        if self.outputs.len() != before_count {
            changed = true;
        }

        changed
    }
}

impl DisplayTopologyService {
    fn refresh_outputs_internal(&mut self) {
        if let Ok(outputs_map) = NiriService::query_outputs() {
            let mut list = Vec::new();
            for (name, output) in outputs_map {
                let mut modes = Vec::new();
                for m in &output.modes {
                    let hz = m.refresh_rate as f64 / 1000.0;
                    modes.push(OutputModeInfo {
                        width: m.width,
                        height: m.height,
                        refresh_mhz: m.refresh_rate,
                        refresh_hz: hz,
                        is_preferred: m.is_preferred,
                    });
                }

                let (w, h, r_hz, scale, px, py, trans) = if let Some(ref log) = output.logical {
                    let cur_hz = output
                        .current_mode
                        .and_then(|idx| output.modes.get(idx))
                        .map(|m| m.refresh_rate as f64 / 1000.0)
                        .unwrap_or(60.0);
                    (
                        log.width as u16,
                        log.height as u16,
                        cur_hz,
                        log.scale,
                        log.x,
                        log.y,
                        format!("{:?}", log.transform),
                    )
                } else if let Some(idx) = output.current_mode
                    && let Some(m) = output.modes.get(idx)
                {
                    (
                        m.width,
                        m.height,
                        m.refresh_rate as f64 / 1000.0,
                        1.0,
                        0,
                        0,
                        "Normal".to_string(),
                    )
                } else {
                    (1920, 1080, 60.0, 1.0, 0, 0, "Normal".to_string())
                };

                list.push(OutputState {
                    name: name.clone(),
                    make: output.make.clone(),
                    model: output.model.clone(),
                    modes,
                    current_mode_index: output.current_mode,
                    width: w,
                    height: h,
                    refresh_hz: r_hz,
                    scale,
                    vrr_supported: output.vrr_supported,
                    vrr_enabled: output.vrr_enabled,
                    pos_x: px,
                    pos_y: py,
                    transform: trans,
                    is_connected: true,
                });
            }
            list.sort_by(|a, b| a.name.cmp(&b.name));
            self.outputs = list;
            if (self.selected_output.is_empty()
                || !self.outputs.iter().any(|o| o.name == self.selected_output))
                && let Some(first) = self.outputs.first()
            {
                self.selected_output = first.name.clone();
            }
        }
    }

    pub fn select_output(name: &str) {
        let mut svc = DisplayTopologyService::write();
        svc.selected_output = name.to_string();
        amane::request_frame();
    }

    pub fn apply_mode(output: &str, width: u16, height: u16, refresh: Option<f64>) {
        let mut svc = DisplayTopologyService::write();
        let cur_state = svc.outputs.iter().find(|o| o.name == output).cloned();

        let previous_mode = if let Some(ref st) = cur_state {
            (st.width, st.height, Some(st.refresh_hz))
        } else {
            (1920, 1080, Some(60.0))
        };
        let previous_scale = cur_state.as_ref().map(|s| s.scale).unwrap_or(1.0);

        let action = OutputAction::Mode {
            mode: ModeToSet::Specific(ConfiguredMode {
                width,
                height,
                refresh,
            }),
        };

        if NiriService::configure_output(output.to_string(), action).is_ok() {
            svc.safe_mode = Some(SafeModePending {
                output_name: output.to_string(),
                previous_mode,
                previous_scale,
                remaining_secs: 12,
                expires_at: Instant::now() + Duration::from_secs(12),
            });
            svc.status_message = Some(format!("Applied {width}x{height} - confirm within 12s"));
            svc.refresh_outputs_internal();
        } else {
            svc.status_message = Some("Failed to apply display mode via Niri IPC".to_string());
        }
        amane::request_frame();
    }

    pub fn apply_scale(output: &str, scale: f64) {
        let mut svc = DisplayTopologyService::write();
        let action = OutputAction::Scale {
            scale: ScaleToSet::Specific(scale),
        };
        if NiriService::configure_output(output.to_string(), action).is_ok() {
            svc.status_message = Some(format!("Scale set to {scale:.2}x"));
            svc.refresh_outputs_internal();
        } else {
            svc.status_message = Some("Failed to apply display scale".to_string());
        }
        amane::request_frame();
    }

    pub fn apply_vrr(output: &str, enabled: bool) {
        let mut svc = DisplayTopologyService::write();
        let action = OutputAction::Vrr {
            vrr: VrrToSet {
                vrr: enabled,
                on_demand: false,
            },
        };
        if NiriService::configure_output(output.to_string(), action).is_ok() {
            svc.status_message = Some(format!(
                "VRR {}",
                if enabled { "enabled" } else { "disabled" }
            ));
            svc.refresh_outputs_internal();
        } else {
            svc.status_message = Some("Failed to toggle VRR".to_string());
        }
        amane::request_frame();
    }

    pub fn apply_position(output: &str, x: i32, y: i32) {
        let mut svc = DisplayTopologyService::write();
        let action = OutputAction::Position {
            position: PositionToSet::Specific(ConfiguredPosition { x, y }),
        };
        if NiriService::configure_output(output.to_string(), action).is_ok() {
            svc.status_message = Some(format!("Position set to ({x}, {y})"));
            svc.refresh_outputs_internal();
        } else {
            svc.status_message = Some("Failed to apply output position".to_string());
        }
        amane::request_frame();
    }

    pub fn confirm_safe_mode() {
        let mut svc = DisplayTopologyService::write();
        svc.safe_mode = None;
        svc.status_message = Some("Display configuration confirmed".to_string());
        amane::request_frame();
    }

    pub fn revert_safe_mode() {
        let mut svc = DisplayTopologyService::write();
        if let Some(safe) = svc.safe_mode.take() {
            let output = safe.output_name;
            let (prev_w, prev_h, prev_ref) = safe.previous_mode;
            let prev_scale = safe.previous_scale;

            let _ = NiriService::configure_output(
                output.clone(),
                OutputAction::Mode {
                    mode: ModeToSet::Specific(ConfiguredMode {
                        width: prev_w,
                        height: prev_h,
                        refresh: prev_ref,
                    }),
                },
            );
            let _ = NiriService::configure_output(
                output,
                OutputAction::Scale {
                    scale: ScaleToSet::Specific(prev_scale),
                },
            );
            svc.status_message = Some("Reverted to previous display mode".to_string());
            svc.refresh_outputs_internal();
        }
        amane::request_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_mode_info_formatting() {
        let mode = OutputModeInfo {
            width: 1920,
            height: 1080,
            refresh_mhz: 144003,
            refresh_hz: 144.003,
            is_preferred: true,
        };
        assert_eq!(mode.width, 1920);
        assert_eq!(mode.height, 1080);
        assert!((mode.refresh_hz - 144.003).abs() < 0.001);
        assert!(mode.is_preferred);
    }

    #[test]
    fn test_safe_mode_pending_structure() {
        let now = Instant::now();
        let safe = SafeModePending {
            output_name: "eDP-1".to_string(),
            previous_mode: (1920, 1080, Some(60.0)),
            previous_scale: 1.0,
            remaining_secs: 12,
            expires_at: now + Duration::from_secs(12),
        };
        assert_eq!(safe.output_name, "eDP-1");
        assert_eq!(safe.remaining_secs, 12);
        assert_eq!(safe.previous_mode.0, 1920);
    }
}
