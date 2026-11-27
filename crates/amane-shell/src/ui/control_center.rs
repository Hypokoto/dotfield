use std::time::Duration;

use amane::{
    Align, Audio, Battery, Bluetooth, Brightness, Center, Color, Column, Cpu, Horizontal, Justify,
    Key, Keyboard, Layer, LayerWindow, Margin, Media, Memory, Network, Notifications, OutputTarget,
    Padding, Parent, Pointer, Rectangle, Row, Service, Text, Weight, Widget, Zone,
};

use crate::config::{ConfigService, ShellManifest};
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
    Notifications,
}

pub struct ControlCenterService {
    pub active_view: Option<ActiveView>,
    pub dnd: bool,
}

impl ControlCenterService {
    #[inline]
    pub fn is_open(&self) -> bool {
        self.active_view.is_some()
    }
}

impl Service for ControlCenterService {
    fn new() -> Self {
        Self {
            active_view: None,
            dnd: false,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut cc = ControlCenterService::write();
        cc.active_view = match cc.active_view {
            Some(_) => None,
            None => Some(ActiveView::Main),
        };
        cc.is_open()
    };
    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::ControlCenter,
        );
    }
    amane::request_frame();
}

pub fn open() {
    {
        let mut cc = ControlCenterService::write();
        cc.active_view = Some(ActiveView::Main);
    }
    crate::ui::modal_manager::ModalManager::open_exclusive(
        crate::ui::modal_manager::ModalType::ControlCenter,
    );
    amane::request_frame();
}

pub fn open_view(view: ActiveView) {
    let should_open = {
        let mut cc = ControlCenterService::write();
        if cc.active_view == Some(view) {
            cc.active_view = None;
            false
        } else {
            cc.active_view = Some(view);
            true
        }
    };
    if should_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::ControlCenter,
        );
    }
    amane::request_frame();
}

pub fn close() {
    let mut cc = ControlCenterService::write();
    cc.active_view = None;
    amane::request_frame();
}

pub fn control_center_panel() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let cc = ControlCenterService::read();

    let is_open = cc.is_open() && manifest.control_center.enabled;
    let motion = crate::ui::animation::panel_motion("control_center", is_open);
    if !motion.visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-control-center")
            .child(Rectangle::new().width(Parent).height(Parent));
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = manifest.theme.font_family.clone();
    let current_view = cc.active_view.unwrap_or_default();

    let content: Column = match current_view {
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
        ActiveView::Notifications => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "Notification Center",
                    "󰂚",
                    &tokens,
                    &font,
                )),
                render_notification_center_subview(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(spacing::GAP_MD)
        }
    };

    let panel_height = match current_view {
        ActiveView::Main => {
            let media_h = if Media::read().playing() || !Media::read().title().is_empty() {
                76.0
            } else {
                38.0
            };
            let max_h = (NiriService::focused_output_height() - 64.0).max(560.0);
            (594.0_f32 + media_h).min(max_h)
        }
        ActiveView::Network => adaptive_network_height(),
        ActiveView::Audio => adaptive_audio_height(),
        ActiveView::Bluetooth => adaptive_bluetooth_height(),
        ActiveView::Tray => adaptive_tray_height(),
        ActiveView::Notifications => adaptive_notifications_height(),
    };

    let is_top_bar = true;
    let panel = components::bar_connected_panel(
        380.0,
        panel_height,
        is_top_bar,
        motion,
        Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        },
        tokens.border_subtle,
        content,
        &tokens,
    );

    let (win_w, win_h) = components::bar_connected_window_size(380.0, panel_height);
    let top_margin = (crate::ui::bar::TOP_ISLAND_HEIGHT + 6.0 + 10.0).round() as i32;

    LayerWindow::new()
        .width(win_w)
        .height(win_h)
        .target_output(OutputTarget::Active)
        .anchor_vertical(amane::Vertical::Top)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: top_margin,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(if is_open {
            Keyboard::OnDemand
        } else {
            Keyboard::None
        })
        .input_region(components::bar_connected_input_region(
            380.0,
            panel_height,
            is_top_bar,
        ))
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .space(Zone::Ignore)
        .visible(motion.visible)
        .namespace("dotfield-shell-control-center")
        .child(panel)
}

