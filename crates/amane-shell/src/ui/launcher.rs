use std::time::Duration;

use amane::{
    Align, Apps, Color, Column, DesktopApp, Horizontal, Image, Key, Keyboard, Layer, LayerWindow,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, TextInput, Vertical,
    Widget, Zone,
};

use crate::config::ConfigService;

pub const GRID_COLS: usize = 3;
pub const GRID_VISIBLE_ROWS: usize = 4;

pub struct LauncherService {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub hover_index: Option<usize>,
    pub scroll_row: usize,
}

impl Service for LauncherService {
    fn new() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            hover_index: None,
            scroll_row: 0,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let mut launcher = LauncherService::write();
    launcher.open = !launcher.open;
    launcher.query.clear();
    launcher.selected = 0;
    launcher.scroll_row = 0;
    launcher.hover_index = None;
    TextInput::set_text("launcher-search", "");
}

pub fn close() {
    let mut launcher = LauncherService::write();
    launcher.open = false;
    launcher.query.clear();
    launcher.selected = 0;
    launcher.scroll_row = 0;
    launcher.hover_index = None;
}

pub fn launcher_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let launcher = LauncherService::read();

    let is_visible = launcher.open && manifest.launcher.enabled;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .target_output(OutputTarget::Active)
            .namespace("dotfield-shell-launcher");
    }

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let font = manifest.theme.font_family.clone();

    let apps = Apps::read();
    let mut matched_apps: Vec<DesktopApp> = Vec::new();

    let query_lower = launcher.query.to_lowercase();
    for app in apps.list() {
        if query_lower.is_empty()
            || app.name().to_lowercase().contains(&query_lower)
            || app
                .description()
                .is_some_and(|d| d.to_lowercase().contains(&query_lower))
        {
            matched_apps.push(app.clone());
        }
    }

    let total_results = matched_apps.len();
    let total_rows = total_results.div_ceil(GRID_COLS);

    // Adjust sliding window offset to keep selected item inside visible window
    let selected = if total_results == 0 {
        0
    } else {
        launcher.selected.min(total_results - 1)
    };

    let cur_row = selected / GRID_COLS;
    let mut scroll_row = launcher.scroll_row;
    if cur_row < scroll_row {
        scroll_row = cur_row;
    } else if cur_row >= scroll_row + GRID_VISIBLE_ROWS {
        scroll_row = cur_row + 1 - GRID_VISIBLE_ROWS;
    }

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // Search bar with icon prefix and subtle border
    let search_icon = Text::new("󰍉").size(15.0).color(amber_color).font(&font);
    let search_input = TextInput::new("launcher-search")
        .size(13.0)
        .color(fg_color)
        .placeholder("Search applications or commands...")
        .focused()
        .on_change(|text| {
            let mut launcher = LauncherService::write();
            launcher.query = text;
            launcher.selected = 0;
            launcher.scroll_row = 0;
        })
        .on_submit(|_| {
            submit_selection();
        });

    let count_text = if total_results > 0 {
        format!("{}/{} apps", selected + 1, total_results)
    } else {
        "0 apps".to_string()
    };
    let count_badge = Text::new(count_text)
        .size(10.0)
        .color(Color::from(manifest.palette.muted.as_str()))
        .font(&font);

    let search_row = Row::new(vec![
        Box::new(search_icon),
        Box::new(search_input),
        Box::new(count_badge),
    ])
    .gap(10.0)
    .align(Align::Center);

    let search_box = Rectangle::new()
        .width(Parent)
        .height(42.0)
        .radius(8.0)
        .fill(overlay_color)
        .border(1.0, Color::from("#34343D"))
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 14.0,
            right: 14.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(search_row);

    rows.push(Box::new(search_box));

    // Results grid: 3 columns x 4 rows
    let hover_color = Color::from(manifest.palette.hover.as_str()); // Pale Amber #FFD166
    let steel_color = Color::from(manifest.palette.steel.as_str()); // Steel #8B949E

    let mut grid_rows: Vec<Box<dyn Widget>> = Vec::new();
    let start_idx = scroll_row * GRID_COLS;

    for row_idx in 0..GRID_VISIBLE_ROWS {
        let mut row_cards: Vec<Box<dyn Widget>> = Vec::new();
        for col_idx in 0..GRID_COLS {
            let abs_idx = start_idx + (row_idx * GRID_COLS) + col_idx;
            if abs_idx < total_results {
                let app = &matched_apps[abs_idx];
                let is_selected = abs_idx == selected;
                let is_hovered = launcher.hover_index == Some(abs_idx);
                let launched = app.clone();

                let card_bg = if is_selected {
                    Color::from("#262632") // Elevated Graphite
                } else if is_hovered {
                    Color::from("#202028") // Subtle hover surface
                } else {
                    overlay_color // Graphite #1C1C23
                };

                let card_border = if is_selected {
                    amber_color // Industrial Amber #FFB000
                } else if is_hovered {
                    Color::from("#4E4E5C") // Focused steel-iron border
                } else {
                    Color::from("#2A2A34") // Restrained separator
                };

                let title_color = if is_selected {
                    amber_color
                } else if is_hovered {
                    hover_color
                } else {
                    fg_color
                };

                // Indicator pill for selected / hovered item
                let indicator =
                    Rectangle::new()
                        .width(3.0)
                        .height(40.0)
                        .radius(1.5)
                        .fill(if is_selected {
                            amber_color
                        } else if is_hovered {
                            hover_color
                        } else {
                            Color::TRANSPARENT
                        });

                // Icon with fallback glyph if missing
                let icon_widget: Box<dyn Widget> = match app.icon_path() {
                    Some(path) => Box::new(
                        Rectangle::new()
                            .width(28.0)
                            .height(28.0)
                            .fill(Image::contain(path)),
                    ),
                    None => Box::new(
                        Rectangle::new()
                            .width(28.0)
                            .height(28.0)
                            .radius(4.0)
                            .fill(Color::from("#16161D"))
                            .border(1.0, Color::from("#2A2A34"))
                            .align_child(Align::Center, Align::Center)
                            .child(
                                Text::new("󰵆")
                                    .size(14.0)
                                    .color(if is_selected {
                                        amber_color
                                    } else if is_hovered {
                                        hover_color
                                    } else {
                                        steel_color
                                    })
                                    .font(&font),
                            ),
                    ),
                };

                // Truncate name and description cleanly
                let app_name: String = app.name().chars().take(18).collect();
                let app_desc: String = app
                    .description()
                    .map(|d| d.chars().take(22).collect())
                    .unwrap_or_else(|| "Application".to_string());

                let text_col = Column::new(vec![
                    Box::new(
                        Text::new(app_name)
                            .size(11.5)
                            .color(title_color)
                            .font(&font),
                    ),
                    Box::new(Text::new(app_desc).size(8.5).color(steel_color).font(&font)),
                ])
                .gap(2.0)
                .width(140.0)
                .align(Align::Start);

                let card_content =
                    Row::new(vec![Box::new(indicator), icon_widget, Box::new(text_col)])
                        .gap(7.0)
                        .align(Align::Center);

                let app_card = Rectangle::new()
                    .width(204.0)
                    .height(64.0)
                    .radius(8.0)
                    .fill(card_bg)
                    .border(if is_selected { 1.5 } else { 1.0 }, card_border)
                    .cursor(Pointer)
                    .padding(Padding {
                        top: 6.0,
                        bottom: 6.0,
                        left: 6.0,
                        right: 8.0,
                    })
                    .align_child(Align::Start, Align::Center)
                    .on_hover(move |inside| {
                        let mut launcher = LauncherService::write();
                        if inside {
                            launcher.hover_index = Some(abs_idx);
                        } else if launcher.hover_index == Some(abs_idx) {
                            launcher.hover_index = None;
                        }
                    })
                    .on_click(move |_| {
                        launched.launch();
                        close();
                    })
                    .child(card_content);

                row_cards.push(Box::new(app_card));
            } else {
                let placeholder = Rectangle::new()
                    .width(204.0)
                    .height(64.0)
                    .fill(Color::TRANSPARENT);
                row_cards.push(Box::new(placeholder));
            }
        }
        grid_rows.push(Box::new(
            Row::new(row_cards)
                .width(Parent)
                .gap(10.0)
                .align(Align::Center),
        ));
    }

    if total_results > 0 {
        let app_grid_column = Column::new(grid_rows).width(Parent).gap(8.0);
        let scrollable_container = Rectangle::new()
            .width(Parent)
            .height(280.0)
            .cursor(Pointer)
            .on_scroll(move |scroll: amane::Scroll| {
                let mut launcher = LauncherService::write();
                if scroll.y < 0.0 {
                    if launcher.scroll_row + 1 < total_rows {
                        launcher.scroll_row += 1;
                        let cur_row = launcher.selected / GRID_COLS;
                        if cur_row < launcher.scroll_row {
                            launcher.selected = (launcher.scroll_row * GRID_COLS
                                + (launcher.selected % GRID_COLS))
                                .min(total_results - 1);
                        }
                    }
                } else if launcher.scroll_row > 0 {
                    launcher.scroll_row = launcher.scroll_row.saturating_sub(1);
                    let cur_row = launcher.selected / GRID_COLS;
                    if cur_row >= launcher.scroll_row + GRID_VISIBLE_ROWS {
                        launcher.selected = ((launcher.scroll_row + GRID_VISIBLE_ROWS - 1)
                            * GRID_COLS
                            + (launcher.selected % GRID_COLS))
                            .min(total_results - 1);
                    }
                }
            })
            .child(app_grid_column);

        rows.push(Box::new(scrollable_container));
    } else if !launcher.query.trim().is_empty() {
        let empty_card = Rectangle::new()
            .width(Parent)
            .height(60.0)
            .radius(8.0)
            .fill(overlay_color)
            .border(1.0, Color::from("#34343D"))
            .align_child(Align::Center, Align::Center)
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new("󰛐")
                            .size(16.0)
                            .color(Color::from(manifest.palette.muted.as_str()))
                            .font(&font),
                    ),
                    Box::new(
                        Text::new(format!("No applications matching \"{}\"", launcher.query))
                            .size(11.0)
                            .color(Color::from(manifest.palette.muted.as_str()))
                            .font(&font),
                    ),
                ])
                .gap(4.0)
                .align(Align::Center),
            );
        rows.push(Box::new(empty_card));
    }

    // Footer bar with navigation hints
    let footer_text =
        Text::new("[Arrows] Navigate Grid   [PgUp/PgDn] Page   [↵] Launch   [Esc] Dismiss")
            .size(9.5)
            .color(Color::from(manifest.palette.muted.as_str()))
            .font(&font);

    let footer_row = Rectangle::new()
        .width(Parent)
        .height(20.0)
        .align_child(Align::Center, Align::Center)
        .child(footer_text);

    rows.push(Box::new(footer_row));

    let dialog = Rectangle::new()
        .width(660.0)
        .height(460.0)
        .radius(12.0)
        .fill(bg_color)
        .border(1.0, Color::from("#34343D"))
        .padding(Padding {
            top: 14.0,
            bottom: 12.0,
            left: 14.0,
            right: 14.0,
        })
        .child(Column::new(rows).width(Parent).gap(10.0));

    LayerWindow::new()
        .width(660.0)
        .height(460.0)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .layer(Layer::Overlay)
        .keyboard(Keyboard::OnDemand)
        .on_key(move |key| {
            handle_key(key, total_results);
        })
        .space(Zone::Ignore)
        .visible(is_visible)
        .namespace("dotfield-shell-launcher")
        .child(dialog)
}

