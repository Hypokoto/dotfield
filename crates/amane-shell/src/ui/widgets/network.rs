use amane::{
    Align, Center, Color, Column, Justify, Link, Network, Padding, Parent, Pointer, Rectangle, Row,
    Service, Text, Widget,
};

use crate::config::ShellManifest;

/// Renders the NetworkManager interactive widget.
pub fn render_network_widget(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let surface_color = Color::from(manifest.palette.surface.as_str());
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let hover_color = Color::from(manifest.palette.hover.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());
    let steel_color = Color::from(manifest.palette.steel.as_str());

    let network = Network::read();
    let is_wifi_enabled = network.wifi_enabled();
    let is_connected = network.connected();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Controls Bar: Radio toggle, Scan button, Disconnect button
    let wifi_toggle_btn = Rectangle::new()
        .width(96.0)
        .height(30.0)
        .radius(6.0)
        .fill(if is_wifi_enabled {
            amber_color
        } else {
            overlay_color
        })
        .border(
            1.0,
            if is_wifi_enabled {
                hover_color
            } else {
                border_color
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            Network::set_wifi(!is_wifi_enabled);
        })
        .child(
            Text::new(if is_wifi_enabled {
                "󰤨 Wi-Fi ON"
            } else {
                "󰤭 Wi-Fi OFF"
            })
            .size(11.0)
            .color(if is_wifi_enabled {
                Color::BLACK
            } else {
                fg_color
            })
            .font(font),
        );

    let scan_btn = Rectangle::new()
        .width(80.0)
        .height(30.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Network::scan();
        })
        .child(Text::new("󰑐 Scan").size(11.0).color(amber_color).font(font));

    let disconnect_btn = Rectangle::new()
        .width(96.0)
        .height(30.0)
        .radius(6.0)
        .fill(if is_connected {
            crimson_color
        } else {
            surface_color
        })
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Network::disconnect();
        })
        .child(
            Text::new("󰅙 Disconnect")
                .size(11.0)
                .color(if is_connected {
                    Color::WHITE
                } else {
                    steel_color
                })
                .font(font),
        );

    elements.push(Box::new(
        Row::new(vec![
            Box::new(wifi_toggle_btn),
            Box::new(scan_btn),
            Box::new(disconnect_btn),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    ));

    // Active connection status card
    let (status_icon, status_title, status_sub) = match network.link() {
        Link::Wired => (
            "󰈀",
            "Ethernet Wired".to_string(),
            "Connected at 1 Gbps".to_string(),
        ),
        Link::Wifi => {
            let ssid_name = if network.ssid().is_empty() {
                "Wi-Fi Network".to_string()
            } else {
                network.ssid().to_string()
            };
            (
                "󰤨",
                ssid_name,
                format!("Signal {}% • Connected", network.strength()),
            )
        }
        Link::Other => ("󰈀", "Alternative Link".to_string(), "Connected".to_string()),
        Link::Offline => {
            if network.connecting() {
                (
                    "󰑐",
                    "Connecting...".to_string(),
                    "Negotiating link credentials".to_string(),
                )
            } else {
                (
                    "󰤭",
                    "Disconnected".to_string(),
                    "No active network connection".to_string(),
                )
            }
        }
    };

    let active_card = Rectangle::new()
        .width(Parent)
        .height(52.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(
            1.0,
            if is_connected {
                hover_color
            } else {
                border_color
            },
        )
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(status_icon)
                        .size(20.0)
                        .color(if is_connected {
                            amber_color
                        } else {
                            crimson_color
                        })
                        .font(font),
                ),
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new(status_title)
                                .size(12.0)
                                .color(fg_color)
                                .font(font),
                        ),
                        Box::new(
                            Text::new(status_sub)
                                .size(10.0)
                                .color(steel_color)
                                .font(font),
                        ),
                    ])
                    .gap(2.0),
                ),
            ])
            .gap(10.0)
            .align(Align::Center),
        );

    elements.push(Box::new(active_card));

    // Available access points list
    let ap_header = Row::new(vec![
        Box::new(
            Text::new("Available Access Points")
                .size(10.0)
                .color(amber_color)
                .font(font),
        ),
        Box::new(
            Text::new(format!("{} visible", network.access_points().len()))
                .size(10.0)
                .color(steel_color)
                .font(font),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(ap_header));

    let mut ap_cards: Vec<Box<dyn Widget>> = Vec::new();

    if !is_wifi_enabled {
        ap_cards.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(44.0)
                .radius(6.0)
                .fill(surface_color)
                .border(1.0, border_color)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(Text::new("󰤭").size(14.0).color(steel_color).font(font)),
                        Box::new(
                            Text::new("Wi-Fi is disabled • Toggle above to scan")
                                .size(10.5)
                                .color(steel_color)
                                .font(font),
                        ),
                    ])
                    .gap(6.0)
                    .align(Align::Center),
                ),
        ));
    } else if network.access_points().is_empty() {
        ap_cards.push(Box::new(
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
                            Text::new("No networks found • Click Scan to search")
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
        // Sort access points: active first, then saved, then by signal strength descending
        let mut sorted_aps: Vec<_> = network.access_points().to_vec();
        sorted_aps.sort_by(|a, b| {
            b.active()
                .cmp(&a.active())
                .then_with(|| b.saved().cmp(&a.saved()))
                .then_with(|| b.strength().cmp(&a.strength()))
        });

        for ap in sorted_aps.into_iter().take(6) {
            let ssid = ap.ssid().to_string();
            let display_ssid = if ssid.is_empty() {
                "<Hidden SSID>".to_string()
            } else if ssid.chars().count() > 22 {
                format!("{}...", ssid.chars().take(20).collect::<String>())
            } else {
                ssid.clone()
            };

            let strength = ap.strength();
            let wifi_glyph = if strength >= 75 {
                "󰤨"
            } else if strength >= 50 {
                "󰤥"
            } else if strength >= 25 {
                "󰤢"
            } else {
                "󰤟"
            };

            let lock_badge = if ap.secured() { "󰌾" } else { "󰌿" };
            let is_active = ap.active();

            let click_ssid = ssid.clone();
            let ap_row = Rectangle::new()
                .width(Parent)
                .height(34.0)
                .radius(6.0)
                .fill(if is_active {
                    Color::from("#2A2415") // Subtle amber tint
                } else {
                    overlay_color
                })
                .border(1.0, if is_active { hover_color } else { border_color })
                .cursor(Pointer)
                .padding(Padding {
                    top: 2.0,
                    bottom: 2.0,
                    left: 8.0,
                    right: 8.0,
                })
                .on_click(move |_| {
                    if !click_ssid.is_empty() {
                        Network::connect(&click_ssid, None);
                    }
                })
                .child(
                    Row::new(vec![
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Text::new(wifi_glyph)
                                        .size(13.0)
                                        .color(if is_active { amber_color } else { fg_color })
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(display_ssid)
                                        .size(11.0)
                                        .color(if is_active { amber_color } else { fg_color })
                                        .font(font),
                                ),
                            ])
                            .gap(8.0)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Text::new(lock_badge)
                                        .size(11.0)
                                        .color(steel_color)
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(format!("{strength}%"))
                                        .size(10.0)
                                        .color(if is_active { amber_color } else { steel_color })
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(if is_active { "󰄬" } else { "󰁔" })
                                        .size(11.0)
                                        .color(if is_active { amber_color } else { steel_color })
                                        .font(font),
                                ),
                            ])
                            .gap(6.0)
                            .align(Align::Center),
                        ),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                );

            ap_cards.push(Box::new(ap_row));
        }
    }

    elements.push(Box::new(Column::new(ap_cards).width(Parent).gap(5.0)));

    Box::new(Column::new(elements).width(Parent).gap(8.0))
}
