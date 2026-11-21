use amane::{
    Align, Bluetooth, Center, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Service,
    Text, Widget,
};

use crate::config::ShellManifest;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

/// Renders the BlueZ Bluetooth interactive widget.
pub fn render_bluetooth_widget(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);

    let bluetooth = Bluetooth::read();
    let is_available = bluetooth.available();
    let is_powered = bluetooth.powered();
    let is_scanning = bluetooth.scanning();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Controls Bar: Power toggle, Discovery/Scan button
    let power_btn = Rectangle::new()
        .width(110.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(if is_powered {
            tokens.accent_tint
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_powered {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            Bluetooth::set_powered(!is_powered);
        })
        .child(
            Text::new(if is_powered {
                "󰂯 BlueZ ON"
            } else {
                "󰂲 BlueZ OFF"
            })
            .size(typography::SECONDARY)
            .color(if is_powered {
                tokens.accent
            } else {
                tokens.text_primary
            })
            .font(&tokens.font_sans),
        );

    let scan_btn = Rectangle::new()
        .width(110.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(if is_scanning {
            tokens.accent_tint
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_scanning {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            if is_scanning {
                Bluetooth::stop_scan();
            } else {
                Bluetooth::start_scan();
            }
        })
        .child(
            Text::new(if is_scanning {
                "󰑐 Scanning..."
            } else {
                "󰑐 Scan Devices"
            })
            .size(typography::SECONDARY)
            .color(tokens.accent)
            .font(&tokens.font_sans),
        );

    let status_summary = Rectangle::new()
        .width(110.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .align_child(Center, Center)
        .child(
            Text::new(if !is_available {
                "No Adapter"
            } else if !is_powered {
                "Disabled"
            } else {
                "Ready"
            })
            .size(typography::SECONDARY)
            .color(tokens.text_secondary)
            .font(&tokens.font_sans),
        );

    elements.push(Box::new(
        Row::new(vec![
            Box::new(power_btn),
            Box::new(scan_btn),
            Box::new(status_summary),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    ));

    // Devices section header
    let connected_count = bluetooth.devices().iter().filter(|d| d.connected()).count();
    let devices_header = Row::new(vec![
        Box::new(
            Text::new("Bluetooth Devices")
                .size(typography::BADGE)
                .color(tokens.accent)
                .font(&tokens.font_sans),
        ),
        Box::new(
            Text::new(format!(
                "{} connected • {} total",
                connected_count,
                bluetooth.devices().len()
            ))
            .size(typography::BADGE)
            .color(tokens.text_secondary)
            .font(font),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(devices_header));

    // Devices list
    let mut dev_cards: Vec<Box<dyn Widget>> = Vec::new();

    if !is_available {
        dev_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(44.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_danger)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰂲")
                                .size(typography::TITLE_SM)
                                .color(tokens.danger)
                                .font(font),
                        ),
                        Box::new(
                            Text::new("No BlueZ adapter detected")
                                .size(typography::SECONDARY)
                                .color(tokens.danger)
                                .font(font),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
                ),
        ));
    } else if !is_powered {
        dev_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(44.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰂲")
                                .size(typography::TITLE_SM)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new("Bluetooth is disabled • Turn on above")
                                .size(typography::BADGE)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
                ),
        ));
    } else if bluetooth.devices().is_empty() {
        dev_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(108.0)
                .radius(radius::MD)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .align_child(Center, Center)
                .child(
                    Column::new(vec![
                        Box::new(
                            Text::new("󰂯")
                                .size(28.0)
                                .color(tokens.accent)
                                .font(&tokens.font_mono),
                        ),
                        Box::new(
                            Text::new("No Bluetooth Devices Found")
                                .size(typography::BODY)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new("Set your device to pairing mode to discover it")
                                .size(typography::CAPTION)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(140.0)
                                .height(26.0)
                                .radius(radius::SM)
                                .fill(tokens.accent_tint)
                                .border(1.0, tokens.border_focus)
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| {
                                    Bluetooth::start_scan();
                                })
                                .child(
                                    Text::new("󰑐 Scan for Devices")
                                        .size(typography::CAPTION)
                                        .color(tokens.accent)
                                        .font(&tokens.font_sans),
                                ),
                        ),
                    ])
                    .gap(4.0)
                    .align(Align::Center),
                ),
        ));
    } else {
        for device in bluetooth.devices().iter().take(6) {
            let path = device.path().to_string();
            let is_connected = device.connected();
            let is_paired = device.paired();
            let dev_name = if device.name().is_empty() {
                device.address().to_string()
            } else if device.name().chars().count() > 18 {
                format!("{}...", device.name().chars().take(16).collect::<String>())
            } else {
                device.name().to_string()
            };

            let dev_icon = {
                let icon_str = device.icon().to_lowercase();
                if icon_str.contains("headset") || icon_str.contains("audio") {
                    "󰋋"
                } else if icon_str.contains("keyboard") {
                    "󰌌"
                } else if icon_str.contains("mouse") {
                    "󰍽"
                } else {
                    "󰂯"
                }
            };

            let dev_path = path.clone();
            let action_btn = Rectangle::new()
                .width(76.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(if is_connected {
                    tokens.danger_subtle
                } else if is_paired {
                    tokens.accent_tint
                } else {
                    tokens.surface_elevated
                })
                .border(
                    1.0,
                    if is_connected {
                        tokens.border_danger
                    } else if is_paired {
                        tokens.border_focus
                    } else {
                        tokens.border_subtle
                    },
                )
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| {
                    if is_connected {
                        Bluetooth::disconnect(&dev_path);
                    } else if is_paired {
                        Bluetooth::connect(&dev_path);
                    } else {
                        Bluetooth::pair(&dev_path);
                    }
                })
                .child(
                    Text::new(if is_connected {
                        "Disconnect"
                    } else if is_paired {
                        "Connect"
                    } else {
                        "Pair"
                    })
                    .size(typography::BADGE)
                    .color(if is_connected {
                        tokens.danger
                    } else if is_paired {
                        tokens.accent
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
                );

            let row = Rectangle::new()
                .width(Parent)
                .height(36.0)
                .radius(radius::SM)
                .fill(if is_connected {
                    tokens.accent_tint
                } else {
                    tokens.surface_elevated
                })
                .border(
                    1.0,
                    if is_connected {
                        tokens.border_focus
                    } else {
                        tokens.border_subtle
                    },
                )
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
                                    Text::new(dev_icon)
                                        .size(typography::TITLE_SM)
                                        .color(if is_connected {
                                            tokens.accent
                                        } else {
                                            tokens.text_secondary
                                        })
                                        .font(&tokens.font_mono),
                                ),
                                Box::new(
                                    Text::new(dev_name)
                                        .size(typography::SECONDARY)
                                        .color(if is_connected {
                                            tokens.accent
                                        } else {
                                            tokens.text_primary
                                        })
                                        .font(&tokens.font_sans),
                                ),
                                Box::new(
                                    Text::new(if let Some(bat) = device.battery() {
                                        format!("󰂄 {bat}%")
                                    } else {
                                        String::new()
                                    })
                                    .size(typography::CAPTION)
                                    .color(tokens.accent)
                                    .font(&tokens.font_mono),
                                ),
                            ])
                            .gap(spacing::GAP_SM)
                            .align(Align::Center),
                        ),
                        Box::new(action_btn),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                );

            dev_cards.push(Box::new(row));
        }
    }

    elements.push(Box::new(
        Column::new(dev_cards).width(Parent).gap(spacing::GAP_XS),
    ));

    Box::new(Column::new(elements).width(Parent).gap(spacing::GAP_SM))
}
