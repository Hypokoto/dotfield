use parking_lot::RwLock;
use std::sync::LazyLock;
use std::time::Duration;

use amane::{
    Align, Audio, Battery, Bluetooth, Brightness, Center, Color, Column, Cpu, Horizontal, Justify,
    Layer, LayerWindow, Margin, Media, Memory, Network, Notifications, Padding, Parent, Pointer,
    Rectangle, Row, Scroll, Service, Stack, Text, Vertical, Widget, Zone,
};

use crate::config::ShellManifest;
use crate::services::niri::NiriService;

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

static CONFIG: LazyLock<RwLock<ShellManifest>> =
    LazyLock::new(|| RwLock::new(ShellManifest::default()));

pub fn set_config(manifest: ShellManifest) {
    *CONFIG.write() = manifest;
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
    let manifest = CONFIG.read().clone();
    let cc = ControlCenterService::read();

    let is_visible = cc.open && manifest.control_center.enabled;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .namespace("dotfield-shell-control-center");
    }

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let font = manifest.theme.font_family.clone();

    let content: Column = match cc.view {
        ActiveView::Main => {
            let mut sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_header(&manifest, &font)),
                Box::new(render_system_overview(&manifest, &font)),
                Box::new(render_quick_toggles(&manifest, cc.dnd, &font)),
                Box::new(render_widget_shortcuts(&manifest, &font)),
                Box::new(render_sliders(&manifest, &font)),
                Box::new(render_appearance(&manifest, &font)),
            ];

            if manifest.control_center.show_mpris {
                sections.push(Box::new(render_media(&manifest, &font)));
            }

            sections.push(Box::new(render_notifications_drawer(&manifest, &font)));
            sections.push(Box::new(render_power_actions(&manifest, &font)));

            Column::new(sections).width(Parent).gap(10.0)
        }
        ActiveView::Network => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "NetworkManager",
                    "󰤨",
                    &manifest,
                    &font,
                )),
                crate::ui::widgets::network::render_network_widget(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(10.0)
        }
        ActiveView::Audio => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "PipeWire Audio",
                    "󰕾",
                    &manifest,
                    &font,
                )),
                crate::ui::widgets::audio::render_audio_widget(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(10.0)
        }
        ActiveView::Bluetooth => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "BlueZ Bluetooth",
                    "󰂯",
                    &manifest,
                    &font,
                )),
                crate::ui::widgets::bluetooth::render_bluetooth_widget(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(10.0)
        }
        ActiveView::Tray => {
            let sections: Vec<Box<dyn Widget>> = vec![
                Box::new(render_subview_header(
                    "System Tray & Apps",
                    "󰀻",
                    &manifest,
                    &font,
                )),
                crate::ui::tray::render_tray_drawer(&manifest, &font),
            ];
            Column::new(sections).width(Parent).gap(10.0)
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
        .radius(12.0)
        .fill(bg_color)
        .border(1.0, Color::from("#34343D"))
        .padding(Padding {
            top: 14.0,
            bottom: 14.0,
            left: 14.0,
            right: 14.0,
        })
        .child(content);

    LayerWindow::new()
        .width(380.0)
        .height(panel_height)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 38,
            right: 12,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .visible(is_visible)
        .namespace("dotfield-shell-control-center")
        .child(panel)
}

