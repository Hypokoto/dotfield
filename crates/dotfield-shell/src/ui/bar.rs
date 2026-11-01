use chrono::Local;
use parking_lot::RwLock;
use std::sync::LazyLock;

use amane::{
    Align, Audio, Battery, Center, Color, Cpu, Full, Justify, Layer, LayerWindow, Memory, Network,
    Notifications, Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Text, Vertical,
    Widget, Workspaces, Zone,
};

use crate::config::ShellManifest;
use crate::services::niri::NiriWindowService;

static CONFIG: LazyLock<RwLock<ShellManifest>> =
    LazyLock::new(|| RwLock::new(ShellManifest::default()));

pub fn set_config(manifest: ShellManifest) {
    *CONFIG.write() = manifest;
}

pub fn top_bar() -> LayerWindow {
    let manifest = CONFIG.read().clone();
    let bar_cfg = &manifest.bar;

    let height = bar_cfg.height as f32;
    let bg_color = Color::from(manifest.palette.background.as_str());

    let mut sections: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Left section: Workspace Radar
    if bar_cfg.show_workspaces {
        sections.push(Box::new(render_workspaces(&manifest)));
    }

    // 2. Center section: Active Window Title
    if bar_cfg.show_title {
        sections.push(Box::new(render_title(&manifest)));
    }

    // 3. Right section: System Telemetry (CPU, Mem, Audio, Battery, Network, Clock, Control Center)
    if bar_cfg.show_telemetry {
        sections.push(Box::new(render_telemetry(&manifest)));
    }

    let content_row = Row::new(sections)
        .width(Parent)
        .height(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let background_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(bg_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .child(content_row);

    LayerWindow::new()
        .width(Full)
        .height(height)
        .anchor_vertical(Vertical::Top)
        .layer(Layer::Top)
        .space(Zone::Reserve)
        .namespace("dotfield-shell-bar")
        .child(background_container)
}

fn render_workspaces(manifest: &ShellManifest) -> Rectangle {
    let workspaces = Workspaces::read();
    let focused_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let urgent_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let active_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let idle_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let font = &manifest.theme.font_family;

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    for ws in workspaces.list() {
        let id = ws.id();
        let idx = ws.index();
        let label = format!("{idx}");

        let fill = if ws.focused() {
            focused_color
        } else if ws.urgent() {
            urgent_color
        } else if ws.windows() > 0 {
            active_color
        } else {
            idle_color
        };

        let tile_border = if ws.focused() {
            Color::from("#FFD166")
        } else if ws.urgent() {
            Color::from("#FF6B76")
        } else {
            border_color
        };

        let text_color = if ws.focused() { Color::BLACK } else { fg_color };

        let tile = Rectangle::new()
            .width(28.0)
            .height(24.0)
            .radius(4.0)
            .fill(fill)
            .border(1.0, tile_border)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| Workspaces::focus(id))
            .child(Text::new(label).size(11.0).color(text_color).font(font));

        buttons.push(Box::new(tile));
    }

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .child(Row::new(buttons).gap(6.0).align(Align::Center))
}

fn render_title(manifest: &ShellManifest) -> Rectangle {
    let win = NiriWindowService::read();
    let text = if win.title.is_empty() {
        if win.app_id.is_empty() {
            "Dotfield".to_string()
        } else {
            win.app_id.clone()
        }
    } else {
        win.title.clone()
    };

    // Truncate long title cleanly
    let display_text = if text.chars().count() > 60 {
        format!("{}...", text.chars().take(57).collect::<String>())
    } else {
        text
    };

    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let font = &manifest.theme.font_family;

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Center, Center)
        .child(
            Text::new(display_text)
                .size(11.0)
                .color(fg_color)
                .font(font),
        )
}

