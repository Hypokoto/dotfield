use std::process::Command;
use std::time::{Duration, Instant};

use amane::{
    Align, Center, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Zone,
};

use crate::config::ConfigService;

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
    let mut rec = ScreenRecordService::write();
    rec.open = !rec.open;
}

pub fn open() {
    let mut rec = ScreenRecordService::write();
    rec.open = true;
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
            .visible(false)
            .namespace("dotfield-shell-screen-record");
    }

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let crimson_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let steel_color = Color::from(manifest.palette.steel.as_str());
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
                    Box::new(Text::new("󰑋").size(16.0).color(crimson_color).font(&font)),
                    Box::new(
                        Text::new("RECORDING IN PROGRESS")
                            .size(11.0)
                            .color(crimson_color)
                            .font(&font),
                    ),
                ])
                .gap(6.0)
                .align(Align::Center),
            ),
            Box::new(
                Text::new(time_formatted)
                    .size(16.0)
                    .color(amber_color)
                    .font(&font),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let info_line = Text::new("Saving MP4 stream to ~/Videos/Recordings/")
            .size(10.0)
            .color(steel_color)
            .font(&font);

        let stop_btn = Rectangle::new()
            .width(Parent)
            .height(44.0)
            .radius(8.0)
            .fill(crimson_color)
            .border(1.0, Color::from("#FF6B76"))
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                stop_recording();
            })
            .child(
                Row::new(vec![
                    Box::new(Text::new("󰙦").size(16.0).color(Color::WHITE).font(&font)),
                    Box::new(
                        Text::new("STOP & SAVE RECORDING")
                            .size(12.0)
                            .color(Color::WHITE)
                            .font(&font),
                    ),
                ])
                .gap(8.0)
                .align(Align::Center),
            );

        Column::new(vec![
            Box::new(header),
            Box::new(info_line),
            Box::new(stop_btn),
        ])
        .width(Parent)
        .gap(14.0)
    } else {
        // Idle view: Select Region vs Fullscreen with audio toggle
        let header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(Text::new("󰑋").size(15.0).color(amber_color).font(&font)),
                    Box::new(
                        Text::new("WAYLAND SCREEN RECORDER")
                            .size(11.0)
                            .color(fg_color)
                            .font(&font),
                    ),
                    Box::new(
                        Text::new("• wf-recorder")
                            .size(10.0)
                            .color(steel_color)
                            .font(&font),
                    ),
                ])
                .gap(6.0)
                .align(Align::Center),
            ),
            Box::new(
                Rectangle::new()
                    .width(26.0)
                    .height(26.0)
                    .radius(4.0)
                    .fill(overlay_color)
                    .border(1.0, border_color)
                    .cursor(Pointer)
                    .align_child(Center, Center)
                    .on_click(|_| {
                        close();
                    })
                    .child(Text::new("󰅖").size(12.0).color(steel_color).font(&font)),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        // Action Cards: Region vs Fullscreen
        let region_card = Rectangle::new()
            .width(196.0)
            .height(72.0)
            .radius(8.0)
            .fill(overlay_color)
            .border(1.0, border_color)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                start_area();
            })
            .child(
                Column::new(vec![
                    Box::new(Text::new("󰒉").size(22.0).color(amber_color).font(&font)),
                    Box::new(
                        Text::new("Record Region")
                            .size(11.0)
                            .color(fg_color)
                            .font(&font),
                    ),
                    Box::new(
                        Text::new("Select area with slurp")
                            .size(9.0)
                            .color(steel_color)
                            .font(&font),
                    ),
                ])
                .gap(2.0)
                .align(Align::Center),
            );

        let full_card = Rectangle::new()
            .width(196.0)
            .height(72.0)
            .radius(8.0)
            .fill(overlay_color)
            .border(1.0, border_color)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                start_fullscreen();
            })
            .child(
                Column::new(vec![
                    Box::new(Text::new("󰍹").size(22.0).color(amber_color).font(&font)),
                    Box::new(
                        Text::new("Fullscreen")
                            .size(11.0)
                            .color(fg_color)
                            .font(&font),
                    ),
                    Box::new(
                        Text::new("Record entire monitor")
                            .size(9.0)
                            .color(steel_color)
                            .font(&font),
                    ),
                ])
                .gap(2.0)
                .align(Align::Center),
            );

        let mode_row = Row::new(vec![Box::new(region_card), Box::new(full_card)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        // Audio toggle & hints
        let audio_btn = Rectangle::new()
            .width(136.0)
            .height(28.0)
            .radius(6.0)
            .fill(if rec.with_audio {
                amber_color
            } else {
                overlay_color
            })
            .border(
                1.0,
                if rec.with_audio {
                    Color::from("#FFD166")
                } else {
                    border_color
                },
            )
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(|_| {
                toggle_audio();
            })
            .child(
                Text::new(if rec.with_audio {
                    "󰍬 Mic Audio: ON"
                } else {
                    "󰍭 Mic Audio: OFF"
                })
                .size(10.0)
                .color(if rec.with_audio {
                    Color::BLACK
                } else {
                    steel_color
                })
                .font(&font),
            );

        let path_hint = Text::new("Saves to ~/Videos/Recordings")
            .size(9.5)
            .color(steel_color)
            .font(&font);

        let controls_row = Row::new(vec![Box::new(audio_btn), Box::new(path_hint)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        Column::new(vec![
            Box::new(header),
            Box::new(mode_row),
            Box::new(controls_row),
        ])
        .width(Parent)
        .gap(10.0)
    };

    let panel = Rectangle::new()
        .width(440.0)
        .height(180.0)
        .radius(12.0)
        .fill(bg_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 14.0,
            bottom: 12.0,
            left: 16.0,
            right: 16.0,
        })
        .child(content_box);

    LayerWindow::new()
        .width(440.0)
        .height(180.0)
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
        ScreenRecordService::write().open = false;
        toggle();
        assert!(ScreenRecordService::read().open);

        toggle_audio();
        assert!(ScreenRecordService::read().with_audio);

        close();
        assert!(!ScreenRecordService::read().open);
    }
}
