use amane::{
    Align, Center, Column, Justify, Link, Network, Padding, Parent, Pointer, Rectangle, Row,
    Service, Text, Widget,
};

use crate::config::ShellManifest;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

/// Renders the NetworkManager interactive widget.
pub fn render_network_widget(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);

    let network = Network::read();
    let is_wifi_enabled = network.wifi_enabled();
    let is_connected = network.connected();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Controls Bar: Radio toggle, Scan button, Disconnect button
    let wifi_toggle_btn = Rectangle::new()
        .width(102.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(if is_wifi_enabled {
            tokens.accent_tint
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_wifi_enabled {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            Network::set_wifi(!is_wifi_enabled);
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(if is_wifi_enabled { "󰤨" } else { "󰤭" })
                        .size(13.0)
                        .color(if is_wifi_enabled {
                            tokens.accent
                        } else {
                            tokens.text_primary
                        })
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new(if is_wifi_enabled {
                        "Wi-Fi ON"
                    } else {
                        "Wi-Fi OFF"
                    })
                    .size(typography::SECONDARY)
                    .color(if is_wifi_enabled {
                        tokens.accent
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        );

    let scan_btn = Rectangle::new()
        .width(80.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Network::scan();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰑐")
                        .size(13.0)
                        .color(tokens.accent)
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Scan")
                        .size(typography::SECONDARY)
                        .color(tokens.accent)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        );

    let disconnect_btn = Rectangle::new()
        .width(108.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(if is_connected {
            tokens.danger_subtle
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_connected {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Network::disconnect();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰅙")
                        .size(13.0)
                        .color(if is_connected {
                            tokens.danger
                        } else {
                            tokens.text_muted
                        })
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Disconnect")
                        .size(typography::SECONDARY)
                        .color(if is_connected {
                            tokens.danger
                        } else {
                            tokens.text_muted
                        })
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
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
            top: 6.0,
            bottom: 6.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(status_icon)
                        .size(typography::DISPLAY)
                        .color(if is_connected {
                            tokens.accent
                        } else {
                            tokens.danger
                        })
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new(status_title)
                                .size(typography::BODY)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(status_sub)
                                .size(typography::BADGE)
                                .color(tokens.text_secondary)
                                .font(&tokens.font_mono),
                        ),
                    ])
                    .gap(2.0),
                ),
            ])
            .gap(spacing::GAP_MD)
            .align(Align::Center),
        );

    elements.push(Box::new(active_card));

    // Available access points list
    let ap_header = Row::new(vec![
        Box::new(
            Text::new("Available Access Points")
                .size(typography::BADGE)
                .color(tokens.accent)
                .font(&tokens.font_sans),
        ),
        Box::new(
            Text::new(format!("{} visible", network.access_points().len()))
                .size(typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_mono),
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
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .align_child(Center, Center)
                .child(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰤭")
                                .size(typography::TITLE_SM)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new("Wi-Fi is disabled • Toggle above to scan")
                                .size(typography::BADGE)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
                ),
        ));
    } else if network.access_points().is_empty() {
        ap_cards.push(Box::new(
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
                            Text::new("󰑐")
                                .size(typography::TITLE_SM)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new("No networks found • Click Scan to search")
                                .size(typography::BADGE)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
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
                .radius(radius::SM)
                .fill(if is_active {
                    tokens.accent_tint
                } else {
                    tokens.surface_elevated
                })
                .border(
                    1.0,
                    if is_active {
                        tokens.border_focus
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
                                        .size(typography::BODY_LG)
                                        .color(if is_active {
                                            tokens.accent
                                        } else {
                                            tokens.text_primary
                                        })
                                        .font(&tokens.font_mono),
                                ),
                                Box::new(
                                    Text::new(display_ssid)
                                        .size(typography::SECONDARY)
                                        .color(if is_active {
                                            tokens.accent
                                        } else {
                                            tokens.text_primary
                                        })
                                        .font(&tokens.font_sans),
                                ),
                            ])
                            .gap(spacing::GAP_SM)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Text::new(lock_badge)
                                        .size(typography::SECONDARY)
                                        .color(tokens.text_secondary)
                                        .font(&tokens.font_mono),
                                ),
                                Box::new(
                                    Text::new(format!("{strength}%"))
                                        .size(typography::BADGE)
                                        .color(if is_active {
                                            tokens.accent
                                        } else {
                                            tokens.text_secondary
                                        })
                                        .font(&tokens.font_mono),
                                ),
                                Box::new(
                                    Text::new(if is_active { "󰄬" } else { "󰁔" })
                                        .size(typography::SECONDARY)
                                        .color(if is_active {
                                            tokens.accent
                                        } else {
                                            tokens.text_secondary
                                        })
                                        .font(&tokens.font_mono),
                                ),
                            ])
                            .gap(spacing::GAP_SM)
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

    elements.push(Box::new(
        Column::new(ap_cards).width(Parent).gap(spacing::GAP_XS),
    ));

    Box::new(Column::new(elements).width(Parent).gap(spacing::GAP_SM))
}
