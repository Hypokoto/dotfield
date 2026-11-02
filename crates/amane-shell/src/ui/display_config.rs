use std::process::Command;
use std::sync::LazyLock;
use std::time::Duration;

use amane::{
    Align, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin, Padding,
    Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};
use parking_lot::RwLock;

use crate::config::ShellManifest;

#[derive(Debug, Clone)]
pub struct DisplayPreset {
    pub id: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub command: &'static str,
}

pub static PRESETS: &[DisplayPreset] = &[
    DisplayPreset {
        id: "144hz",
        icon: "󰍹",
        title: "144Hz High Performance",
        subtitle: "eDP-1: 1920x1080 @ 144Hz (Gaming & High Refresh)",
        command: "wlr-randr --output eDP-1 --mode 1920x1080@144.003Hz",
    },
    DisplayPreset {
        id: "60hz",
        icon: "󰌢",
        title: "60Hz Battery Saver",
        subtitle: "eDP-1: 1920x1080 @ 60Hz (Reduced GPU/Display Power)",
        command: "wlr-randr --output eDP-1 --mode 1920x1080@60.001Hz",
    },
    DisplayPreset {
        id: "extend-right",
        icon: "󰍺",
        title: "Extend Display (Right)",
        subtitle: "Position external monitor to the right of eDP-1 (1920,0)",
        command: "ext=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}'); [ -n \"$ext\" ] && wlr-randr --output eDP-1 --pos 0,0 --output \"$ext\" --pos 1920,0",
    },
    DisplayPreset {
        id: "mirror",
        icon: "󰍺",
        title: "Mirror / Clone Display",
        subtitle: "Clone internal display to external monitor (pos 0,0)",
        command: "ext=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}'); [ -n \"$ext\" ] && wlr-randr --output eDP-1 --pos 0,0 --output \"$ext\" --pos 0,0",
    },
    DisplayPreset {
        id: "laptop-only",
        icon: "󰍹",
        title: "Laptop Screen Only",
        subtitle: "Enable eDP-1 and power down any external monitors",
        command: "ext=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}'); [ -n \"$ext\" ] && wlr-randr --output eDP-1 --on --output \"$ext\" --off",
    },
    DisplayPreset {
        id: "clamshell",
        icon: "󰍺",
        title: "External Screen Only (Clamshell)",
        subtitle: "Disable internal eDP-1 and route video to external display",
        command: "ext=$(wlr-randr 2>/dev/null | grep -E '^[a-zA-Z0-9_-]+' | grep -v 'eDP-1' | head -n 1 | awk '{print $1}'); [ -n \"$ext\" ] && wlr-randr --output \"$ext\" --on --pos 0,0 --output eDP-1 --off",
    },
    DisplayPreset {
        id: "kanshi",
        icon: "󰑐",
        title: "Reload Kanshi Profiles",
        subtitle: "Trigger hotplug autoconfig re-evaluation daemon",
        command: "pkill -HUP kanshi 2>/dev/null || (setsid kanshi </dev/null >/dev/null 2>&1 &)",
    },
];

pub struct DisplayConfigService {
    pub open: bool,
    pub selected: usize,
    pub hover_index: Option<usize>,
    pub status_message: Option<String>,
}