fn handle_key(key: Key, count: usize) {
    match key {
        Key::Down => {
            let mut launcher = LauncherService::write();
            if count > 0 {
                if launcher.selected + GRID_COLS < count {
                    launcher.selected += GRID_COLS;
                } else if launcher.selected < count - 1 {
                    launcher.selected = count - 1;
                }
                let cur_row = launcher.selected / GRID_COLS;
                if cur_row >= launcher.scroll_row + GRID_VISIBLE_ROWS {
                    launcher.scroll_row = cur_row + 1 - GRID_VISIBLE_ROWS;
                }
            }
        }
        Key::Up => {
            let mut launcher = LauncherService::write();
            if launcher.selected >= GRID_COLS {
                launcher.selected -= GRID_COLS;
            } else {
                launcher.selected = 0;
            }
            let cur_row = launcher.selected / GRID_COLS;
            if cur_row < launcher.scroll_row {
                launcher.scroll_row = cur_row;
            }
        }
        Key::Right => {
            let mut launcher = LauncherService::write();
            if count > 0 && launcher.selected + 1 < count {
                launcher.selected += 1;
                let cur_row = launcher.selected / GRID_COLS;
                if cur_row >= launcher.scroll_row + GRID_VISIBLE_ROWS {
                    launcher.scroll_row = cur_row + 1 - GRID_VISIBLE_ROWS;
                }
            }
        }
        Key::Left => {
            let mut launcher = LauncherService::write();
            launcher.selected = launcher.selected.saturating_sub(1);
            let cur_row = launcher.selected / GRID_COLS;
            if cur_row < launcher.scroll_row {
                launcher.scroll_row = cur_row;
            }
        }
        Key::PageDown => {
            let mut launcher = LauncherService::write();
            if count > 0 {
                let step = GRID_COLS * GRID_VISIBLE_ROWS;
                launcher.selected = (launcher.selected + step).min(count - 1);
                let cur_row = launcher.selected / GRID_COLS;
                if cur_row >= launcher.scroll_row + GRID_VISIBLE_ROWS {
                    launcher.scroll_row = cur_row + 1 - GRID_VISIBLE_ROWS;
                }
            }
        }
        Key::PageUp => {
            let mut launcher = LauncherService::write();
            let step = GRID_COLS * GRID_VISIBLE_ROWS;
            launcher.selected = launcher.selected.saturating_sub(step);
            let cur_row = launcher.selected / GRID_COLS;
            if cur_row < launcher.scroll_row {
                launcher.scroll_row = cur_row;
            }
        }
        Key::Home => {
            let mut launcher = LauncherService::write();
            launcher.selected = 0;
            launcher.scroll_row = 0;
        }
        Key::End => {
            let mut launcher = LauncherService::write();
            if count > 0 {
                launcher.selected = count - 1;
                let cur_row = launcher.selected / GRID_COLS;
                if cur_row >= launcher.scroll_row + GRID_VISIBLE_ROWS {
                    launcher.scroll_row = cur_row + 1 - GRID_VISIBLE_ROWS;
                }
            }
        }
        Key::Escape => {
            close();
        }
        Key::Enter => {
            submit_selection();
        }
        _ => {}
    }
}

fn submit_selection() {
    let launcher = LauncherService::read();
    let apps = Apps::read();
    let query_lower = launcher.query.to_lowercase();
    let selected = launcher.selected;

    let mut matched_apps: Vec<DesktopApp> = Vec::new();
    for app in apps.list() {
        if query_lower.is_empty()
            || app.name().to_lowercase().contains(&query_lower)
            || app
                .description()
                .is_some_and(|d| d.to_lowercase().contains(&query_lower))
        {
            matched_apps.push(app.clone());
        }
    }

    drop(launcher); // Release read lock before close() acquires write lock

    if let Some(target) = matched_apps.get(selected) {
        target.launch();
        close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_service_default() {
        let svc = LauncherService::new();
        assert!(!svc.open);
        assert!(svc.query.is_empty());
        assert_eq!(svc.selected, 0);
        assert_eq!(svc.scroll_row, 0);
    }

    #[test]
    fn test_launcher_toggle_and_close() {
        close();
        assert!(!LauncherService::read().open);

        toggle();
        assert!(LauncherService::read().open);

        close();
        assert!(!LauncherService::read().open);
    }
}
