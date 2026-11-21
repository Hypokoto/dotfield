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
        ActiveView::Main => {
            let media_h = if Media::read().playing() || !Media::read().title().is_empty() {
                76.0
            } else {
                36.0
            };
            24.0 + 28.0 + 48.0 + 142.0 + 76.0 + media_h + 34.0 + 56.0 + 30.0 + 70.0
        }
        ActiveView::Calendar => 480.0,
        ActiveView::Network => adaptive_network_height(),
        ActiveView::Audio => adaptive_audio_height(),
        ActiveView::Bluetooth => adaptive_bluetooth_height(),
        ActiveView::Tray => adaptive_tray_height(),
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

fn render_system_overview(tokens: &ThemeTokens, _font: &str) -> Rectangle {
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
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new(format!("{cpu}%"))
                        .size(typography::TITLE_SM)
                        .color(tokens.text_primary)
                        .font(&tokens.font_mono),
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
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new(format!("{mem_pct}% ({used_gib:.1}/{total_gib:.0}G)"))
                        .size(typography::CAPTION)
                        .color(tokens.text_primary)
                        .font(&tokens.font_mono),
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
                    Text::new("󰚥 Power")
                        .size(typography::BADGE)
                        .color(tokens.accent)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new(bat_text)
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_mono),
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

fn adaptive_network_height() -> f32 {
    let network = Network::read();
    let ap_count = network.access_points().len().min(6);
    if ap_count == 0 {
        270.0
    } else {
        (200.0 + ap_count as f32 * 38.0).clamp(270.0, 480.0)
    }
}

fn adaptive_audio_height() -> f32 {
    let audio = Audio::read();
    let sinks_count = audio.sinks().len().min(4);
    (270.0 + sinks_count as f32 * 38.0).clamp(320.0, 500.0)
}

fn adaptive_bluetooth_height() -> f32 {
    let bluetooth = Bluetooth::read();
    let dev_count = bluetooth.devices().len();
    if dev_count == 0 {
        260.0
    } else {
        let count = dev_count.min(5);
        (190.0 + count as f32 * 42.0).clamp(260.0, 460.0)
    }
}

fn adaptive_tray_height() -> f32 {
    let tray = crate::services::tray::TrayService::read();
    let count = tray.items.len().min(6);
    (170.0 + count as f32 * 42.0).clamp(240.0, 420.0)
}

fn render_split_tile<F1, F2>(
    icon: &'static str,
    label: &str,
    is_active: bool,
    tokens: &ThemeTokens,
    on_toggle: F1,
    on_details: F2,
) -> Rectangle
where
    F1: Fn() + Send + Sync + 'static,
    F2: Fn() + Send + Sync + 'static,
{
    let tile_w = 172.0;
    let tile_h = 42.0;

    let (bg_color, border_color, icon_color, text_color) = if is_active {
        (
            tokens.accent_tint,
            tokens.border_focus,
            tokens.accent,
            tokens.text_primary,
        )
    } else {
        (
            tokens.surface_elevated,
            tokens.border_subtle,
            tokens.text_muted,
            tokens.text_secondary,
        )
    };

    let left_btn = Rectangle::new()
        .width(132.0)
        .height(tile_h)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 4.0,
        })
        .align_child(Align::Start, Align::Center)
        .on_click(move |_| on_toggle())
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(icon)
                        .size(16.0)
                        .color(icon_color)
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new(label)
                        .size(typography::SECONDARY)
                        .color(text_color)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        );

    let divider = Rectangle::new()
        .width(1.0)
        .height(18.0)
        .fill(Color::from("#ffffff14"));

    let right_btn = Rectangle::new()
        .width(38.0)
        .height(tile_h)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_details())
        .child(
            Text::new("󰅂")
                .size(14.0)
                .color(if is_active {
                    tokens.accent
                } else {
                    tokens.text_muted
                })
                .font(&tokens.font_mono),
        );

    Rectangle::new()
        .width(tile_w)
        .height(tile_h)
        .radius(radius::MD)
        .fill(bg_color)
        .border(1.0, border_color)
        .child(
            Row::new(vec![
                Box::new(left_btn),
                Box::new(divider),
                Box::new(right_btn),
            ])
            .align(Align::Center),
        )
}

fn render_action_tile<F>(
    icon: &'static str,
    label: &str,
    is_active: bool,
    is_danger: bool,
    tokens: &ThemeTokens,
    on_click: F,
) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    let tile_w = 172.0;
    let tile_h = 42.0;

    let (bg_color, border_color, icon_color, text_color) = if is_danger && is_active {
        (
            tokens.danger_subtle,
            tokens.border_danger,
            tokens.danger,
            tokens.text_primary,
        )
    } else if is_active {
        (
            tokens.accent_tint,
            tokens.border_focus,
            tokens.accent,
            tokens.text_primary,
        )
    } else {
        (
            tokens.surface_elevated,
            tokens.border_subtle,
            tokens.text_muted,
            tokens.text_secondary,
        )
    };

    Rectangle::new()
        .width(tile_w)
        .height(tile_h)
        .radius(radius::MD)
        .fill(bg_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .on_click(move |_| on_click())
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(icon)
                        .size(16.0)
                        .color(icon_color)
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new(label)
                        .size(typography::SECONDARY)
                        .color(text_color)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        )
}

