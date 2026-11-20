use std::time::{Duration, Instant};

use chrono::{Datelike, Local, NaiveDate};

use amane::{
    Align, Audio, Battery, Bluetooth, Brightness, Center, Color, Column, Cpu, Horizontal, Justify,
    Key, Keyboard, Layer, LayerWindow, Margin, Media, Memory, Network, Notifications, OutputTarget,
    Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
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
    Calendar,
    Tray,
}

pub struct ControlCenterService {
    pub open: bool,
    pub open_time: Option<Instant>,
    pub dnd: bool,
    pub view: ActiveView,
}

impl Service for ControlCenterService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
            dnd: false,
            view: ActiveView::Main,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut cc = ControlCenterService::write();
        cc.open = !cc.open;
        cc.open_time = if cc.open { Some(Instant::now()) } else { None };
        if !cc.open {
            cc.view = ActiveView::Main;
        }
        cc.open
    };
    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::ControlCenter,
        );
    }
}

pub fn open_view(view: ActiveView) {
    let should_open = {
        let mut cc = ControlCenterService::write();
        if cc.open && cc.view == view {
            cc.open = false;
            cc.open_time = None;
            cc.view = ActiveView::Main;
            false
        } else {
            if !cc.open {
                cc.open_time = Some(Instant::now());
            }
            cc.open = true;
            cc.view = view;
            true
        }
    };
    if should_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::ControlCenter,
        );
    }
}

