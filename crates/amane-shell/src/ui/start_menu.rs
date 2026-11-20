//! Centered Start Menu & Application Grid.
//! Windows 11 / Material You style centered bottom-dock anchored flyout.
//! Provides search input, 4x3 pinned grid, A-Z application list, and session power controls.

use std::time::{Duration, Instant};

use amane::{
    Align, Apps, Center, Color, Column, DesktopApp, Horizontal, Key, Keyboard, Layer, LayerWindow,
    Margin, OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, TextInput,
    Vertical, Widget, Zone,
};

use crate::config::ConfigService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub const PINNED_COLS: usize = 4;
pub const PINNED_ROWS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartMenuViewMode {
    Pinned,
    AllApps,
}

pub struct StartMenuService {
    pub open: bool,
    pub open_time: Option<Instant>,
    pub query: String,
    pub selected: usize,
    pub view_mode: StartMenuViewMode,
}

impl Service for StartMenuService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
            query: String::new(),
            selected: 0,
            view_mode: StartMenuViewMode::Pinned,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut menu = StartMenuService::write();
        menu.open = !menu.open;
        menu.open_time = if menu.open {
            Some(Instant::now())
        } else {
            None
        };
        menu.query.clear();
        menu.selected = 0;
        menu.view_mode = StartMenuViewMode::Pinned;
        TextInput::set_text("start-menu-search", "");
        menu.open
    };

    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::StartMenu,
        );
    }
}

pub fn close() {
    let mut menu = StartMenuService::write();
    menu.open = false;
    menu.open_time = None;
    menu.query.clear();
    menu.selected = 0;
}