fn render_quick_toggles(tokens: &ThemeTokens, dnd: bool, _font: &str) -> Rectangle {
    let network = Network::read();
    let bluetooth = Bluetooth::read();
    let is_presenting = crate::services::presentation::PresentationService::global().is_active();
    let rec_active = crate::ui::screen_record::ScreenRecordService::read().is_recording;

    let wifi_active = network.connected();
    let wifi_label = if wifi_active && !network.ssid().is_empty() {
        let ssid = network.ssid();
        if ssid.len() > 10 {
            format!("{}…", &ssid[..9])
        } else {
            ssid.to_string()
        }
    } else if wifi_active {
        "Wi-Fi On".to_string()
    } else {
        "Wi-Fi Off".to_string()
    };

    let wifi_tile = render_split_tile(
        if wifi_active { "󰤨" } else { "󰤭" },
        &wifi_label,
        wifi_active,
        tokens,
        || {
            let net = Network::read();
            Network::set_wifi(!net.wifi_enabled());
        },
        || {
            open_view(ActiveView::Network);
        },
    );

    let bt_active = bluetooth.powered();
    let bt_label = if bt_active {
        "Bluetooth"
    } else {
        "Bluetooth Off"
    };
    let bt_tile = render_split_tile(
        if bt_active { "󰂯" } else { "󰂲" },
        bt_label,
        bt_active,
        tokens,
        move || {
            Bluetooth::set_powered(!bt_active);
        },
        || {
            open_view(ActiveView::Bluetooth);
        },
    );

    let row1 = Row::new(vec![Box::new(wifi_tile), Box::new(bt_tile)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let dnd_tile = render_action_tile(
        if dnd { "󰂛" } else { "󰂚" },
        "Do Not Disturb",
        dnd,
        true,
        tokens,
        || {
            let mut cc = ControlCenterService::write();
            cc.dnd = !cc.dnd;
        },
    );

    let disp_tile = render_action_tile("󰍹", "Display Profiles", false, false, tokens, || {
        crate::ui::display_config::toggle();
    });

    let row2 = Row::new(vec![Box::new(dnd_tile), Box::new(disp_tile)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let rec_tile = render_action_tile(
        "󰑋",
        if rec_active {
            "Recording"
        } else {
            "Record Screen"
        },
        rec_active,
        true,
        tokens,
        || {
            crate::ui::screen_record::toggle();
        },
    );

    let pres_tile = render_action_tile(
        "󰐩",
        if is_presenting {
            "Presenting"
        } else {
            "Presentation"
        },
        is_presenting,
        false,
        tokens,
        || {
            let _ = crate::services::presentation::PresentationService::global().toggle();
        },
    );

    let row3 = Row::new(vec![Box::new(rec_tile), Box::new(pres_tile)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new().width(Parent).height(142.0).child(
        Column::new(vec![Box::new(row1), Box::new(row2), Box::new(row3)])
            .width(Parent)
            .gap(8.0),
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

fn render_sliders(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let audio = Audio::read();
    let vol = audio.volume();
    let is_muted = audio.muted();
    let brightness = Brightness::read();
    let bright = brightness.percent();

    let slider_w = 356.0;

    // Volume Slider: full-width 356px with interactive leading mute toggle icon
    let vol_slider = components::fluent_slider(
        vol as f32 / 100.0,
        slider_w,
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
        Some(|| {
            Audio::toggle_mute();
        }),
    );

    // Brightness Slider: full-width 356px
    let bright_slider = components::fluent_slider(
        bright as f32 / 100.0,
        slider_w,
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
        None::<fn()>,
    );

    Rectangle::new().width(Parent).height(76.0).child(
        Column::new(vec![Box::new(vol_slider), Box::new(bright_slider)]).gap(spacing::GAP_SM),
    )
}

fn render_media(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let media = Media::read();
    let playing = media.playing();
    let has_media = !media.title().is_empty();

    let play_pause_icon = if playing { "󰏤" } else { "󰐊" };

    if !has_media && !playing {
        // Minimal idle status card
        return Rectangle::new()
            .width(Parent)
            .height(36.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 12.0,
                right: 12.0,
            })
            .align_child(Align::Start, Align::Center)
            .child(
                Row::new(vec![
                    Box::new(
                        Text::new("󰎆")
                            .size(typography::BODY_LG)
                            .color(tokens.text_muted)
                            .font(&tokens.font_mono),
                    ),
                    Box::new(
                        Text::new("Idle • No media playing")
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            );
    }

    let title_text = if media.title().is_empty() {
        "Audio playback active".to_string()
    } else if media.artist().is_empty() {
        media.title().to_string()
    } else {
        format!("{} - {}", media.artist(), media.title())
    };

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
                            Row::new(vec![
                                Box::new(
                                    Text::new("󰎆")
                                        .size(typography::BODY)
                                        .color(tokens.accent)
                                        .font(&tokens.font_mono),
                                ),
                                Box::new(
                                    Text::new(title_text)
                                        .size(typography::SECONDARY)
                                        .color(tokens.text_primary)
                                        .font(&tokens.font_sans),
                                ),
                            ])
                            .gap(spacing::GAP_XS)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Text::new(if playing { "Playing" } else { "Paused" })
                                .size(typography::CAPTION)
                                .color(if playing {
                                    tokens.success
                                } else {
                                    tokens.steel
                                })
                                .font(&tokens.font_sans),
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
