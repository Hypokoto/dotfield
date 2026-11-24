use amane::{
    Align, Audio, Battery, Center, Color, Column, Full, Layer, LayerWindow, Network, Notifications,
    Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Text, Vertical, Widget, Workspaces,
    Zone,
};

use crate::config::{ConfigService, ShellManifest};
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

pub fn bar_layer_properties(manifest: &ShellManifest) -> (f32, Zone, bool) {
    let height = if manifest.bar.height == 34.0
        || manifest.bar.height == 32.0
        || manifest.bar.position == "bottom"
    {
        48.0
    } else {
        manifest.bar.height as f32
    };
    (height, Zone::Reserve, true)
}

fn build_top_bar(monitor_name: Option<&str>) -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let tokens = ThemeTokens::from_manifest(&manifest);
    let (height, bar_zone, is_visible) = bar_layer_properties(&manifest);
    let font = manifest.theme.font_family.clone();

    // 1. Left Cluster: Windows 11 Widgets Board trigger (Telemetry) + Desktop workspace switcher
    let mut left_items: Vec<Box<dyn Widget>> = Vec::new();

    if manifest.bar.show_telemetry {
        left_items.push(Box::new(render_compact_telemetry(&tokens, &font)));
    }

    if manifest.bar.show_workspaces {
        let (ws_widget, _) = render_workspaces(&manifest, &tokens, monitor_name);
        left_items.push(Box::new(ws_widget));
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

    // 2. Center Cluster: Windows 11 style centered dock
    let (dock_widget, dock_width) = crate::ui::dock::render_dock(&manifest, &tokens);
    let center_container = Rectangle::new()
        .width(dock_width)
        .height(Parent)
        .align_child(Center, Center)
        .child(Row::new(vec![dock_widget]));

    // 3. Right Cluster: Telemetry + System Tray + Quick Settings + Clock + Show Desktop
    let right_row = render_right_cluster(&manifest, &tokens, monitor_name, &font);
    let right_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .align_child(Align::End, Align::Center)
        .child(right_row);

    let content_row = Row::new(vec![
        Box::new(left_container),
        Box::new(center_container),
        Box::new(right_container),
    ])
    .width(Parent)
    .height(Parent)
    .align(Align::Center);

    // Smooth Clay Matte Dark Clay background with subtle Clay Lip border outline
    let background_container = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: spacing::PAD_MD,
            right: spacing::PAD_SM,
        })
        .child(content_row);

    LayerWindow::new()
        .width(Full)
        .height(height)
        .anchor_vertical(Vertical::Bottom)
        .layer(Layer::Top)
        .space(bar_zone)
        .visible(is_visible)
        .namespace("dotfield-shell-bar")
        .child(background_container)
}

/// Compact weather/telemetry pill reading persistent /proc pread descriptors
fn render_compact_telemetry(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let telem = TelemetryService::read();
    let cpu = telem.cpu_percent;
    let mem = telem.mem_percent;

    let is_crit = cpu >= 95 || mem >= 90;
    let is_anom = cpu >= 85 || mem >= 80;

    let bg_color = if is_crit {
        tokens.danger
    } else if is_anom {
        tokens.surface_active
    } else {
        tokens.surface_elevated
    };

    let border_color = if is_crit {
        tokens.border_danger
    } else if is_anom {
        tokens.border_focus
    } else {
        tokens.border_subtle
    };

    let glyph_color = if is_crit { Color::WHITE } else { tokens.accent };

    let text_color = if is_crit {
        Color::WHITE
    } else if is_anom {
        tokens.accent
    } else {
        tokens.text_primary
    };

    let mut row_items: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Text::new("󰍛")
                .size(14.5)
                .color(glyph_color)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(format!("{cpu}%"))
                .size(typography::BODY)
                .color(text_color)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new("󰘚")
                .size(14.5)
                .color(glyph_color)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(format!("{mem}%"))
                .size(typography::BODY)
                .color(text_color)
                .font(&tokens.font_mono),
        ),
    ];

    let has_culprit = (is_crit || is_anom || telem.anomaly_active) && telem.culprit_name.is_some();
    let pill_width = if has_culprit {
        if let Some(ref name) = telem.culprit_name {
            let short_name = if name.len() > 8 {
                format!("{:.7}…", name)
            } else {
                name.clone()
            };
            row_items.push(Box::new(
                Text::new(format!("󰓅 {short_name}"))
                    .size(typography::CAPTION)
                    .color(if is_crit { Color::WHITE } else { tokens.accent })
                    .font(&tokens.font_mono),
            ));
        }
        186.0
    } else {
        116.0
    };

    let content_row = Row::new(row_items).gap(6.0).align(Align::Center);

    Rectangle::new()
        .width(pill_width)
        .height(34.0)
        .radius(radius::MD)
        .fill(bg_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("ghostty")
                .arg("-e")
                .arg("btop")
                .spawn();
        })
        .child(content_row)
}

