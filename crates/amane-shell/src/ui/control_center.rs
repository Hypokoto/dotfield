use std::time::Duration;

use amane::{
    Align, Audio, Battery, Bluetooth, Brightness, Center, Color, Column, Cpu, Horizontal, Justify,
    Key, Keyboard, Layer, LayerWindow, Margin, Media, Memory, Network, Notifications, OutputTarget,
    Padding, Parent, Pointer, Rectangle, Row, Scroll, Service, Stack, Text, Vertical, Widget, Zone,
};

use crate::config::ConfigService;
use crate::services::niri::NiriService;
use crate::ui::components;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveView {
    #[default]
    Main,
    Network,
    Audio,
    Bluetooth,
    Tray,
}

pub struct ControlCenterService {
    pub open: bool,
    pub dnd: bool,
    pub view: ActiveView,
}

impl Service for ControlCenterService {
    fn new() -> Self {
        Self {
            open: false,
            dnd: false,
            view: ActiveView::Main,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let mut cc = ControlCenterService::write();
    cc.open = !cc.open;
    if !cc.open {
        cc.view = ActiveView::Main;
    }
}

pub fn open_view(view: ActiveView) {
    let mut cc = ControlCenterService::write();
    cc.open = true;
    cc.view = view;
}

pub fn close() {
    let mut cc = ControlCenterService::write();
    cc.open = false;
    cc.view = ActiveView::Main;
}

pub fn control_center_panel() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let cc = ControlCenterService::read();

    let is_visible = cc.open && manifest.control_center.enabled;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-control-center");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let bg_color = tokens.surface;
    let font = manifest.theme.font_family.clone();

    let content: Column = match cc.view {
        ActiveView::Main => {
            let mut sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_header(&tokens)),
                Box::new(render_system_overview(&tokens, &font)),
                Box::new(render_quick_toggles(&tokens, cc.dnd, &font)),
                Box::new(render_widget_shortcuts(&tokens, &font)),
                Box::new(render_sliders(&tokens, &font)),
                Box::new(render_appearance(&tokens, &font)),
            ];

            if manifest.control_center.show_mpris {
                sections.push(Box::new(render_media(&tokens, &font)));
            }

            sections.push(Box::new(render_notifications_drawer(&tokens, &font)));
            sections.push(Box::new(render_power_actions(&tokens, &font)));

            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
        ActiveView::Network => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header("NetworkManager", "󰤨", &tokens, &font)),
                crate::ui::widgets::network::render_network_widget(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
        ActiveView::Audio => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header("PipeWire Audio", "󰕾", &tokens, &font)),
                crate::ui::widgets::audio::render_audio_widget(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
        ActiveView::Bluetooth => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "BlueZ Bluetooth",
                    "󰂯",
                    &tokens,
                    &font,
                )),
                crate::ui::widgets::bluetooth::render_bluetooth_widget(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
        ActiveView::Tray => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "System Tray & Apps",
                    "󰀻",
                    &tokens,
                    &font,
                )),
                crate::ui::tray::render_tray_drawer(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
    };

    let panel_height = match cc.view {
        ActiveView::Main => 660.0,
        ActiveView::Network => 480.0,
        ActiveView::Audio => 380.0,
        ActiveView::Bluetooth => 420.0,
        ActiveView::Tray => 340.0,
    };

    let panel = Rectangle::new()
        .width(380.0)
        .height(panel_height)
        .radius(radius::LG)
        .fill(bg_color)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(content);

    LayerWindow::new()
        .width(380.0)
        .height(panel_height)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 38,
            right: 12,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Top)
        .keyboard(if is_visible {
            Keyboard::OnDemand
        } else {
            Keyboard::None
        })
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .space(Zone::Ignore)
        .visible(is_visible)
        .namespace("dotfield-shell-control-center")
        .child(panel)
}

fn render_header(tokens: &ThemeTokens) -> Rectangle {
    components::modal_header("󰒓", "Control Center", Some("Niri+Amane"), tokens, close)
}

