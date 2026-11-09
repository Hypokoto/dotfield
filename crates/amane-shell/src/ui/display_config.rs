use std::process::Command;
use std::time::Duration;

use amane::{
    Align, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin, OutputTarget,
    Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};

use crate::config::ConfigService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

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
    // 1. Check native Niri IPC outputs first
    if let Ok(output) = Command::new("niri").args(["msg", "-j", "outputs"]).output()
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        && let Some(map) = val.as_object()
    {
        for key in map.keys() {
            if !key.starts_with("eDP") && !key.starts_with("Writeback") {
                return Some(key.clone());
            }
        }
    }

    // 2. Check Sway outputs
    if let Ok(output) = Command::new("swaymsg").args(["-t", "get_outputs"]).output()
        && let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        && let Some(arr) = val.as_array()
    {
        for item in arr {
            if let Some(name) = item.get("name").and_then(|n| n.as_str())
                && !name.starts_with("eDP")
                && !name.starts_with("Writeback")
            {
                return Some(name.to_string());
            }
        }
    }

    // 3. Check wlr-randr outputs
    if let Ok(output) = Command::new("wlr-randr").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with(' ') || line.is_empty() {
                continue;
            }
            if let Some(first) = line.split_whitespace().next()
                && !first.starts_with("eDP")
                && !first.starts_with("Writeback")
            {
                return Some(first.to_string());
            }
        }
    }

    // 4. Direct DRM sysfs fallback for connected displays that might be unmapped
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            let status_file = path.join("status");
            if let Ok(content) = std::fs::read_to_string(&status_file)
                && content.trim() == "connected"
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
            {
                let port = name.split_once('-').map(|(_, p)| p).unwrap_or(name);
                if !port.starts_with("eDP") && !port.starts_with("Writeback") {
                    return Some(port.to_string());
                }
            }
        }
    }

    None
}