fn render_header(tokens: &ThemeTokens) -> Rectangle {
    let title_row = Row::new(vec![
        Box::new(
            Text::new("󱄅")
                .size(typography::TITLE_SM)
                .color(tokens.accent)
                .font(&tokens.font_mono)
                .tight(),
        ),
        Box::new(
            Text::new("Control Center")
                .size(typography::TITLE_SM)
                .weight(Weight::SemiBold)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let settings_btn = Rectangle::new()
        .width(26.0)
        .height(26.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            crate::ui::settings::toggle();
        })
        .child(
            Text::new("󰒓")
                .size(13.0)
                .color(tokens.text_secondary)
                .font(&tokens.font_mono)
                .tight(),
        );

    let close_btn = components::close_button(close, tokens);

    let actions = Row::new(vec![Box::new(settings_btn), Box::new(close_btn)])
        .gap(spacing::GAP_XS)
        .align(Align::Center);

    let header_row = Row::new(vec![Box::new(title_row), Box::new(actions)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(28.0)
        .align_child(Align::Start, Align::Center)
        .child(header_row)
}

fn render_system_overview(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let cpu = Cpu::read().percent();
    let mem = Memory::read();
    let mem_pct = mem.percent();
    let used_gib = mem.used_kib() as f64 / (1024.0 * 1024.0);
    let total_gib = mem.total_kib() as f64 / (1024.0 * 1024.0);

    let battery = Battery::read();

    let mut metrics: Vec<Box<dyn Widget>> = Vec::new();

    // CPU Bento Small Widget
    let cpu_color = if cpu > 80 {
        tokens.danger
    } else if cpu > 50 {
        tokens.warning
    } else {
        tokens.accent
    };

    let cpu_card = components::ios_widget_small(
        "CPU",
        "󰍛",
        cpu_color,
        &format!("{cpu}%"),
        Some("Load"),
        112.0,
        84.0,
        tokens,
    );
    metrics.push(Box::new(cpu_card));

    // RAM Bento Small Widget
    let ram_color = if mem_pct > 85 {
        tokens.danger
    } else if mem_pct > 70 {
        tokens.warning
    } else {
        tokens.accent
    };

    let ram_card = components::ios_widget_small(
        "RAM",
        "󰘚",
        ram_color,
        &format!("{mem_pct}%"),
        Some(&format!("{used_gib:.1}/{total_gib:.0}G")),
        112.0,
        84.0,
        tokens,
    );
    metrics.push(Box::new(ram_card));

    // Battery / Power Bento Small Widget
    let (bat_icon, bat_metric, bat_sub) = if battery.present() {
        let icon = if battery.charging() { "󰂄" } else { "󰁹" };
        (
            icon,
            format!("{}%", battery.percent()),
            if battery.charging() {
                "Charging"
            } else {
                "Battery"
            },
        )
    } else {
        ("󰚥", "Online".to_string(), "AC Power")
    };
    let bat_card = components::ios_widget_small(
        "Power",
        bat_icon,
        tokens.accent,
        &bat_metric,
        Some(bat_sub),
        112.0,
        84.0,
        tokens,
    );
    metrics.push(Box::new(bat_card));

    Rectangle::new().width(Parent).height(84.0).child(
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

fn adaptive_notifications_height() -> f32 {
    let notifs = Notifications::read();
    let count = notifs.list().len().min(5);
    if count == 0 {
        240.0
    } else {
        (170.0 + count as f32 * 78.0).clamp(260.0, 560.0)
    }
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

    // 1. Row 1: Two authentic iOS Bento Split Connectivity widgets (172x68 each)
    let wifi_tile = components::ios_connectivity_widget(
        if wifi_active { "󰤨" } else { "󰤭" },
        "Wi-Fi",
        &wifi_label,
        wifi_active,
        172.0,
        68.0,
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
        let count = bluetooth.devices().len();
        if count > 0 {
            format!("{count} Connected")
        } else {
            "On".to_string()
        }
    } else {
        "Off".to_string()
    };
    let bt_tile = components::ios_connectivity_widget(
        if bt_active { "󰂯" } else { "󰂲" },
        "Bluetooth",
        &bt_label,
        bt_active,
        172.0,
        68.0,
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

    // 2. Row 2: Four authentic iOS Squircle Bento toggles (82x72 each)
    let dnd_toggle = components::ios_control_toggle(
        if dnd { "󰂛" } else { "󰂚" },
        "Focus",
        dnd,
        82.0,
        72.0,
        tokens,
        || {
            let mut cc = ControlCenterService::write();
            cc.dnd = !cc.dnd;
        },
    );

    let rec_toggle = components::ios_control_toggle(
        "󰑋",
        if rec_active { "Recording" } else { "Record" },
        rec_active,
        82.0,
        72.0,
        tokens,
        || {
            crate::ui::screen_record::toggle();
        },
    );

    let disp_toggle =
        components::ios_control_toggle("󰍹", "Displays", false, 82.0, 72.0, tokens, || {
            crate::ui::display_config::toggle();
        });

    let pres_toggle = components::ios_control_toggle(
        "󰐩",
        if is_presenting {
            "Presenting"
        } else {
            "Present"
        },
        is_presenting,
        82.0,
        72.0,
        tokens,
        || {
            let _ = crate::services::presentation::PresentationService::global().toggle();
        },
    );

    let row2 = Row::new(vec![
        Box::new(dnd_toggle),
        Box::new(rec_toggle),
        Box::new(disp_toggle),
        Box::new(pres_toggle),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    Rectangle::new().width(Parent).height(148.0).child(
        Column::new(vec![Box::new(row1), Box::new(row2)])
            .width(Parent)
            .gap(8.0),
    )
}

fn render_subview_header(title: &str, icon: &str, tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let back_btn = Rectangle::new()
        .width(86.0)
        .height(28.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let mut cc = ControlCenterService::write();
            cc.active_view = Some(ActiveView::Main);
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰅁")
                        .size(11.0)
                        .color(tokens.accent)
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Overview")
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
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
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(title)
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let close_btn = components::close_button(close, tokens);
    let close_box = Rectangle::new()
        .width(86.0)
        .height(28.0)
        .align_child(Align::End, Align::Center)
        .child(close_btn);

    Rectangle::new().width(Parent).height(28.0).child(
        Row::new(vec![
            Box::new(back_btn),
            Box::new(title_block),
            Box::new(close_box),
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

    // Volume Slider: full-width 356px chunky iOS Pill Slider with embedded mute toggle
    let vol_slider = components::ios_pill_slider(
        vol as f32 / 100.0,
        slider_w,
        46.0,
        if is_muted { "󰖁" } else { "󰕾" },
        "Volume",
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

    // Brightness Slider: full-width 356px chunky iOS Pill Slider
    let bright_slider = components::ios_pill_slider(
        bright as f32 / 100.0,
        slider_w,
        46.0,
        "󰃠",
        "Brightness",
        false,
        tokens,
        |step| {
            let current = Brightness::read().percent() as i32;
            let target = (current + step).clamp(1, 100) as u8;
            Brightness::set(target);
        },
        None::<fn()>,
    );

    Rectangle::new().width(Parent).height(100.0).child(
        Column::new(vec![Box::new(vol_slider), Box::new(bright_slider)]).gap(spacing::GAP_SM),
    )
}

fn render_media(tokens: &ThemeTokens, font: &str) -> Rectangle {
    let media = Media::read();
    let playing = media.playing();
    let has_media = !media.title().is_empty();

    let play_pause_icon = if playing { "󰏤" } else { "󰐊" };

    if !has_media && !playing {
        // Minimal idle status card with iOS squircle styling
        return Rectangle::new()
            .width(Parent)
            .height(38.0)
            .radius(radius::IOS_WIDGET_SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_specular)
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
                            .color(tokens.accent)
                            .font(&tokens.font_mono),
                    ),
                    Box::new(
                        Text::new("Music • No media playing")
                            .size(typography::SECONDARY)
                            .color(tokens.text_secondary)
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
                tokens.accent
            } else if val > 0.1 {
                tokens.accent_hover
            } else {
                tokens.border_subtle
            });
        bar_widgets.push(Box::new(bar));
    }
    let spectrum_row = Row::new(bar_widgets).gap(2.0).align(Align::End);

    Rectangle::new()
        .width(Parent)
        .height(76.0)
        .radius(radius::IOS_WIDGET_MD)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_specular)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 12.0,
            right: 12.0,
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
                                    tokens.accent
                                } else {
                                    tokens.text_muted
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
                                .radius(radius::SM)
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
                                        .height(28.0)
                                        .radius(radius::PILL)
                                        .fill(tokens.surface_active)
                                        .border(1.0, tokens.border_specular)
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
                                        .width(36.0)
                                        .height(28.0)
                                        .radius(radius::PILL)
                                        .fill(tokens.accent)
                                        .border(1.0, tokens.border_specular)
                                        .cursor(Pointer)
                                        .align_child(Center, Center)
                                        .on_click(|_| Media::play_pause())
                                        .child(
                                            Text::new(play_pause_icon)
                                                .size(typography::BODY_LG)
                                                .color(tokens.text_on_accent)
                                                .font(font),
                                        ),
                                ),
                                Box::new(
                                    Rectangle::new()
                                        .width(32.0)
                                        .height(28.0)
                                        .radius(radius::PILL)
                                        .fill(tokens.surface_active)
                                        .border(1.0, tokens.border_specular)
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

fn render_appearance(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let manifest = ConfigService::read().manifest.clone();
    let style_name = if manifest.theme.style.eq_ignore_ascii_case("cyber-synthwave") {
        "Cyber Synthwave"
    } else {
        &manifest.theme.style
    };

    let theme_badge = Row::new(vec![
        Box::new(
            Text::new("󰏘")
                .size(typography::BODY_LG)
                .color(tokens.accent)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(style_name)
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let wallpaper_btn = Rectangle::new()
        .width(104.0)
        .height(26.0)
        .radius(radius::PILL)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("sh")
                .args([
                    "-c",
                    "if command -v dotfield-wallpaper >/dev/null 2>&1; then \
                        dotfield-wallpaper next; \
                     elif [ -x \"$HOME/.config/dotfield/scripts/wallpaper.sh\" ]; then \
                        \"$HOME/.config/dotfield/scripts/wallpaper.sh\" next; \
                     elif [ -x \"$HOME/dev/projects/dotfield/scripts/wallpaper.sh\" ]; then \
                        \"$HOME/dev/projects/dotfield/scripts/wallpaper.sh\" next; \
                     fi",
                ])
                .spawn();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰸉")
                        .size(12.0)
                        .color(tokens.accent)
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Next Wall")
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        );

    let session_btn = Rectangle::new()
        .width(96.0)
        .height(26.0)
        .radius(radius::PILL)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("sh")
                .args([
                    "-c",
                    "command -v dotfield-session >/dev/null 2>&1 && dotfield-session save || ~/.config/dotfield/scripts/session-manager.sh save",
                ])
                .spawn();
        })
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰆓")
                        .size(12.0)
                        .color(tokens.accent)
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Save State")
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center),
        );

    let actions_row = Row::new(vec![Box::new(wallpaper_btn), Box::new(session_btn)])
        .gap(spacing::GAP_SM)
        .align(Align::Center);

    let row = Row::new(vec![Box::new(theme_badge), Box::new(actions_row)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(34.0)
        .radius(radius::IOS_WIDGET_SM)
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

fn render_notifications_drawer(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let notifications = Notifications::read();
    let notifs_count = notifications.list().len();

    let clear_btn: Box<dyn Widget> = if notifs_count > 0 {
        Box::new(
            Rectangle::new()
                .width(54.0)
                .height(22.0)
                .radius(radius::PILL)
                .fill(tokens.danger)
                .border(1.0, tokens.border_danger)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| Notifications::clear())
                .child(
                    Text::new("Clear")
                        .size(typography::BADGE)
                        .color(tokens.text_on_danger)
                        .font(&tokens.font_sans)
                        .tight(),
                ),
        )
    } else {
        Box::new(
            Rectangle::new()
                .width(44.0)
                .height(20.0)
                .radius(radius::PILL)
                .fill(tokens.surface)
                .border(1.0, tokens.border_specular)
                .align_child(Center, Center)
                .child(
                    Text::new("Idle")
                        .size(typography::BADGE)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans)
                        .tight(),
                ),
        )
    };

    let header_row = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Text::new("󰂚")
                        .size(13.0)
                        .color(if notifs_count > 0 {
                            tokens.accent
                        } else {
                            tokens.text_muted
                        })
                        .font(&tokens.font_mono)
                        .tight(),
                ),
                Box::new(
                    Text::new(format!("Notifications ({notifs_count})"))
                        .size(typography::SECONDARY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        ),
        clear_btn,
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(56.0)
        .radius(radius::IOS_WIDGET_SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .on_click(|_| {
            let mut cc = ControlCenterService::write();
            cc.active_view = Some(ActiveView::Notifications);
        })
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
                        "No new notifications (click to view center)"
                    } else {
                        "Recent notifications active (click to open)"
                    })
                    .size(typography::BADGE)
                    .color(tokens.text_muted)
                    .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_XS),
        )
}

fn render_notification_center_subview(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);
    let notifs = Notifications::read();
    let notifs_list = notifs.list();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Top action bar: count badge + Clear All button
    let count = notifs_list.len();
    let count_text = Text::new(format!("{count} active alerts"))
        .size(typography::SECONDARY)
        .color(tokens.text_secondary)
        .font(font);

    let clear_all_btn = if count > 0 {
        Box::new(
            Rectangle::new()
                .width(84.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(tokens.surface_active)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| {
                    Notifications::clear();
                    amane::request_frame();
                })
                .child(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰃢")
                                .size(11.0)
                                .color(tokens.danger)
                                .font(&tokens.font_mono)
                                .tight(),
                        ),
                        Box::new(
                            Text::new("Clean All")
                                .size(typography::CAPTION)
                                .color(tokens.text_primary)
                                .font(font)
                                .tight(),
                        ),
                    ])
                    .gap(4.0)
                    .align(Align::Center),
                ),
        ) as Box<dyn Widget>
    } else {
        Box::new(
            Text::new("")
                .size(typography::CAPTION)
                .color(Color::TRANSPARENT),
        ) as Box<dyn Widget>
    };

    let toolbar = Row::new(vec![Box::new(count_text), clear_all_btn])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    elements.push(Box::new(toolbar));

    if notifs_list.is_empty() {
        let empty_card = Rectangle::new()
            .width(Parent)
            .height(110.0)
            .radius(radius::MD)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .align_child(Center, Center)
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new("󰂚")
                            .size(24.0)
                            .color(tokens.text_muted)
                            .font(&tokens.font_mono)
                            .tight(),
                    ),
                    Box::new(
                        Text::new("No active notifications")
                            .size(typography::BODY)
                            .color(tokens.text_muted)
                            .font(font),
                    ),
                    Box::new(
                        Text::new("You're all caught up!")
                            .size(typography::CAPTION)
                            .color(tokens.text_secondary)
                            .font(font),
                    ),
                ])
                .gap(spacing::GAP_XS)
                .align(Center),
            );
        elements.push(Box::new(empty_card));
    } else {
        for notif in notifs_list.iter().take(5) {
            let id = notif.id();
            let (border_col, header_col) = match notif.urgency() {
                amane::Urgency::Critical => (tokens.border_danger, tokens.danger),
                amane::Urgency::Low => (tokens.border_subtle, tokens.text_muted),
                amane::Urgency::Normal => (tokens.border_subtle, tokens.accent),
            };

            let app_name = if notif.app_name().is_empty() {
                "Notification"
            } else {
                notif.app_name()
            };

            let mark_read_btn = Rectangle::new()
                .width(20.0)
                .height(20.0)
                .radius(radius::PILL)
                .fill(tokens.surface_track)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| {
                    let n_list = Notifications::read();
                    let read_key = n_list
                        .list()
                        .iter()
                        .find(|n| n.id() == id)
                        .and_then(|target| {
                            target.actions().iter().find(|a| {
                                let k = a.key().to_lowercase();
                                let l = a.label().to_lowercase();
                                k.contains("read") || l.contains("read")
                            })
                        })
                        .map(|a| a.key().to_string());

                    if let Some(key) = read_key {
                        Notifications::invoke(id, &key);
                    }
                    Notifications::dismiss(id);
                    amane::request_frame();
                })
                .child(
                    Text::new("󰄬")
                        .size(10.0)
                        .color(tokens.accent)
                        .font(&tokens.font_mono)
                        .tight(),
                );

            let dismiss_btn = Rectangle::new()
                .width(20.0)
                .height(20.0)
                .radius(radius::PILL)
                .fill(tokens.surface_track)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| {
                    Notifications::dismiss(id);
                    amane::request_frame();
                })
                .child(
                    Text::new("󰅖")
                        .size(10.0)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_mono)
                        .tight(),
                );

            let header_actions = Row::new(vec![Box::new(mark_read_btn), Box::new(dismiss_btn)])
                .gap(6.0)
                .align(Align::Center);

            let notif_header = Row::new(vec![
                Box::new(
                    Text::new(app_name)
                        .size(typography::CAPTION)
                        .weight(Weight::SemiBold)
                        .color(header_col)
                        .font(font),
                ),
                Box::new(header_actions),
            ])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

            let mut card_children: Vec<Box<dyn Widget>> = vec![
                Box::new(notif_header),
                Box::new(
                    Text::new(notif.summary())
                        .size(typography::SECONDARY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(font),
                ),
            ];

            if !notif.body().is_empty() {
                card_children.push(Box::new(
                    Text::new(notif.body())
                        .size(typography::CAPTION)
                        .color(tokens.text_secondary)
                        .font(font),
                ));
            }

            let card_h = if notif.body().is_empty() { 56.0 } else { 72.0 };
            let card = Rectangle::new()
                .width(Parent)
                .height(card_h)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, border_col)
                .cursor(Pointer)
                .padding(Padding {
                    top: 6.0,
                    bottom: 6.0,
                    left: 10.0,
                    right: 10.0,
                })
                .on_click(move |_| {
                    Notifications::click(id);
                    amane::request_frame();
                })
                .child(
                    Column::new(card_children)
                        .width(Parent)
                        .gap(spacing::GAP_XS),
                );

            elements.push(Box::new(card));
        }
    }

    Box::new(Column::new(elements).width(Parent).gap(spacing::GAP_SM))
}

