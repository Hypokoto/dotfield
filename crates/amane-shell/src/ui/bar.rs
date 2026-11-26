use amane::{
    Align, Audio, Battery, Center, Color, Horizontal, Layer, LayerWindow, Margin, Network,
    Notifications, OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Text,
    Vertical, Weight, Widget, Workspaces, Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::ui::tokens::{ThemeTokens, radius, typography};

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

pub fn bar_layer_properties(manifest: &ShellManifest) -> (f32, Zone, bool, Vertical) {
    let raw_h = manifest.bar.height as f32;
    let height = if raw_h < 40.0 { 40.0 } else { raw_h };
    let anchor = bar_anchor_vertical(manifest);
    (height, Zone::Reserve, true, anchor)
}

pub fn bar_anchor_vertical(manifest: &ShellManifest) -> Vertical {
    if manifest.bar.position == "top" {
        Vertical::Top
    } else {
        Vertical::Bottom
    }
}

pub fn bar_edge_margins(manifest: &ShellManifest, gap: f32) -> (i32, i32) {
    let (bar_h, _, _, anchor) = bar_layer_properties(manifest);
    let clearance = (bar_h + 8.0 + gap).max(0.0).round() as i32;
    match anchor {
        Vertical::Top => (clearance, 0),
        Vertical::Bottom => (0, clearance),
        _ => (0, 0),
    }
}

pub const TOP_ISLAND_HEIGHT: f32 = 40.0;

pub fn top_island_monitor(monitor: &amane::Monitor) -> LayerWindow {
    build_top_island(Some(&monitor.name))
}

fn build_top_island(monitor_name: Option<&str>) -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let tokens = ThemeTokens::from_manifest(&manifest);
    let (_, _, is_visible, _) = bar_layer_properties(&manifest);

    // Auto-hide when a fullscreen window is active on this output
    // Reading NiriWindowService subscribes this LayerWindow to changes so it auto-toggles
    let niri_win = crate::services::niri::NiriWindowService::read();
    let is_fullscreen = niri_win.is_fullscreen;
    let visible = is_visible && !is_fullscreen;
    let zone = if is_fullscreen {
        Zone::Ignore
    } else {
        Zone::Reserve
    };

    let mut items: Vec<Box<dyn Widget>> = Vec::new();
    let mut content_w = 0.0_f32;

    if manifest.bar.show_workspaces {
        let (ws_widget, ws_w) = render_workspaces(&manifest, &tokens, monitor_name);
        items.push(Box::new(ws_widget));
        content_w += ws_w;
    }

    // Compact Clock + Date trigger (opens Calendar)
    let clock = crate::services::clock::ClockService::read();
    let clock_label = format!("{}  {}", clock.time(), clock.date());
    let clock_w = 136.0;
    let clock_pill = Rectangle::new()
        .width(clock_w)
        .height(28.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::calendar::toggle();
        })
        .child(
            Text::new(clock_label)
                .size(typography::SECONDARY)
                .weight(Weight::SemiBold)
                .color(tokens.accent)
                .font(&tokens.font_mono)
                .tight(),
        );
    items.push(Box::new(clock_pill));
    content_w += clock_w;

    // Quick Settings (Wi-Fi, Volume, Battery -> opens Control Center)
    let qs_pill = render_quick_settings_pill(&tokens, &manifest.theme.font_family);
    items.push(Box::new(qs_pill));
    content_w += 90.0;

    // Notification counter / bell pill
    let notifs_count = Notifications::read().list().len();
    let notif_w = if notifs_count > 0 { 44.0 } else { 28.0 };
    let notif_pill = Rectangle::new()
        .width(notif_w)
        .height(28.0)
        .radius(radius::PILL)
        .fill(if notifs_count > 0 {
            tokens.danger
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if notifs_count > 0 {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::control_center::open_view(
                crate::ui::control_center::ActiveView::Notifications,
            );
        })
        .child(if notifs_count > 0 {
            Text::new(format!("󰂚 {notifs_count}"))
                .size(typography::CAPTION)
                .color(tokens.text_on_danger)
                .font(&tokens.font_mono)
                .tight()
        } else {
            Text::new("󰂚")
                .size(13.0)
                .color(tokens.text_secondary)
                .font(&tokens.font_mono)
                .tight()
        });
    items.push(Box::new(notif_pill));
    content_w += notif_w;

    // Width: sum of all pill widths + inter-pill gaps (8px each) + 24px island horizontal padding
    let gap_total = (items.len().saturating_sub(1) as f32) * 8.0;
    let island_w = content_w + gap_total + 24.0;

    let island_row = Row::new(items)
        .gap(8.0)
        .align(Align::Center)
        .width(Parent)
        .height(Parent);

    let island_frame = Rectangle::new()
        .width(island_w)
        .height(TOP_ISLAND_HEIGHT)
        .radius(radius::PILL)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 12.0,
            right: 12.0,
        })
        .align_child(Align::Center, Align::Center)
        .child(island_row);

    LayerWindow::new()
        .width(island_w)
        .height(TOP_ISLAND_HEIGHT)
        .target_output(monitor_name.map_or(OutputTarget::Active, |n| {
            OutputTarget::Named(Box::leak(n.to_string().into_boxed_str()))
        }))
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 6,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Top)
        .space(zone)
        .visible(visible)
        .namespace("dotfield-shell-top-island")
        .child(island_frame)
}

