use std::time::Duration;

use amane::{
    Align, Apps, Color, Column, DesktopApp, Horizontal, Image, Key, Keyboard, Layer, LayerWindow,
    OutputTarget, Padding, Parent, Rectangle, Row, Service, Text, TextInput, Vertical, Widget,
    Zone,
};

use crate::config::ConfigService;
use crate::ui::components::{action_card, action_card_interactive, modal_shell, status_badge};
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

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

pub struct ShellAction {
    pub id: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
}

pub fn get_shell_actions() -> Vec<ShellAction> {
    vec![
        ShellAction {
            id: "display-config",
            icon: "󰍹",
            title: "Display Configurator",
            subtitle: "Switch multi-monitor layouts via Kanshi",
        },
        ShellAction {
            id: "terminal-manager",
            icon: "󰆍",
            title: "Terminal Manager",
            subtitle: "Select and launch terminal emulator",
        },
        ShellAction {
            id: "powermenu",
            icon: "󰐥",
            title: "Power Menu",
            subtitle: "Lock, suspend, restart, or power off",
        },
        ShellAction {
            id: "calendar",
            icon: "󰸗",
            title: "Calendar HUD",
            subtitle: "View current date and monthly schedule",
        },
        ShellAction {
            id: "control-center",
            icon: "󰕾",
            title: "Control Center",
            subtitle: "Manage volume, network, and bluetooth",
        },
        ShellAction {
            id: "reload",
            icon: "󰑐",
            title: "Reload Shell",
            subtitle: "Reload dotfield manifest and styles",
        },
    ]
}