pub fn close() {
    let mut cc = ControlCenterService::write();
    cc.open = false;
    cc.open_time = None;
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
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-control-center");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = manifest.theme.font_family.clone();

    let content: Column = match cc.view {
        ActiveView::Main => {
            let mut sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_header(&tokens)),
                Box::new(render_system_overview(&tokens, &font)),
                Box::new(render_quick_toggles(&tokens, cc.dnd, &font)),
                Box::new(render_widget_shortcuts(&tokens, &font)),
                Box::new(render_sliders(&tokens, &font)),
            ];

            if manifest.control_center.show_mpris {
                sections.push(Box::new(render_media(&tokens, &font)));
            }

            sections.push(Box::new(render_appearance(&tokens, &font)));
            sections.push(Box::new(render_notifications_drawer(&tokens, &font)));
            sections.push(Box::new(render_power_actions(&tokens, &font)));

            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
        ActiveView::Calendar => render_calendar_subview(&manifest, &tokens, &font),
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
        ActiveView::Main => 680.0,
        ActiveView::Calendar => 520.0,
        ActiveView::Network => 480.0,
        ActiveView::Audio => 380.0,
        ActiveView::Bluetooth => 420.0,
        ActiveView::Tray => 340.0,
    };

    let panel = Rectangle::new()
        .width(380.0)
        .height(panel_height)
        .radius(radius::XL)
        .fill(tokens.acrylic_bg)
        .border(1.0, tokens.acrylic_border)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(content);

    let transition = crate::ui::animation::ModalTransition::compute(
        cc.open_time,
        crate::ui::animation::DEFAULT_MODAL_ANIMATION_DURATION,
    );
    let panel = transition.apply(panel);

    let (dock_height, _, _) = crate::ui::bar::bar_layer_properties(&manifest);
    let bottom_margin = (dock_height + 10.0) as i32;

    LayerWindow::new()
        .width(380.0)
        .height(panel_height)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Bottom)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 0,
            right: 12,
            bottom: bottom_margin,
            left: 0,
        })
        .layer(Layer::Overlay)
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
    let is_presenting = crate::services::presentation::PresentationService::global().is_active();

    // Row 1: Wi-Fi, Bluetooth, DND
    let wifi_active = network.connected();
    let wifi_btn = Rectangle::new()
        .width(112.0)
        .height(36.0)
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

    let bt_active = bluetooth.powered();
    let bt_btn = Rectangle::new()
        .width(112.0)
        .height(36.0)
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

    let dnd_btn = Rectangle::new()
        .width(112.0)
        .height(36.0)
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

    let row1 = Row::new(vec![
        Box::new(wifi_btn),
        Box::new(bt_btn),
        Box::new(dnd_btn),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    // Row 2: Display, Record, Present
    let disp_btn = Rectangle::new()
        .width(112.0)
        .height(36.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::display_config::toggle();
        })
        .child(
            Text::new("󰍹 Display")
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(font),
        );

    let rec_active = crate::ui::screen_record::ScreenRecordService::read().is_recording;
    let rec_btn = Rectangle::new()
        .width(112.0)
        .height(36.0)
        .radius(radius::SM)
        .fill(if rec_active {
            tokens.danger
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if rec_active {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::screen_record::toggle();
        })
        .child(
            Text::new(if rec_active {
                "󰑋 REC"
            } else {
                "󰑋 Record"
            })
            .size(typography::SECONDARY)
            .color(if rec_active {
                Color::WHITE
            } else {
                tokens.text_primary
            })
            .font(font),
        );

    let present_btn = Rectangle::new()
        .width(112.0)
        .height(36.0)
        .radius(radius::SM)
        .fill(if is_presenting {
            tokens.accent
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_presenting {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = crate::services::presentation::PresentationService::global().toggle();
        })
        .child(
            Text::new(if is_presenting {
                "󰐩 Present"
            } else {
                "󰐩 Normal"
            })
            .size(typography::SECONDARY)
            .color(if is_presenting {
                Color::BLACK
            } else {
                tokens.text_primary
            })
            .font(font),
        );

    let row2 = Row::new(vec![
        Box::new(disp_btn),
        Box::new(rec_btn),
        Box::new(present_btn),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    Rectangle::new().width(Parent).height(80.0).child(
        Column::new(vec![Box::new(row1), Box::new(row2)])
            .width(Parent)
            .gap(8.0),
    )
}

fn render_widget_shortcuts(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let make_shortcut = |icon: &'static str, label: &'static str, view: ActiveView| {
        Rectangle::new()
            .width(66.0)
            .height(28.0)
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
                .gap(2.0)
                .align(Align::Center),
            )
    };

    Rectangle::new().width(Parent).height(28.0).child(
        Row::new(vec![
            Box::new(make_shortcut("󰤨", "Net", ActiveView::Network)),
            Box::new(make_shortcut("󰕾", "Audio", ActiveView::Audio)),
            Box::new(make_shortcut("󰂯", "BlueZ", ActiveView::Bluetooth)),
            Box::new(make_shortcut("󰸗", "Cal", ActiveView::Calendar)),
            Box::new(make_shortcut("󰀻", "Tray", ActiveView::Tray)),
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

    // Volume Slider Row with Mute Button using WinUI 3 fluent_slider
    let vol_slider_w = 280.0;
    let vol_slider = components::fluent_slider(
        vol as f32 / 100.0,
        vol_slider_w,
        34.0,
        if is_muted { "󰖁" } else { "󰕾" },
        &format!("Volume {vol}%"),
        is_muted,
        tokens,
        |step| {
            let current = Audio::read().volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_volume(target);
        },
    );

    let mute_btn = Rectangle::new()
        .width(66.0)
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

    // Brightness Slider Row using WinUI 3 fluent_slider
    let bright_slider_w = 356.0;
    let bright_slider = components::fluent_slider(
        bright as f32 / 100.0,
        bright_slider_w,
        34.0,
        "󰃠",
        &format!("Brightness {bright}%"),
        false,
        tokens,
        |step| {
            let current = Brightness::read().percent() as i32;
            let target = (current + step).clamp(1, 100) as u8;
            Brightness::set(target);
        },
    );
    rows.push(Box::new(bright_slider));

    Rectangle::new()
        .width(Parent)
        .height(76.0)
        .child(Column::new(rows).gap(spacing::GAP_SM))
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

    // CAVA Audio Spectrum Visualizer integrated directly into the media card
    let spectrum = crate::services::cava::CavaService::spectrum();
    let mut bar_widgets: Vec<Box<dyn Widget>> = Vec::new();
    for &val in spectrum.iter() {
        let bar_h = (val * 24.0).clamp(3.0, 24.0);
        let bar = Rectangle::new()
            .width(5.0)
            .height(bar_h)
            .radius(radius::XS)
            .fill(if val > 0.6 {
                tokens.accent_hover
            } else if val > 0.1 {
                tokens.accent
            } else {
                tokens.steel
            });
        bar_widgets.push(Box::new(bar));
    }
    let spectrum_row = Row::new(bar_widgets).gap(2.0).align(Align::End);

    Rectangle::new()
        .width(Parent)
        .height(76.0)
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
                    Row::new(vec![
                        Box::new(
                            Text::new(title_text)
                                .size(typography::SECONDARY)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new(if playing { "● PLAYING" } else { "● IDLE" })
                                .size(typography::CAPTION)
                                .color(if playing {
                                    tokens.success
                                } else {
                                    tokens.steel
                                })
                                .font(font),
                        ),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Rectangle::new()
                                .width(140.0)
                                .height(28.0)
                                .fill(tokens.surface_track)
                                .radius(radius::XS)
                                .padding(Padding {
                                    top: 2.0,
                                    bottom: 2.0,
                                    left: 4.0,
                                    right: 4.0,
                                })
                                .align_child(Center, Align::End)
                                .child(spectrum_row),
                        ),
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Rectangle::new()
                                        .width(32.0)
                                        .height(26.0)
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
                                        .width(32.0)
                                        .height(26.0)
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
                                        .width(32.0)
                                        .height(26.0)
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
                            .gap(spacing::GAP_SM)
                            .align(Align::Center),
                        ),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                ),
            ])
            .gap(spacing::GAP_XS),
        )
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
        .height(56.0)
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
                Box::new(header_row),
                Box::new(
                    Text::new(if notifs_count == 0 {
                        "No new notifications"
                    } else {
                        "Recent notifications active"
                    })
                    .size(typography::BADGE)
                    .color(tokens.text_muted)
                    .font(font),
                ),
            ])
            .gap(spacing::GAP_XS),
        )
}