pub fn apply_preset(index: usize) {
    if let Some(preset) = PRESETS.get(index) {
        let title = preset.title;
        let ext_opt = get_external_display();

        let switch_kanshi = |profile: &str| -> bool {
            Command::new("kanshictl")
                .args(["switch", profile])
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        };

        match preset.id {
            "144hz" => {
                let profile = if let Some(ref ext) = ext_opt {
                    if ext.starts_with("DP") {
                        "docked-dp"
                    } else {
                        "docked-dual"
                    }
                } else {
                    "undocked"
                };

                if !switch_kanshi(profile) {
                    let _ = Command::new("niri")
                        .args(["msg", "output", "eDP-1", "mode", "1920x1080@144.003"])
                        .status();
                }
            }
            "60hz" => {
                let profile = if let Some(ref ext) = ext_opt {
                    if ext.starts_with("DP") {
                        "docked-dp-60hz"
                    } else {
                        "docked-dual-60hz"
                    }
                } else {
                    "undocked-60hz"
                };

                if !switch_kanshi(profile) {
                    let _ = Command::new("niri")
                        .args(["msg", "output", "eDP-1", "mode", "1920x1080@60.001"])
                        .status();
                }
            }
            "extend-right" => {
                if let Some(ref ext) = ext_opt {
                    let profile = if ext.starts_with("DP") {
                        "docked-dp"
                    } else {
                        "docked-dual"
                    };

                    if !switch_kanshi(profile) {
                        let _ = Command::new("niri")
                            .args(["msg", "output", "eDP-1", "on"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", "eDP-1", "position", "set", "0", "0"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "on"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "position", "set", "1920", "0"])
                            .status();
                    }
                }
            }
            "mirror" => {
                if let Some(ref ext) = ext_opt {
                    let profile = if ext.starts_with("DP") {
                        "docked-mirror-dp"
                    } else {
                        "docked-mirror-hdmi"
                    };

                    if !switch_kanshi(profile) {
                        let _ = Command::new("niri")
                            .args(["msg", "output", "eDP-1", "on"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", "eDP-1", "position", "set", "0", "0"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "on"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "position", "set", "0", "0"])
                            .status();
                    }
                }
            }
            "laptop-only" => {
                if let Some(ref ext) = ext_opt {
                    let profile = if ext.starts_with("DP") {
                        "laptop-only-dp"
                    } else {
                        "laptop-only-hdmi"
                    };

                    if !switch_kanshi(profile) {
                        // Crucial: Turn internal display on BEFORE turning external display off
                        let _ = Command::new("niri")
                            .args(["msg", "output", "eDP-1", "on"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "off"])
                            .status();
                    }
                } else {
                    let _ = switch_kanshi("undocked");
                    let _ = Command::new("niri")
                        .args(["msg", "output", "eDP-1", "on"])
                        .status();
                }
            }
            "clamshell" => {
                if let Some(ref ext) = ext_opt {
                    let profile = if ext.starts_with("DP") {
                        "clamshell-dp"
                    } else {
                        "clamshell-hdmi"
                    };

                    if !switch_kanshi(profile) {
                        // Crucial: Turn external display on BEFORE turning internal display off
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "on"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", ext, "position", "set", "0", "0"])
                            .status();
                        let _ = Command::new("niri")
                            .args(["msg", "output", "eDP-1", "off"])
                            .status();
                    }
                }
            }
            "kanshi" => {
                let _ = Command::new("kanshictl").arg("reload").status();
                let status = Command::new("systemctl")
                    .args(["--user", "restart", "kanshi.service"])
                    .status();
                if !status.is_ok_and(|s| s.success()) {
                    let _ = Command::new("pkill").args(["-HUP", "kanshi"]).status();
                }
            }
            _ => {}
        }

        // Trigger wallpaper refresh for newly configured active displays
        let home = std::env::var("HOME").unwrap_or_default();
        let wp_script = format!("{home}/.config/dotfield/scripts/wallpaper.sh");
        if std::path::Path::new(&wp_script).exists() {
            let _ = Command::new("systemd-run")
                .args([
                    "--user",
                    "--scope",
                    "--quiet",
                    "sh",
                    "-c",
                    &format!("sleep 0.3; exec {wp_script}"),
                ])
                .spawn();
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
    let manifest = ConfigService::read().manifest.clone();
    let svc = DisplayConfigService::read();

    let is_visible = svc.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-display-config");
    }
    let selected_idx = svc.selected;

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = manifest.theme.font_family.clone();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // Standardized Modal Header with close button and context tag
    let header = crate::ui::components::modal_header(
        "󰍹",
        "Display Configurator",
        Some("wlr-randr"),
        &tokens,
        close,
    );
    rows.push(Box::new(header));

    // Preset options
    let total_presets = PRESETS.len();
    let mut preset_rows: Vec<Box<dyn Widget>> = Vec::new();

    for (idx, preset) in PRESETS.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let is_hovered = svc.hover_index == Some(idx);

        let (card_bg, border_color, title_color, icon_color) = if is_selected {
            (
                tokens.surface_selected,
                tokens.border_focus,
                tokens.accent,
                tokens.accent,
            )
        } else if is_hovered {
            (
                tokens.surface_hover,
                tokens.border_hover,
                tokens.text_primary,
                tokens.accent,
            )
        } else {
            (
                tokens.surface_elevated,
                tokens.border_subtle,
                tokens.text_primary,
                tokens.text_secondary,
            )
        };

        let icon_box = Rectangle::new()
            .width(28.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_selected {
                tokens.surface_active
            } else {
                tokens.surface
            })
            .border(
                1.0,
                if is_selected {
                    tokens.border_focus
                } else {
                    tokens.border_subtle
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
            .size(typography::BODY)
            .color(title_color)
            .font(&font);

        let subtitle_widget = Text::new(preset.subtitle)
            .size(typography::BADGE)
            .color(if is_hovered {
                tokens.text_secondary
            } else {
                tokens.text_muted
            })
            .font(&font);

        let text_column = Column::new(vec![Box::new(title_widget), Box::new(subtitle_widget)])
            .gap(2.0)
            .align(Align::Start);

        let left_content = Row::new(vec![Box::new(icon_box), Box::new(text_column)])
            .gap(spacing::GAP_MD)
            .align(Align::Center);

        let mut card_content: Vec<Box<dyn Widget>> = vec![Box::new(left_content)];

        if is_selected {
            let apply_badge =
                crate::ui::components::status_badge("↵ Apply", tokens.accent, &tokens);
            card_content.push(Box::new(apply_badge));
        }

        let preset_card = Rectangle::new()
            .width(Parent)
            .height(44.0)
            .radius(radius::MD)
            .fill(card_bg)
            .border(1.0, border_color)
            .cursor(Pointer)
            .padding(Padding {
                top: 4.0,
                bottom: 4.0,
                left: spacing::PAD_SM,
                right: spacing::PAD_MD,
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

    let preset_list = Column::new(preset_rows).width(Parent).gap(spacing::GAP_SM);
    rows.push(Box::new(preset_list));

    // Footer bar with standardized keycaps
    let footer_row = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(crate::ui::components::keycap("↑/↓", &tokens)),
                Box::new(
                    Text::new("Navigate")
                        .size(typography::CAPTION)
                        .color(tokens.text_muted)
                        .font(&font),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        ),
        Box::new(
            Row::new(vec![
                Box::new(crate::ui::components::keycap("↵", &tokens)),
                Box::new(
                    Text::new("Apply")
                        .size(typography::CAPTION)
                        .color(tokens.text_muted)
                        .font(&font),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        ),
        Box::new(
            Row::new(vec![
                Box::new(crate::ui::components::keycap("Esc", &tokens)),
                Box::new(
                    Text::new("Dismiss")
                        .size(typography::CAPTION)
                        .color(tokens.text_muted)
                        .font(&font),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        ),
    ])
    .width(Parent)
    .justify(Justify::Center)
    .align(Align::Center)
    .gap(spacing::GAP_MD);

    rows.push(Box::new(footer_row));

    let dialog = Rectangle::new()
        .width(440.0)
        .height(450.0)
        .radius(radius::LG)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(Column::new(rows).width(Parent).gap(spacing::GAP_MD));

    LayerWindow::new()
        .width(440.0)
        .height(450.0)
        .target_output(OutputTarget::Active)
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