fn render_workspaces(
    _manifest: &ShellManifest,
    tokens: &ThemeTokens,
    monitor_name: Option<&str>,
) -> (Row, f32) {
    let workspaces = Workspaces::read();

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
        .max(2);

    let mut total_width = 0.0;
    let gap = 4.0;

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

        // Display workspace if active/focused, occupied, or urgent
        let should_display = is_focused || is_occupied || is_urgent || num <= 2;
        if !should_display {
            continue;
        }

        let pill_width = 34.0;

        let fill = if is_focused {
            tokens.surface_active
        } else if is_urgent {
            tokens.danger
        } else if is_occupied {
            tokens.surface_elevated
        } else {
            Color::TRANSPARENT
        };

        let tile_border = if is_focused {
            tokens.border_focus
        } else if is_urgent {
            tokens.border_danger
        } else if is_occupied {
            tokens.border_subtle
        } else {
            Color::TRANSPARENT
        };

        let text_color = if is_focused {
            tokens.accent
        } else if is_urgent {
            Color::WHITE
        } else if is_occupied {
            tokens.text_primary
        } else {
            tokens.text_muted
        };

        let target_name = if num <= CANONICAL_WORKSPACES.len() {
            CANONICAL_WORKSPACES[num - 1].to_string()
        } else {
            format!("{num}")
        };

        let tile = Rectangle::new()
            .width(pill_width)
            .height(34.0)
            .radius(radius::MD)
            .fill(fill)
            .border(1.0, tile_border)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                if let Some(id) = ws_id {
                    Workspaces::focus(id);
                } else {
                    Workspaces::focus_name(target_name.clone());
                }
            })
            .child(
                Text::new(format!("{num}"))
                    .size(typography::BODY)
                    .color(text_color)
                    .font(&tokens.font_mono),
            );

        buttons.push(Box::new(tile));
        total_width += pill_width;
    }

    if buttons.is_empty() {
        let pill_width = 34.0;
        let target_name = CANONICAL_WORKSPACES[0].to_string();
        let tile = Rectangle::new()
            .width(pill_width)
            .height(34.0)
            .radius(radius::MD)
            .fill(tokens.surface_active)
            .border(1.0, tokens.border_focus)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                Workspaces::focus_name(target_name.clone());
            })
            .child(
                Text::new("1")
                    .size(typography::BODY)
                    .color(tokens.accent)
                    .font(&tokens.font_mono),
            );

        buttons.push(Box::new(tile));
        total_width += pill_width;
    }

    if buttons.len() > 1 {
        total_width += (buttons.len() - 1) as f32 * gap;
    }

    (Row::new(buttons).gap(gap).align(Align::Center), total_width)
}

fn render_right_cluster(
    manifest: &ShellManifest,
    tokens: &ThemeTokens,
    monitor_name: Option<&str>,
    font: &str,
) -> Row {
    let mut pills: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Presentation Mode Stopwatch (if active on primary display)
    let presentation = PresentationService::global();
    let is_presenting = presentation.is_active() && is_presentation_primary_display(monitor_name);
    if is_presenting {
        let elapsed_str = presentation
            .elapsed_formatted()
            .unwrap_or_else(|| "00:00".to_string());
        let pres_label = format!("󰐩 Presenting {elapsed_str}");
        let pill_w = 144.0;

        let pres_pill = Rectangle::new()
            .width(pill_w)
            .height(34.0)
            .radius(radius::MD)
            .fill(tokens.surface_active)
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
                PresentationService::global().toggle();
            })
            .child(
                Text::new(pres_label)
                    .size(typography::SECONDARY)
                    .color(tokens.accent)
                    .font(&tokens.font_sans),
            );
        pills.push(Box::new(pres_pill));
    }

    // 3. Active Screen Recording Pill
    let is_recording = crate::ui::screen_record::ScreenRecordService::read().is_recording;
    let rec_elapsed = crate::ui::screen_record::ScreenRecordService::read()
        .started_at
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);
    if is_recording {
        let mins = rec_elapsed / 60;
        let secs = rec_elapsed % 60;
        let rec_text = format!("󰑋 Rec {mins:02}:{secs:02}");
        let rec_pill = Rectangle::new()
            .width(108.0)
            .height(34.0)
            .radius(radius::MD)
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
                    .color(tokens.text_primary)
                    .font(&tokens.font_sans),
            );
        pills.push(Box::new(rec_pill));
    }

    // 4. System Tray (StatusNotifierItem applications)
    pills.push(crate::ui::tray::render_tray_pill(
        manifest,
        font,
        monitor_name,
    ));

    // 5. Quick Settings Pill: Unified cluster for Wi-Fi, Volume, Battery
    let qs_pill = render_quick_settings_pill(tokens, font);
    pills.push(Box::new(qs_pill));

    // 6. Stacked Two-Line Clock & Notification Center (HH:MM and DD/MM/YYYY)
    let clock_pill = render_stacked_clock_pill(tokens, font);
    pills.push(Box::new(clock_pill));

    // 7. Far-Right "Show Desktop" / Task View sliver
    let show_desktop_sliver = Rectangle::new()
        .width(6.0)
        .height(34.0)
        .radius(radius::SM)
        .fill(Color::TRANSPARENT)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = crate::services::niri::NiriService::toggle_overview();
        })
        .child(
            Rectangle::new()
                .width(2.0)
                .height(18.0)
                .radius(radius::PILL)
                .fill(tokens.border_subtle),
        );
    pills.push(Box::new(show_desktop_sliver));

    Row::new(pills).gap(spacing::GAP_SM).align(Align::Center)
}

