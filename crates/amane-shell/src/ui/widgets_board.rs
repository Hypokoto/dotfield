//! Left Slide-In Widgets Board.
//! Displays host info, dynamic quotes, telemetry gauges, and Cava audio visualizer.
//! Features 180ms cubic ease-out slide-in from the left screen edge and 0 Hz frame parking when at rest.

use std::time::{Duration, Instant};

use amane::{
    Align, Center, Color, Column, Full, Horizontal, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};

use crate::config::ConfigService;
use crate::services::cava::CavaService;
use crate::services::telemetry::TelemetryService;
use crate::ui::components::status_badge;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub struct WidgetsBoardService {
    pub open: bool,
    pub open_time: Option<Instant>,
}

impl Service for WidgetsBoardService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut board = WidgetsBoardService::write();
        board.open = !board.open;
        board.open_time = if board.open {
            Some(Instant::now())
        } else {
            None
        };
        board.open
    };

    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::Widgets,
        );
    }
}

pub fn close() {
    let mut board = WidgetsBoardService::write();
    board.open = false;
    board.open_time = None;
}

pub fn widgets_board_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let board = WidgetsBoardService::read();

    if !board.open {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .layer(Layer::Overlay)
            .target_output(OutputTarget::Active)
            .namespace("dotfield-shell-widgets");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = &tokens.font_family;

    let dock_height = manifest.bar.height as f32;
    let is_bottom_bar = manifest.bar.position == "bottom";

    // Header Card
    let username = std::env::var("USER").unwrap_or_else(|_| "developer".to_string());
    let hostname = std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "blacksite".to_string());

    let header_card = Rectangle::new()
        .width(Parent)
        .height(68.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Row::new(vec![
                Box::new(
                    Rectangle::new()
                        .width(44.0)
                        .height(44.0)
                        .radius(radius::PILL)
                        .fill(tokens.surface_elevated)
                        .border(1.0, tokens.accent)
                        .align_child(Center, Center)
                        .child(Text::new("󰣇").size(24.0).color(tokens.accent).font(font)),
                ),
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new(format!("{}@{}", username, hostname))
                                .size(typography::TITLE_SM)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new("Dotfield / Amane Desktop Suite")
                                .size(typography::CAPTION)
                                .color(tokens.text_secondary)
                                .font(font),
                        ),
                    ])
                    .gap(2.0),
                ),
            ])
            .gap(spacing::GAP_MD)
            .align(Align::Center),
        );

    // Audio / Spectrum Visualizer Card
    let spectrum = CavaService::spectrum();
    let mut bar_widgets: Vec<Box<dyn Widget>> = Vec::new();
    for &val in spectrum.iter() {
        let bar_h = (val * 48.0).clamp(4.0, 48.0);
        let bar = Rectangle::new()
            .width(8.0)
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

    let spectrum_row = Row::new(bar_widgets).gap(4.0).align(Align::End);

    let audio_card = Rectangle::new()
        .width(Parent)
        .height(112.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰝚 Audio Spectrum")
                                .size(typography::TITLE_SM)
                                .color(tokens.accent)
                                .font(font),
                        ),
                        Box::new(status_badge(
                            if CavaService::is_audio_active() {
                                "PLAYING"
                            } else {
                                "IDLE"
                            },
                            if CavaService::is_audio_active() {
                                tokens.success
                            } else {
                                tokens.steel
                            },
                            &tokens,
                        )),
                    ])
                    .width(Parent)
                    .align(Align::Center),
                ),
                Box::new(
                    Rectangle::new()
                        .width(Parent)
                        .height(56.0)
                        .fill(tokens.surface_elevated)
                        .radius(radius::SM)
                        .padding(Padding {
                            top: 4.0,
                            bottom: 4.0,
                            left: 8.0,
                            right: 8.0,
                        })
                        .align_child(Center, Align::End)
                        .child(spectrum_row),
                ),
            ])
            .gap(spacing::GAP_SM),
        );

    // Quote / Manifesto Card
    let quote_card = Rectangle::new()
        .width(Parent)
        .height(84.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("󰃀 Controlled Chaos")
                        .size(typography::TITLE_SM)
                        .color(tokens.accent_hover)
                        .font(font),
                ),
                Box::new(
                    Text::new("\"Organic shapes, unexpected layout, visual silence, and clarity. A focused workspace with controlled visual tension.\"")
                        .size(typography::BODY - 1.0)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
            ])
            .gap(spacing::GAP_XS),
        );

    // Telemetry Card
    let telemetry = TelemetryService::read();
    let cpu_pct = telemetry.cpu_percent as f32;
    let ram_pct = telemetry.mem_percent as f32;

    let telemetry_card = Rectangle::new()
        .width(Parent)
        .height(108.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("󰄪 System Telemetry")
                        .size(typography::TITLE_SM)
                        .color(tokens.accent)
                        .font(font),
                ),
                Box::new(render_gauge_row(
                    "CPU Load",
                    cpu_pct,
                    tokens.accent,
                    font,
                    &tokens,
                )),
                Box::new(render_gauge_row(
                    "Memory",
                    ram_pct,
                    tokens.accent_hover,
                    font,
                    &tokens,
                )),
            ])
            .gap(spacing::GAP_SM),
        );

    // Footer actions
    let close_btn = Rectangle::new()
        .width(Parent)
        .height(36.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(|_| {
            close();
        })
        .child(
            Text::new("Close Widgets (Esc)")
                .size(typography::CAPTION)
                .color(tokens.text_secondary)
                .font(font),
        );

    let board_column = Column::new(vec![
        Box::new(header_card),
        Box::new(audio_card),
        Box::new(quote_card),
        Box::new(telemetry_card),
        Box::new(close_btn),
    ])
    .gap(spacing::GAP_MD)
    .width(Parent);

    let container = Rectangle::new()
        .width(360.0)
        .height(Parent)
        .fill(tokens.bg)
        .border(1.0, tokens.border_subtle)
        .radius(radius::LG)
        .padding(Padding {
            top: spacing::PAD_LG,
            bottom: spacing::PAD_LG,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(board_column);

    let top_margin = 12;
    let bottom_margin = if is_bottom_bar {
        (dock_height + 12.0) as i32
    } else {
        12
    };

    LayerWindow::new()
        .width(360.0)
        .height(Full)
        .anchor_horizontal(Horizontal::Left)
        .anchor_vertical(Vertical::Top)
        .margin(Margin {
            top: top_margin,
            right: 0,
            bottom: bottom_margin,
            left: 12,
        })
        .layer(Layer::Overlay)
        .target_output(OutputTarget::Active)
        .namespace("dotfield-shell-widgets")
        .space(Zone::Ignore)
        .keyboard(Keyboard::OnDemand)
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .visible(true)
        .child(container)
}

fn render_gauge_row(
    label: &str,
    pct: f32,
    gauge_color: Color,
    font: &str,
    tokens: &ThemeTokens,
) -> Row {
    let bar_width = 160.0;
    let filled_width = (bar_width * (pct / 100.0)).clamp(0.0, bar_width);

    let gauge_bg = Rectangle::new()
        .width(bar_width)
        .height(8.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .child(
            Rectangle::new()
                .width(filled_width)
                .height(8.0)
                .radius(radius::PILL)
                .fill(gauge_color),
        );

    Row::new(vec![
        Box::new(
            Text::new(label)
                .size(typography::BODY - 1.0)
                .color(tokens.text_secondary)
                .font(font),
        ),
        Box::new(gauge_bg),
        Box::new(
            Text::new(format!("{:.0}%", pct))
                .size(typography::CAPTION)
                .color(tokens.text_primary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widgets_board_toggle() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        assert!(!WidgetsBoardService::read().open);
        toggle();
        assert!(WidgetsBoardService::read().open);
        close();
        assert!(!WidgetsBoardService::read().open);
    }
}
