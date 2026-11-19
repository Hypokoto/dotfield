use amane::{
    Align, Audio, Bluetooth, Center, Color, Full, Layer, LayerWindow, Network, Notifications,
    Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Text, Vertical, Widget, Workspaces,
    Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::services::niri::NiriWindowService;
use crate::services::presentation::PresentationService;
use crate::services::telemetry::TelemetryService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

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

const WS_LABELS: [&str; 10] = ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"];

fn build_top_bar(monitor_name: Option<&str>) -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let bar_cfg = &manifest.bar;
    let tokens = ThemeTokens::from_manifest(&manifest);

    let height = bar_cfg.height as f32;

    let telem = TelemetryService::read();
    let focused_win = crate::services::niri::NiriService::focused_window();
    let is_full_width = focused_win.is_full_width;

    // Contextual Focus Shield:
    // When focused on a 1.0 full-width column:
    // - If normal (no anomaly): auto-hide top bar completely and reserve 0 zone (true distraction-free full width).
    // - If anomaly tripped: surface floating anomaly HUD capsule with Zone::Respect (exclusive_zone = 0)
    //   so no layout thrashing or window resizing occurs.
    if is_full_width && !telem.is_anomalous() {
        return LayerWindow::new()
            .width(Full)
            .height(0.0)
            .anchor_vertical(Vertical::Top)
            .layer(Layer::Top)
            .space(Zone::Respect)
            .visible(false)
            .namespace("dotfield-shell-bar")
            .child(Rectangle::new().width(0.0).height(0.0));
    }

    let bar_zone = if is_full_width {
        Zone::Respect
    } else {
        Zone::Reserve
    };

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
        .fill(Color::TRANSPARENT)
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
        .space(bar_zone)
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

        // Only display workspace if it has open applications (occupied), is active/focused, or urgent
        let should_display = is_focused || is_occupied || is_urgent;
        if !should_display {
            continue;
        }

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

        let label_owned;
        let label: &str = if num >= 1 && num <= WS_LABELS.len() {
            WS_LABELS[num - 1]
        } else {
            label_owned = format!("{num:02}");
            &label_owned
        };

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

    // Fallback: If no workspace is occupied or focused yet, at least show workspace 01
    if buttons.is_empty() {
        let pill_width = 34.0;
        let target_name = CANONICAL_WORKSPACES[0].to_string();
        let tile = Rectangle::new()
            .width(pill_width)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.accent)
            .border(1.0, tokens.accent)
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 4.0,
                right: 4.0,
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                Workspaces::focus_name(target_name.clone());
            })
            .child(
                Text::new("01")
                    .size(typography::CAPTION + 0.5)
                    .color(tokens.bg)
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
    let display_text = if !win.display_title.is_empty() {
        win.display_title.as_str()
    } else if !win.title.is_empty() {
        win.title.as_str()
    } else if !win.app_id.is_empty() {
        win.app_id.as_str()
    } else {
        "Dotfield"
    };

    let font = &manifest.theme.font_family;
    let title_w = if win.layout_width > 0.0 {
        win.layout_width
    } else {
        (display_text.chars().count() as f32 * 8.5 + 40.0).clamp(64.0, 280.0)
    };

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

    // Exception HUD: Ghost Capsule (trips on CPU > 85%, RAM > 80%, PSI stalls, or Battery <= 20% discharging)
    let telem = TelemetryService::read();
    let mut anomaly_pill_w = 0.0;
    if telem.is_anomalous() {
        let (label, is_crit) = if telem.cpu_stalled {
            ("󱐋 CPU STALL".to_string(), true)
        } else if telem.mem_stalled {
            ("󱐋 MEM STALL".to_string(), true)
        } else if telem.cpu_percent >= 85 {
            let detail = if let (Some(name), Some(pid)) = (&telem.culprit_name, telem.culprit_pid) {
                format!("󰍛 {}% {} ({})", telem.cpu_percent, name, pid)
            } else {
                format!("󰍛 {}%", telem.cpu_percent)
            };
            (detail, telem.is_critical())
        } else if telem.mem_percent >= 80 {
            let detail = if let Some(name) = &telem.culprit_name {
                format!("󰘚 {}% {}", telem.mem_percent, name)
            } else {
                format!("󰘚 {}%", telem.mem_percent)
            };
            (detail, telem.is_critical())
        } else if telem.battery_present
            && !telem.battery_charging
            && telem.battery_percent.unwrap_or(100) <= 20
        {
            let bat_p = telem.battery_percent.unwrap_or(0);
            (format!("󰁺 {bat_p}% LOW BAT"), bat_p <= 10)
        } else {
            (format!("󰍛 {}%", telem.cpu_percent), false)
        };

        let border_col = if is_crit {
            tokens.border_danger
        } else {
            tokens.border_focus
        };
        let text_col = if is_crit {
            tokens.danger
        } else {
            tokens.accent
        };
        let pill_w = (label.chars().count() as f32 * 7.5).max(64.0) + 16.0;
        anomaly_pill_w = pill_w;

        let anomaly_pill = Rectangle::new()
            .width(pill_w)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, border_col)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .color(text_col)
                    .font(font),
            );
        pills.push(Box::new(anomaly_pill));
    }

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

    // Notifications status badge (with Contextual Focus Shield in 01 DEV and 04 SECURITY)
    let notifs = Notifications::read();
    let notifs_count = notifs.list().len();
    let in_focus_shield = Workspaces::read().list().iter().any(|ws| {
        (ws.focused() || ws.active())
            && ws
                .name()
                .map(|n| n.contains("01 DEV") || n.contains("04 SECURITY"))
                .unwrap_or(false)
    });
    let has_critical = notifs
        .list()
        .iter()
        .any(|n| n.urgency() == amane::Urgency::Critical);

    let mut notif_pill_w = 0.0;
    if notifs_count > 0 {
        let (pill_fill, border_col, text_col, icon_label, width) =
            if in_focus_shield && !has_critical {
                (
                    tokens.surface,
                    tokens.border_subtle,
                    tokens.steel,
                    "•".to_string(),
                    24.0,
                )
            } else {
                (
                    tokens.danger,
                    tokens.border_danger,
                    Color::WHITE,
                    format!("󰂚 {notifs_count}"),
                    52.0,
                )
            };
        notif_pill_w = width;

        let notif_pill = Rectangle::new()
            .width(width)
            .height(26.0)
            .radius(radius::SM)
            .fill(pill_fill)
            .border(1.0, border_col)
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: if in_focus_shield && !has_critical {
                    4.0
                } else {
                    8.0
                },
                right: if in_focus_shield && !has_critical {
                    4.0
                } else {
                    8.0
                },
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                crate::ui::control_center::toggle();
            })
            .child(
                Text::new(icon_label)
                    .size(if in_focus_shield && !has_critical {
                        14.0
                    } else {
                        typography::SECONDARY
                    })
                    .color(text_col)
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

    // Presentation Mode HUD Capsule (Director Stopwatch, rendered exclusively on primary display)
    let presentation = PresentationService::global();
    let is_presenting = presentation.is_active() && is_presentation_primary_display(monitor_name);
    let mut pres_pill_w = 0.0;
    if is_presenting {
        let elapsed_str = presentation
            .elapsed_formatted()
            .unwrap_or_else(|| "00:00".to_string());
        let pres_label = format!("󰐩 [PRESENTING] {elapsed_str}");
        let pill_w = 144.0;
        pres_pill_w = pill_w;

        let pres_pill = Rectangle::new()
            .width(pill_w)
            .height(26.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_focus)
            .cursor(Pointer)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .on_click(|_| {
                PresentationService::global().toggle();
            })
            .child(
                Text::new(pres_label)
                    .size(typography::SECONDARY)
                    .color(tokens.accent)
                    .font(font),
            );
        pills.push(Box::new(pres_pill));
    }

    // Clock (clickable: toggles interactive monthly Calendar)
    let clock_service = crate::services::clock::ClockService::read();
    let now = clock_service.formatted();
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
    let mut telemetry_w: f32 = 76.0 + 84.0 + 132.0 + 28.0; // Audio (76), Net (84), Clock (132), Power (28)
    if telem.is_anomalous() {
        telemetry_w += anomaly_pill_w;
    }
    if bluetooth.available() {
        telemetry_w += 54.0;
    }
    if notifs_count > 0 {
        telemetry_w += notif_pill_w;
    }
    if is_recording {
        telemetry_w += 108.0;
    }
    if is_presenting {
        telemetry_w += pres_pill_w;
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

/// Determines whether the given monitor is the primary display for presentation director HUD.
/// Laptop eDP displays are always primary; non-eDP external displays are secondary.
/// In single-monitor rehearsal environments, the single active display is treated as primary.
pub fn is_presentation_primary_display(monitor_name: Option<&str>) -> bool {
    let Some(name) = monitor_name else {
        return true;
    };
    if name.starts_with("eDP") {
        return true;
    }

    let workspaces = Workspaces::read();
    let all_outputs: std::collections::HashSet<String> = workspaces
        .list()
        .iter()
        .filter_map(|ws| ws.output().map(|s| s.to_string()))
        .collect();

    // If only one display output is active in the workspace hierarchy, allow presentation HUD
    all_outputs.len() <= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_presentation_primary_display() {
        assert!(is_presentation_primary_display(None));
        assert!(is_presentation_primary_display(Some("eDP-1")));
        assert!(is_presentation_primary_display(Some("eDP-2")));
    }
}