/// Unified Quick Settings pill combining Wi-Fi, Volume, and Battery status
fn render_quick_settings_pill(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let network = Network::read();
    let net_icon = if network.connected() {
        if network.ssid().is_empty() {
            "󰈀"
        } else {
            "󰤨"
        }
    } else {
        "󰤭"
    };

    let audio = Audio::read();
    let vol_icon = if audio.muted() {
        "󰖁"
    } else if audio.volume() > 50 {
        "󰕾"
    } else if audio.volume() > 0 {
        "󰖀"
    } else {
        "󰕿"
    };

    let battery = Battery::read();
    let bat_icon = if battery.charging() {
        "󰂄"
    } else if battery.percent() <= 10 {
        "󰁺"
    } else if battery.percent() <= 20 {
        "󰁻"
    } else {
        "󰁹"
    };

    let net_color = if network.connected() {
        tokens.accent
    } else {
        tokens.text_muted
    };

    let vol_color = if audio.muted() {
        tokens.text_muted
    } else {
        tokens.accent
    };

    let bat_color = if battery.charging() {
        tokens.accent_tint
    } else if battery.percent() <= 10 {
        tokens.danger
    } else if battery.percent() <= 20 {
        tokens.accent_tint
    } else {
        tokens.accent
    };

    let content_row = Row::new(vec![
        Box::new(
            Text::new(net_icon)
                .size(16.0)
                .color(net_color)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(vol_icon)
                .size(16.0)
                .color(vol_color)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(bat_icon)
                .size(16.0)
                .color(bat_color)
                .font(&tokens.font_mono),
        ),
    ])
    .gap(10.0)
    .align(Align::Center);

    Rectangle::new()
        .width(90.0)
        .height(34.0)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
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
            crate::ui::control_center::toggle();
        })
        .on_scroll(|scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            let current = Audio::read().volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_volume(target);
        })
        .child(content_row)
}

/// Two-line stacked clock and notification center pill (HH:MM / DD/MM/YYYY) with Smooth Clay styling
fn render_stacked_clock_pill(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let clock = crate::services::clock::ClockService::read();
    let time_str = clock.time();
    let date_str = clock.date();

    let notifs = Notifications::read();
    let notifs_count = notifs.list().len();

    let clock_column = Column::new(vec![
        Box::new(
            Text::new(time_str)
                .size(typography::BODY_LG)
                .color(tokens.accent)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(date_str)
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_sans),
        ),
    ])
    .align(Center)
    .gap(1.0);

    let mut row_items: Vec<Box<dyn Widget>> = vec![Box::new(clock_column)];

    if notifs_count > 0 {
        let notif_badge = Rectangle::new()
            .width(18.0)
            .height(18.0)
            .radius(radius::PILL)
            .fill(tokens.danger)
            .border(1.0, tokens.border_danger)
            .align_child(Center, Center)
            .child(
                Text::new(format!("{notifs_count}"))
                    .size(typography::CAPTION - 1.0)
                    .color(Color::WHITE)
                    .font(&tokens.font_mono),
            );
        row_items.push(Box::new(notif_badge));
    }

    Rectangle::new()
        .width(94.0)
        .height(34.0)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::calendar::toggle();
        })
        .child(Row::new(row_items).gap(6.0).align(Align::Center))
}

/// Determines whether the given monitor is the primary display for presentation director HUD.
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

    #[test]
    fn test_top_bar_permanent_visibility() {
        let manifest = ShellManifest::default();
        let (height, zone, visible) = bar_layer_properties(&manifest);
        assert!(visible);
        assert_eq!(zone, Zone::Reserve);
        assert_eq!(height, 48.0);
    }
}
