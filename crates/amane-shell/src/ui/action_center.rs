//! Right Slide-In Action Center & Calendar Hub.
//! Windows 11 / Material You style right slide-in flyout.
//! Provides notification history, 3x2 quick toggles, interactive volume/brightness sliders, and compact calendar.

use std::time::{Duration, Instant};

use amane::{
    Align, Audio, Center, Color, Column, Full, Horizontal, Key, Keyboard, Layer, LayerWindow,
    Margin, Notifications, OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text,
    Vertical, Widget, Zone,
};
use chrono::{Datelike, Local};

use crate::config::ConfigService;
use crate::services::presentation::PresentationService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub struct ActionCenterService {
    pub open: bool,
    pub open_time: Option<Instant>,
}

impl Service for ActionCenterService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut center = ActionCenterService::write();
        center.open = !center.open;
        center.open_time = if center.open {
            Some(Instant::now())
        } else {
            None
        };
        center.open
    };

    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::ActionCenter,
        );
    }
}

pub fn close() {
    let mut center = ActionCenterService::write();
    center.open = false;
    center.open_time = None;
}

pub fn action_center_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let center = ActionCenterService::read();

    if !center.open {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .layer(Layer::Overlay)
            .target_output(OutputTarget::Active)
            .namespace("dotfield-shell-action-center");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = &tokens.font_family;

    let dock_height = manifest.bar.height as f32;
    let is_bottom_bar = manifest.bar.position == "bottom";

    // 1. Notification Hub Card
    let notifications = Notifications::read();
    let notif_list = notifications.list();
    let notif_count = notif_list.len();

    let notif_header = Row::new(vec![
        Box::new(
            Text::new(format!("Notifications ({})", notif_count))
                .size(typography::TITLE_SM)
                .color(tokens.text_primary)
                .font(font),
        ),
        Box::new(
            Rectangle::new()
                .width(64.0)
                .height(24.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .padding(Padding {
                    top: 2.0,
                    bottom: 2.0,
                    left: 6.0,
                    right: 6.0,
                })
                .cursor(Pointer)
                .on_click(|_| {
                    let notifs = Notifications::read();
                    let ids: Vec<u32> = notifs.list().iter().map(|n| n.id()).collect();
                    drop(notifs);
                    for id in ids {
                        Notifications::dismiss(id);
                    }
                })
                .child(
                    Text::new("Clear all")
                        .size(typography::CAPTION)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
        ),
    ])
    .width(Parent)
    .align(Align::Center);

    let notif_content: Box<dyn Widget> = if notif_count == 0 {
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(64.0)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(Text::new("󰂚").size(16.0).color(tokens.steel).font(font)),
                        Box::new(
                            Text::new("No new notifications")
                                .size(typography::BODY - 1.0)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
                ),
        )
    } else {
        let mut items: Vec<Box<dyn Widget>> = Vec::new();
        for notif in notif_list.iter().take(2) {
            let app_name = notif.app_name();
            let summary = notif.summary();

            let item = Rectangle::new()
                .width(Parent)
                .height(40.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .padding(Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 8.0,
                    right: 8.0,
                })
                .child(
                    Column::new(vec![
                        Box::new(
                            Text::new(app_name)
                                .size(typography::CAPTION)
                                .color(tokens.accent)
                                .font(font),
                        ),
                        Box::new(
                            Text::new(summary)
                                .size(typography::BODY - 1.0)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                    ])
                    .gap(2.0),
                );
            items.push(Box::new(item));
        }

        Box::new(Column::new(items).gap(spacing::GAP_XS).width(Parent))
    };

    let notif_card = Rectangle::new()
        .width(Parent)
        .height(130.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(Column::new(vec![Box::new(notif_header), notif_content]).gap(spacing::GAP_SM));

    // 2. Quick Settings 3x2 Grid
    let is_presenting = PresentationService::global().is_active();
    let audio = Audio::read();
    let is_muted = audio.muted();
    let vol_pct = audio.volume() as u32;

    let quick_toggles = Column::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(render_toggle_tile(
                    "󰖩",
                    "WiFi",
                    true,
                    tokens.accent,
                    font,
                    &tokens,
                    || {
                        let _ = std::process::Command::new("amane-msg")
                            .arg("network")
                            .spawn();
                    },
                )),
                Box::new(render_toggle_tile(
                    "󰂯",
                    "Bluetooth",
                    true,
                    tokens.accent,
                    font,
                    &tokens,
                    || {
                        let _ = std::process::Command::new("amane-msg")
                            .arg("bluetooth")
                            .spawn();
                    },
                )),
                Box::new(render_toggle_tile(
                    "󰐩",
                    "Present",
                    is_presenting,
                    tokens.accent_hover,
                    font,
                    &tokens,
                    || {
                        PresentationService::global().toggle();
                    },
                )),
            ])
            .gap(spacing::GAP_SM)
            .width(Parent),
        ),
        Box::new(
            Row::new(vec![
                Box::new(render_toggle_tile(
                    if is_muted { "󰝟" } else { "󰕾" },
                    "Mute",
                    is_muted,
                    tokens.danger,
                    font,
                    &tokens,
                    || {
                        Audio::toggle_mute();
                    },
                )),
                Box::new(render_toggle_tile(
                    "󰍹",
                    "Display",
                    false,
                    tokens.steel,
                    font,
                    &tokens,
                    || {
                        let _ = std::process::Command::new("amane-msg")
                            .arg("display-config")
                            .spawn();
                    },
                )),
                Box::new(render_toggle_tile(
                    "󰕮",
                    "Record",
                    false,
                    tokens.steel,
                    font,
                    &tokens,
                    || {
                        let _ = std::process::Command::new("amane-msg")
                            .arg("screen-recorder")
                            .spawn();
                    },
                )),
            ])
            .gap(spacing::GAP_SM)
            .width(Parent),
        ),
    ])
    .gap(spacing::GAP_SM)
    .width(Parent);

    // 3. Volume Slider
    let vol_slider = Row::new(vec![
        Box::new(Text::new("󰕾").size(14.0).color(tokens.accent).font(font)),
        Box::new(
            Rectangle::new()
                .width(220.0)
                .height(8.0)
                .radius(radius::PILL)
                .fill(tokens.surface_elevated)
                .child(
                    Rectangle::new()
                        .width(220.0 * (vol_pct as f32 / 100.0).clamp(0.0, 1.0))
                        .height(8.0)
                        .radius(radius::PILL)
                        .fill(tokens.accent),
                ),
        ),
        Box::new(
            Text::new(format!("{}%", vol_pct))
                .size(typography::CAPTION)
                .color(tokens.text_primary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_MD)
    .align(Align::Center);

    let sliders_card = Rectangle::new()
        .width(Parent)
        .height(170.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(vec![
                Box::new(quick_toggles),
                Box::new(
                    Rectangle::new()
                        .width(Parent)
                        .height(1.0)
                        .fill(tokens.border_subtle),
                ),
                Box::new(vol_slider),
            ])
            .gap(spacing::GAP_MD),
        );

    // 4. Compact Calendar Card
    let now = Local::now();
    let month_str = now.format("%B %Y").to_string();
    let today_day = now.day();

    let calendar_card = Rectangle::new()
        .width(Parent)
        .height(170.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰸗 Calendar")
                                .size(typography::TITLE_SM)
                                .color(tokens.accent)
                                .font(font),
                        ),
                        Box::new(
                            Text::new(&month_str)
                                .size(typography::BODY)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                    ])
                    .width(Parent)
                    .align(Align::Center),
                ),
                Box::new(render_calendar_grid(today_day, font, &tokens)),
            ])
            .gap(spacing::GAP_SM),
        );

    // Footer
    let close_btn = Rectangle::new()
        .width(Parent)
        .height(36.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            close();
        })
        .child(
            Text::new("Close Action Center (Esc)")
                .size(typography::CAPTION)
                .color(tokens.text_secondary)
                .font(font),
        );

    let main_column = Column::new(vec![
        Box::new(notif_card),
        Box::new(sliders_card),
        Box::new(calendar_card),
        Box::new(close_btn),
    ])
    .gap(spacing::GAP_MD)
    .width(Parent);

    let container = Rectangle::new()
        .width(380.0)
        .height(Parent)
        .fill(tokens.bg)
        .border(1.0, tokens.border_subtle)
        .radius(radius::LG)
        .padding(Padding {
            top: spacing::PAD_LG,
            bottom: spacing::PAD_LG,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(main_column);

    let top_margin = 12;
    let bottom_margin = if is_bottom_bar {
        (dock_height + 12.0) as i32
    } else {
        12
    };

    LayerWindow::new()
        .width(380.0)
        .height(Full)
        .anchor_horizontal(Horizontal::Right)
        .anchor_vertical(Vertical::Top)
        .margin(Margin {
            top: top_margin,
            right: 12,
            bottom: bottom_margin,
            left: 0,
        })
        .layer(Layer::Overlay)
        .target_output(OutputTarget::Active)
        .space(Zone::Ignore)
        .keyboard(Keyboard::OnDemand)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .visible(true)
        .namespace("dotfield-shell-action-center")
        .child(container)
}

fn render_toggle_tile<F>(
    icon: &'static str,
    label: &'static str,
    active: bool,
    active_color: Color,
    font: &str,
    tokens: &ThemeTokens,
    on_click: F,
) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    Rectangle::new()
        .width(104.0)
        .height(52.0)
        .radius(radius::SM)
        .fill(if active {
            tokens.surface_elevated
        } else {
            tokens.surface
        })
        .border(
            1.0,
            if active {
                active_color
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(
            Column::new(vec![
                Box::new(
                    Text::new(icon)
                        .size(16.0)
                        .color(if active {
                            active_color
                        } else {
                            tokens.text_secondary
                        })
                        .font(font),
                ),
                Box::new(
                    Text::new(label)
                        .size(typography::CAPTION - 0.5)
                        .color(if active {
                            tokens.text_primary
                        } else {
                            tokens.text_secondary
                        })
                        .font(font),
                ),
            ])
            .align(Center)
            .gap(2.0),
        )
}

fn render_calendar_grid(today: u32, font: &str, tokens: &ThemeTokens) -> Column {
    let day_headers = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let header_widgets: Vec<Box<dyn Widget>> = day_headers
        .iter()
        .map(|&d| {
            Box::new(
                Rectangle::new()
                    .width(44.0)
                    .height(20.0)
                    .align_child(Center, Center)
                    .child(
                        Text::new(d)
                            .size(typography::CAPTION)
                            .color(tokens.text_secondary)
                            .font(font),
                    ),
            ) as Box<dyn Widget>
        })
        .collect();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();
    rows.push(Box::new(
        Row::new(header_widgets).width(Parent).align(Align::Center),
    ));

    // Render 4 weeks for compact calendar
    for week in 0..4 {
        let mut week_days: Vec<Box<dyn Widget>> = Vec::new();
        for d in 1..=7 {
            let day_num = week * 7 + d;
            let is_today = day_num == today;

            let day_tile = Rectangle::new()
                .width(44.0)
                .height(24.0)
                .radius(radius::SM)
                .fill(if is_today {
                    tokens.accent
                } else {
                    Color::TRANSPARENT
                })
                .align_child(Center, Center)
                .child(
                    Text::new(format!("{}", day_num))
                        .size(typography::CAPTION)
                        .color(if is_today {
                            tokens.bg
                        } else {
                            tokens.text_primary
                        })
                        .font(font),
                );
            week_days.push(Box::new(day_tile));
        }
        rows.push(Box::new(
            Row::new(week_days).width(Parent).align(Align::Center),
        ));
    }

    Column::new(rows).gap(2.0).width(Parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_center_toggle() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        assert!(!ActionCenterService::read().open);
        toggle();
        assert!(ActionCenterService::read().open);
        close();
        assert!(!ActionCenterService::read().open);
    }
}