fn render_power_actions(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let mut btns: Vec<Box<dyn Widget>> = Vec::new();

    // Lock
    let lock_btn = Rectangle::new()
        .width(66.0)
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
        .width(66.0)
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
        .width(66.0)
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
        .width(66.0)
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
        .width(66.0)
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

fn render_calendar_subview(
    _manifest: &crate::config::ShellManifest,
    tokens: &ThemeTokens,
    font: &str,
) -> Column {
    let cal = crate::ui::calendar::CalendarService::read();
    let now = Local::now();
    let today_year = now.year();
    let today_month = now.month();
    let today_day = now.day();

    let time_str = now.format("%H:%M:%S").to_string();
    let full_date_str = now.format("%A, %B %d, %Y").to_string();

    // 1. Subview Header
    let subview_hdr = render_subview_header("Calendar & Events", "󰸗", tokens, font);

    // 2. Hero Digital Clock Banner
    let clock_banner = Rectangle::new()
        .width(Parent)
        .height(56.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_XS,
            bottom: spacing::PAD_XS,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Row::new(vec![Box::new(
                Column::new(vec![
                    Box::new(
                        Text::new(time_str)
                            .size(typography::DISPLAY)
                            .color(tokens.accent)
                            .font(font),
                    ),
                    Box::new(
                        Text::new(full_date_str)
                            .size(typography::BADGE)
                            .color(tokens.text_secondary)
                            .font(font),
                    ),
                ])
                .gap(2.0),
            )])
            .align(Align::Center),
        );

    // 3. Month navigation
    let month_name = match cal.view_month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };
    let month_label = format!("{} {}", month_name, cal.view_year);

    let nav_row = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Rectangle::new()
                        .width(28.0)
                        .height(26.0)
                        .radius(radius::SM)
                        .fill(tokens.surface_elevated)
                        .border(1.0, tokens.border_subtle)
                        .cursor(Pointer)
                        .align_child(Center, Center)
                        .on_click(|_| {
                            crate::ui::calendar::prev_month();
                        })
                        .child(Text::new("❮").size(11.0).color(tokens.accent).font(font)),
                ),
                Box::new(
                    Text::new(month_label)
                        .size(typography::BODY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
                Box::new(
                    Rectangle::new()
                        .width(28.0)
                        .height(26.0)
                        .radius(radius::SM)
                        .fill(tokens.surface_elevated)
                        .border(1.0, tokens.border_subtle)
                        .cursor(Pointer)
                        .align_child(Center, Center)
                        .on_click(|_| {
                            crate::ui::calendar::next_month();
                        })
                        .child(Text::new("❯").size(11.0).color(tokens.accent).font(font)),
                ),
            ])
            .gap(spacing::GAP_MD)
            .align(Align::Center),
        ),
        Box::new(
            Rectangle::new()
                .width(52.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| {
                    crate::ui::calendar::reset_today();
                })
                .child(
                    Text::new("Today")
                        .size(typography::BADGE)
                        .color(tokens.accent)
                        .font(font),
                ),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    // 4. Weekday headers
    let weekday_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let mut weekday_pills: Vec<Box<dyn Widget>> = Vec::new();
    for label in weekday_labels {
        let pill = Rectangle::new()
            .width(44.0)
            .height(20.0)
            .align_child(Center, Center)
            .child(
                Text::new(label)
                    .size(typography::BADGE)
                    .color(tokens.text_muted)
                    .font(font),
            );
        weekday_pills.push(Box::new(pill));
    }
    let weekday_row = Row::new(weekday_pills)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    // 5. Calendar days grid
    let first_day = NaiveDate::from_ymd_opt(cal.view_year, cal.view_month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    let start_col = first_day.weekday().num_days_from_monday() as usize;

    let (next_y, next_m) = if cal.view_month == 12 {
        (cal.view_year + 1, 1)
    } else {
        (cal.view_year, cal.view_month + 1)
    };
    let next_first = NaiveDate::from_ymd_opt(next_y, next_m, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
    let days_in_month = (next_first - first_day).num_days() as u32;

    let (prev_y, prev_m) = if cal.view_month == 1 {
        (cal.view_year - 1, 12)
    } else {
        (cal.view_year, cal.view_month - 1)
    };
    let prev_first = NaiveDate::from_ymd_opt(prev_y, prev_m, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
    let days_in_prev_month = (first_day - prev_first).num_days() as u32;

    let mut grid_cells: Vec<(u32, bool, bool)> = Vec::new();

    for i in (0..start_col).rev() {
        let day_num = days_in_prev_month - i as u32;
        grid_cells.push((day_num, false, false));
    }

    for day in 1..=days_in_month {
        let is_today =
            cal.view_year == today_year && cal.view_month == today_month && day == today_day;
        grid_cells.push((day, true, is_today));
    }

    let total_cells = if grid_cells.len() > 35 { 42 } else { 35 };
    let mut next_day = 1;
    while grid_cells.len() < total_cells {
        grid_cells.push((next_day, false, false));
        next_day += 1;
    }

    let mut week_rows: Vec<Box<dyn Widget>> = Vec::new();
    for week_idx in 0..(total_cells / 7) {
        let mut row_cells: Vec<Box<dyn Widget>> = Vec::new();
        for day_idx in 0..7 {
            let cell_idx = week_idx * 7 + day_idx;
            let (day_num, is_current_month, is_today) = grid_cells[cell_idx];

            let bg_color = if is_today {
                tokens.warning // Controlled Chaos Industrial Amber (#FFB000)
            } else {
                Color::TRANSPARENT
            };

            let text_color = if is_today {
                Color::BLACK
            } else if is_current_month {
                tokens.text_primary
            } else {
                tokens.steel
            };

            let cell = Rectangle::new()
                .width(44.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(bg_color)
                .cursor(Pointer)
                .align_child(Center, Center)
                .child(
                    Text::new(format!("{day_num}"))
                        .size(11.0)
                        .color(text_color)
                        .font(font),
                );

            row_cells.push(Box::new(cell));
        }

        let week_row = Row::new(row_cells)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);
        week_rows.push(Box::new(week_row));
    }

    let cal_card = Rectangle::new()
        .width(Parent)
        .height(250.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_SM,
            bottom: spacing::PAD_SM,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(vec![
                Box::new(nav_row),
                Box::new(weekday_row),
                Box::new(Column::new(week_rows).width(Parent).gap(2.0)),
            ])
            .gap(spacing::GAP_SM),
        );

    // 6. Notifications in Calendar View
    let notifs_drawer = render_notifications_drawer(tokens, font);

    Column::new(vec![
        Box::new(subview_hdr),
        Box::new(clock_banner),
        Box::new(cal_card),
        Box::new(notifs_drawer),
    ])
    .width(Parent)
    .gap(spacing::GAP_MD)
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
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        toggle();
        assert!(ControlCenterService::read().open);
        open_view(ActiveView::Network);
        assert_eq!(ControlCenterService::read().view, ActiveView::Network);
        open_view(ActiveView::Calendar);
        assert_eq!(ControlCenterService::read().view, ActiveView::Calendar);
        close();
        assert!(!ControlCenterService::read().open);
    }
}