pub fn execute_shell_action(id: &str) {
    match id {
        "display-config" => {
            crate::ui::display_config::toggle();
        }
        "terminal-manager" => {
            crate::ui::terminal_manager::toggle();
        }
        "powermenu" => {
            crate::ui::powermenu::toggle();
        }
        "calendar" => {
            crate::ui::calendar::toggle();
        }
        "control-center" => {
            crate::ui::control_center::toggle();
        }
        "reload" => {
            let path = crate::config::ShellManifest::default_path();
            if let Ok(manifest) = crate::config::ShellManifest::load_from_path(&path) {
                crate::config::ConfigService::write().manifest = manifest;
            }
        }
        _ => {}
    }
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

    let tokens = ThemeTokens::from_manifest(&manifest);
    let query_trimmed = launcher.query.trim();

    let (content_widget, total_results) = if let Some(cmd) = query_trimmed.strip_prefix(':') {
        render_command_mode(cmd.trim(), &tokens)
    } else if let Some(act_query) = query_trimmed.strip_prefix('>') {
        render_action_mode(
            act_query.trim(),
            launcher.selected,
            launcher.hover_index,
            &tokens,
        )
    } else {
        render_app_mode(
            &launcher.query,
            launcher.selected,
            launcher.hover_index,
            launcher.scroll_row,
            &tokens,
        )
    };

    let mode_badge = if query_trimmed.starts_with(':') {
        status_badge(": Command", tokens.accent, &tokens)
    } else if query_trimmed.starts_with('>') {
        status_badge("> Action", tokens.accent, &tokens)
    } else {
        let count_text = if total_results > 0 {
            format!(
                "{}/{} apps",
                (launcher.selected + 1).min(total_results),
                total_results
            )
        } else {
            "0 apps".to_string()
        };
        status_badge(&count_text, tokens.accent, &tokens)
    };

    let search_icon = Text::new("󰍉")
        .size(typography::TITLE_SM)
        .color(tokens.accent)
        .font(&tokens.font_family)
        .tight();

    let search_input = TextInput::new("launcher-search")
        .size(typography::BODY)
        .color(tokens.text_primary)
        .placeholder("Search apps, ': <cmd>' to run, '> <action>' for shell")
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

    let search_row = Row::new(vec![
        Box::new(search_icon),
        Box::new(search_input),
        Box::new(mode_badge),
    ])
    .gap(spacing::GAP_MD)
    .align(Align::Center);

    let search_box = Rectangle::new()
        .width(Parent)
        .height(42.0)
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
        .child(search_row);

    let body_column = Column::new(vec![Box::new(search_box), content_widget])
        .gap(spacing::GAP_MD)
        .width(Parent);

    let dialog = modal_shell(
        680.0,
        480.0,
        "󰵆",
        "Application Launcher",
        Some("Niri"),
        Box::new(body_column),
        vec![
            ("Arrows", "Navigate"),
            ("↵ Enter", "Launch"),
            ("Esc", "Dismiss"),
            (":", "Terminal Exec"),
            (">", "Actions"),
        ],
        close,
        &tokens,
    );

    LayerWindow::new()
        .width(680.0)
        .height(480.0)
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

fn render_command_mode(cmd: &str, tokens: &ThemeTokens) -> (Box<dyn Widget>, usize) {
    let icon_box = Box::new(
        Rectangle::new()
            .width(28.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .align_child(Align::Center, Align::Center)
            .child(
                Text::new("󰆍")
                    .size(typography::TITLE_SM)
                    .color(tokens.accent)
                    .font(&tokens.font_family),
            ),
    );

    let title = if cmd.is_empty() {
        "Type command to execute...".to_string()
    } else {
        format!("Run: {}", cmd)
    };

    let card = action_card(
        648.0,
        64.0,
        icon_box,
        &title,
        Some("Executes inside terminal emulator via terminal-select.sh"),
        Some("↵ Run"),
        true,
        false,
        move || {
            submit_selection();
        },
        tokens,
    );

    let container = Rectangle::new()
        .width(Parent)
        .height(310.0)
        .child(Column::new(vec![Box::new(card)]).width(Parent));

    (Box::new(container), 1)
}

fn render_action_mode(
    act_query: &str,
    selected: usize,
    hover_index: Option<usize>,
    tokens: &ThemeTokens,
) -> (Box<dyn Widget>, usize) {
    let all_actions = get_shell_actions();
    let q = act_query.to_lowercase();
    let matched: Vec<_> = all_actions
        .into_iter()
        .filter(|a| q.is_empty() || a.id.contains(&q) || a.title.to_lowercase().contains(&q))
        .collect();

    let total_results = matched.len();
    if total_results == 0 {
        let empty_card = Rectangle::new()
            .width(Parent)
            .height(72.0)
            .radius(radius::MD)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .align_child(Align::Center, Align::Center)
            .child(
                Text::new(format!("No shell actions matching \">{}\"", act_query))
                    .size(typography::SECONDARY)
                    .color(tokens.text_secondary)
                    .font(&tokens.font_family),
            );
        let container = Rectangle::new()
            .width(Parent)
            .height(310.0)
            .child(empty_card);
        return (Box::new(container), 0);
    }

    let mut action_rows: Vec<Box<dyn Widget>> = Vec::new();
    for (idx, action) in matched.iter().enumerate() {
        let is_selected = idx == selected;
        let is_hovered = hover_index == Some(idx);
        let action_id = action.id;

        let icon_box = Box::new(
            Rectangle::new()
                .width(28.0)
                .height(28.0)
                .radius(radius::SM)
                .fill(tokens.surface)
                .border(1.0, tokens.border_subtle)
                .align_child(Align::Center, Align::Center)
                .child(
                    Text::new(action.icon)
                        .size(typography::TITLE_SM)
                        .color(tokens.accent)
                        .font(&tokens.font_family),
                ),
        );

        let card = action_card_interactive(
            648.0,
            46.0,
            icon_box,
            action.title,
            Some(action.subtitle),
            Some(action.id),
            is_selected,
            is_hovered,
            move |inside| {
                let mut launcher = LauncherService::write();
                if inside {
                    launcher.hover_index = Some(idx);
                } else if launcher.hover_index == Some(idx) {
                    launcher.hover_index = None;
                }
            },
            move || {
                execute_shell_action(action_id);
                close();
            },
            tokens,
        );
        action_rows.push(Box::new(card));
    }

    let action_column = Column::new(action_rows).width(Parent).gap(spacing::GAP_SM);

    let container = Rectangle::new()
        .width(Parent)
        .height(310.0)
        .child(action_column);

    (Box::new(container), total_results)
}

fn render_app_mode(
    query: &str,
    selected: usize,
    hover_index: Option<usize>,
    scroll_row: usize,
    tokens: &ThemeTokens,
) -> (Box<dyn Widget>, usize) {
    let apps = Apps::read();
    let mut matched_apps: Vec<DesktopApp> = Vec::new();

    let query_lower = query.to_lowercase();
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

    let sel = if total_results == 0 {
        0
    } else {
        selected.min(total_results - 1)
    };

    let cur_row = sel / GRID_COLS;
    let mut scroll = scroll_row;
    if cur_row < scroll {
        scroll = cur_row;
    } else if cur_row >= scroll + GRID_VISIBLE_ROWS {
        scroll = cur_row + 1 - GRID_VISIBLE_ROWS;
    }

    if total_results == 0 {
        let empty_card = Rectangle::new()
            .width(Parent)
            .height(72.0)
            .radius(radius::MD)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .align_child(Align::Center, Align::Center)
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new("󰛐")
                            .size(typography::TITLE_SM)
                            .color(tokens.text_muted)
                            .font(&tokens.font_family),
                    ),
                    Box::new(
                        Text::new(format!("No applications matching \"{}\"", query))
                            .size(typography::SECONDARY)
                            .color(tokens.text_secondary)
                            .font(&tokens.font_family),
                    ),
                ])
                .gap(spacing::GAP_XS)
                .align(Align::Center),
            );
        let container = Rectangle::new()
            .width(Parent)
            .height(310.0)
            .child(empty_card);
        return (Box::new(container), 0);
    }

    let mut grid_rows: Vec<Box<dyn Widget>> = Vec::new();
    let start_idx = scroll * GRID_COLS;

    for row_idx in 0..GRID_VISIBLE_ROWS {
        let mut row_cards: Vec<Box<dyn Widget>> = Vec::new();
        for col_idx in 0..GRID_COLS {
            let abs_idx = start_idx + (row_idx * GRID_COLS) + col_idx;
            if abs_idx < total_results {
                let app = &matched_apps[abs_idx];
                let is_selected = abs_idx == sel;
                let is_hovered = hover_index == Some(abs_idx);
                let launched = app.clone();

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
                            .radius(radius::SM)
                            .fill(tokens.surface)
                            .border(1.0, tokens.border_subtle)
                            .align_child(Align::Center, Align::Center)
                            .child(
                                Text::new("󰵆")
                                    .size(typography::TITLE_SM)
                                    .color(if is_selected {
                                        tokens.accent
                                    } else {
                                        tokens.text_secondary
                                    })
                                    .font(&tokens.font_family),
                            ),
                    ),
                };

                let app_name: String = app.name().chars().take(18).collect();
                let app_desc: String = app
                    .description()
                    .map(|d| d.chars().take(22).collect())
                    .unwrap_or_else(|| "Application".to_string());

                let app_card = action_card_interactive(
                    204.0,
                    64.0,
                    icon_widget,
                    &app_name,
                    Some(&app_desc),
                    None,
                    is_selected,
                    is_hovered,
                    move |inside| {
                        let mut launcher = LauncherService::write();
                        if inside {
                            launcher.hover_index = Some(abs_idx);
                        } else if launcher.hover_index == Some(abs_idx) {
                            launcher.hover_index = None;
                        }
                    },
                    move || {
                        launched.launch();
                        close();
                    },
                    tokens,
                );

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

    let app_grid_column = Column::new(grid_rows).width(Parent).gap(8.0);
    let scrollable_container = Rectangle::new()
        .width(Parent)
        .height(310.0)
        .cursor(amane::Pointer)
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
                    launcher.selected = ((launcher.scroll_row + GRID_VISIBLE_ROWS - 1) * GRID_COLS
                        + (launcher.selected % GRID_COLS))
                        .min(total_results - 1);
                }
            }
        })
        .child(app_grid_column);

    (Box::new(scrollable_container), total_results)
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
    let query_trimmed = launcher.query.trim().to_string();
    let selected = launcher.selected;

    if let Some(cmd) = query_trimmed.strip_prefix(':') {
        let cmd = cmd.trim();
        if !cmd.is_empty() {
            let _ = std::process::Command::new("terminal-select.sh")
                .args(["-e", cmd])
                .spawn();
        }
        drop(launcher);
        close();
        return;
    }

    if let Some(act_query) = query_trimmed.strip_prefix('>') {
        let actions = get_shell_actions();
        let q = act_query.trim().to_lowercase();
        let matched: Vec<_> = actions
            .into_iter()
            .filter(|a| q.is_empty() || a.id.contains(&q) || a.title.to_lowercase().contains(&q))
            .collect();
        drop(launcher);
        if let Some(target) = matched.get(selected) {
            execute_shell_action(target.id);
        }
        close();
        return;
    }

    let apps = Apps::read();
    let query_lower = query_trimmed.to_lowercase();
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
    drop(launcher);

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

    #[test]
    fn test_get_shell_actions() {
        let actions = get_shell_actions();
        assert!(actions.iter().any(|a| a.id == "display-config"));
        assert!(actions.iter().any(|a| a.id == "terminal-manager"));
        assert!(actions.iter().any(|a| a.id == "reload"));
    }
}
