use chrono::Local;

use amane::{
    Align, Audio, Battery, Bluetooth, Center, Color, Cpu, Full, Layer, LayerWindow, Memory,
    Network, Notifications, Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Text,
    Vertical, Widget, Workspaces, Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::services::niri::NiriWindowService;
use crate::ui::tokens::{ThemeTokens, border, radius, spacing, typography};

pub fn top_bar_monitor(monitor: &amane::Monitor) -> LayerWindow {
    build_top_bar(Some(&monitor.name))
}

const CANONICAL_WORKSPACES: [&str; 5] = [
    "01 DEV",
    "02 SYSTEM",
    "03 RESEARCH",
    "04 SECURITY",
    "05 SCRATCH",
];

fn build_top_bar(monitor_name: Option<&str>) -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let bar_cfg = &manifest.bar;
    let tokens = ThemeTokens::from_manifest(&manifest);

    let height = bar_cfg.height as f32;

    // Left section: BLACKSITE Brand & Arch Indicator & Active Window Title
    let mut left_items: Vec<Box<dyn Widget>> = Vec::new();
    left_items.push(Box::new(render_brand(&manifest, &tokens)));
    if bar_cfg.show_title {
        left_items.push(Box::new(render_title(&manifest, &tokens)));
    }
    let left_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Align::Start, Align::Center)
        .child(
            Row::new(left_items)
                .gap(spacing::GAP_MD)
                .align(Align::Center),
        );

    // Center section: 5 Workspaces
    let (center_widget, ws_width) = if bar_cfg.show_workspaces {
        render_workspaces(&manifest, &tokens, monitor_name)
    } else {
        (Row::new(Vec::new()), 0.0)
    };
    let center_container = Rectangle::new()
        .width(ws_width)
        .height(Parent)
        .align_child(Center, Center)
        .child(center_widget);

    // Right section: System Telemetry
    let right_widget = if bar_cfg.show_telemetry {
        render_telemetry(&manifest, &tokens, monitor_name)
    } else {
        Rectangle::new().width(0.0).height(0.0)
    };
    let right_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Align::End, Align::Center)
        .child(right_widget);

    let content_row = Row::new(vec![
        Box::new(left_container),
        Box::new(center_container),
        Box::new(right_container),
    ])
    .width(Parent)
    .height(Parent)
    .align(Align::Center);

    let background_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(tokens.bg)
        .border(border::THIN, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
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

fn render_brand(manifest: &ShellManifest, tokens: &ThemeTokens) -> Rectangle {
    let font = &manifest.theme.font_family;
    Rectangle::new()
        .width(116.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::launcher::toggle();
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("󰣇").size(13.0).color(tokens.accent).font(font)),
                Box::new(
                    Text::new("BLACKSITE")
                        .size(typography::CAPTION + 0.5)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ])
            .gap(6.0)
            .align(Align::Center),
        )
}

fn render_workspaces(
    manifest: &ShellManifest,
    tokens: &ThemeTokens,
    monitor_name: Option<&str>,
) -> (Row, f32) {
    let workspaces = Workspaces::read();
    let font = &manifest.theme.font_family;

    let mut buttons: Vec<Box<dyn Widget>> = Vec::new();

    let all_workspaces = workspaces.list();
    let relevant_workspaces: Vec<_> = all_workspaces
        .iter()
        .filter(|ws| {
            if let Some(m) = monitor_name {
                ws.output() == Some(m) || ws.output().is_none()
            } else {
                true
            }
        })
        .collect();

    let max_idx = relevant_workspaces
        .iter()
        .map(|ws| ws.index() as usize)
        .max()
        .unwrap_or(0)
        .max(5);

    let mut total_width = 0.0;
    let gap = 6.0;

    for num in 1..=max_idx {
        let ws_match = relevant_workspaces.iter().find(|ws| {
            if ws.index() as usize == num {
                return true;
            }
            if let Some(name) = ws.name() {
                let trimmed = name.trim();
                let prefix2 = format!("{num:02}");
                let prefix1 = format!("{num}");
                if trimmed.starts_with(&prefix2) || trimmed.starts_with(&prefix1) {
                    return true;
                }
            }
            false
        });

        let is_focused = ws_match
            .map(|ws| ws.focused() || ws.active())
            .unwrap_or(false);
        let is_urgent = ws_match.map(|ws| ws.urgent()).unwrap_or(false);
        let is_occupied = ws_match.map(|ws| ws.windows() > 0).unwrap_or(false);
        let ws_id = ws_match.map(|ws| ws.id());

        let pill_width = 34.0;

        let fill = if is_focused {
            tokens.accent
        } else if is_urgent {
            tokens.danger
        } else if is_occupied {
            tokens.surface_elevated
        } else {
            tokens.surface
        };

        let tile_border = if is_focused {
            tokens.accent
        } else if is_urgent {
            tokens.border_danger
        } else {
            tokens.border_subtle
        };

        let text_color = if is_focused {
            tokens.bg
        } else if is_urgent || is_occupied {
            tokens.text_primary
        } else {
            tokens.steel
        };

        let target_name = if num <= CANONICAL_WORKSPACES.len() {
            CANONICAL_WORKSPACES[num - 1].to_string()
        } else {
            format!("{num}")
        };

        let label = format!("{num:02}");

        let tile = Rectangle::new()
            .width(pill_width)
            .height(26.0)
            .radius(radius::SM)
            .fill(fill)
            .border(1.0, tile_border)
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 4.0,
                right: 4.0,
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                if let Some(id) = ws_id {
                    Workspaces::focus(id);
                } else {
                    Workspaces::focus_name(target_name.clone());
                }
            })
            .child(
                Text::new(label)
                    .size(typography::CAPTION + 0.5)
                    .color(text_color)
                    .font(font),
            );

        buttons.push(Box::new(tile));
        total_width += pill_width;
    }

    if buttons.len() > 1 {
        total_width += (buttons.len() - 1) as f32 * gap;
    }

    (Row::new(buttons).gap(gap).align(Align::Center), total_width)
}

