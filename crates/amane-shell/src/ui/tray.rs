use amane::{
    Align, Center, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Widget,
};

use crate::config::ShellManifest;
use crate::services::tray::TrayService;
use crate::ui::tokens::{ThemeTokens, radius};

/// Renders the compact Status Tray pill for the top bar.
pub fn render_tray_pill(
    manifest: &ShellManifest,
    font: &str,
    _monitor_name: Option<&str>,
) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);

    let tray = TrayService::read();
    let mut items: Vec<Box<dyn Widget>> = Vec::new();

    // External SNI App Items (Discord, Steam, Telegram, OBS, etc.)
    for (idx, item) in tray.items.iter().enumerate().take(6) {
        let glyph = item.glyph();
        let app_pill = Rectangle::new()
            .width(24.0)
            .height(24.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |btn| {
                if btn == amane::Button::Right {
                    TrayService::context_menu(idx);
                } else {
                    TrayService::activate(idx);
                }
            })
            .child(
                Text::new(glyph)
                    .size(14.0)
                    .color(tokens.accent)
                    .font(font)
                    .tight(),
            );

        items.push(Box::new(app_pill));
    }

    // Fallback: If no external SNI apps are registered, show a subtle chevron up icon
    if items.is_empty() {
        items.push(Box::new(
            Text::new("󰅃")
                .size(16.0)
                .color(tokens.text_muted)
                .font(font)
                .tight(),
        ));
    }

    let pill_w = if tray.items.is_empty() {
        32.0
    } else {
        (items.len() as f32 * 26.0) + 12.0
    };

    let mut tray_container = Rectangle::new()
        .width(pill_w)
        .height(34.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 4.0,
            right: 4.0,
        })
        .align_child(Center, Center);

    if tray.items.is_empty() {
        tray_container = tray_container.cursor(Pointer).on_click(move |_| {
            crate::ui::control_center::open_view(crate::ui::control_center::ActiveView::Tray);
        });
    }

    let tray_container = tray_container.child(Row::new(items).gap(4.0).align(Align::Center));

    Box::new(tray_container)
}

/// Renders the full Status Notifier Item (SNI) management drawer in Control Center.
pub fn render_tray_drawer(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);
    let tray = TrayService::read();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // D-Bus Watcher status banner
    let watcher_banner = Rectangle::new()
        .width(Parent)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Row::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰘳")
                                .size(12.0)
                                .color(tokens.accent)
                                .font(font)
                                .tight(),
                        ),
                        Box::new(
                            Text::new("StatusNotifierWatcher")
                                .size(10.0)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                    ])
                    .gap(6.0)
                    .align(Align::Center),
                ),
                Box::new(
                    Text::new(if tray.watcher_active {
                        "● Listening"
                    } else {
                        "○ Offline"
                    })
                    .size(10.0)
                    .color(if tray.watcher_active {
                        tokens.success
                    } else {
                        tokens.danger
                    })
                    .font(font),
                ),
            ])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
        );

    elements.push(Box::new(watcher_banner));

    // Registered app items section
    let items_header = Row::new(vec![
        Box::new(
            Text::new("Status Notifier Applications")
                .size(10.0)
                .color(tokens.accent)
                .font(font),
        ),
        Box::new(
            Text::new(format!("{} active", tray.items.len()))
                .size(10.0)
                .color(tokens.text_muted)
                .font(font),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(items_header));

    let mut app_rows: Vec<Box<dyn Widget>> = Vec::new();

    if tray.items.is_empty() {
        app_rows.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(84.0)
                .radius(radius::MD)
                .fill(tokens.surface)
                .border(1.0, tokens.border_subtle)
                .padding(Padding {
                    top: 12.0,
                    bottom: 12.0,
                    left: 14.0,
                    right: 14.0,
                })
                .align_child(Center, Center)
                .child(
                    Column::new(vec![
                        Box::new(
                            Text::new("󰀻")
                                .size(22.0)
                                .color(tokens.text_muted)
                                .font(font)
                                .tight(),
                        ),
                        Box::new(
                            Text::new("No Background Apps")
                                .size(11.0)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new("SNI status items appear here automatically")
                                .size(9.5)
                                .color(tokens.text_muted)
                                .font(font),
                        ),
                    ])
                    .gap(4.0)
                    .align(Align::Center),
                ),
        ));
    } else {
        for (idx, item) in tray.items.iter().enumerate() {
            let glyph = item.glyph();
            let title = if item.title.is_empty() {
                item.id.clone()
            } else {
                item.title.clone()
            };
            let status = if item.status.is_empty() {
                "Active".to_string()
            } else {
                item.status.clone()
            };

            let row = Rectangle::new()
                .width(Parent)
                .height(36.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .padding(Padding {
                    top: 2.0,
                    bottom: 2.0,
                    left: 8.0,
                    right: 8.0,
                })
                .child(
                    Row::new(vec![
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Text::new(glyph)
                                        .size(14.0)
                                        .color(tokens.accent)
                                        .font(font)
                                        .tight(),
                                ),
                                Box::new(
                                    Text::new(title)
                                        .size(11.0)
                                        .color(tokens.text_primary)
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(format!("• {status}"))
                                        .size(9.0)
                                        .color(tokens.text_muted)
                                        .font(font),
                                ),
                            ])
                            .gap(6.0)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Rectangle::new()
                                        .width(62.0)
                                        .height(24.0)
                                        .radius(radius::SM)
                                        .fill(tokens.surface)
                                        .border(1.0, tokens.border_subtle)
                                        .cursor(Pointer)
                                        .align_child(Center, Center)
                                        .on_click(move |_| {
                                            TrayService::activate(idx);
                                        })
                                        .child(
                                            Text::new("Open")
                                                .size(10.0)
                                                .color(tokens.text_primary)
                                                .font(font),
                                        ),
                                ),
                                Box::new(
                                    Rectangle::new()
                                        .width(52.0)
                                        .height(24.0)
                                        .radius(radius::SM)
                                        .fill(tokens.surface)
                                        .border(1.0, tokens.border_subtle)
                                        .cursor(Pointer)
                                        .align_child(Center, Center)
                                        .on_click(move |_| {
                                            TrayService::context_menu(idx);
                                        })
                                        .child(
                                            Text::new("Menu")
                                                .size(10.0)
                                                .color(tokens.text_muted)
                                                .font(font),
                                        ),
                                ),
                                Box::new(
                                    Rectangle::new()
                                        .width(52.0)
                                        .height(24.0)
                                        .radius(radius::SM)
                                        .fill(tokens.surface)
                                        .border(1.0, tokens.danger)
                                        .cursor(Pointer)
                                        .align_child(Center, Center)
                                        .on_click(move |_| {
                                            TrayService::quit(idx);
                                        })
                                        .child(
                                            Text::new("Quit")
                                                .size(10.0)
                                                .color(tokens.danger)
                                                .font(font),
                                        ),
                                ),
                            ])
                            .gap(4.0)
                            .align(Align::Center),
                        ),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                );

            app_rows.push(Box::new(row));
        }
    }

    elements.push(Box::new(Column::new(app_rows).width(Parent).gap(5.0)));

    Box::new(Column::new(elements).width(Parent).gap(8.0))
}