fn render_system_overview(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let cpu = Cpu::read().percent();
    let mem = Memory::read();
    let mem_pct = mem.percent();
    let used_gib = mem.used_kib() as f64 / (1024.0 * 1024.0);
    let total_gib = mem.total_kib() as f64 / (1024.0 * 1024.0);

    let battery = Battery::read();

    let mut metrics: Vec<Box<dyn Widget>> = Vec::new();

    // CPU Card
    let cpu_color = if cpu > 80 {
        tokens.danger
    } else if cpu > 50 {
        tokens.border_focus
    } else {
        tokens.accent
    };

    let cpu_card = Rectangle::new()
        .width(112.0)
        .height(48.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("󰍛 CPU")
                        .size(typography::BADGE)
                        .color(cpu_color)
                        .font(font),
                ),
                Box::new(
                    Text::new(format!("{cpu}%"))
                        .size(typography::TITLE_SM)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ])
            .gap(2.0),
        );
    metrics.push(Box::new(cpu_card));

    // RAM Card
    let ram_color = if mem_pct > 85 {
        tokens.danger
    } else if mem_pct > 70 {
        tokens.border_focus
    } else {
        tokens.accent
    };

    let ram_card = Rectangle::new()
        .width(112.0)
        .height(48.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("󰘚 RAM")
                        .size(typography::BADGE)
                        .color(ram_color)
                        .font(font),
                ),
                Box::new(
                    Text::new(format!("{mem_pct}% ({used_gib:.1}/{total_gib:.0}G)"))
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ])
            .gap(2.0),
        );
    metrics.push(Box::new(ram_card));

    // Battery / Power Card
    let bat_text = if battery.present() {
        let icon = if battery.charging() { "󰂄" } else { "󰁹" };
        format!("{icon} {}%", battery.percent())
    } else {
        "󰚥 AC Online".to_string()
    };
    let bat_card = Rectangle::new()
        .width(112.0)
        .height(48.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("POWER")
                        .size(typography::BADGE)
                        .color(tokens.accent)
                        .font(font),
                ),
                Box::new(
                    Text::new(bat_text)
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ])
            .gap(2.0),
        );
    metrics.push(Box::new(bat_card));

    Rectangle::new().width(Parent).height(48.0).child(
        Row::new(metrics)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
    )
}