pub fn start_menu_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let menu = StartMenuService::read();

    if !menu.open {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .layer(Layer::Overlay)
            .target_output(OutputTarget::Active)
            .namespace("dotfield-shell-start-menu");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let font = &tokens.font_family;

    let dock_height = manifest.bar.height as f32;
    let is_bottom_bar = manifest.bar.position == "bottom";

    // 1. Top Search Bar
    let search_input = TextInput::new("start-menu-search")
        .size(typography::BODY)
        .color(tokens.text_primary)
        .placeholder("Search apps, settings, and files...")
        .focused()
        .on_change(|text| {
            let mut menu = StartMenuService::write();
            menu.query = text;
            menu.selected = 0;
            if !menu.query.is_empty() {
                menu.view_mode = StartMenuViewMode::AllApps;
            }
        });

    let search_bar = Rectangle::new()
        .width(Parent)
        .height(44.0)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .align_child(Align::Start, Align::Center)
        .child(
            Row::new(vec![
                Box::new(Text::new("").size(14.0).color(tokens.accent).font(font)),
                Box::new(search_input),
            ])
            .gap(spacing::GAP_MD)
            .width(Parent)
            .align(Align::Center),
        );

    // 2. Apps Content: Pinned Grid or Search/AllApps List
    let apps = Apps::read();
    let query_lower = menu.query.trim().to_lowercase();

    let matched_apps: Vec<DesktopApp> = apps
        .list()
        .iter()
        .filter(|&a| {
            query_lower.is_empty()
                || a.name().to_lowercase().contains(&query_lower)
                || a.description()
                    .is_some_and(|d| d.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .collect();

    let total_results = matched_apps.len();
    let current_view = menu.view_mode;

    let header_row = Row::new(vec![
        Box::new(
            Text::new(
                if current_view == StartMenuViewMode::Pinned && query_lower.is_empty() {
                    "Pinned Apps"
                } else {
                    "All Applications"
                },
            )
            .size(typography::TITLE_SM)
            .color(tokens.text_primary)
            .font(font),
        ),
        Box::new(
            Rectangle::new()
                .width(88.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .padding(Padding {
                    top: 2.0,
                    bottom: 2.0,
                    left: 8.0,
                    right: 8.0,
                })
                .cursor(Pointer)
                .on_click(|_| {
                    let mut menu = StartMenuService::write();
                    menu.view_mode = match menu.view_mode {
                        StartMenuViewMode::Pinned => StartMenuViewMode::AllApps,
                        StartMenuViewMode::AllApps => StartMenuViewMode::Pinned,
                    };
                })
                .child(
                    Text::new(if current_view == StartMenuViewMode::Pinned {
                        "All apps ❯"
                    } else {
                        "❮ Pinned"
                    })
                    .size(typography::CAPTION)
                    .color(tokens.accent)
                    .font(font),
                ),
        ),
    ])
    .width(Parent)
    .align(Align::Center);

    let grid_content: Box<dyn Widget> = if total_results == 0 {
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(340.0)
                .align_child(Center, Center)
                .child(
                    Text::new("No applications found")
                        .size(typography::BODY)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
        )
    } else {
        let mut rows: Vec<Box<dyn Widget>> = Vec::new();
        let max_items = (PINNED_COLS * PINNED_ROWS).min(total_results);

        for r in 0..PINNED_ROWS {
            let mut col_items: Vec<Box<dyn Widget>> = Vec::new();
            for c in 0..PINNED_COLS {
                let idx = r * PINNED_COLS + c;
                if idx < max_items {
                    let app = &matched_apps[idx];
                    let is_selected = idx == menu.selected;
                    let launched_app = app.clone();

                    let icon_widget: Box<dyn Widget> = match app.icon_path() {
                        Some(path) => Box::new(
                            Rectangle::new()
                                .width(28.0)
                                .height(28.0)
                                .fill(amane::Image::contain(path)),
                        ),
                        None => Box::new(
                            Rectangle::new()
                                .width(28.0)
                                .height(28.0)
                                .radius(radius::SM)
                                .fill(tokens.surface)
                                .align_child(Center, Center)
                                .child(Text::new("󰵆").size(16.0).color(tokens.accent).font(font)),
                        ),
                    };

                    let app_name: String = app.name().chars().take(12).collect();

                    let tile = Rectangle::new()
                        .width(116.0)
                        .height(84.0)
                        .radius(radius::MD)
                        .fill(if is_selected {
                            tokens.surface_elevated
                        } else {
                            tokens.surface
                        })
                        .border(
                            1.0,
                            if is_selected {
                                tokens.accent
                            } else {
                                tokens.border_subtle
                            },
                        )
                        .cursor(Pointer)
                        .padding(Padding {
                            top: 8.0,
                            bottom: 8.0,
                            left: 4.0,
                            right: 4.0,
                        })
                        .align_child(Center, Center)
                        .on_click(move |_| {
                            launched_app.launch();
                            close();
                        })
                        .child(
                            Column::new(vec![
                                icon_widget,
                                Box::new(
                                    Text::new(&app_name)
                                        .size(typography::CAPTION)
                                        .color(tokens.text_primary)
                                        .font(font),
                                ),
                            ])
                            .align(Center)
                            .gap(spacing::GAP_XS),
                        );

                    col_items.push(Box::new(tile));
                } else {
                    // Empty placeholder
                    col_items.push(Box::new(Rectangle::new().width(116.0).height(84.0)));
                }
            }
            rows.push(Box::new(
                Row::new(col_items)
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
            ));
        }

        Box::new(
            Column::new(rows)
                .gap(spacing::GAP_SM)
                .width(Parent)
                .align(Align::Center),
        )
    };

    // 3. User & Session Footer
    let username = std::env::var("USER").unwrap_or_else(|_| "developer".to_string());
    let user_info = Row::new(vec![
        Box::new(
            Rectangle::new()
                .width(32.0)
                .height(32.0)
                .radius(radius::PILL)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.accent)
                .align_child(Center, Center)
                .child(Text::new("").size(14.0).color(tokens.accent).font(font)),
        ),
        Box::new(
            Text::new(&username)
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(font),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let power_buttons = Row::new(vec![
        Box::new(render_power_icon(
            "󰌾",
            "Lock",
            tokens.text_secondary,
            font,
            &tokens,
            || {
                let _ = std::process::Command::new("swaylock").spawn();
                close();
            },
        )),
        Box::new(render_power_icon(
            "󰒲",
            "Sleep",
            tokens.text_secondary,
            font,
            &tokens,
            || {
                let _ = std::process::Command::new("systemctl")
                    .arg("suspend")
                    .spawn();
                close();
            },
        )),
        Box::new(render_power_icon(
            "󰜉",
            "Reboot",
            tokens.accent_hover,
            font,
            &tokens,
            || {
                let _ = std::process::Command::new("systemctl")
                    .arg("reboot")
                    .spawn();
                close();
            },
        )),
        Box::new(render_power_icon(
            "󰐥",
            "Shutdown",
            tokens.danger,
            font,
            &tokens,
            || {
                let _ = std::process::Command::new("systemctl")
                    .arg("poweroff")
                    .spawn();
                close();
            },
        )),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let footer_row = Row::new(vec![Box::new(user_info), Box::new(power_buttons)])
        .width(Parent)
        .align(Align::Center);

    let start_menu_content = Column::new(vec![
        Box::new(search_bar),
        Box::new(header_row),
        grid_content,
        Box::new(
            Rectangle::new()
                .width(Parent)
                .height(1.0)
                .fill(tokens.border_subtle),
        ),
        Box::new(footer_row),
    ])
    .gap(spacing::GAP_MD)
    .width(Parent);

    let container = Rectangle::new()
        .width(540.0)
        .height(560.0)
        .fill(tokens.bg)
        .border(1.0, tokens.border_subtle)
        .radius(radius::LG)
        .padding(Padding {
            top: spacing::PAD_LG,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_LG,
            right: spacing::PAD_LG,
        })
        .child(start_menu_content);

    let bottom_margin = if is_bottom_bar {
        (dock_height + 12.0) as i32
    } else {
        12
    };
    let top_margin = if is_bottom_bar {
        12
    } else {
        (dock_height + 12.0) as i32
    };

    LayerWindow::new()
        .width(540.0)
        .height(560.0)
        .anchor_horizontal(Horizontal::Middle)
        .anchor_vertical(if is_bottom_bar {
            Vertical::Bottom
        } else {
            Vertical::Top
        })
        .margin(Margin {
            top: top_margin,
            right: 0,
            bottom: bottom_margin,
            left: 0,
        })
        .layer(Layer::Overlay)
        .target_output(OutputTarget::Active)
        .namespace("dotfield-shell-start-menu")
        .space(Zone::Ignore)
        .keyboard(Keyboard::OnDemand)
        .on_key(|key| match key {
            Key::Escape => close(),
            Key::Enter => {
                let apps = Apps::read();
                let menu = StartMenuService::read();
                let query_lower = menu.query.trim().to_lowercase();
                let matched: Vec<DesktopApp> = apps
                    .list()
                    .iter()
                    .filter(|&a| {
                        query_lower.is_empty() || a.name().to_lowercase().contains(&query_lower)
                    })
                    .cloned()
                    .collect();
                if let Some(target) = matched.get(menu.selected) {
                    target.launch();
                    close();
                }
            }
            Key::Right => {
                let mut menu = StartMenuService::write();
                menu.selected = (menu.selected + 1).min(PINNED_COLS * PINNED_ROWS - 1);
            }
            Key::Left => {
                let mut menu = StartMenuService::write();
                menu.selected = menu.selected.saturating_sub(1);
            }
            Key::Down => {
                let mut menu = StartMenuService::write();
                menu.selected = (menu.selected + PINNED_COLS).min(PINNED_COLS * PINNED_ROWS - 1);
            }
            Key::Up => {
                let mut menu = StartMenuService::write();
                menu.selected = menu.selected.saturating_sub(PINNED_COLS);
            }
            _ => {}
        })
        .visible(true)
        .child(container)
}

fn render_power_icon<F>(
    icon: &'static str,
    _tooltip: &'static str,
    color: Color,
    font: &str,
    tokens: &ThemeTokens,
    on_click: F,
) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    Rectangle::new()
        .width(32.0)
        .height(32.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(
            Text::new(icon)
                .size(typography::BODY)
                .color(color)
                .font(font),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_menu_toggle() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        assert!(!StartMenuService::read().open);
        toggle();
        assert!(StartMenuService::read().open);
        close();
        assert!(!StartMenuService::read().open);
    }
}