fn render_header(manifest: &ShellManifest, font: &str) -> Rectangle {
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let muted_color = Color::from(manifest.palette.muted.as_str());

    let title_block = Row::new(vec![
        Box::new(Text::new("󰒓").size(14.0).color(amber_color).font(font)),
        Box::new(
            Text::new("CONTROL CENTER")
                .size(11.0)
                .color(fg_color)
                .font(font),
        ),
        Box::new(
            Text::new("• Niri+Amane")
                .size(10.0)
                .color(muted_color)
                .font(font),
        ),
    ])
    .gap(8.0)
    .align(Align::Center);

    let close_btn = Rectangle::new()
        .width(24.0)
        .height(24.0)
        .radius(4.0)
        .fill(Color::from(manifest.palette.overlay.as_str()))
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| close())
        .child(Text::new("󰅖").size(11.0).color(fg_color).font(font));

    let header_row = Row::new(vec![Box::new(title_block), Box::new(close_btn)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(28.0)
        .child(header_row)
}

fn render_system_overview(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Industrial Amber #FFB000
    let hover_color = Color::from(manifest.palette.hover.as_str()); // Pale Amber #FFD166
    let fg_color = Color::from(manifest.palette.foreground.as_str()); // Bone #E8E6E3
    let crimson_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D

    let cpu = Cpu::read().percent();
    let mem = Memory::read();
    let mem_pct = mem.percent();
    let used_gib = mem.used_kib() as f64 / (1024.0 * 1024.0);
    let total_gib = mem.total_kib() as f64 / (1024.0 * 1024.0);

    let battery = Battery::read();

    let mut metrics: Vec<Box<dyn Widget>> = Vec::new();

    // CPU Card
    let cpu_card = Rectangle::new()
        .width(112.0)
        .height(48.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
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
                        .size(10.0)
                        .color(if cpu > 80 {
                            crimson_color
                        } else if cpu > 50 {
                            hover_color
                        } else {
                            amber_color
                        })
                        .font(font),
                ),
                Box::new(
                    Text::new(format!("{cpu}%"))
                        .size(13.0)
                        .color(fg_color)
                        .font(font),
                ),
            ])
            .gap(2.0),
        );
    metrics.push(Box::new(cpu_card));

    // RAM Card
    let ram_card = Rectangle::new()
        .width(112.0)
        .height(48.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
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
                        .size(10.0)
                        .color(if mem_pct > 85 {
                            crimson_color
                        } else if mem_pct > 70 {
                            hover_color
                        } else {
                            amber_color
                        })
                        .font(font),
                ),
                Box::new(
                    Text::new(format!("{mem_pct}% ({used_gib:.1}/{total_gib:.0}G)"))
                        .size(11.0)
                        .color(fg_color)
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
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 8.0,
            right: 8.0,
        })
        .child(
            Column::new(vec![
                Box::new(Text::new("POWER").size(10.0).color(amber_color).font(font)),
                Box::new(Text::new(bat_text).size(11.0).color(fg_color).font(font)),
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

fn render_quick_toggles(manifest: &ShellManifest, dnd: bool, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Industrial Amber #FFB000
    let crimson_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let fg_color = Color::from(manifest.palette.foreground.as_str()); // Bone #E8E6E3
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D

    let network = Network::read();
    let bluetooth = Bluetooth::read();

    let mut toggles: Vec<Box<dyn Widget>> = Vec::new();

    // Wi-Fi Toggle
    let wifi_active = network.connected();
    let wifi_btn = Rectangle::new()
        .width(110.0)
        .height(38.0)
        .radius(6.0)
        .fill(if wifi_active {
            amber_color
        } else {
            overlay_color
        })
        .border(
            1.0,
            if wifi_active {
                Color::from("#FFD166")
            } else {
                border_color
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
            .size(11.0)
            .color(if wifi_active { Color::BLACK } else { fg_color })
            .font(font),
        );
    toggles.push(Box::new(wifi_btn));

    // Bluetooth Toggle
    let bt_active = bluetooth.powered();
    let bt_btn = Rectangle::new()
        .width(110.0)
        .height(38.0)
        .radius(6.0)
        .fill(if bt_active {
            amber_color
        } else {
            overlay_color
        })
        .border(
            1.0,
            if bt_active {
                Color::from("#FFD166")
            } else {
                border_color
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
            .size(11.0)
            .color(if bt_active { Color::BLACK } else { fg_color })
            .font(font),
        );
    toggles.push(Box::new(bt_btn));

    // Do Not Disturb (DND)
    let dnd_btn = Rectangle::new()
        .width(110.0)
        .height(38.0)
        .radius(6.0)
        .fill(if dnd { crimson_color } else { overlay_color })
        .border(
            1.0,
            if dnd {
                Color::from("#FF6B76")
            } else {
                border_color
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
                .size(11.0)
                .color(if dnd { Color::WHITE } else { fg_color })
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

fn render_widget_shortcuts(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());

    let net_shortcut = Rectangle::new()
        .width(82.0)
        .height(30.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            open_view(ActiveView::Network);
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("󰤨").size(12.0).color(amber_color).font(font)),
                Box::new(Text::new("Net ❯").size(10.0).color(fg_color).font(font)),
            ])
            .gap(3.0)
            .align(Align::Center),
        );

    let audio_shortcut = Rectangle::new()
        .width(82.0)
        .height(30.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            open_view(ActiveView::Audio);
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("󰕾").size(12.0).color(amber_color).font(font)),
                Box::new(Text::new("Audio ❯").size(10.0).color(fg_color).font(font)),
            ])
            .gap(3.0)
            .align(Align::Center),
        );

    let bt_shortcut = Rectangle::new()
        .width(82.0)
        .height(30.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            open_view(ActiveView::Bluetooth);
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("󰂯").size(12.0).color(amber_color).font(font)),
                Box::new(Text::new("BlueZ ❯").size(10.0).color(fg_color).font(font)),
            ])
            .gap(3.0)
            .align(Align::Center),
        );

    let tray_shortcut = Rectangle::new()
        .width(82.0)
        .height(30.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            open_view(ActiveView::Tray);
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("󰀻").size(12.0).color(amber_color).font(font)),
                Box::new(Text::new("Tray ❯").size(10.0).color(fg_color).font(font)),
            ])
            .gap(3.0)
            .align(Align::Center),
        );

    Rectangle::new().width(Parent).height(30.0).child(
        Row::new(vec![
            Box::new(net_shortcut),
            Box::new(audio_shortcut),
            Box::new(bt_shortcut),
            Box::new(tray_shortcut),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    )
}

fn render_subview_header(
    title: &str,
    icon: &str,
    manifest: &ShellManifest,
    font: &str,
) -> Rectangle {
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let hover_color = Color::from(manifest.palette.hover.as_str());
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());

    let back_btn = Rectangle::new()
        .width(82.0)
        .height(28.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let mut cc = ControlCenterService::write();
            cc.view = ActiveView::Main;
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("❮").size(11.0).color(amber_color).font(font)),
                Box::new(Text::new("Overview").size(10.0).color(fg_color).font(font)),
            ])
            .gap(4.0)
            .align(Align::Center),
        );

    let title_block = Row::new(vec![
        Box::new(Text::new(icon).size(14.0).color(amber_color).font(font)),
        Box::new(Text::new(title).size(12.0).color(fg_color).font(font)),
    ])
    .gap(6.0)
    .align(Align::Center);

    let close_btn = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            close();
        })
        .child(Text::new("󰅖").size(12.0).color(hover_color).font(font));

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