fn render_power_button<F>(
    glyph: &'static str,
    label: &'static str,
    icon_col: Color,
    border_col: Color,
    bg_col: Color,
    tokens: &ThemeTokens,
    callback: F,
) -> Box<dyn Widget>
where
    F: Fn() + Send + Sync + 'static,
{
    let btn = Rectangle::new()
        .width(66.0)
        .height(32.0)
        .radius(radius::PILL)
        .fill(bg_col)
        .border(1.0, border_col)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| callback())
        .child(
            Row::new(vec![
                Box::new(
                    Text::new(glyph)
                        .size(12.0)
                        .color(icon_col)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
                Box::new(
                    Text::new(label)
                        .size(typography::BADGE)
                        .color(if bg_col == tokens.danger {
                            tokens.text_on_danger
                        } else {
                            tokens.text_primary
                        })
                        .font(&tokens.font_sans)
                        .tight(),
                ),
            ])
            .gap(4.0)
            .align(Align::Center),
        );
    Box::new(btn)
}

fn render_power_actions(tokens: &ThemeTokens, _font: &str) -> Rectangle {
    let btns: Vec<Box<dyn Widget>> = vec![
        render_power_button(
            "󰌾",
            "Lock",
            tokens.text_secondary,
            tokens.border_subtle,
            tokens.surface_elevated,
            tokens,
            || {
                let _ = std::process::Command::new("sh")
                    .args(["-c", "~/.config/dotfield/scripts/lock.sh"])
                    .spawn();
                close();
            },
        ),
        render_power_button(
            "󰒲",
            "Sleep",
            tokens.text_secondary,
            tokens.border_subtle,
            tokens.surface_elevated,
            tokens,
            || {
                let _ = std::process::Command::new("systemctl")
                    .arg("suspend")
                    .spawn();
                close();
            },
        ),
        render_power_button(
            "󰜉",
            "Reboot",
            tokens.text_secondary,
            tokens.border_subtle,
            tokens.surface_elevated,
            tokens,
            || {
                let _ = std::process::Command::new("systemctl")
                    .arg("reboot")
                    .spawn();
                close();
            },
        ),
        render_power_button(
            "󰐥",
            "Power",
            tokens.text_on_danger,
            tokens.border_danger,
            tokens.danger,
            tokens,
            || {
                let _ = std::process::Command::new("systemctl")
                    .arg("poweroff")
                    .spawn();
                close();
            },
        ),
        render_power_button(
            "󰗽",
            "Quit",
            tokens.text_secondary,
            tokens.border_subtle,
            tokens.surface_elevated,
            tokens,
            || {
                let _ = NiriService::quit(false);
                close();
            },
        ),
    ];

    Rectangle::new().width(Parent).height(32.0).child(
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
        assert!(!svc.is_open());
        assert_eq!(svc.active_view, None);
        assert!(!svc.dnd);
    }

    #[test]
    fn test_control_center_lifecycle_and_views() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        toggle();
        assert_eq!(
            ControlCenterService::read().active_view,
            Some(ActiveView::Main)
        );
        open_view(ActiveView::Network);
        assert_eq!(
            ControlCenterService::read().active_view,
            Some(ActiveView::Network)
        );
        open_view(ActiveView::Audio);
        assert_eq!(
            ControlCenterService::read().active_view,
            Some(ActiveView::Audio)
        );
        close();
        assert!(!ControlCenterService::read().is_open());
    }
}
