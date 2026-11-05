use amane::{
    Align, Audio, Center, Color, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Scroll,
    Service, SinkType, Text, Widget,
};

use crate::config::ShellManifest;

/// Renders the PipeWire Audio interactive widget.
pub fn render_audio_widget(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let surface_color = Color::from(manifest.palette.surface.as_str());
    let overlay_color = Color::from(manifest.palette.overlay.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str());
    let hover_color = Color::from(manifest.palette.hover.as_str());
    let crimson_color = Color::from(manifest.palette.accent.as_str());
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str());
    let steel_color = Color::from(manifest.palette.steel.as_str());

    let audio = Audio::read();
    let vol = audio.volume();
    let is_muted = audio.muted();
    let mic_vol = audio.microphone_volume();
    let is_mic_muted = audio.microphone_muted();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Master Output Section (Speaker / Headphones)
    let output_header = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Text::new(if is_muted { "󰖁" } else { "󰕾" })
                        .size(14.0)
                        .color(if is_muted { crimson_color } else { amber_color })
                        .font(font),
                ),
                Box::new(
                    Text::new("PIPEWIRE OUTPUT")
                        .size(10.0)
                        .color(amber_color)
                        .font(font),
                ),
            ])
            .gap(6.0)
            .align(Align::Center),
        ),
        Box::new(
            Text::new(if is_muted {
                "MUTED".to_string()
            } else {
                format!("{vol}%")
            })
            .size(11.0)
            .color(if is_muted { crimson_color } else { hover_color })
            .font(font),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(output_header));

    // Master Output Volume Track & Interactive Slider
    let slider_w: f32 = 352.0;
    let fill_w: f32 = (slider_w * (vol as f32 / 100.0)).clamp(0.0_f32, slider_w);

    let output_slider_card = Rectangle::new()
        .width(Parent)
        .height(38.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(
            1.0,
            if is_muted {
                Color::from("#FF6B76")
            } else {
                border_color
            },
        )
        .cursor(Pointer)
        .on_scroll(|scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            let current = Audio::read().volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_volume(target);
        })
        .child(Row::new(vec![
            // Visual progress bar inside track
            Box::new(
                Rectangle::new()
                    .width(fill_w)
                    .height(38.0)
                    .radius(6.0)
                    .fill(if is_muted {
                        Color::from("#3D2024")
                    } else {
                        Color::from("#382E15")
                    }),
            ),
        ]));

    elements.push(Box::new(output_slider_card));

    // Output Controls: Mute toggle, Presets (25%, 50%, 75%, 100%)
    let mute_btn = Rectangle::new()
        .width(80.0)
        .height(28.0)
        .radius(4.0)
        .fill(if is_muted {
            crimson_color
        } else {
            surface_color
        })
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Audio::toggle_mute();
        })
        .child(
            Text::new(if is_muted { "󰖁 Unmute" } else { "󰕾 Mute" })
                .size(10.0)
                .color(if is_muted { Color::WHITE } else { fg_color })
                .font(font),
        );

    let mut preset_btns: Vec<Box<dyn Widget>> = vec![Box::new(mute_btn)];
    for preset in [25, 50, 75, 100] {
        let is_selected = !is_muted && (vol as i32 - preset as i32).abs() <= 5;
        let btn = Rectangle::new()
            .width(58.0)
            .height(28.0)
            .radius(4.0)
            .fill(if is_selected {
                amber_color
            } else {
                surface_color
            })
            .border(1.0, border_color)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                Audio::set_volume(preset);
            })
            .child(
                Text::new(format!("{preset}%"))
                    .size(10.0)
                    .color(if is_selected { Color::BLACK } else { fg_color })
                    .font(font),
            );
        preset_btns.push(Box::new(btn));
    }

    elements.push(Box::new(
        Row::new(preset_btns)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
    ));

    // 2. Output Sinks (Devices) Selection Section
    let sinks = audio.sinks();
    if !sinks.is_empty() {
        let sinks_header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(Text::new("󰓃").size(13.0).color(amber_color).font(font)),
                    Box::new(
                        Text::new("OUTPUT DEVICES")
                            .size(10.0)
                            .color(amber_color)
                            .font(font),
                    ),
                ])
                .gap(6.0)
                .align(Align::Center),
            ),
            Box::new(
                Text::new(format!("{} AVAILABLE", sinks.len()))
                    .size(9.0)
                    .color(steel_color)
                    .font(font),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        elements.push(Box::new(sinks_header));

        let mut sink_cards: Vec<Box<dyn Widget>> = Vec::new();
        for sink in sinks {
            let is_default = sink.is_default;
            let sink_name = sink.name.clone();
            let sink_desc = sink.description.clone();

            let icon_str = match sink.sink_type {
                SinkType::Bluetooth => "󰂯",
                SinkType::Hdmi => "󰍹",
                SinkType::Headphones => "󰋋",
                SinkType::Usb => "󰓃",
                SinkType::Speaker => "󰌢",
            };

            // Format friendly name: clean up alsa suffixes or display names
            let display_name = if sink_desc.len() > 32 {
                format!("{}…", &sink_desc[..31])
            } else {
                sink_desc.clone()
            };

            let card = Rectangle::new()
                .width(Parent)
                .height(34.0)
                .radius(5.0)
                .fill(if is_default {
                    overlay_color
                } else {
                    surface_color
                })
                .border(
                    1.0,
                    if is_default {
                        amber_color
                    } else {
                        border_color
                    },
                )
                .cursor(Pointer)
                .padding(Padding {
                    top: 2.0,
                    bottom: 2.0,
                    left: 10.0,
                    right: 10.0,
                })
                .on_click(move |_| {
                    Audio::set_default_sink(&sink_name);
                    let _ = std::process::Command::new("notify-send")
                        .args([
                            "-a",
                            "dotfield",
                            "-c",
                            "audio",
                            "-r",
                            "9994",
                            "-h",
                            "string:x-canonical-private-synchronous:audio-sink",
                            "-t",
                            "1500",
                            "Audio Output",
                            &sink_desc,
                        ])
                        .spawn();
                })
                .child(
                    Row::new(vec![
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Text::new(icon_str)
                                        .size(13.0)
                                        .color(if is_default { amber_color } else { fg_color })
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(display_name)
                                        .size(10.0)
                                        .color(if is_default { fg_color } else { steel_color })
                                        .font(font),
                                ),
                            ])
                            .gap(8.0)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Text::new(if is_default { "ACTIVE" } else { "SELECT" })
                                .size(9.0)
                                .color(if is_default { amber_color } else { steel_color })
                                .font(font),
                        ),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                );

            sink_cards.push(Box::new(card));
        }

        elements.push(Box::new(Column::new(sink_cards).width(Parent).gap(4.0)));
    }

    // 3. Microphone Input Section
    let mic_header = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Text::new(if is_mic_muted { "󰍭" } else { "󰍬" })
                        .size(14.0)
                        .color(if is_mic_muted {
                            crimson_color
                        } else {
                            amber_color
                        })
                        .font(font),
                ),
                Box::new(
                    Text::new("PIPEWIRE MICROPHONE")
                        .size(10.0)
                        .color(amber_color)
                        .font(font),
                ),
            ])
            .gap(6.0)
            .align(Align::Center),
        ),
        Box::new(
            Text::new(if is_mic_muted {
                "MUTED".to_string()
            } else {
                format!("{mic_vol}%")
            })
            .size(11.0)
            .color(if is_mic_muted {
                crimson_color
            } else {
                hover_color
            })
            .font(font),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(mic_header));

    // Microphone Slider Card
    let mic_fill_w: f32 = (slider_w * (mic_vol as f32 / 100.0)).clamp(0.0_f32, slider_w);
    let mic_slider_card = Rectangle::new()
        .width(Parent)
        .height(38.0)
        .radius(6.0)
        .fill(overlay_color)
        .border(
            1.0,
            if is_mic_muted {
                Color::from("#FF6B76")
            } else {
                border_color
            },
        )
        .cursor(Pointer)
        .on_scroll(|scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            let current = Audio::read().microphone_volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_microphone_volume(target);
        })
        .child(Row::new(vec![Box::new(
            Rectangle::new()
                .width(mic_fill_w)
                .height(38.0)
                .radius(6.0)
                .fill(if is_mic_muted {
                    Color::from("#3D2024")
                } else {
                    Color::from("#382E15")
                }),
        )]));

    elements.push(Box::new(mic_slider_card));

    // Mic Controls: Mic Mute toggle, Step -5%, Step +5%
    let mic_mute_btn = Rectangle::new()
        .width(110.0)
        .height(28.0)
        .radius(4.0)
        .fill(if is_mic_muted {
            crimson_color
        } else {
            surface_color
        })
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            Audio::toggle_microphone_mute();
        })
        .child(
            Text::new(if is_mic_muted {
                "󰍭 Mic Unmute"
            } else {
                "󰍬 Mic Mute"
            })
            .size(10.0)
            .color(if is_mic_muted { Color::WHITE } else { fg_color })
            .font(font),
        );

    let mic_minus_btn = Rectangle::new()
        .width(110.0)
        .height(28.0)
        .radius(4.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let cur = Audio::read().microphone_volume() as i32;
            let target = (cur - 5).clamp(0, 100) as u8;
            Audio::set_microphone_volume(target);
        })
        .child(
            Text::new("󰍬 -5% Gain")
                .size(10.0)
                .color(fg_color)
                .font(font),
        );

    let mic_plus_btn = Rectangle::new()
        .width(110.0)
        .height(28.0)
        .radius(4.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let cur = Audio::read().microphone_volume() as i32;
            let target = (cur + 5).clamp(0, 100) as u8;
            Audio::set_microphone_volume(target);
        })
        .child(
            Text::new("󰍬 +5% Gain")
                .size(10.0)
                .color(fg_color)
                .font(font),
        );

    elements.push(Box::new(
        Row::new(vec![
            Box::new(mic_mute_btn),
            Box::new(mic_minus_btn),
            Box::new(mic_plus_btn),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    ));

    // 3. Audio Subsystem Telemetry Badge
    let info_card = Rectangle::new()
        .width(Parent)
        .height(34.0)
        .radius(6.0)
        .fill(surface_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .child(
            Row::new(vec![
                Box::new(Text::new("󰓃").size(13.0).color(amber_color).font(font)),
                Box::new(
                    Text::new("PipeWire Audio Server • Low-Latency Stream")
                        .size(10.0)
                        .color(steel_color)
                        .font(font),
                ),
            ])
            .gap(8.0)
            .align(Align::Center),
        );

    elements.push(Box::new(info_card));

    Box::new(Column::new(elements).width(Parent).gap(8.0))
}
