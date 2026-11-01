use std::sync::LazyLock;
use std::time::Duration;

use amane::{
    Align, Apps, Color, Column, DesktopApp, Horizontal, Image, Key, Keyboard, Layer, LayerWindow,
    Padding, Parent, Pointer, Rectangle, Row, Service, Text, TextInput, Vertical, Widget, Zone,
};
use parking_lot::RwLock;

use crate::config::ShellManifest;

pub struct LauncherService {
    pub open: bool,
    pub query: String,
    pub selected: usize,
    pub hover_index: Option<usize>,
}

impl Service for LauncherService {
    fn new() -> Self {
        Self {
            open: false,
            query: String::new(),
            selected: 0,
            hover_index: None,
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
    let mut launcher = LauncherService::write();
    launcher.open = !launcher.open;
    launcher.query.clear();
    launcher.selected = 0;
    launcher.hover_index = None;
    TextInput::set_text("launcher-search", "");
}

pub fn close() {
    let mut launcher = LauncherService::write();
    launcher.open = false;
    launcher.query.clear();
    launcher.selected = 0;
    launcher.hover_index = None;
}

pub fn launcher_overlay() -> LayerWindow {
    let manifest = CONFIG.read().clone();
    let launcher = LauncherService::read();

    let is_visible = launcher.open && manifest.launcher.enabled;

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

    let visible_count = manifest.launcher.max_results;
    let total_results = matched_apps.len();

    // Adjust sliding window offset to keep selected item inside visible window
    let selected = if total_results == 0 {
        0
    } else {
        launcher.selected.min(total_results - 1)
    };

    let scroll_offset = if total_results <= visible_count {
        0
    } else if selected >= visible_count {
        (selected + 1).saturating_sub(visible_count)
    } else {
        0
    };

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Search Bar with icon prefix and sleek border
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
        })
        .on_submit(|_| {
            submit_selection();
        });

    let count_text = if total_results > 0 {
        format!("{}/{}", selected + 1, total_results)
    } else {
        "0".to_string()
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

    // 2. Results list with sliding viewport window
    let mut app_rows: Vec<Box<dyn Widget>> = Vec::new();
    let visible_slice = matched_apps
        .iter()
        .skip(scroll_offset)
        .take(visible_count)
        .enumerate();

    let hover_color = Color::from(manifest.palette.hover.as_str()); // Pale Amber #FFD166
    let steel_color = Color::from(manifest.palette.steel.as_str()); // Steel #8B949E
    let border_color_token = Color::from(manifest.palette.border.as_str()); // Iron #34343D

    for (rel_idx, app) in visible_slice {
        let abs_idx = scroll_offset + rel_idx;
        let is_selected = abs_idx == selected;
        let is_hovered = launcher.hover_index == Some(abs_idx);
        let launched = app.clone();

        let row_bg = if is_selected {
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

        let mut row_children: Vec<Box<dyn Widget>> = Vec::new();

        // Indicator pill for selected / hovered item
        let indicator = Rectangle::new()
            .width(3.0)
            .height(20.0)
            .radius(2.0)
            .fill(if is_selected {
                amber_color
            } else if is_hovered {
                hover_color
            } else {
                Color::TRANSPARENT
            });
        row_children.push(Box::new(indicator));

        // Icon with fallback glyph if missing
        let icon_widget: Box<dyn Widget> = match app.icon_path() {
            Some(path) => Box::new(
                Rectangle::new()
                    .width(24.0)
                    .height(24.0)
                    .fill(Image::contain(path)),
            ),
            None => Box::new(
                Rectangle::new()
                    .width(24.0)
                    .height(24.0)
                    .radius(4.0)
                    .fill(Color::from("#16161D"))
                    .border(1.0, Color::from("#2A2A34"))
                    .align_child(Align::Center, Align::Center)
                    .child(
                        Text::new("󰵆")
                            .size(13.0)
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
        row_children.push(icon_widget);

        // App Name with dynamic hover / selection contrast
        let title_color = if is_selected {
            amber_color
        } else if is_hovered {
            hover_color
        } else {
            fg_color
        };
        row_children.push(Box::new(
            Text::new(app.name())
                .size(12.0)
                .color(title_color)
                .font(&font),
        ));

        let left_group = Row::new(row_children).gap(8.0).align(Align::Center);
        let mut card_content: Vec<Box<dyn Widget>> = vec![Box::new(left_group)];

        // Badge on right for selected item
        if is_selected {
            let enter_badge = Rectangle::new()
                .width(62.0)
                .height(20.0)
                .radius(4.0)
                .fill(Color::from("#0A0A0C"))
                .border(1.0, border_color_token)
                .padding(Padding {
                    top: 1.0,
                    bottom: 1.0,
                    left: 6.0,
                    right: 6.0,
                })
                .align_child(Align::Center, Align::Center)
                .child(
                    Text::new("↵ Launch")
                        .size(9.0)
                        .color(amber_color)
                        .font(&font),
                );
            card_content.push(Box::new(enter_badge));
        }

        let app_card = Rectangle::new()
            .width(Parent)
            .height(38.0)
            .radius(8.0)
            .fill(row_bg)
            .border(1.0, card_border)
            .cursor(Pointer)
            .padding(Padding {
                top: 4.0,
                bottom: 4.0,
                left: 6.0,
                right: 10.0,
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
            .child(
                Row::new(card_content)
                    .width(Parent)
                    .justify(amane::Justify::SpaceBetween)
                    .align(Align::Center),
            );

        app_rows.push(Box::new(app_card));
    }

    if !app_rows.is_empty() {
        let app_list_column = Column::new(app_rows).width(Parent).gap(5.0);
        let scrollable_container = Rectangle::new()
            .width(Parent)
            .height(340.0)
            .cursor(Pointer)
            .on_scroll(move |scroll: amane::Scroll| {
                let mut launcher = LauncherService::write();
                if scroll.y < 0.0 {
                    if total_results > 0 && launcher.selected + 1 < total_results {
                        launcher.selected += 1;
                    }
                } else {
                    launcher.selected = launcher.selected.saturating_sub(1);
                }
            })
            .child(app_list_column);

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

    // 3. Footer Bar with Navigation Hints
    let footer_text =
        Text::new("[↑/↓] Navigate   [Home/End] First/Last   [↵] Launch   [Esc] Dismiss")
            .size(10.0)
            .color(Color::from(manifest.palette.muted.as_str()))
            .font(&font);

    let footer_row = Rectangle::new()
        .width(Parent)
        .height(20.0)
        .align_child(Align::Center, Align::Center)
        .child(footer_text);

    rows.push(Box::new(footer_row));

    let dialog = Rectangle::new()
        .width(440.0)
        .height(470.0)
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
        .width(440.0)
        .height(470.0)
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
            if count > 0 && launcher.selected + 1 < count {
                launcher.selected += 1;
            }
        }
        Key::Up => {
            let mut launcher = LauncherService::write();
            launcher.selected = launcher.selected.saturating_sub(1);
        }
        Key::Home => {
            let mut launcher = LauncherService::write();
            launcher.selected = 0;
        }
        Key::End => {
            let mut launcher = LauncherService::write();
            if count > 0 {
                launcher.selected = count - 1;
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
