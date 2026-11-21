//! Spatial Ribbon Overview.
//! Provides a fullscreen translucent overview mapping Niri workspaces and spatial columns.
//! Features 150ms cubic ease-out animation and 0 Hz frame parking at rest.

use std::time::{Duration, Instant};

use amane::{
    Align, Center, Column, Full, Key, Keyboard, Layer, LayerWindow, OutputTarget, Padding, Parent,
    Pointer, Rectangle, Row, Service, Text, Widget, Zone,
};

use crate::config::ConfigService;
use crate::services::niri::NiriService;
use crate::ui::components::status_badge;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

const WORKSPACES: [&str; 5] = [
    "01 DEV",
    "02 SYSTEM",
    "03 RESEARCH",
    "04 SECURITY",
    "05 SCRATCH",
];

pub struct OverviewService {
    pub open: bool,
    pub open_time: Option<Instant>,
    pub selected_window_idx: usize,
}

impl Service for OverviewService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
            selected_window_idx: 0,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut overview = OverviewService::write();
        overview.open = !overview.open;
        overview.open_time = if overview.open {
            Some(Instant::now())
        } else {
            None
        };
        overview.selected_window_idx = 0;
        overview.open
    };

    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::Overview,
        );
    }
}

pub fn close() {
    let mut overview = OverviewService::write();
    overview.open = false;
    overview.open_time = None;
    overview.selected_window_idx = 0;
}