fn render_quick_toggles(tokens: &ThemeTokens, dnd: bool, font: &str) -> Rectangle {
    let network = Network::read();
    let bluetooth = Bluetooth::read();

    let mut toggles: Vec<Box<dyn Widget>> = Vec::new();

    // Wi-Fi Toggle
    let wifi_active = network.connected();
    let wifi_btn = Rectangle::new()
        .width(110.0)
        .height(38.0)
        .radius(radius::SM)
        .fill(if wifi_active {
            tokens.accent
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if wifi_active {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let net = Network::read();
            Network::set_wifi(!net.wifi_enabled());
        })
        .child(
            Text::new(if wifi_active {
                "󰤨 Wi-Fi"
            } else {
                "󰤭 Wi-Fi"
            })
            .size(typography::SECONDARY)
            .color(if wifi_active {
                Color::BLACK
            } else {
                tokens.text_primary
            })
            .font(font),
        );
    toggles.push(Box::new(wifi_btn));

    // Bluetooth Toggle
    let bt_active = bluetooth.powered();
    let bt_btn = Rectangle::new()
        .width(110.0)
        .height(38.0)
        .radius(radius::SM)
        .fill(if bt_active {
            tokens.accent
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if bt_active {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| {
            Bluetooth::set_powered(!bt_active);
        })
        .child(
            Text::new(if bt_active {
                "󰂯 BT On"
            } else {
                "󰂲 BT Off"
            })
            .size(typography::SECONDARY)
            .color(if bt_active {
                Color::BLACK
            } else {
                tokens.text_primary
            })
            .font(font),
        );
    toggles.push(Box::new(bt_btn));

    // Do Not Disturb (DND)
    let dnd_btn = Rectangle::new()
        .width(110.0)
        .height(38.0)
        .radius(radius::SM)
        .fill(if dnd {
            tokens.danger
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if dnd {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let mut cc = ControlCenterService::write();
            cc.dnd = !cc.dnd;
        })
        .child(
            Text::new(if dnd { "󰂛 DND" } else { "󰂚 DND" })
                .size(typography::SECONDARY)
                .color(if dnd {
                    Color::WHITE
                } else {
                    tokens.text_primary
                })
                .font(font),
        );
    toggles.push(Box::new(dnd_btn));

    Rectangle::new().width(Parent).height(38.0).child(
        Row::new(toggles)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
    )
}

fn render_widget_shortcuts(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let make_shortcut = |icon: &'static str, label: &'static str, view: ActiveView| {
        Rectangle::new()
            .width(82.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                open_view(view);
            })
            .child(
                Row::new(vec![
                    Box::new(
                        Text::new(icon)
                            .size(typography::BODY)
                            .color(tokens.accent)
                            .font(font),
                    ),
                    Box::new(
                        Text::new(label)
                            .size(typography::CAPTION)
                            .color(tokens.text_secondary)
                            .font(font),
                    ),
                ])
                .gap(3.0)
                .align(Align::Center),
            )
    };

    Rectangle::new().width(Parent).height(30.0).child(
        Row::new(vec![
            Box::new(make_shortcut("󰤨", "Net ❯", ActiveView::Network)),
            Box::new(make_shortcut("󰕾", "Audio ❯", ActiveView::Audio)),
            Box::new(make_shortcut("󰂯", "BlueZ ❯", ActiveView::Bluetooth)),
            Box::new(make_shortcut("󰀻", "Tray ❯", ActiveView::Tray)),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    )
}

fn render_subview_header(title: &str, icon: &str, tokens: &ThemeTokens, font: &str) -> Rectangle {
    let back_btn = Rectangle::new()
        .width(82.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let mut cc = ControlCenterService::write();
            cc.view = ActiveView::Main;
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("❮").size(11.0).color(tokens.accent).font(font)),
                Box::new(
                    Text::new("Overview")
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        );

    let title_block = Row::new(vec![
        Box::new(
            Text::new(icon)
                .size(typography::BODY)
                .color(tokens.accent)
                .font(font),
        ),
        Box::new(
            Text::new(title)
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let close_btn = components::close_button(close, tokens);

    Rectangle::new().width(Parent).height(28.0).child(
        Row::new(vec![
            Box::new(back_btn),
            Box::new(title_block),
            Box::new(close_btn),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    )
}

fn render_sliders(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let audio = Audio::read();
    let vol = audio.volume();
    let is_muted = audio.muted();
    let brightness = Brightness::read();
    let bright = brightness.percent();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // Volume Slider Row with Mute Button and Stacked Visual Track
    let vol_slider_w = 280.0;
    let vol_fill_w = (vol_slider_w * (vol as f32 / 100.0)).clamp(0.0, vol_slider_w);
    let vol_fill_color = if is_muted {
        tokens.danger_subtle
    } else {
        tokens.accent_subtle
    };

    let vol_bg_rect = Rectangle::new()
        .width(vol_slider_w)
        .height(34.0)
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle);

    let vol_fill_rect = Rectangle::new()
        .width(vol_fill_w)
        .height(34.0)
        .radius(radius::SM)
        .fill(vol_fill_color);

    let vol_label_row = Row::new(vec![
        Box::new(
            Text::new(if is_muted { "󰖁" } else { "󰕾" })
                .size(typography::BODY_LG)
                .color(if is_muted {
                    tokens.danger
                } else {
                    tokens.accent
                })
                .font(font),
        ),
        Box::new(
            Text::new(format!("Volume {vol}%"))
                .size(typography::SECONDARY)
                .color(if is_muted {
                    tokens.text_secondary
                } else {
                    tokens.text_primary
                })
                .font(font),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let vol_content = Rectangle::new()
        .width(vol_slider_w)
        .height(34.0)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(vol_label_row);

    let vol_stack = Stack::new(vec![
        Box::new(vol_bg_rect),
        Box::new(vol_fill_rect),
        Box::new(vol_content),
    ]);

    let vol_slider = Rectangle::new()
        .width(vol_slider_w)
        .height(34.0)
        .cursor(Pointer)
        .on_scroll(|scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            let current = Audio::read().volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_volume(target);
        })
        .child(vol_stack);

    let mute_btn = Rectangle::new()
        .width(60.0)
        .height(34.0)
        .radius(radius::SM)
        .fill(if is_muted {
            tokens.danger
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_muted {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Audio::toggle_mute();
        })
        .child(
            Text::new(if is_muted { "Muted" } else { "Mute" })
                .size(typography::BADGE)
                .color(if is_muted {
                    Color::WHITE
                } else {
                    tokens.text_primary
                })
                .font(font),
        );

    let vol_row = Row::new(vec![Box::new(vol_slider), Box::new(mute_btn)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);
    rows.push(Box::new(vol_row));

    // Brightness Slider Row with Stacked Visual Track
    let bright_slider_w = 352.0;
    let bright_fill_w = (bright_slider_w * (bright as f32 / 100.0)).clamp(0.0, bright_slider_w);
    let bright_fill_color = tokens.accent_subtle;

    let bright_bg_rect = Rectangle::new()
        .width(bright_slider_w)
        .height(34.0)
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle);

    let bright_fill_rect = Rectangle::new()
        .width(bright_fill_w)
        .height(34.0)
        .radius(radius::SM)
        .fill(bright_fill_color);

    let bright_label_row = Row::new(vec![
        Box::new(
            Text::new("󰃠")
                .size(typography::BODY_LG)
                .color(tokens.accent)
                .font(font),
        ),
        Box::new(
            Text::new(format!("Brightness {bright}%"))
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let bright_content = Rectangle::new()
        .width(bright_slider_w)
        .height(34.0)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(bright_label_row);

    let bright_stack = Stack::new(vec![
        Box::new(bright_bg_rect),
        Box::new(bright_fill_rect),
        Box::new(bright_content),
    ]);

    let bright_row = Rectangle::new()
        .width(bright_slider_w)
        .height(34.0)
        .cursor(Pointer)
        .on_scroll(|scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            let current = Brightness::read().percent() as i32;
            let target = (current + step).clamp(1, 100) as u8;
            Brightness::set(target);
        })
        .child(bright_stack);

    rows.push(Box::new(bright_row));

    Rectangle::new()
        .width(Parent)
        .height(76.0)
        .child(Column::new(rows).gap(spacing::GAP_SM))
}

fn render_appearance(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let theme_badge = Row::new(vec![
        Box::new(
            Text::new("󰔎")
                .size(typography::BODY_LG)
                .color(tokens.accent)
                .font(font),
        ),
        Box::new(
            Text::new("Controlled Chaos Theme")
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let wallpaper_btn = Rectangle::new()
        .width(110.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.accent)
        .border(1.0, tokens.accent_hover)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("sh")
                .args(["-c", "~/.config/dotfield/scripts/wallpaper.sh"])
                .spawn();
        })
        .child(
            Text::new("󰸉 Next Wall")
                .size(typography::BADGE)
                .color(Color::BLACK)
                .font(font),
        );

    let row = Row::new(vec![Box::new(theme_badge), Box::new(wallpaper_btn)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(34.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(row)
}

fn render_media(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let media = Media::read();
    let playing = media.playing();

    let title_text = if media.title().is_empty() {
        "No media playing".to_string()
    } else {
        format!("{} - {}", media.artist(), media.title())
    };

    let play_pause_icon = if playing { "󰏤" } else { "󰐊" };

    Rectangle::new()
        .width(Parent)
        .height(64.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Text::new(title_text)
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Rectangle::new()
                                .width(34.0)
                                .height(24.0)
                                .radius(radius::SM)
                                .fill(tokens.surface_track)
                                .border(1.0, tokens.border_subtle)
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| Media::previous())
                                .child(
                                    Text::new("󰒮")
                                        .size(typography::SECONDARY)
                                        .color(tokens.text_primary)
                                        .font(font),
                                ),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(34.0)
                                .height(24.0)
                                .radius(radius::SM)
                                .fill(tokens.accent)
                                .border(1.0, tokens.accent_hover)
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| Media::play_pause())
                                .child(
                                    Text::new(play_pause_icon)
                                        .size(typography::BODY_LG)
                                        .color(Color::BLACK)
                                        .font(font),
                                ),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(34.0)
                                .height(24.0)
                                .radius(radius::SM)
                                .fill(tokens.surface_track)
                                .border(1.0, tokens.border_subtle)
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| Media::next())
                                .child(
                                    Text::new("󰒭")
                                        .size(typography::SECONDARY)
                                        .color(tokens.text_primary)
                                        .font(font),
                                ),
                        ),
                    ])
                    .gap(spacing::GAP_MD)
                    .align(Align::Center),
                ),
            ])
            .gap(spacing::GAP_XS),
        )
}

fn render_notifications_drawer(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let notifications = Notifications::read();
    let notifs_count = notifications.list().len();

    let header_row = Row::new(vec![
        Box::new(
            Text::new(format!("Notifications ({notifs_count})"))
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(font),
        ),
        Box::new(
            Rectangle::new()
                .width(56.0)
                .height(20.0)
                .radius(radius::SM)
                .fill(tokens.danger)
                .border(1.0, tokens.border_danger)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| Notifications::clear())
                .child(
                    Text::new("Clear")
                        .size(typography::BADGE)
                        .color(Color::WHITE)
                        .font(font),
                ),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(80.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 8.0,
            bottom: 8.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Column::new(vec![
                Box::new(header_row),
                Box::new(
                    Text::new(if notifs_count == 0 {
                        "No new notifications"
                    } else {
                        "Recent notifications shown in overlay"
                    })
                    .size(typography::BADGE)
                    .color(tokens.text_muted)
                    .font(font),
                ),
            ])
            .gap(spacing::GAP_SM),
        )
}

fn render_power_actions(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let mut btns: Vec<Box<dyn Widget>> = Vec::new();

    // Lock
    let lock_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("sh")
                .args(["-c", "~/.config/dotfield/scripts/lock.sh"])
                .spawn();
            close();
        })
        .child(
            Text::new("󰌾 Lock")
                .size(typography::BADGE)
                .color(tokens.text_primary)
                .font(font),
        );
    btns.push(Box::new(lock_btn));

    // Suspend
    let suspend_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("systemctl")
                .arg("suspend")
                .spawn();
            close();
        })
        .child(
            Text::new("󰒲 Sleep")
                .size(typography::BADGE)
                .color(tokens.text_primary)
                .font(font),
        );
    btns.push(Box::new(suspend_btn));

    // Reboot
    let reboot_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("systemctl")
                .arg("reboot")
                .spawn();
            close();
        })
        .child(
            Text::new("󰜉 Reboot")
                .size(typography::BADGE)
                .color(tokens.text_primary)
                .font(font),
        );
    btns.push(Box::new(reboot_btn));

    // Poweroff
    let poweroff_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.danger)
        .border(1.0, tokens.border_danger)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("systemctl")
                .arg("poweroff")
                .spawn();
            close();
        })
        .child(
            Text::new("󰐥 Power")
                .size(typography::BADGE)
                .color(Color::WHITE)
                .font(font),
        );
    btns.push(Box::new(poweroff_btn));

    // Logout
    let logout_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = NiriService::quit(false);
            close();
        })
        .child(
            Text::new("󰗽 Quit")
                .size(typography::BADGE)
                .color(tokens.text_primary)
                .font(font),
        );
    btns.push(Box::new(logout_btn));

    Rectangle::new().width(Parent).height(30.0).child(
        Row::new(btns)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_center_service_default() {
        let svc = ControlCenterService::new();
        assert!(!svc.open);
        assert!(!svc.dnd);
    }

    #[test]
    fn test_control_center_lifecycle_and_views() {
        close();
        assert!(!ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Main);

        toggle();
        assert!(ControlCenterService::read().open);

        close();
        assert!(!ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Main);

        open_view(ActiveView::Network);
        assert!(ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Network);

        open_view(ActiveView::Audio);
        assert!(ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Audio);

        open_view(ActiveView::Bluetooth);
        assert!(ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Bluetooth);

        open_view(ActiveView::Tray);
        assert!(ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Tray);

        close();
        assert!(!ControlCenterService::read().open);
        assert_eq!(ControlCenterService::read().view, ActiveView::Main);
    }
}