fn render_title(manifest: &ShellManifest, tokens: &ThemeTokens) -> Rectangle {
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
    let display_text = if text.chars().count() > 28 {
        format!("{}...", text.chars().take(25).collect::<String>())
    } else {
        text
    };

    let font = &manifest.theme.font_family;
    let title_w = (display_text.chars().count() as f32 * 8.5 + 40.0).clamp(64.0, 280.0);

    Rectangle::new()
        .width(title_w)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .child(
            Row::new(vec![
                Box::new(Text::new("󰣆").size(12.0).color(tokens.accent).font(font)),
                Box::new(
                    Text::new(display_text)
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ])
            .gap(6.0)
            .align(Align::Center),
        )
}

fn render_telemetry(
    manifest: &ShellManifest,
    tokens: &ThemeTokens,
    monitor_name: Option<&str>,
) -> Rectangle {
    let font = &manifest.theme.font_family;
    let mut pills: Vec<Box<dyn Widget>> = Vec::new();

    // CPU telemetry
    let cpu = Cpu::read().percent();
    let cpu_color = if cpu > 80 {
        tokens.danger
    } else if cpu > 50 {
        tokens.accent
    } else {
        tokens.steel
    };
    let cpu_pill = Rectangle::new()
        .width(58.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .child(
            Text::new(format!("󰍛 {cpu}%"))
                .size(typography::SECONDARY)
                .color(cpu_color)
                .font(font),
        );
    pills.push(Box::new(cpu_pill));

    // Memory telemetry
    let mem = Memory::read().percent();
    let mem_color = if mem > 85 {
        tokens.danger
    } else if mem > 70 {
        tokens.accent
    } else {
        tokens.steel
    };
    let mem_pill = Rectangle::new()
        .width(58.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .child(
            Text::new(format!("󰘚 {mem}%"))
                .size(typography::SECONDARY)
                .color(mem_color)
                .font(font),
        );
    pills.push(Box::new(mem_pill));

    // Audio status (PipeWire: interactive click to open Audio widget, scroll to adjust volume)
    let audio = Audio::read();
    let mic_muted = audio.microphone_muted();
    let vol_text = if audio.muted() {
        "󰖁 MUTED".to_string()
    } else if mic_muted {
        format!("󰕾 {}% 󰍭", audio.volume())
    } else {
        format!("󰕾 {}%", audio.volume())
    };
    let vol_pill = Rectangle::new()
        .width(76.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(
            1.0,
            if audio.muted() {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .on_click(move |_| {
            crate::ui::control_center::open_view(crate::ui::control_center::ActiveView::Audio);
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
                .size(typography::SECONDARY)
                .color(if audio.muted() {
                    tokens.danger
                } else {
                    tokens.steel
                })
                .font(font),
        );
    pills.push(Box::new(vol_pill));

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
            .width(68.0)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 6.0,
                right: 6.0,
            })
            .align_child(Center, Center)
            .child(
                Text::new(bat_text)
                    .size(typography::SECONDARY)
                    .color(if battery.percent() < 20 && !battery.charging() {
                        tokens.danger
                    } else {
                        tokens.steel
                    })
                    .font(font),
            );
        pills.push(Box::new(bat_pill));
    }

    // Network status (NetworkManager: interactive click to open Network widget)
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
        .width(84.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| {
            crate::ui::control_center::open_view(crate::ui::control_center::ActiveView::Network);
        })
        .child(
            Text::new(net_text)
                .size(typography::SECONDARY)
                .color(if network.connected() {
                    tokens.steel
                } else {
                    tokens.danger
                })
                .font(font),
        );
    pills.push(Box::new(net_pill));

    // Bluetooth status (BlueZ: interactive click to open Bluetooth widget)
    let bluetooth = Bluetooth::read();
    if bluetooth.available() {
        let bt_powered = bluetooth.powered();
        let connected_count = bluetooth.devices().iter().filter(|d| d.connected()).count();
        let bt_text = if !bt_powered {
            "󰂲 Off".to_string()
        } else if connected_count > 0 {
            format!("󰂱 {connected_count}")
        } else {
            "󰂯 On".to_string()
        };
        let bt_pill = Rectangle::new()
            .width(54.0)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.surface)
            .border(
                1.0,
                if connected_count > 0 {
                    tokens.border_focus
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 6.0,
                right: 6.0,
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                crate::ui::control_center::open_view(
                    crate::ui::control_center::ActiveView::Bluetooth,
                );
            })
            .child(
                Text::new(bt_text)
                    .size(typography::SECONDARY)
                    .color(if connected_count > 0 {
                        tokens.accent
                    } else if bt_powered {
                        tokens.steel
                    } else {
                        tokens.text_muted
                    })
                    .font(font),
            );
        pills.push(Box::new(bt_pill));
    }

    // Notifications status badge (if any active)
    let notifs = Notifications::read();
    let notifs_count = notifs.list().len();
    if notifs_count > 0 {
        let notif_pill = Rectangle::new()
            .width(52.0)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.danger)
            .border(1.0, tokens.border_danger)
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                crate::ui::control_center::toggle();
            })
            .child(
                Text::new(format!("󰂚 {notifs_count}"))
                    .size(typography::SECONDARY)
                    .color(Color::WHITE)
                    .font(font),
            );
        pills.push(Box::new(notif_pill));
    }

    // System Tray (StatusNotifierItem applications & status cluster)
    pills.push(crate::ui::tray::render_tray_pill(
        manifest,
        font,
        monitor_name,
    ));

    // Active Screen Recording indicator (Crimson with timer, click to open recorder overlay)
    let is_recording = crate::ui::screen_record::ScreenRecordService::read().is_recording;
    let rec_elapsed = crate::ui::screen_record::ScreenRecordService::read()
        .started_at
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);
    if is_recording {
        let mins = rec_elapsed / 60;
        let secs = rec_elapsed % 60;
        let rec_text = format!("󰑋 REC {mins:02}:{secs:02}");
        let rec_pill = Rectangle::new()
            .width(108.0)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.danger)
            .border(1.0, tokens.border_danger)
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                crate::ui::screen_record::toggle();
            })
            .child(
                Text::new(rec_text)
                    .size(typography::SECONDARY)
                    .color(Color::WHITE)
                    .font(font),
            );
        pills.push(Box::new(rec_pill));
    }

    // Clock (clickable: toggles interactive monthly Calendar)
    let now = Local::now().format("%a %b %d  %H:%M").to_string();
    let clock_pill = Rectangle::new()
        .width(132.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| {
            crate::ui::calendar::toggle();
        })
        .child(
            Text::new(now)
                .size(typography::SECONDARY)
                .color(tokens.accent)
                .font(font),
        );
    pills.push(Box::new(clock_pill));

    // Powermenu icon button (Crimson #E63946)
    let power_pill = Rectangle::new()
        .width(28.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            crate::ui::powermenu::toggle();
        })
        .child(Text::new("󰐥").size(12.0).color(tokens.danger).font(font));
    pills.push(Box::new(power_pill));

    let pills_count = pills.len();
    let mut telemetry_w: f32 = 58.0 + 58.0 + 76.0 + 84.0 + 132.0 + 28.0;
    if battery.present() {
        telemetry_w += 68.0;
    }
    if bluetooth.available() {
        telemetry_w += 54.0;
    }
    if notifs_count > 0 {
        telemetry_w += 52.0;
    }
    if is_recording {
        telemetry_w += 108.0;
    }
    let tray_items_len = crate::services::tray::TrayService::read().items.len();
    let tray_w = if tray_items_len == 0 {
        32.0
    } else {
        (tray_items_len.min(6) as f32 * 26.0) + 12.0
    };
    telemetry_w += tray_w;
    if pills_count > 1 {
        telemetry_w += (pills_count - 1) as f32 * 6.0;
    }

    Rectangle::new()
        .width(telemetry_w)
        .height(Parent)
        .child(Row::new(pills).gap(6.0).align(Align::Center))
}