impl Service for DisplayConfigService {
    fn new() -> Self {
        Self {
            open: false,
            selected: 0,
            hover_index: None,
            status_message: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

static CONFIG: LazyLock<RwLock<ShellManifest>> =
    LazyLock::new(|| RwLock::new(ShellManifest::default()));

pub fn set_config(manifest: ShellManifest) {
    *CONFIG.write() = manifest;
}

pub fn toggle() {
    let mut svc = DisplayConfigService::write();
    svc.open = !svc.open;
    svc.selected = 0;
    svc.hover_index = None;
    svc.status_message = None;
}

pub fn open() {
    let mut svc = DisplayConfigService::write();
    svc.open = true;
    svc.selected = 0;
    svc.hover_index = None;
    svc.status_message = None;
}

pub fn close() {
    let mut svc = DisplayConfigService::write();
    svc.open = false;
    svc.hover_index = None;
    svc.status_message = None;
}

fn get_external_display() -> Option<String> {
    let output = Command::new("wlr-randr").output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.starts_with(' ') || line.is_empty() {
            continue;
        }
        if let Some(first) = line.split_whitespace().next()
            && !first.starts_with("eDP")
        {
            return Some(first.to_string());
        }
    }
    None
}

pub fn apply_preset(index: usize) {
    if let Some(preset) = PRESETS.get(index) {
        let title = preset.title;

        match preset.id {
            "144hz" => {
                let _ = Command::new("wlr-randr")
                    .args(["--output", "eDP-1", "--mode", "1920x1080@144.003Hz"])
                    .spawn();
            }
            "60hz" => {
                let _ = Command::new("wlr-randr")
                    .args(["--output", "eDP-1", "--mode", "1920x1080@60.001Hz"])
                    .spawn();
            }
            "extend-right" => {
                if let Some(ext) = get_external_display() {
                    let _ = Command::new("wlr-randr")
                        .args([
                            "--output", "eDP-1", "--pos", "0,0", "--output", &ext, "--pos",
                            "1920,0",
                        ])
                        .spawn();
                }
            }
            "mirror" => {
                if let Some(ext) = get_external_display() {
                    let _ = Command::new("wlr-randr")
                        .args([
                            "--output", "eDP-1", "--pos", "0,0", "--output", &ext, "--pos", "0,0",
                        ])
                        .spawn();
                }
            }
            "laptop-only" => {
                if let Some(ext) = get_external_display() {
                    let _ = Command::new("wlr-randr")
                        .args(["--output", "eDP-1", "--on", "--output", &ext, "--off"])
                        .spawn();
                }
            }
            "clamshell" => {
                if let Some(ext) = get_external_display() {
                    let _ = Command::new("wlr-randr")
                        .args([
                            "--output", &ext, "--on", "--pos", "0,0", "--output", "eDP-1", "--off",
                        ])
                        .spawn();
                }
            }
            "kanshi" => {
                use std::os::unix::process::CommandExt;
                let status = Command::new("pkill").args(["-HUP", "kanshi"]).status();
                if status.is_err() || !status.unwrap().success() {
                    let _ = Command::new("kanshi")
                        .process_group(0)
                        .stdin(std::process::Stdio::null())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn();
                }
            }
            _ => {}
        }

        let _ = Command::new("notify-send")
            .args([
                "-a",
                "dotfield",
                "-u",
                "normal",
                "Display Configurator",
                &format!("Applied preset: {title}"),
            ])
            .spawn();

        close();
    }
}

pub fn display_config_overlay() -> LayerWindow {
    let manifest = CONFIG.read().clone();
    let svc = DisplayConfigService::read();

    let is_visible = svc.open;
    let selected_idx = svc.selected;

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let fg_color = Color::from(manifest.palette.foreground.as_str()); // Bone #E8E6E3
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let muted_color = Color::from(manifest.palette.muted.as_str()); // Ash #A1A1AA
    let font = manifest.theme.font_family.clone();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Header Box
    let header_icon = Text::new("󰍹").size(16.0).color(amber_color).font(&font);

    let header_title = Text::new("Display Configurator")
        .size(13.0)
        .color(fg_color)
        .font(&font);

    let badge_rect = Rectangle::new()
        .width(68.0)
        .height(18.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, Color::from("#34343D"))
        .padding(Padding {
            top: 1.0,
            bottom: 1.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Align::Center, Align::Center)
        .child(
            Text::new("wlr-randr")
                .size(9.0)
                .color(amber_color)
                .font(&font),
        );

    let header_left = Row::new(vec![Box::new(header_icon), Box::new(header_title)])
        .gap(8.0)
        .align(Align::Center);

    let header_row = Rectangle::new()
        .width(Parent)
        .height(28.0)
        .align_child(Align::Start, Align::Center)
        .child(
            Row::new(vec![Box::new(header_left), Box::new(badge_rect)])
                .width(Parent)
                .justify(Justify::SpaceBetween)
                .align(Align::Center),
        );

    rows.push(Box::new(header_row));

    // 2. Preset Options
    let total_presets = PRESETS.len();
    let mut preset_rows: Vec<Box<dyn Widget>> = Vec::new();

    let hover_color = Color::from(manifest.palette.hover.as_str()); // Pale Amber #FFD166
    let steel_color = Color::from(manifest.palette.steel.as_str()); // Steel #8B949E
    let border_color_token = Color::from(manifest.palette.border.as_str()); // Iron #34343D

    for (idx, preset) in PRESETS.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let is_hovered = svc.hover_index == Some(idx);

        let card_bg = if is_selected {
            Color::from("#262632") // Elevated Graphite
        } else if is_hovered {
            Color::from("#202028") // Subtle hover surface
        } else {
            bg_color // Obsidian #121216
        };

        let border_color = if is_selected {
            amber_color // Industrial Amber #FFB000
        } else if is_hovered {
            Color::from("#4E4E5C") // Focused steel-iron border
        } else {
            border_color_token // Iron #34343D
        };

        let icon_color = if is_selected {
            amber_color
        } else if is_hovered {
            hover_color
        } else {
            steel_color
        };

        let title_color = if is_selected {
            amber_color
        } else if is_hovered {
            hover_color
        } else {
            fg_color
        };

        let icon_box = Rectangle::new()
            .width(28.0)
            .height(28.0)
            .radius(6.0)
            .fill(if is_selected {
                Color::from("#23232D")
            } else {
                Color::from("#16161D")
            })
            .border(
                1.0,
                if is_selected {
                    amber_color
                } else {
                    Color::from("#2A2A34")
                },
            )
            .align_child(Align::Center, Align::Center)
            .child(
                Text::new(preset.icon)
                    .size(15.0)
                    .color(icon_color)
                    .font(&font),
            );

        let title_widget = Text::new(preset.title)
            .size(12.0)
            .color(title_color)
            .font(&font);

        let subtitle_widget = Text::new(preset.subtitle)
            .size(10.0)
            .color(if is_hovered { steel_color } else { muted_color })
            .font(&font);

        let text_column = Column::new(vec![Box::new(title_widget), Box::new(subtitle_widget)])
            .gap(2.0)
            .align(Align::Start);

        let left_content = Row::new(vec![Box::new(icon_box), Box::new(text_column)])
            .gap(10.0)
            .align(Align::Center);

        let mut card_content: Vec<Box<dyn Widget>> = vec![Box::new(left_content)];

        if is_selected {
            let apply_badge = Rectangle::new()
                .width(58.0)
                .height(20.0)
                .radius(4.0)
                .fill(Color::from("#0A0A0C"))
                .border(1.0, border_color_token)
                .padding(Padding {
                    top: 1.0,
                    bottom: 1.0,
                    left: 6.0,
                    right: 6.0,
                })
                .align_child(Align::Center, Align::Center)
                .child(
                    Text::new("↵ Apply")
                        .size(9.0)
                        .color(amber_color)
                        .font(&font),
                );
            card_content.push(Box::new(apply_badge));
        }

        let preset_card = Rectangle::new()
            .width(Parent)
            .height(44.0)
            .radius(8.0)
            .fill(card_bg)
            .border(1.0, border_color)
            .cursor(Pointer)
            .padding(Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 10.0,
            })
            .align_child(Align::Start, Align::Center)
            .on_hover(move |inside| {
                let mut s = DisplayConfigService::write();
                if inside {
                    s.hover_index = Some(idx);
                } else if s.hover_index == Some(idx) {
                    s.hover_index = None;
                }
            })
            .on_click(move |_| {
                apply_preset(idx);
            })
            .child(
                Row::new(card_content)
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
            );

        preset_rows.push(Box::new(preset_card));
    }

    let preset_list = Column::new(preset_rows).width(Parent).gap(6.0);
    rows.push(Box::new(preset_list));

    // 3. Footer Bar with Navigation Hints
    let footer_text = Text::new("[↑/↓] Navigate   [↵] Apply Preset   [Esc] Dismiss")
        .size(10.0)
        .color(muted_color)
        .font(&font);

    let footer_row = Rectangle::new()
        .width(Parent)
        .height(20.0)
        .align_child(Align::Center, Align::Center)
        .child(footer_text);

    rows.push(Box::new(footer_row));

    let dialog = Rectangle::new()
        .width(440.0)
        .height(430.0)
        .radius(12.0)
        .fill(bg_color)
        .border(1.0, Color::from("#34343D"))
        .padding(Padding {
            top: 14.0,
            bottom: 12.0,
            left: 14.0,
            right: 14.0,
        })
        .child(Column::new(rows).width(Parent).gap(10.0));

    LayerWindow::new()
        .width(440.0)
        .height(430.0)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::OnDemand)
        .on_key(move |key| {
            handle_key(key, total_presets);
        })
        .space(Zone::Ignore)
        .visible(is_visible)
        .namespace("dotfield-shell-display-config")
        .child(dialog)
}

fn handle_key(key: Key, count: usize) {
    match key {
        Key::Down => {
            let mut svc = DisplayConfigService::write();
            if count > 0 && svc.selected + 1 < count {
                svc.selected += 1;
            }
        }
        Key::Up => {
            let mut svc = DisplayConfigService::write();
            svc.selected = svc.selected.saturating_sub(1);
        }
        Key::Escape => {
            close();
        }
        Key::Enter => {
            let svc = DisplayConfigService::read();
            let sel = svc.selected;
            drop(svc);
            apply_preset(sel);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_presets_present() {
        assert_eq!(PRESETS.len(), 7);
        assert_eq!(PRESETS[0].id, "144hz");
        assert_eq!(PRESETS[1].id, "60hz");
    }

    #[test]
    fn test_display_service_default() {
        let svc = DisplayConfigService::new();
        assert!(!svc.open);
        assert_eq!(svc.selected, 0);
    }

    #[test]
    fn test_display_toggle_and_close() {
        close();
        assert!(!DisplayConfigService::read().open);

        toggle();
        assert!(DisplayConfigService::read().open);

        close();
        assert!(!DisplayConfigService::read().open);
    }
}