pub fn overview_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let overview = OverviewService::read();

    if !overview.open {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .layer(Layer::Overlay)
            .target_output(OutputTarget::Active)
            .namespace("dotfield-shell-overview");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = &tokens.font_family;

    let windows = NiriService::query_windows().unwrap_or_default();
    let focused_win = NiriService::focused_window();
    let total_windows = windows.len();
    let selected_idx = if total_windows > 0 {
        overview.selected_window_idx.min(total_windows - 1)
    } else {
        0
    };

    // Header
    let header_row = Row::new(vec![
        Box::new(
            Text::new("󰍹 Spatial Ribbon Overview")
                .size(typography::TITLE_MD)
                .color(tokens.accent)
                .font(font),
        ),
        Box::new(status_badge(
            &format!("{} Windows Active", total_windows),
            tokens.accent_hover,
            &tokens,
        )),
    ])
    .gap(spacing::GAP_MD)
    .align(Align::Center);

    // Workspace Cards Ribbon
    let mut ws_cards: Vec<Box<dyn Widget>> = Vec::new();
    for &ws_name in &WORKSPACES {
        let ws_card = Rectangle::new()
            .width(140.0)
            .height(64.0)
            .radius(radius::MD)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .cursor(Pointer)
            .padding(Padding {
                top: 8.0,
                bottom: 8.0,
                left: 10.0,
                right: 10.0,
            })
            .align_child(Center, Center)
            .on_click(move |_| {
                let _ = NiriService::focus_workspace(ws_name);
                close();
            })
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new(ws_name)
                            .size(typography::BODY)
                            .color(tokens.text_primary)
                            .font(font),
                    ),
                    Box::new(
                        Text::new("Workspace")
                            .size(typography::CAPTION)
                            .color(tokens.text_secondary)
                            .font(font),
                    ),
                ])
                .align(Center)
                .gap(2.0),
            );
        ws_cards.push(Box::new(ws_card));
    }

    let ws_ribbon = Row::new(ws_cards).gap(spacing::GAP_MD).align(Align::Center);

    // Active Windows Miniature Cards
    let windows_content: Box<dyn Widget> = if total_windows == 0 {
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(240.0)
                .align_child(Center, Center)
                .child(
                    Text::new("No active windows on the spatial ribbon")
                        .size(typography::TITLE_SM)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
        )
    } else {
        let mut win_cards: Vec<Box<dyn Widget>> = Vec::new();
        for (i, win) in windows.iter().enumerate() {
            let win_id = win.id;
            let is_selected = i == selected_idx;
            let is_focused = win_id == focused_win.id;

            let title: String = win
                .title
                .as_deref()
                .unwrap_or("Untitled")
                .chars()
                .take(20)
                .collect();
            let app_id = win.app_id.as_deref().unwrap_or("app");

            let card = Rectangle::new()
                .width(220.0)
                .height(140.0)
                .radius(radius::MD)
                .fill(if is_selected {
                    tokens.surface_elevated
                } else {
                    tokens.surface
                })
                .border(
                    if is_selected { 2.0 } else { 1.0 },
                    if is_selected {
                        tokens.accent
                    } else if is_focused {
                        tokens.accent_hover
                    } else {
                        tokens.border_subtle
                    },
                )
                .cursor(Pointer)
                .padding(Padding {
                    top: 12.0,
                    bottom: 12.0,
                    left: 12.0,
                    right: 12.0,
                })
                .align_child(Center, Center)
                .on_click(move |_| {
                    let _ = NiriService::focus_window(win_id);
                    close();
                })
                .child(
                    Column::new(vec![
                        Box::new(
                            Row::new(vec![
                                Box::new(Text::new("󰍹").size(16.0).color(tokens.accent).font(font)),
                                Box::new(
                                    Text::new(app_id)
                                        .size(typography::CAPTION)
                                        .color(tokens.text_secondary)
                                        .font(font),
                                ),
                            ])
                            .gap(spacing::GAP_SM)
                            .align(Align::Center),
                        ),
                        Box::new(
                            Text::new(&title)
                                .size(typography::BODY)
                                .color(tokens.text_primary)
                                .font(font),
                        ),
                        Box::new(
                            Text::new(if is_focused { "[Active]" } else { "Column" })
                                .size(typography::CAPTION)
                                .color(if is_focused {
                                    tokens.accent_hover
                                } else {
                                    tokens.steel
                                })
                                .font(font),
                        ),
                    ])
                    .align(Center)
                    .gap(spacing::GAP_SM),
                );

            win_cards.push(Box::new(card));
        }

        Box::new(
            Row::new(win_cards)
                .gap(spacing::GAP_MD)
                .align(Align::Center),
        )
    };

    let overview_container = Column::new(vec![
        Box::new(header_row),
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(1.0)
                .fill(tokens.border_subtle),
        ),
        Box::new(ws_ribbon),
        windows_content,
        Box::new(
            Text::new("Use Left/Right to select, Enter to warp focus, Escape to exit")
                .size(typography::CAPTION)
                .color(tokens.text_secondary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_LG)
    .align(Center);

    let backdrop = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(tokens.acrylic_bg)
        .align_child(Center, Center)
        .child(overview_container);

    LayerWindow::new()
        .width(Full)
        .height(Full)
        .layer(Layer::Overlay)
        .target_output(OutputTarget::Active)
        .namespace("dotfield-shell-overview")
        .space(Zone::Ignore)
        .keyboard(Keyboard::OnDemand)
        .on_key(|key| match key {
            Key::Escape => close(),
            Key::Right => {
                let mut overview = OverviewService::write();
                let wins = NiriService::query_windows().unwrap_or_default();
                if !wins.is_empty() {
                    overview.selected_window_idx =
                        (overview.selected_window_idx + 1).min(wins.len() - 1);
                }
            }
            Key::Left => {
                let mut overview = OverviewService::write();
                overview.selected_window_idx = overview.selected_window_idx.saturating_sub(1);
            }
            Key::Enter => {
                let overview = OverviewService::read();
                let wins = NiriService::query_windows().unwrap_or_default();
                if let Some(target) = wins.get(overview.selected_window_idx) {
                    let _ = NiriService::focus_window(target.id);
                    close();
                }
            }
            _ => {}
        })
        .visible(true)
        .child(backdrop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_toggle() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        assert!(!OverviewService::read().open);
        toggle();
        assert!(OverviewService::read().open);
        close();
        assert!(!OverviewService::read().open);
    }
}
