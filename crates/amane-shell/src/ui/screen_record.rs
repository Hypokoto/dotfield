use std::process::Command;
use std::time::{Duration, Instant};

use amane::{
    Align, Center, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Zone,
};

use crate::config::ConfigService;
use crate::ui::components;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub struct ScreenRecordService {
    pub open: bool,
    pub is_recording: bool,
    pub with_audio: bool,
    pub started_at: Option<Instant>,
}

fn check_recording_status() -> bool {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let pid_file = format!("{runtime_dir}/dotfield-recording.pid");
    let Ok(content) = std::fs::read_to_string(&pid_file) else {
        return false;
    };
    let Ok(pid) = content.trim().parse::<u32>() else {
        return false;
    };
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

impl Service for ScreenRecordService {
    fn new() -> Self {
        let recording = check_recording_status();
        Self {
            open: false,
            is_recording: recording,
            with_audio: false,
            started_at: if recording {
                Some(Instant::now())
            } else {
                None
            },
        }
    }

    fn interval() -> Duration {
        // Poll every second to update timer and detect external start/stop
        Duration::from_secs(1)
    }

    fn update(&mut self) -> bool {
        let active = check_recording_status();
        let mut changed = false;
        if active != self.is_recording {
            self.is_recording = active;
            if active {
                self.started_at = Some(Instant::now());
            } else {
                self.started_at = None;
            }
            changed = true;
        } else if self.is_recording {
            changed = true;
        }
        changed
    }
}

pub fn toggle() {
    let is_open = {
        let mut rec = ScreenRecordService::write();
        rec.open = !rec.open;
        rec.open
    };
    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::ScreenRecord,
        );
    }
}

pub fn open() {
    let mut rec = ScreenRecordService::write();
    rec.open = true;
    drop(rec);

    crate::ui::modal_manager::ModalManager::open_exclusive(
        crate::ui::modal_manager::ModalType::ScreenRecord,
    );
}

pub fn close() {
    let mut rec = ScreenRecordService::write();
    rec.open = false;
}

pub fn toggle_audio() {
    let mut rec = ScreenRecordService::write();
    rec.with_audio = !rec.with_audio;
}

pub fn start_fullscreen() {
    let with_audio = ScreenRecordService::read().with_audio;
    close();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let script = format!("{home}/.config/dotfield/scripts/screen-record.sh");
    let fallback = "scripts/screen-record.sh";
    let cmd = if std::path::Path::new(&script).exists() {
        &script
    } else {
        fallback
    };

    let mut args = vec!["--fullscreen"];
    if with_audio {
        args.push("--audio");
    }

    let _ = Command::new("bash").arg(cmd).args(&args).spawn();
    let mut rec = ScreenRecordService::write();
    rec.is_recording = true;
    rec.started_at = Some(Instant::now());
}

pub fn start_area() {
    let with_audio = ScreenRecordService::read().with_audio;
    close();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let script = format!("{home}/.config/dotfield/scripts/screen-record.sh");
    let fallback = "scripts/screen-record.sh";
    let cmd = if std::path::Path::new(&script).exists() {
        &script
    } else {
        fallback
    };

    let mut args = vec!["--area"];
    if with_audio {
        args.push("--audio");
    }

    let _ = Command::new("bash").arg(cmd).args(&args).spawn();
    let mut rec = ScreenRecordService::write();
    rec.is_recording = true;
    rec.started_at = Some(Instant::now());
}

pub fn stop_recording() {
    close();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let script = format!("{home}/.config/dotfield/scripts/screen-record.sh");
    let fallback = "scripts/screen-record.sh";
    let cmd = if std::path::Path::new(&script).exists() {
        &script
    } else {
        fallback
    };

    let _ = Command::new("bash").arg(cmd).arg("--stop").spawn();
    let mut rec = ScreenRecordService::write();
    rec.is_recording = false;
    rec.started_at = None;
}

