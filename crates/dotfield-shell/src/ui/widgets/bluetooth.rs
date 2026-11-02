use amane::{
    Align, Bluetooth, Center, Color, Column, Justify, Padding, Parent, Pointer, Rectangle, Row,
    Service, Text, Widget,
};

use crate::config::ShellManifest;

/// Renders the BlueZ Bluetooth interactive widget.
pub fn render_bluetooth_widget(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let surface_color = Color::from(manifest.palette.surface.as_str());
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let hover_color = Color::from(manifest.palette.hover.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());
    let steel_color = Color::from(manifest.palette.steel.as_str());

    let bluetooth = Bluetooth::read();
    let is_available = bluetooth.available();
    let is_powered = bluetooth.powered();
    let is_scanning = bluetooth.scanning();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Controls Bar: Power toggle, Discovery/Scan button
    let power_btn = Rectangle::new()
        .width(110.0)
        .height(30.0)
        .radius(6.0)
        .fill(if is_powered {
            amber_color
        } else {
            overlay_color
        })
        .border(
            1.0,
            if is_powered {
                hover_color
            } else {
                border_color
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
            .size(11.0)
            .color(if is_powered { Color::BLACK } else { fg_color })
            .font(font),
        );

    let scan_btn = Rectangle::new()
        .width(110.0)
        .height(30.0)
        .radius(6.0)
        .fill(if is_scanning {
            amber_color
        } else {
            overlay_color
        })
        .border(1.0, border_color)
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
            .size(11.0)
            .color(if is_scanning {
                Color::BLACK
            } else {
                amber_color
            })
            .font(font),
        );

    let status_summary = Rectangle::new()
        .width(110.0)
        .height(30.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .align_child(Center, Center)
        .child(
            Text::new(if !is_available {
                "No Adapter"
            } else if !is_powered {
                "Disabled"
            } else {
                "Ready"
            })
            .size(11.0)
            .color(steel_color)
            .font(font),
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

    // 2. Devices Section Header
    let connected_count = bluetooth.devices().iter().filter(|d| d.connected()).count();
    let devices_header = Row::new(vec![
        Box::new(
            Text::new("BLUETOOTH DEVICES")
                .size(10.0)
                .color(amber_color)
                .font(font),
        ),
        Box::new(
            Text::new(format!(
                "{} connected • {} total",
                connected_count,
                bluetooth.devices().len()
            ))
            .size(10.0)
            .color(steel_color)
            .font(font),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(devices_header));

    // 3. Devices List
    let mut dev_cards: Vec<Box<dyn Widget>> = Vec::new();

    if !is_available {
        dev_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(44.0)
                .radius(6.0)
                .fill(surface_color)
                .border(1.0, border_color)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(Text::new("󰂲").size(14.0).color(crimson_color).font(font)),
                        Box::new(
                            Text::new("No BlueZ adapter detected")
                                .size(11.0)
                                .color(crimson_color)
                                .font(font),
                        ),
                    ])
                    .gap(6.0)
                    .align(Align::Center),
                ),
        ));
    } else if !is_powered {
        dev_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(44.0)
                .radius(6.0)
                .fill(surface_color)
                .border(1.0, border_color)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(Text::new("󰂲").size(14.0).color(steel_color).font(font)),
                        Box::new(
                            Text::new("Bluetooth is disabled • Turn on above")
                                .size(10.5)
                                .color(steel_color)
                                .font(font),
                        ),
                    ])
                    .gap(6.0)
                    .align(Align::Center),
                ),
        ));
    } else if bluetooth.devices().is_empty() {
        dev_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(44.0)
                .radius(6.0)
                .fill(surface_color)
                .border(1.0, border_color)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(Text::new("󰑐").size(14.0).color(steel_color).font(font)),
                        Box::new(
                            Text::new("No devices found • Click Scan to search")
                                .size(10.5)
                                .color(steel_color)
                                .font(font),
                        ),
                    ])
                    .gap(6.0)
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
                .radius(4.0)
                .fill(if is_connected {
                    crimson_color
                } else if is_paired {
                    amber_color
                } else {
                    surface_color
                })
                .border(1.0, border_color)
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
                    .size(10.0)
                    .color(if is_connected {
                        Color::WHITE
                    } else if is_paired {
                        Color::BLACK
                    } else {
                        fg_color
                    })
                    .font(font),
                );

            let row = Rectangle::new()
                .width(Parent)
                .height(36.0)
                .radius(6.0)
                .fill(if is_connected {
                    Color::from("#2A2415")
                } else {
                    overlay_color
                })
                .border(
                    1.0,
                    if is_connected {
                        hover_color
                    } else {
                        border_color
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
                                        .size(14.0)
                                        .color(if is_connected {
                                            amber_color
                                        } else {
                                            steel_color
                                        })
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(dev_name)
                                        .size(11.0)
                                        .color(if is_connected { amber_color } else { fg_color })
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(if let Some(bat) = device.battery() {
                                        format!("󰂄 {bat}%")
                                    } else {
                                        String::new()
                                    })
                                    .size(9.0)
                                    .color(hover_color)
                                    .font(font),
                                ),
                            ])
                            .gap(6.0)
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

    elements.push(Box::new(Column::new(dev_cards).width(Parent).gap(5.0)));

    Box::new(Column::new(elements).width(Parent).gap(8.0))
}
