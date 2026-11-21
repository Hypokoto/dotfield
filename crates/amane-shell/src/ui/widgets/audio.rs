use amane::{
    Align, Audio, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Service, SinkType,
    Text, Widget,
};

use crate::config::ShellManifest;
use crate::ui::components;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

/// Renders the PipeWire Audio interactive widget.
pub fn render_audio_widget(manifest: &ShellManifest, _font: &str) -> Box<dyn Widget> {
    let tokens = ThemeTokens::from_manifest(manifest);

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
                        .size(typography::BODY_LG)
                        .color(if is_muted {
                            tokens.danger
                        } else {
                            tokens.accent
                        })
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Output Volume")
                        .size(typography::BODY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        ),
        Box::new(
            Text::new(if is_muted {
                "Muted".to_string()
            } else {
                format!("{vol}%")
            })
            .size(typography::SECONDARY)
            .color(if is_muted {
                tokens.danger
            } else {
                tokens.accent
            })
            .font(&tokens.font_mono),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(output_header));

    // Master Output Fluent Pill Slider (356px width with interactive mute button)
    let output_slider = components::fluent_slider(
        vol as f32 / 100.0,
        356.0,
        36.0,
        if is_muted { "󰖁" } else { "󰕾" },
        "Master Volume",
        is_muted,
        &tokens,
        |step| {
            let current = Audio::read().volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_volume(target);
        },
        Some(|| {
            Audio::toggle_mute();
        }),
    );
    elements.push(Box::new(output_slider));

    // 2. Output sinks (devices) selection section
    let sinks = audio.sinks();
    if !sinks.is_empty() {
        let sinks_header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(
                        Text::new("󰓃")
                            .size(typography::BODY)
                            .color(tokens.accent)
                            .font(&tokens.font_mono),
                    ),
                    Box::new(
                        Text::new("Output Devices")
                            .size(typography::BODY)
                            .color(tokens.text_primary)
                            .font(&tokens.font_sans),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            ),
            Box::new(
                Text::new(format!("{} available", sinks.len()))
                    .size(typography::CAPTION)
                    .color(tokens.text_secondary)
                    .font(&tokens.font_mono),
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
                    tokens.accent_tint
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
                                            tokens.text_secondary
                                        })
                                        .font(&tokens.font_mono),
                                ),
                                Box::new(
                                    Text::new(display_name)
                                        .size(typography::SECONDARY)
                                        .color(tokens.text_primary)
                                        .font(&tokens.font_sans),
                                ),
                            ])
                            .gap(spacing::GAP_SM)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Text::new(if is_default { "󰄬 Active" } else { "Select" })
                                .size(typography::CAPTION)
                                .color(if is_default {
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
                );

            sink_cards.push(Box::new(card));
        }

        elements.push(Box::new(
            Column::new(sink_cards).width(Parent).gap(spacing::GAP_XS),
        ));
    }

    // 3. Microphone input section
    let mic_header = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Text::new(if is_mic_muted { "󰍭" } else { "󰍬" })
                        .size(typography::BODY_LG)
                        .color(if is_mic_muted {
                            tokens.danger
                        } else {
                            tokens.accent
                        })
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("Microphone Input")
                        .size(typography::BODY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        ),
        Box::new(
            Text::new(if is_mic_muted {
                "Muted".to_string()
            } else {
                format!("{mic_vol}%")
            })
            .size(typography::SECONDARY)
            .color(if is_mic_muted {
                tokens.danger
            } else {
                tokens.accent
            })
            .font(&tokens.font_mono),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    elements.push(Box::new(mic_header));

    // Microphone Fluent Pill Slider (356px width with interactive mute button)
    let mic_slider = components::fluent_slider(
        mic_vol as f32 / 100.0,
        356.0,
        36.0,
        if is_mic_muted { "󰍭" } else { "󰍬" },
        "Input Gain",
        is_mic_muted,
        &tokens,
        |step| {
            let current = Audio::read().microphone_volume() as i32;
            let target = (current + step).clamp(0, 100) as u8;
            Audio::set_microphone_volume(target);
        },
        Some(|| {
            Audio::toggle_microphone_mute();
        }),
    );
    elements.push(Box::new(mic_slider));

    // 4. Audio subsystem telemetry badge
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
                        .font(&tokens.font_mono),
                ),
                Box::new(
                    Text::new("PipeWire Audio Server • Low-Latency Stream")
                        .size(typography::BADGE)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        );

    elements.push(Box::new(info_card));

    Box::new(Column::new(elements).width(Parent).gap(spacing::GAP_SM))
}