/// Renders the fullscreen centered Screen Recorder overlay dialog.
pub fn screen_record_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let rec = ScreenRecordService::read();

    let is_visible = rec.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-screen-record");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = manifest.theme.font_family.clone();

    let is_active = rec.is_recording;

    let content_box = if is_active {
        // Active recording in progress view
        let elapsed_secs = rec.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        let mins = elapsed_secs / 60;
        let secs = elapsed_secs % 60;
        let time_formatted = format!("{mins:02}:{secs:02}");

        let header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(components::status_badge(
                        "REC: LIVE",
                        tokens.danger,
                        &tokens,
                    )),
                    Box::new(
                        Text::new("Recording in Progress")
                            .size(typography::BODY)
                            .color(tokens.text_primary)
                            .font(&font),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            ),
            Box::new(
                Text::new(time_formatted)
                    .size(typography::TITLE_SM)
                    .color(tokens.accent)
                    .font(&font),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let telemetry_line = Row::new(vec![
            Box::new(
                Text::new("1920x1080 @ 144Hz")
                    .size(typography::CAPTION)
                    .color(tokens.text_secondary)
                    .font(&font),
            ),
            Box::new(
                Text::new("•")
                    .size(typography::CAPTION)
                    .color(tokens.text_muted)
                    .font(&font),
            ),
            Box::new(
                Text::new("H.264 VAAPI")
                    .size(typography::CAPTION)
                    .color(tokens.text_secondary)
                    .font(&font),
            ),
            Box::new(
                Text::new("•")
                    .size(typography::CAPTION)
                    .color(tokens.text_muted)
                    .font(&font),
            ),
            Box::new(
                Text::new("~/Videos/Recordings/")
                    .size(typography::CAPTION)
                    .color(tokens.text_muted)
                    .font(&font),
            ),
        ])
        .gap(6.0)
        .align(Align::Center);

        let stop_btn = Rectangle::new()
            .width(Parent)
            .height(44.0)
            .radius(radius::SM)
            .fill(tokens.danger)
            .border(1.0, tokens.border_danger)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                stop_recording();
            })
            .child(
                Row::new(vec![
                    Box::new(Text::new("󰙦").size(16.0).color(Color::WHITE).font(&font)),
                    Box::new(
                        Text::new("Stop Recording")
                            .size(typography::BODY)
                            .color(Color::WHITE)
                            .font(&font),
                    ),
                    Box::new(components::keycap("Space", &tokens)),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            );

        Column::new(vec![
            Box::new(header),
            Box::new(telemetry_line),
            Box::new(stop_btn),
        ])
        .width(Parent)
        .gap(spacing::GAP_MD)
    } else {
        // Idle view: Select Region vs Fullscreen with audio toggle
        let header =
            components::modal_header("⏺", "Screen Recorder", Some("wf-recorder"), &tokens, close);

        // Action Cards: Region vs Fullscreen
        let region_card = Rectangle::new()
            .width(236.0)
            .height(84.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                start_area();
            })
            .child(
                Column::new(vec![
                    Box::new(
                        Row::new(vec![
                            Box::new(Text::new("⛶").size(20.0).color(tokens.accent).font(&font)),
                            Box::new(
                                Text::new("Region")
                                    .size(typography::BODY)
                                    .color(tokens.text_primary)
                                    .font(&font),
                            ),
                            Box::new(components::keycap("R", &tokens)),
                        ])
                        .gap(spacing::GAP_SM)
                        .align(Align::Center),
                    ),
                    Box::new(
                        Text::new("Select area with slurp")
                            .size(typography::CAPTION)
                            .color(tokens.text_muted)
                            .font(&font),
                    ),
                ])
                .gap(4.0)
                .align(Align::Center),
            );

        let full_card = Rectangle::new()
            .width(236.0)
            .height(84.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                start_fullscreen();
            })
            .child(
                Column::new(vec![
                    Box::new(
                        Row::new(vec![
                            Box::new(Text::new("󰍹").size(20.0).color(tokens.accent).font(&font)),
                            Box::new(
                                Text::new("Fullscreen")
                                    .size(typography::BODY)
                                    .color(tokens.text_primary)
                                    .font(&font),
                            ),
                            Box::new(components::keycap("F", &tokens)),
                        ])
                        .gap(spacing::GAP_SM)
                        .align(Align::Center),
                    ),
                    Box::new(
                        Text::new("Record entire display")
                            .size(typography::CAPTION)
                            .color(tokens.text_muted)
                            .font(&font),
                    ),
                ])
                .gap(4.0)
                .align(Align::Center),
            );

        let mode_row = Row::new(vec![Box::new(region_card), Box::new(full_card)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        // Audio toggle & hints
        let (audio_bg, audio_border, audio_fg) = if rec.with_audio {
            (tokens.surface_selected, tokens.border_focus, tokens.success)
        } else {
            (
                tokens.surface_elevated,
                tokens.border_subtle,
                tokens.text_muted,
            )
        };

        let audio_btn = Rectangle::new()
            .width(180.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(audio_bg)
            .border(1.0, audio_border)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                toggle_audio();
            })
            .child(
                Row::new(vec![
                    Box::new(
                        Text::new(if rec.with_audio {
                            "󰍬 Mic Audio: On"
                        } else {
                            "󰍭 Mic Audio: Off"
                        })
                        .size(typography::SECONDARY)
                        .color(audio_fg)
                        .font(&font),
                    ),
                    Box::new(components::keycap("A", &tokens)),
                ])
                .gap(6.0)
                .align(Align::Center),
            );

        let telemetry_info = Row::new(vec![
            Box::new(
                Text::new("1080p @ 144Hz")
                    .size(typography::CAPTION)
                    .color(tokens.text_secondary)
                    .font(&font),
            ),
            Box::new(
                Text::new("•")
                    .size(typography::CAPTION)
                    .color(tokens.text_muted)
                    .font(&font),
            ),
            Box::new(
                Text::new("H.264 VAAPI")
                    .size(typography::CAPTION)
                    .color(tokens.text_muted)
                    .font(&font),
            ),
        ])
        .gap(6.0)
        .align(Align::Center);

        let controls_row = Row::new(vec![Box::new(audio_btn), Box::new(telemetry_info)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        Column::new(vec![
            Box::new(header),
            Box::new(mode_row),
            Box::new(controls_row),
        ])
        .width(Parent)
        .gap(spacing::GAP_MD)
    };

    let panel = Rectangle::new()
        .width(520.0)
        .height(218.0)
        .radius(radius::LG)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(content_box);

    LayerWindow::new()
        .width(520.0)
        .height(218.0)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(if is_visible {
            Keyboard::OnDemand
        } else {
            Keyboard::None
        })
        .visible(is_visible)
        .namespace("dotfield-shell-screen-record")
        .on_key(|key| match key {
            Key::Escape | Key::Character('q') | Key::Character('Q') => {
                close();
            }
            Key::Character('r') | Key::Character('R') => {
                start_area();
            }
            Key::Character('f') | Key::Character('F') => {
                start_fullscreen();
            }
            Key::Character('a') | Key::Character('A') => {
                toggle_audio();
            }
            Key::Space | Key::Enter => {
                let is_rec = ScreenRecordService::read().is_recording;
                if is_rec {
                    stop_recording();
                } else {
                    start_area();
                }
            }
            _ => {}
        })
        .child(panel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_record_service_default() {
        let rec = ScreenRecordService::new();
        assert!(!rec.open);
        assert!(!rec.with_audio);
    }

    #[test]
    fn test_screen_record_toggle_and_close() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        ScreenRecordService::write().open = false;
        toggle();
        assert!(ScreenRecordService::read().open);

        toggle_audio();
        assert!(ScreenRecordService::read().with_audio);

        close();
        assert!(!ScreenRecordService::read().open);
    }
}
