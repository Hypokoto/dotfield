use amane::{
    Align, Audio, Center, Color, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Scroll,
    Service, SinkType, Text, Widget,
};

use crate::config::ShellManifest;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

/// Renders the PipeWire Audio interactive widget.
pub fn render_audio_widget(manifest: &ShellManifest, font: &str) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);

    let audio = Audio::read();
    let vol = audio.volume();
    let is_muted = audio.muted();
    let mic_vol = audio.microphone_volume();
    let is_mic_muted = audio.microphone_muted();

    let mut elements: Vec<Box<dyn Widget>> = Vec::new();

    // Master Output Section (Speaker / Headphones)
    let output_header = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Text::new(if is_muted { "󰖁" } else { "󰕾" })
                        .size(typography::TITLE_SM)
                        .color(if is_muted {
                            tokens.danger
                        } else {
                            tokens.accent
                        })
                        .font(font),
                ),
                Box::new(
                    Text::new("PIPEWIRE OUTPUT")
                        .size(typography::BADGE)
                        .color(tokens.accent)
                        .font(font),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        ),
        Box::new(
            Text::new(if is_muted {
                "MUTED".to_string()
            } else {
                format!("{vol}%")
            })
            .size(typography::SECONDARY)
            .color(if is_muted {
                tokens.danger
            } else {
                tokens.accent_hover
            })
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
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(
            1.0,
            if is_muted {
                tokens.border_danger
            } else {
                tokens.border_subtle
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
                    .radius(radius::SM)
                    .fill(if is_muted {
                        tokens.danger_subtle
                    } else {
                        tokens.accent_subtle
                    }),
            ),
        ]));

    elements.push(Box::new(output_slider_card));

    // Output Controls: Mute toggle, Presets (25%, 50%, 75%, 100%)
    let mute_btn = Rectangle::new()
        .width(80.0)
        .height(28.0)
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
            Text::new(if is_muted { "󰖁 Unmute" } else { "󰕾 Mute" })
                .size(typography::BADGE)
                .color(if is_muted {
                    Color::WHITE
                } else {
                    tokens.text_primary
                })
                .font(font),
        );

    let mut preset_btns: Vec<Box<dyn Widget>> = vec![Box::new(mute_btn)];
    for preset in [25, 50, 75, 100] {
        let is_selected = !is_muted && (vol as i32 - preset as i32).abs() <= 5;
        let btn = Rectangle::new()
            .width(58.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_selected {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_selected {
                    tokens.border_focus
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| {
                Audio::set_volume(preset);
            })
            .child(
                Text::new(format!("{preset}%"))
                    .size(typography::BADGE)
                    .color(if is_selected {
                        Color::BLACK
                    } else {
                        tokens.text_primary
                    })
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

    // Output sinks (devices) selection section
    let sinks = audio.sinks();
    if !sinks.is_empty() {
        let sinks_header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(
                        Text::new("󰓃")
                            .size(typography::BODY_LG)
                            .color(tokens.accent)
                            .font(font),
                    ),
                    Box::new(
                        Text::new("OUTPUT DEVICES")
                            .size(typography::BADGE)
                            .color(tokens.accent)
                            .font(font),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            ),
            Box::new(
                Text::new(format!("{} AVAILABLE", sinks.len()))
                    .size(typography::CAPTION)
                    .color(tokens.text_secondary)
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
            let display_name = if sink_desc.chars().count() > 32 {
                format!("{}…", sink_desc.chars().take(31).collect::<String>())
            } else {
                sink_desc.clone()
            };

            let card = Rectangle::new()
                .width(Parent)
                .height(34.0)
                .radius(radius::SM)
                .fill(if is_default {
                    tokens.surface_selected
                } else {
                    tokens.surface_elevated
                })
                .border(
                    1.0,
                    if is_default {
                        tokens.border_focus
                    } else {
                        tokens.border_subtle
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
                    if amane::Notifications::read().running() {
                        amane::Notifications::notify(
                            "dotfield",
                            "Audio Output",
                            sink_desc.clone(),
                            amane::Urgency::Low,
                        );
                    } else {
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
                    }
                })
                .child(
                    Row::new(vec![
                        Box::new(
                            Row::new(vec![
                                Box::new(
                                    Text::new(icon_str)
                                        .size(typography::BODY_LG)
                                        .color(if is_default {
                                            tokens.accent
                                        } else {
                                            tokens.text_primary
                                        })
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(display_name)
                                        .size(typography::BADGE)
                                        .color(if is_default {
                                            tokens.text_primary
                                        } else {
                                            tokens.text_secondary
                                        })
                                        .font(font),
                                ),
                            ])
                            .gap(spacing::GAP_SM)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Text::new(if is_default { "ACTIVE" } else { "SELECT" })
                                .size(typography::CAPTION)
                                .color(if is_default {
                                    tokens.accent
                                } else {
                                    tokens.text_secondary
                                })
                                .font(font),
                        ),
                    ])
                    .width(Parent)
                    .justify(Justify::SpaceBetween)
                    .align(Align::Center),
                );

            sink_cards.push(Box::new(card));
        }

        elements.push(Box::new(
            Column::new(sink_cards).width(Parent).gap(spacing::GAP_XS),
        ));
    }

    // Microphone input section
    let mic_header = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Text::new(if is_mic_muted { "󰍭" } else { "󰍬" })
                        .size(typography::TITLE_SM)
                        .color(if is_mic_muted {
                            tokens.danger
                        } else {
                            tokens.accent
                        })
                        .font(font),
                ),
                Box::new(
                    Text::new("PIPEWIRE MICROPHONE")
                        .size(typography::BADGE)
                        .color(tokens.accent)
                        .font(font),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        ),
        Box::new(
            Text::new(if is_mic_muted {
                "MUTED".to_string()
            } else {
                format!("{mic_vol}%")
            })
            .size(typography::SECONDARY)
            .color(if is_mic_muted {
                tokens.danger
            } else {
                tokens.accent_hover
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
        .radius(radius::SM)
        .fill(tokens.surface_track)
        .border(
            1.0,
            if is_mic_muted {
                tokens.border_danger
            } else {
                tokens.border_subtle
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
                .radius(radius::SM)
                .fill(if is_mic_muted {
                    tokens.danger_subtle
                } else {
                    tokens.accent_subtle
                }),
        )]));

    elements.push(Box::new(mic_slider_card));

    // Mic Controls: Mic Mute toggle, Step -5%, Step +5%
    let mic_mute_btn = Rectangle::new()
        .width(110.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(if is_mic_muted {
            tokens.danger
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if is_mic_muted {
                tokens.border_danger
            } else {
                tokens.border_subtle
            },
        )
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
            .size(typography::BADGE)
            .color(if is_mic_muted {
                Color::WHITE
            } else {
                tokens.text_primary
            })
            .font(font),
        );

    let mic_minus_btn = Rectangle::new()
        .width(110.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let cur = Audio::read().microphone_volume() as i32;
            let target = (cur - 5).clamp(0, 100) as u8;
            Audio::set_microphone_volume(target);
        })
        .child(
            Text::new("󰍬 -5% Gain")
                .size(typography::BADGE)
                .color(tokens.text_primary)
                .font(font),
        );

    let mic_plus_btn = Rectangle::new()
        .width(110.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            let cur = Audio::read().microphone_volume() as i32;
            let target = (cur + 5).clamp(0, 100) as u8;
            Audio::set_microphone_volume(target);
        })
        .child(
            Text::new("󰍬 +5% Gain")
                .size(typography::BADGE)
                .color(tokens.text_primary)
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

    // Audio subsystem telemetry badge
    let info_card = Rectangle::new()
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
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰓃")
                        .size(typography::BODY_LG)
                        .color(tokens.accent)
                        .font(font),
                ),
                Box::new(
                    Text::new("PipeWire Audio Server • Low-Latency Stream")
                        .size(typography::BADGE)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        );

    elements.push(Box::new(info_card));

    Box::new(Column::new(elements).width(Parent).gap(spacing::GAP_SM))
}