fn render_telemetry(manifest: &ShellManifest) -> Rectangle {
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let surface_color = Color::from(manifest.palette.surface.as_str());
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let font = &manifest.theme.font_family;

    let mut pills: Vec<Box<dyn Widget>> = Vec::new();

    // CPU telemetry
    let cpu = Cpu::read().percent();
    let cpu_color = if cpu > 80 {
        crimson_color
    } else if cpu > 50 {
        amber_color
    } else {
        fg_color
    };
    let cpu_pill = Rectangle::new()
        .width(66.0)
        .height(26.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .child(
            Text::new(format!("󰍛 {cpu}%"))
                .size(11.0)
                .color(cpu_color)
                .font(font),
        );
    pills.push(Box::new(cpu_pill));

    // Memory telemetry
    let mem = Memory::read().percent();
    let mem_color = if mem > 85 {
        crimson_color
    } else if mem > 70 {
        amber_color
    } else {
        fg_color
    };
    let mem_pill = Rectangle::new()
        .width(66.0)
        .height(26.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .child(
            Text::new(format!("󰘚 {mem}%"))
                .size(11.0)
                .color(mem_color)
                .font(font),
        );
    pills.push(Box::new(mem_pill));

    // Audio status (interactive click to mute, scroll to adjust)
    let audio = Audio::read();
    let vol_text = if audio.muted() {
        "󰖁 MUTED".to_string()
    } else {
        format!("󰕾 {}%", audio.volume())
    };
    let vol_pill = Rectangle::new()
        .width(82.0)
        .height(26.0)
        .radius(6.0)
        .fill(surface_color)
        .border(
            1.0,
            if audio.muted() {
                Color::from("#FF6B76")
            } else {
                border_color
            },
        )
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .on_click(|_| {
            Audio::toggle_mute();
        })
        .on_scroll(|scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            let current = Audio::read().volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_volume(target);
        })
        .align_child(Center, Center)
        .child(
            Text::new(vol_text)
                .size(11.0)
                .color(if audio.muted() {
                    crimson_color
                } else {
                    fg_color
                })
                .font(font),
        );
    pills.push(Box::new(vol_pill));

    // Battery status
    let battery = Battery::read();
    if battery.present() {
        let bat_icon = if battery.charging() {
            "󰂄"
        } else if battery.percent() > 80 {
            "󰁹"
        } else if battery.percent() > 40 {
            "󰁾"
        } else {
            "󰁺"
        };
        let bat_text = format!("{} {}%", bat_icon, battery.percent());
        let bat_pill = Rectangle::new()
            .width(76.0)
            .height(26.0)
            .radius(6.0)
            .fill(surface_color)
            .border(1.0, border_color)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .child(
                Text::new(bat_text)
                    .size(11.0)
                    .color(if battery.percent() < 20 && !battery.charging() {
                        crimson_color
                    } else {
                        fg_color
                    })
                    .font(font),
            );
        pills.push(Box::new(bat_pill));
    }

    // Network status
    let network = Network::read();
    let net_text = if network.connected() {
        if network.ssid().is_empty() {
            "󰈀 Wired".to_string()
        } else {
            format!("󰤨 {}", network.ssid())
        }
    } else {
        "󰤭 Offline".to_string()
    };
    let net_pill = Rectangle::new()
        .width(96.0)
        .height(26.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .child(
            Text::new(net_text)
                .size(11.0)
                .color(if network.connected() {
                    fg_color
                } else {
                    crimson_color
                })
                .font(font),
        );
    pills.push(Box::new(net_pill));

    // Notifications status badge (if any active)
    let notifs = Notifications::read();
    let notifs_count = notifs.list().len();
    if notifs_count > 0 {
        let notif_pill = Rectangle::new()
            .width(52.0)
            .height(26.0)
            .radius(6.0)
            .fill(crimson_color)
            .border(1.0, Color::from("#FF6B76"))
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .on_click(|_| {
                crate::ui::control_center::toggle();
            })
            .child(
                Text::new(format!("󰂚 {notifs_count}"))
                    .size(11.0)
                    .color(Color::WHITE)
                    .font(font),
            );
        pills.push(Box::new(notif_pill));
    }

    // Clock (clickable: toggles Control Center)
    let now = Local::now().format("%a %b %d  %H:%M").to_string();
    let clock_pill = Rectangle::new()
        .width(136.0)
        .height(26.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::control_center::toggle();
        })
        .child(Text::new(now).size(11.0).color(amber_color).font(font));
    pills.push(Box::new(clock_pill));

    // Control Center icon button
    let cc_pill = Rectangle::new()
        .width(30.0)
        .height(26.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::control_center::toggle();
        })
        .child(Text::new("󰒓").size(12.0).color(amber_color).font(font));
    pills.push(Box::new(cc_pill));

    Rectangle::new()
        .width(Parent)
        .height(Parent)
        .child(Row::new(pills).gap(8.0).align(Align::Center))
}