fn render_sliders(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Industrial Amber #FFB000
    let crimson_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let fg_color = Color::from(manifest.palette.foreground.as_str()); // Bone #E8E6E3
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let steel_color = Color::from(manifest.palette.steel.as_str()); // Steel #8B949E

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
        Color::from("#3D2024") // Muted Crimson tint
    } else {
        Color::from("#382E15") // Rich Amber tint
    };

    let vol_bg_rect = Rectangle::new()
        .width(vol_slider_w)
        .height(34.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color);

    let vol_fill_rect = Rectangle::new()
        .width(vol_fill_w)
        .height(34.0)
        .radius(6.0)
        .fill(vol_fill_color);

    let vol_label_row = Row::new(vec![
        Box::new(
            Text::new(if is_muted { "󰖁" } else { "󰕾" })
                .size(13.0)
                .color(if is_muted { crimson_color } else { amber_color })
                .font(font),
        ),
        Box::new(
            Text::new(format!("Volume {vol}%"))
                .size(11.0)
                .color(if is_muted { steel_color } else { fg_color })
                .font(font),
        ),
    ])
    .gap(8.0)
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
        .radius(6.0)
        .fill(if is_muted {
            crimson_color
        } else {
            overlay_color
        })
        .border(
            1.0,
            if is_muted {
                Color::from("#FF6B76")
            } else {
                border_color
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Audio::toggle_mute();
        })
        .child(
            Text::new(if is_muted { "Muted" } else { "Mute" })
                .size(10.0)
                .color(if is_muted { Color::WHITE } else { fg_color })
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
    let bright_fill_color = Color::from("#382E15"); // Rich Amber tint

    let bright_bg_rect = Rectangle::new()
        .width(bright_slider_w)
        .height(34.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color);

    let bright_fill_rect = Rectangle::new()
        .width(bright_fill_w)
        .height(34.0)
        .radius(6.0)
        .fill(bright_fill_color);

    let bright_label_row = Row::new(vec![
        Box::new(Text::new("󰃠").size(13.0).color(amber_color).font(font)),
        Box::new(
            Text::new(format!("Brightness {bright}%"))
                .size(11.0)
                .color(fg_color)
                .font(font),
        ),
    ])
    .gap(8.0)
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
        .height(72.0)
        .child(Column::new(rows).gap(8.0))
}

