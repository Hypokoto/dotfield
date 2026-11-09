use chrono::Local;

use amane::{
    Align, Audio, Battery, Bluetooth, Center, Color, Cpu, Full, Justify, Layer, LayerWindow,
    Memory, Network, Notifications, Padding, Parent, Pointer, Rectangle, Row, Scroll, Service,
    Text, Vertical, Widget, Workspaces, Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::services::niri::NiriWindowService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub fn top_bar_monitor(monitor: &amane::Monitor) -> LayerWindow {
    build_top_bar(Some(&monitor.name))
}

fn build_top_bar(monitor_name: Option<&str>) -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let bar_cfg = &manifest.bar;
    let tokens = ThemeTokens::from_manifest(&manifest);

    let height = bar_cfg.height as f32;

    let mut sections: Vec<Box<dyn Widget>> = Vec::new();

    // Left section: Workspace Radar and Active Window Title
    let mut left_items: Vec<Box<dyn Widget>> = Vec::new();
    if bar_cfg.show_workspaces {
        left_items.push(Box::new(render_workspaces(
            &manifest,
            &tokens,
            monitor_name,
        )));
    }
    if bar_cfg.show_title {
        left_items.push(Box::new(render_title(&manifest, &tokens)));
    }
    if !left_items.is_empty() {
        sections.push(Box::new(
            Row::new(left_items)
                .gap(spacing::GAP_MD)
                .align(Align::Center),
        ));
    }

    // Right section: System Telemetry
    if bar_cfg.show_telemetry {
        sections.push(Box::new(render_telemetry(&manifest, &tokens, monitor_name)));
    }

    let content_row = Row::new(sections)
        .width(Parent)
        .height(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let background_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(tokens.bg)
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

fn render_workspaces(
    manifest: &ShellManifest,
    tokens: &ThemeTokens,
    monitor_name: Option<&str>,
) -> Rectangle {
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

    for ws in relevant_workspaces {
        let id = ws.id();
        let idx = ws.index();
        let label = format!("{idx}");

        let fill = if ws.focused() {
            tokens.accent
        } else if ws.urgent() {
            tokens.danger
        } else if ws.windows() > 0 {
            tokens.surface_elevated
        } else {
            tokens.surface
        };

        let tile_border = if ws.focused() {
            tokens.border_focus
        } else if ws.urgent() {
            tokens.border_danger
        } else {
            tokens.border_subtle
        };

        let text_color = if ws.focused() {
            Color::BLACK
        } else {
            tokens.text_primary
        };
        let mon = monitor_name.map(String::from);

        let tile = Rectangle::new()
            .width(28.0)
            .height(24.0)
            .radius(radius::SM)
            .fill(fill)
            .border(1.0, tile_border)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                if let Some(ref name) = mon {
                    amane::focus_monitor(name);
                }
                Workspaces::focus(id);
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .color(text_color)
                    .font(font),
            );

        buttons.push(Box::new(tile));
    }

    let ws_width = if buttons.is_empty() {
        28.0
    } else {
        (buttons.len() as f32 * 28.0) + ((buttons.len().saturating_sub(1)) as f32 * 6.0)
    };

    Rectangle::new()
        .width(ws_width)
        .height(Parent)
        .child(Row::new(buttons).gap(6.0).align(Align::Center))
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
    let title_w = (display_text.chars().count() as f32 * 7.2 + 34.0).clamp(64.0, 240.0);

    Rectangle::new()
        .width(title_w)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: spacing::PAD_SM,
            right: spacing::PAD_SM,
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
        tokens.text_primary
    };
    let cpu_pill = Rectangle::new()
        .width(66.0)
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
        tokens.text_primary
    };
    let mem_pill = Rectangle::new()
        .width(66.0)
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
    let mon_audio = monitor_name.map(String::from);
    let vol_pill = Rectangle::new()
        .width(88.0)
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
            left: 8.0,
            right: 8.0,
        })
        .on_click(move |_| {
            if let Some(ref name) = mon_audio {
                amane::focus_monitor(name);
            }
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
                    tokens.text_primary
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
            .width(76.0)
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
                Text::new(bat_text)
                    .size(typography::SECONDARY)
                    .color(if battery.percent() < 20 && !battery.charging() {
                        tokens.danger
                    } else {
                        tokens.text_primary
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
    let mon_net = monitor_name.map(String::from);
    let net_pill = Rectangle::new()
        .width(96.0)
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
        .on_click(move |_| {
            if let Some(ref name) = mon_net {
                amane::focus_monitor(name);
            }
            crate::ui::control_center::open_view(crate::ui::control_center::ActiveView::Network);
        })
        .child(
            Text::new(net_text)
                .size(typography::SECONDARY)
                .color(if network.connected() {
                    tokens.text_primary
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
        let mon_bt = monitor_name.map(String::from);
        let bt_pill = Rectangle::new()
            .width(62.0)
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
                if let Some(ref name) = mon_bt {
                    amane::focus_monitor(name);
                }
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
                        tokens.text_primary
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
        let mon_notif = monitor_name.map(String::from);
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
                if let Some(ref name) = mon_notif {
                    amane::focus_monitor(name);
                }
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
        let mon_rec = monitor_name.map(String::from);
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
                if let Some(ref name) = mon_rec {
                    amane::focus_monitor(name);
                }
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
    let mon_clock = monitor_name.map(String::from);
    let clock_pill = Rectangle::new()
        .width(162.0)
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
        .on_click(move |_| {
            if let Some(ref name) = mon_clock {
                amane::focus_monitor(name);
            }
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
    let mon_power = monitor_name.map(String::from);
    let power_pill = Rectangle::new()
        .width(28.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            if let Some(ref name) = mon_power {
                amane::focus_monitor(name);
            }
            crate::ui::powermenu::toggle();
        })
        .child(Text::new("󰐥").size(12.0).color(tokens.danger).font(font));
    pills.push(Box::new(power_pill));

    let pills_count = pills.len();
    let mut telemetry_w: f32 = 66.0 + 66.0 + 88.0 + 96.0 + 162.0 + 28.0;
    if battery.present() {
        telemetry_w += 76.0;
    }
    if bluetooth.available() {
        telemetry_w += 62.0;
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