fn build_top_bar(monitor_name: Option<&str>) -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let tokens = ThemeTokens::from_manifest(&manifest);
    let (height, bar_zone, is_visible, _) = bar_layer_properties(&manifest);

    // Auto-hide dock island when a fullscreen window is active
    // Reading NiriWindowService subscribes this LayerWindow to changes so it auto-toggles
    let niri_win = crate::services::niri::NiriWindowService::read();
    let is_fullscreen = niri_win.is_fullscreen;
    let visible = is_visible && !is_fullscreen;
    let effective_zone = if is_fullscreen {
        Zone::Ignore
    } else {
        bar_zone
    };

    let (dock_widget, dock_width) = crate::ui::dock::render_dock(&manifest, &tokens);
    let dock_island_w = (dock_width + 24.0).max(160.0);

    let dock_frame = Rectangle::new()
        .width(dock_island_w)
        .height(height)
        .radius(radius::PILL)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 12.0,
            right: 12.0,
        })
        .align_child(Align::Center, Align::Center)
        .child(Row::new(vec![dock_widget]).align(Align::Center));

    LayerWindow::new()
        .width(dock_island_w)
        .height(height)
        .target_output(monitor_name.map_or(OutputTarget::Active, |n| {
            OutputTarget::Named(Box::leak(n.to_string().into_boxed_str()))
        }))
        .anchor_vertical(Vertical::Bottom)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 8,
            left: 0,
        })
        .layer(Layer::Top)
        .space(effective_zone)
        .visible(visible)
        .namespace("dotfield-shell-bar")
        .child(dock_frame)
}

fn render_workspaces(
    _manifest: &ShellManifest,
    tokens: &ThemeTokens,
    monitor_name: Option<&str>,
) -> (Rectangle, f32) {
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
    let gap = 3.0;

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

        let pill_width = 28.0;

        let fill = if is_focused {
            tokens.accent
        } else if is_urgent {
            tokens.danger
        } else if is_occupied {
            tokens.surface_track
        } else {
            Color::TRANSPARENT
        };

        let tile_border = if is_focused {
            tokens.border_focus
        } else if is_urgent {
            tokens.border_danger
        } else {
            Color::TRANSPARENT
        };

        let text_color = if is_focused {
            tokens.text_on_accent
        } else if is_urgent {
            tokens.text_on_danger
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
            .height(28.0)
            .radius(radius::PILL)
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
        let pill_width = 28.0;
        let target_name = CANONICAL_WORKSPACES[0].to_string();
        let tile = Rectangle::new()
            .width(pill_width)
            .height(28.0)
            .radius(radius::PILL)
            .fill(tokens.accent)
            .border(1.0, tokens.border_focus)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                Workspaces::focus_name(target_name.clone());
            })
            .child(
                Text::new("1")
                    .size(typography::BODY)
                    .color(tokens.text_on_accent)
                    .font(&tokens.font_mono),
            );

        buttons.push(Box::new(tile));
        total_width += pill_width;
    }

    if buttons.len() > 1 {
        total_width += (buttons.len() - 1) as f32 * gap;
    }

    let capsule_width = total_width + 8.0;
    let capsule = Rectangle::new()
        .width(capsule_width)
        .height(34.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 4.0,
            right: 4.0,
        })
        .align_child(Center, Center)
        .child(Row::new(buttons).gap(gap).align(Align::Center));

    (capsule, capsule_width)
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
        tokens.success
    } else if battery.percent() <= 10 {
        tokens.danger
    } else if battery.percent() <= 20 {
        tokens.warning
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
        .radius(radius::PILL)
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
        let (height, zone, visible, anchor) = bar_layer_properties(&manifest);
        assert!(visible);
        assert_eq!(zone, Zone::Reserve);
        assert_eq!(height, 48.0);
        assert_eq!(anchor, Vertical::Bottom);
    }
}