fn render_appearance(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());

    let theme_badge = Row::new(vec![
        Box::new(Text::new("󰔎").size(13.0).color(amber_color).font(font)),
        Box::new(
            Text::new("Controlled Chaos Theme")
                .size(11.0)
                .color(fg_color)
                .font(font),
        ),
    ])
    .gap(8.0)
    .align(Align::Center);

    let wallpaper_btn = Rectangle::new()
        .width(110.0)
        .height(26.0)
        .radius(4.0)
        .fill(amber_color)
        .border(1.0, Color::from("#FFD166"))
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("sh")
                .args(["-c", "~/.config/dotfield/scripts/wallpaper.sh"])
                .spawn();
        })
        .child(
            Text::new("󰸉 Next Wall")
                .size(10.0)
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
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(row)
}

fn render_media(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());

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
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Column::new(vec![
                Box::new(Text::new(title_text).size(11.0).color(fg_color).font(font)),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Rectangle::new()
                                .width(34.0)
                                .height(24.0)
                                .radius(4.0)
                                .fill(Color::from(manifest.palette.muted.as_str()))
                                .border(1.0, Color::from("#262630"))
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| Media::previous())
                                .child(Text::new("󰒮").size(11.0).color(fg_color).font(font)),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(34.0)
                                .height(24.0)
                                .radius(4.0)
                                .fill(amber_color)
                                .border(1.0, Color::from("#FFD166"))
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| Media::play_pause())
                                .child(
                                    Text::new(play_pause_icon)
                                        .size(13.0)
                                        .color(Color::BLACK)
                                        .font(font),
                                ),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(34.0)
                                .height(24.0)
                                .radius(4.0)
                                .fill(Color::from(manifest.palette.muted.as_str()))
                                .border(1.0, Color::from("#262630"))
                                .cursor(Pointer)
                                .align_child(Center, Center)
                                .on_click(|_| Media::next())
                                .child(Text::new("󰒭").size(11.0).color(fg_color).font(font)),
                        ),
                    ])
                    .gap(10.0)
                    .align(Align::Center),
                ),
            ])
            .gap(4.0),
        )
}

fn render_notifications_drawer(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());

    let notifications = Notifications::read();
    let notifs_count = notifications.list().len();

    let header_row = Row::new(vec![
        Box::new(
            Text::new(format!("Notifications ({notifs_count})"))
                .size(11.0)
                .color(fg_color)
                .font(font),
        ),
        Box::new(
            Rectangle::new()
                .width(56.0)
                .height(20.0)
                .radius(4.0)
                .fill(crimson_color)
                .border(1.0, Color::from("#FF6B76"))
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| Notifications::clear())
                .child(Text::new("Clear").size(10.0).color(Color::WHITE).font(font)),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(80.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(1.0, border_color)
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
                    .size(10.0)
                    .color(Color::from(manifest.palette.muted.as_str()))
                    .font(font),
                ),
            ])
            .gap(8.0),
        )
}

fn render_power_actions(manifest: &ShellManifest, font: &str) -> Rectangle {
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());

    let mut btns: Vec<Box<dyn Widget>> = Vec::new();

    // Lock
    let lock_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("sh")
                .args(["-c", "~/.config/dotfield/scripts/lock.sh"])
                .spawn();
            close();
        })
        .child(Text::new("󰌾 Lock").size(10.0).color(fg_color).font(font));
    btns.push(Box::new(lock_btn));

    // Suspend
    let suspend_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("systemctl")
                .arg("suspend")
                .spawn();
            close();
        })
        .child(Text::new("󰒲 Sleep").size(10.0).color(fg_color).font(font));
    btns.push(Box::new(suspend_btn));

    // Reboot
    let reboot_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = std::process::Command::new("systemctl")
                .arg("reboot")
                .spawn();
            close();
        })
        .child(Text::new("󰜉 Reboot").size(10.0).color(fg_color).font(font));
    btns.push(Box::new(reboot_btn));

    // Poweroff
    let poweroff_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(4.0)
        .fill(crimson_color)
        .border(1.0, Color::from("#9D0208")) // Blood Red critical emphasis
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
                .size(10.0)
                .color(Color::WHITE)
                .font(font),
        );
    btns.push(Box::new(poweroff_btn));

    // Logout
    let logout_btn = Rectangle::new()
        .width(64.0)
        .height(30.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let _ = NiriService::quit(false);
            close();
        })
        .child(Text::new("󰗽 Quit").size(10.0).color(fg_color).font(font));
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
