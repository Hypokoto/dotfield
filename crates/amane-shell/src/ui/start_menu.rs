//! Centered Start Menu & Unified Desktop Launcher.
//! Windows 11 Fluent style bottom-dock anchored flyout (640px x 604px, 12px corner radius).
//! Features:
//! - Pinned Apps Grid (6 columns x 3 rows) with smart multi-line label formatting (no ugly 10-char chop)
//! - Universal Instant-Focus Search with live matching
//! - Terminal Command Mode (": <cmd>") for direct CLI execution
//! - System Action Mode ("> <action>") for display, power, calendar, record, lock controls
//! - Full Alphabetical All-Apps View (2-column scrollable cards)
//! - 4 Curated Quick Actions in Recommended section (Terminal, Yazi, Neovim, Btop)
//! - User Profile & One-Click Power Controls (Lock, Sleep, Reboot, Shutdown)
//! - Full Keyboard Accessibility (Arrows, Enter, Escape, PageUp, PageDown)

use std::path::PathBuf;
use std::time::{Duration, Instant};

use amane::{
    Align, Apps, Button, Center, Color, Column, DesktopApp, Horizontal, Image, Justify, Key,
    Keyboard, Layer, LayerWindow, Margin, OutputTarget, Padding, Parent, Pointer, Rectangle, Row,
    Service, Text, TextInput, Vertical, Widget, Zone,
};

use crate::config::ConfigService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub const PINNED_COLS: usize = 6;
pub const PINNED_ROWS: usize = 3;
pub const LIST_COLS: usize = 2;
pub const LIST_VISIBLE_ROWS: usize = 7;

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
    pub scroll_row: usize,
    pub view_mode: StartMenuViewMode,
    pub pinned_apps: Vec<String>,
}

impl Service for StartMenuService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
            query: String::new(),
            selected: 0,
            scroll_row: 0,
            view_mode: StartMenuViewMode::Pinned,
            pinned_apps: load_pinned_apps(),
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn pinned_apps_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/dotfield/pinned_apps.json")
}

pub fn load_pinned_apps() -> Vec<String> {
    let path = pinned_apps_path();
    if let Ok(content) = std::fs::read_to_string(&path)
        && let Ok(apps) = serde_json::from_str::<Vec<String>>(&content)
    {
        return apps;
    }
    // "unpin all by default": empty list initially
    Vec::new()
}

pub fn save_pinned_apps(apps: &[String]) {
    let path = pinned_apps_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(apps) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn is_pinned(app_name: &str) -> bool {
    let menu = StartMenuService::read();
    menu.pinned_apps
        .iter()
        .any(|p| p.eq_ignore_ascii_case(app_name))
}

pub fn pin_app(app_name: &str) {
    let mut menu = StartMenuService::write();
    if !menu
        .pinned_apps
        .iter()
        .any(|p| p.eq_ignore_ascii_case(app_name))
    {
        menu.pinned_apps.push(app_name.to_string());
        save_pinned_apps(&menu.pinned_apps);
    }
}

pub fn unpin_app(app_name: &str) {
    let mut menu = StartMenuService::write();
    if let Some(pos) = menu
        .pinned_apps
        .iter()
        .position(|p| p.eq_ignore_ascii_case(app_name))
    {
        menu.pinned_apps.remove(pos);
        save_pinned_apps(&menu.pinned_apps);
        let total = menu.pinned_apps.len();
        if menu.selected >= total && total > 0 {
            menu.selected = total - 1;
        }
    }
}

pub fn toggle_pin_app(app_name: &str) {
    let pinned = is_pinned(app_name);
    if pinned {
        unpin_app(app_name);
    } else {
        pin_app(app_name);
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
        menu.scroll_row = 0;
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
    menu.scroll_row = 0;
}

/// Smart 2-line label formatter for pinned tiles.
/// Avoids chopping words awkwardly; breaks cleanly across two lines with graceful ellipsis.
pub fn format_tile_label(name: &str) -> (String, Option<String>) {
    let clean = name.trim();
    if clean.chars().count() <= 11 {
        return (clean.to_string(), None);
    }

    let words: Vec<&str> = clean.split_whitespace().collect();
    if words.len() > 1 {
        let first = words[0];
        let rest = words[1..].join(" ");
        let line1: String = first.chars().take(11).collect();
        let line2: String = if rest.chars().count() > 11 {
            let sub: String = rest.chars().take(10).collect();
            format!("{sub}…")
        } else {
            rest
        };
        (line1, Some(line2))
    } else {
        let line1: String = clean.chars().take(10).collect();
        let rem: String = clean.chars().skip(10).take(9).collect();
        let line2 = if clean.chars().count() > 19 {
            format!("{rem}…")
        } else {
            rem
        };
        (line1, Some(line2))
    }
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
            id: "display",
            icon: "󰍹",
            title: "Display Configurator",
            subtitle: "Switch multi-monitor layouts via Kanshi",
        },
        ShellAction {
            id: "power",
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
            id: "record",
            icon: "󰕧",
            title: "Screen Recorder",
            subtitle: "Record active monitor or window area",
        },
        ShellAction {
            id: "lock",
            icon: "󰌾",
            title: "Lock Screen",
            subtitle: "Lock desktop session securely",
        },
    ]
}

pub fn execute_shell_action(id: &str) {
    match id {
        "display" => {
            crate::ui::display_config::toggle();
        }
        "power" => {
            crate::ui::powermenu::toggle();
        }
        "calendar" => {
            crate::ui::calendar::toggle();
        }
        "control-center" => {
            crate::ui::control_center::toggle();
        }
        "record" => {
            crate::ui::screen_record::toggle();
        }
        "lock" => {
            let _ = std::process::Command::new("swaylock").spawn();
        }
        _ => {}
    }
}

pub fn handle_enter_submission() {
    let menu = StartMenuService::read();
    let query = menu.query.trim().to_string();

    if query.starts_with(':') {
        let cmd = query.trim_start_matches(':').trim().to_string();
        if !cmd.is_empty() {
            let _ = std::process::Command::new("ghostty")
                .arg("-e")
                .arg("sh")
                .arg("-c")
                .arg(format!("{}; exec $SHELL", cmd))
                .spawn();
        }
        drop(menu);
        close();
    } else if query.starts_with('>') {
        let action_query = query.trim_start_matches('>').trim().to_lowercase();
        let actions = get_shell_actions();
        let filtered: Vec<_> = actions
            .into_iter()
            .filter(|a| {
                action_query.is_empty()
                    || a.id.contains(&action_query)
                    || a.title.to_lowercase().contains(&action_query)
            })
            .collect();
        if let Some(act) = filtered.get(menu.selected) {
            execute_shell_action(act.id);
        }
        drop(menu);
        close();
    } else {
        let apps = Apps::read();
        let query_lower = query.to_lowercase();
        let is_pinned_view = query_lower.is_empty() && menu.view_mode == StartMenuViewMode::Pinned;
        let target_app: Option<DesktopApp> = if is_pinned_view {
            let mut pinned_list = Vec::new();
            for pin_name in &menu.pinned_apps {
                if let Some(app) = apps
                    .list()
                    .iter()
                    .find(|a| a.name().eq_ignore_ascii_case(pin_name))
                {
                    pinned_list.push(app.clone());
                }
            }
            pinned_list.get(menu.selected).cloned()
        } else {
            let matched: Vec<DesktopApp> = apps
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
            matched.get(menu.selected).cloned()
        };

        if let Some(target) = target_app {
            target.launch();
        }
        drop(menu);
        close();
    }
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

    let dock_height = if manifest.bar.height == 34.0
        || manifest.bar.height == 32.0
        || manifest.bar.position == "bottom"
    {
        48.0
    } else {
        manifest.bar.height as f32
    };

    // 1. Search Bar with instant focus
    let search_input = TextInput::new("start-menu-search")
        .size(typography::BODY)
        .color(tokens.text_primary)
        .placeholder("Search apps, ': <cmd>' for exec, '> <action>' for HUD...")
        .focused()
        .on_change(|text| {
            let mut menu = StartMenuService::write();
            menu.query = text;
            menu.selected = 0;
            menu.scroll_row = 0;
        })
        .on_submit(|_| {
            handle_enter_submission();
        });

    let has_query = !menu.query.trim().is_empty();

    let search_bar = Rectangle::new()
        .width(Parent)
        .height(44.0)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(
            1.0,
            if has_query {
                tokens.accent
            } else {
                tokens.border_subtle
            },
        )
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .align_child(Align::Start, Align::Center)
        .child(
            Row::new(vec![
                Box::new(Text::new("").size(15.0).color(tokens.accent).font(font)),
                Box::new(search_input),
            ])
            .gap(spacing::GAP_MD)
            .width(Parent)
            .align(Align::Center),
        );

    let query_raw = menu.query.trim();
    let query_lower = query_raw.to_lowercase();
    let is_cmd_mode = query_raw.starts_with(':');
    let is_action_mode = query_raw.starts_with('>');

    // 2. Apps Content Resolution
    let apps = Apps::read();
    let all_apps_list: Vec<DesktopApp> = apps.list().to_vec();

    let matched_apps: Vec<DesktopApp> = if is_cmd_mode || is_action_mode {
        Vec::new()
    } else if query_lower.is_empty() {
        all_apps_list.clone()
    } else {
        all_apps_list
            .iter()
            .filter(|&a| {
                a.name().to_lowercase().contains(&query_lower)
                    || a.description()
                        .is_some_and(|d| d.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    };

    let total_results = matched_apps.len();
    let current_view = menu.view_mode;

    // Build the dynamic middle content section
    let (content_section, header_row) = if is_cmd_mode {
        let cmd = query_raw.trim_start_matches(':').trim().to_string();
        let cmd_to_run = cmd.clone();
        let cmd_header = Row::new(vec![
            Box::new(
                Text::new("Terminal Execution")
                    .size(typography::TITLE_SM)
                    .color(tokens.text_primary)
                    .font(font),
            ),
            Box::new(
                Text::new(": <command>")
                    .size(typography::CAPTION)
                    .color(tokens.accent)
                    .font(font),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let cmd_title = if cmd.is_empty() {
            "Type a shell command to execute...".to_string()
        } else {
            format!("Execute: {}", cmd)
        };

        let cmd_card = Rectangle::new()
            .width(Parent)
            .height(72.0)
            .radius(radius::MD)
            .fill(tokens.surface_elevated)
            .border(1.5, tokens.accent)
            .cursor(Pointer)
            .padding(Padding {
                top: spacing::PAD_SM,
                bottom: spacing::PAD_SM,
                left: spacing::PAD_MD,
                right: spacing::PAD_MD,
            })
            .on_click(move |_| {
                if !cmd_to_run.is_empty() {
                    let _ = std::process::Command::new("ghostty")
                        .arg("-e")
                        .arg("sh")
                        .arg("-c")
                        .arg(format!("{}; exec $SHELL", cmd_to_run))
                        .spawn();
                    close();
                }
            })
            .child(
                Row::new(vec![
                    Box::new(Text::new("󰆍").size(24.0).color(tokens.accent).font(font)),
                    Box::new(
                        Column::new(vec![
                            Box::new(
                                Text::new(&cmd_title)
                                    .size(typography::SECONDARY)
                                    .color(tokens.text_primary)
                                    .font(font),
                            ),
                            Box::new(
                                Text::new("Press Enter to launch in Ghostty terminal")
                                    .size(typography::CAPTION)
                                    .color(tokens.text_secondary)
                                    .font(font),
                            ),
                        ])
                        .align(Align::Start)
                        .gap(2.0),
                    ),
                ])
                .gap(spacing::GAP_MD)
                .align(Align::Center),
            );

        (
            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(412.0)
                    .align_child(Align::Start, Align::Start)
                    .child(cmd_card),
            ) as Box<dyn Widget>,
            cmd_header,
        )
    } else if is_action_mode {
        let action_query = query_raw.trim_start_matches('>').trim().to_lowercase();
        let all_actions = get_shell_actions();
        let filtered_actions: Vec<_> = all_actions
            .into_iter()
            .filter(|a| {
                action_query.is_empty()
                    || a.id.contains(&action_query)
                    || a.title.to_lowercase().contains(&action_query)
            })
            .collect();

        let action_header = Row::new(vec![
            Box::new(
                Text::new("System HUD Actions")
                    .size(typography::TITLE_SM)
                    .color(tokens.text_primary)
                    .font(font),
            ),
            Box::new(
                Text::new(format!("{} available", filtered_actions.len()))
                    .size(typography::CAPTION)
                    .color(tokens.accent)
                    .font(font),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let mut action_rows: Vec<Box<dyn Widget>> = Vec::new();
        for (idx, action) in filtered_actions.iter().enumerate().take(7) {
            let is_sel = idx == menu.selected;
            let action_id = action.id;
            let row_card = Rectangle::new()
                .width(Parent)
                .height(48.0)
                .radius(radius::MD)
                .fill(if is_sel {
                    tokens.surface_elevated
                } else {
                    tokens.surface
                })
                .border(
                    1.0,
                    if is_sel {
                        tokens.accent
                    } else {
                        tokens.border_subtle
                    },
                )
                .cursor(Pointer)
                .padding(Padding {
                    top: 4.0,
                    bottom: 4.0,
                    left: 12.0,
                    right: 12.0,
                })
                .on_click(move |_| {
                    execute_shell_action(action_id);
                    close();
                })
                .child(
                    Row::new(vec![
                        Box::new(
                            Text::new(action.icon)
                                .size(18.0)
                                .color(tokens.accent)
                                .font(font),
                        ),
                        Box::new(
                            Column::new(vec![
                                Box::new(
                                    Text::new(action.title)
                                        .size(typography::SECONDARY)
                                        .color(tokens.text_primary)
                                        .font(font),
                                ),
                                Box::new(
                                    Text::new(action.subtitle)
                                        .size(typography::CAPTION)
                                        .color(tokens.text_secondary)
                                        .font(font),
                                ),
                            ])
                            .align(Align::Start)
                            .gap(1.0),
                        ),
                    ])
                    .gap(spacing::GAP_MD)
                    .align(Align::Center),
                );
            action_rows.push(Box::new(row_card));
        }

        (
            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(412.0)
                    .child(Column::new(action_rows).gap(spacing::GAP_SM).width(Parent)),
            ) as Box<dyn Widget>,
            action_header,
        )
    } else if has_query || current_view == StartMenuViewMode::AllApps {
        // Search Results Mode OR All Applications Scrollable List
        let header_title = if has_query {
            "Search Results"
        } else {
            "All Applications"
        };

        let badge_text = if has_query {
            format!("{} found", total_results)
        } else {
            format!("{} total", total_results)
        };

        let header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(
                        Text::new(header_title)
                            .size(typography::TITLE_SM)
                            .color(tokens.text_primary)
                            .font(font),
                    ),
                    Box::new(
                        Text::new(&badge_text)
                            .size(typography::CAPTION)
                            .color(tokens.accent)
                            .font(font),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            ),
            Box::new(
                Rectangle::new()
                    .width(88.0)
                    .height(26.0)
                    .radius(radius::SM)
                    .fill(tokens.surface_elevated)
                    .border(1.0, tokens.border_subtle)
                    .cursor(Pointer)
                    .align_child(Center, Center)
                    .on_click(|_| {
                        let mut menu = StartMenuService::write();
                        if !menu.query.is_empty() {
                            menu.query.clear();
                            TextInput::set_text("start-menu-search", "");
                        }
                        menu.view_mode = StartMenuViewMode::Pinned;
                        menu.selected = 0;
                        menu.scroll_row = 0;
                    })
                    .child(
                        Text::new(if has_query {
                            "Clear 󰅖"
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
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let content_widget: Box<dyn Widget> = if total_results == 0 {
            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(412.0)
                    .align_child(Center, Center)
                    .child(
                        Column::new(vec![
                            Box::new(Text::new("󰛐").size(28.0).color(tokens.steel).font(font)),
                            Box::new(
                                Text::new(format!("No applications found matching \"{}\"", query_raw))
                                    .size(typography::SECONDARY)
                                    .color(tokens.text_secondary)
                                    .font(font),
                            ),
                            Box::new(
                                Text::new("Tip: type ': <cmd>' for terminal command or '> <action>' for HUD")
                                    .size(typography::CAPTION)
                                    .color(tokens.text_muted)
                                    .font(font),
                            ),
                        ])
                        .align(Center)
                        .gap(spacing::GAP_SM),
                    ),
            )
        } else {
            let total_rows = total_results.div_ceil(LIST_COLS);
            let scroll = menu
                .scroll_row
                .min(total_rows.saturating_sub(LIST_VISIBLE_ROWS));
            let start_idx = scroll * LIST_COLS;
            let mut card_rows: Vec<Box<dyn Widget>> = Vec::new();

            for r in 0..LIST_VISIBLE_ROWS {
                let mut col_items: Vec<Box<dyn Widget>> = Vec::new();
                for c in 0..LIST_COLS {
                    let idx = start_idx + r * LIST_COLS + c;
                    if idx < total_results {
                        let app = &matched_apps[idx];
                        let is_selected = idx == menu.selected;
                        let launched = app.clone();

                        let icon_widget: Box<dyn Widget> = match app.icon_path() {
                            Some(path) => Box::new(
                                Rectangle::new()
                                    .width(32.0)
                                    .height(32.0)
                                    .fill(Image::contain(path)),
                            ),
                            None => Box::new(
                                Rectangle::new()
                                    .width(32.0)
                                    .height(32.0)
                                    .radius(radius::SM)
                                    .fill(tokens.surface)
                                    .align_child(Center, Center)
                                    .child(
                                        Text::new("󰵆").size(16.0).color(tokens.accent).font(font),
                                    ),
                            ),
                        };

                        let app_name = app.name().to_string();
                        let app_desc = app.description().unwrap_or("Application").to_string();
                        let desc_truncated: String = if app_desc.chars().count() > 30 {
                            format!("{}…", app_desc.chars().take(28).collect::<String>())
                        } else {
                            app_desc
                        };

                        let is_pinned_app = is_pinned(&app_name);
                        let pin_icon = if is_pinned_app { "󰤰" } else { "󰤱" };
                        let pin_color = if is_pinned_app {
                            tokens.accent
                        } else {
                            tokens.steel
                        };
                        let app_name_for_pin = app_name.clone();
                        let app_name_for_card = app_name.clone();

                        let pin_btn = Rectangle::new()
                            .width(28.0)
                            .height(28.0)
                            .radius(radius::SM)
                            .fill(if is_pinned_app {
                                tokens.surface_elevated
                            } else {
                                tokens.surface
                            })
                            .border(
                                1.0,
                                if is_pinned_app {
                                    tokens.accent
                                } else {
                                    tokens.border_subtle
                                },
                            )
                            .cursor(Pointer)
                            .align_child(Center, Center)
                            .on_click(move |_| {
                                toggle_pin_app(&app_name_for_pin);
                            })
                            .child(Text::new(pin_icon).size(14.0).color(pin_color).font(font));

                        let card = Rectangle::new()
                            .width(292.0)
                            .height(52.0)
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
                                top: 4.0,
                                bottom: 4.0,
                                left: 10.0,
                                right: 8.0,
                            })
                            .align_child(Align::Start, Align::Center)
                            .on_click(move |btn| match btn {
                                Button::Left => {
                                    launched.launch();
                                    close();
                                }
                                Button::Right => {
                                    toggle_pin_app(&app_name_for_card);
                                }
                                _ => {}
                            })
                            .child(
                                Row::new(vec![
                                    Box::new(
                                        Row::new(vec![
                                            icon_widget,
                                            Box::new(
                                                Column::new(vec![
                                                    Box::new(
                                                        Text::new(&app_name)
                                                            .size(typography::SECONDARY)
                                                            .color(tokens.text_primary)
                                                            .font(font),
                                                    ),
                                                    Box::new(
                                                        Text::new(&desc_truncated)
                                                            .size(typography::CAPTION)
                                                            .color(tokens.text_secondary)
                                                            .font(font),
                                                    ),
                                                ])
                                                .align(Align::Start)
                                                .gap(1.0),
                                            ),
                                        ])
                                        .gap(spacing::GAP_MD)
                                        .align(Align::Center),
                                    ),
                                    Box::new(pin_btn),
                                ])
                                .width(Parent)
                                .justify(Justify::SpaceBetween)
                                .align(Align::Center),
                            );

                        col_items.push(Box::new(card));
                    } else {
                        col_items.push(Box::new(Rectangle::new().width(292.0).height(52.0)));
                    }
                }
                card_rows.push(Box::new(
                    Row::new(col_items)
                        .gap(spacing::GAP_MD)
                        .align(Align::Center),
                ));
            }

            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(412.0)
                    .child(Column::new(card_rows).gap(spacing::GAP_SM).width(Parent)),
            ) as Box<dyn Widget>
        };

        (content_widget, header)
    } else {
        // View A: Custom Pinned Apps Grid (6 columns x 3 rows)
        let mut pinned_apps_list: Vec<DesktopApp> = Vec::new();
        for pin_name in &menu.pinned_apps {
            if let Some(app) = all_apps_list
                .iter()
                .find(|a| a.name().eq_ignore_ascii_case(pin_name))
            {
                pinned_apps_list.push(app.clone());
            }
        }
        let total_pinned = pinned_apps_list.len();
        let items_per_page = PINNED_COLS * PINNED_ROWS;
        let total_pages = total_pinned.div_ceil(items_per_page).max(1);
        let current_page = (menu.selected / items_per_page).min(total_pages - 1);

        let badge_text = if total_pages > 1 {
            format!(
                "{} pinned • Page {}/{}",
                total_pinned,
                current_page + 1,
                total_pages
            )
        } else {
            format!("{} pinned", total_pinned)
        };

        let header = Row::new(vec![
            Box::new(
                Row::new(vec![
                    Box::new(
                        Text::new("Pinned")
                            .size(typography::TITLE_SM)
                            .color(tokens.text_primary)
                            .font(font),
                    ),
                    Box::new(
                        Text::new(&badge_text)
                            .size(typography::CAPTION)
                            .color(tokens.accent)
                            .font(font),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            ),
            Box::new(
                Rectangle::new()
                    .width(88.0)
                    .height(26.0)
                    .radius(radius::SM)
                    .fill(tokens.surface_elevated)
                    .border(1.0, tokens.border_subtle)
                    .cursor(Pointer)
                    .align_child(Center, Center)
                    .on_click(|_| {
                        let mut menu = StartMenuService::write();
                        menu.view_mode = StartMenuViewMode::AllApps;
                        menu.selected = 0;
                        menu.scroll_row = 0;
                    })
                    .child(
                        Text::new("All apps ❯")
                            .size(typography::CAPTION)
                            .color(tokens.accent)
                            .font(font),
                    ),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let content_widget: Box<dyn Widget> = if total_pinned == 0 {
            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(268.0)
                    .radius(radius::MD)
                    .fill(tokens.surface)
                    .border(1.0, tokens.border_subtle)
                    .align_child(Center, Center)
                    .child(
                        Column::new(vec![
                            Box::new(
                                Text::new("󰤱")
                                    .size(32.0)
                                    .color(tokens.accent)
                                    .font(&tokens.font_mono),
                            ),
                            Box::new(
                                Text::new("No pinned applications")
                                    .size(typography::BODY)
                                    .color(tokens.text_primary)
                                    .font(&tokens.font_sans),
                            ),
                            Box::new(
                                Text::new(
                                    "Right-click any app or click 󰤱 in 'All apps' to pin it here",
                                )
                                .size(typography::CAPTION)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                            ),
                            Box::new(
                                Rectangle::new()
                                    .width(170.0)
                                    .height(30.0)
                                    .radius(radius::SM)
                                    .fill(tokens.accent_tint)
                                    .border(1.0, tokens.border_focus)
                                    .cursor(Pointer)
                                    .align_child(Center, Center)
                                    .on_click(|_| {
                                        let mut menu = StartMenuService::write();
                                        menu.view_mode = StartMenuViewMode::AllApps;
                                        menu.selected = 0;
                                        menu.scroll_row = 0;
                                    })
                                    .child(
                                        Text::new("Browse All Apps ❯")
                                            .size(typography::SECONDARY)
                                            .color(tokens.accent)
                                            .font(&tokens.font_sans),
                                    ),
                            ),
                        ])
                        .align(Center)
                        .gap(spacing::GAP_SM),
                    ),
            )
        } else {
            let page_start = current_page * items_per_page;
            let tile_height = if total_pages > 1 { 80.0 } else { 84.0 };
            let mut rows: Vec<Box<dyn Widget>> = Vec::new();

            for r in 0..PINNED_ROWS {
                let mut col_items: Vec<Box<dyn Widget>> = Vec::new();
                for c in 0..PINNED_COLS {
                    let page_idx = r * PINNED_COLS + c;
                    let global_idx = page_start + page_idx;
                    if global_idx < total_pinned {
                        let app = &pinned_apps_list[global_idx];
                        let is_selected = global_idx == menu.selected;
                        let launched = app.clone();
                        let pin_name = app.name().to_string();

                        let icon_widget: Box<dyn Widget> = match app.icon_path() {
                            Some(path) => Box::new(
                                Rectangle::new()
                                    .width(32.0)
                                    .height(32.0)
                                    .fill(Image::contain(path)),
                            ),
                            None => Box::new(
                                Rectangle::new()
                                    .width(32.0)
                                    .height(32.0)
                                    .radius(radius::SM)
                                    .fill(tokens.surface)
                                    .align_child(Center, Center)
                                    .child(
                                        Text::new("󰵆").size(18.0).color(tokens.accent).font(font),
                                    ),
                            ),
                        };

                        let (line1, line2_opt) = format_tile_label(app.name());

                        let mut label_children: Vec<Box<dyn Widget>> = vec![
                            icon_widget,
                            Box::new(
                                Text::new(&line1)
                                    .size(typography::CAPTION)
                                    .color(tokens.text_primary)
                                    .font(font),
                            ),
                        ];

                        if let Some(line2) = line2_opt {
                            label_children.push(Box::new(
                                Text::new(&line2)
                                    .size(9.5)
                                    .color(tokens.text_secondary)
                                    .font(font),
                            ));
                        }

                        let tile = Rectangle::new()
                            .width(92.0)
                            .height(tile_height)
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
                                top: 5.0,
                                bottom: 5.0,
                                left: 4.0,
                                right: 4.0,
                            })
                            .align_child(Center, Center)
                            .on_click(move |btn| match btn {
                                Button::Left => {
                                    launched.launch();
                                    close();
                                }
                                Button::Right => {
                                    unpin_app(&pin_name);
                                }
                                _ => {}
                            })
                            .child(Column::new(label_children).align(Center).gap(1.5));

                        col_items.push(Box::new(tile));
                    } else {
                        col_items.push(Box::new(Rectangle::new().width(92.0).height(tile_height)));
                    }
                }
                rows.push(Box::new(
                    Row::new(col_items)
                        .gap(spacing::GAP_SM)
                        .align(Align::Center),
                ));
            }

            let grid_col = Column::new(rows)
                .gap(if total_pages > 1 {
                    6.0
                } else {
                    spacing::GAP_SM
                })
                .width(Parent)
                .align(Align::Center);

            let mut container_children: Vec<Box<dyn Widget>> = vec![Box::new(grid_col)];

            if total_pages > 1 {
                let mut dot_widgets: Vec<Box<dyn Widget>> = Vec::new();
                for p in 0..total_pages {
                    let is_current = p == current_page;
                    let target_sel = p * items_per_page;
                    let dot = Rectangle::new()
                        .width(if is_current { 16.0 } else { 6.0 })
                        .height(6.0)
                        .radius(3.0)
                        .fill(if is_current {
                            tokens.accent
                        } else {
                            tokens.border_subtle
                        })
                        .cursor(Pointer)
                        .on_click(move |_| {
                            let mut menu = StartMenuService::write();
                            menu.selected = target_sel;
                        });
                    dot_widgets.push(Box::new(dot));
                }

                let dots_row = Row::new(dot_widgets).gap(6.0).align(Align::Center);
                container_children.push(Box::new(dots_row));
            }

            let container_column = Column::new(container_children)
                .gap(6.0)
                .width(Parent)
                .align(Align::Center);

            Box::new(
                Rectangle::new()
                    .width(Parent)
                    .height(268.0)
                    .on_scroll(move |scroll| {
                        if total_pages > 1 {
                            let mut menu = StartMenuService::write();
                            let items = PINNED_COLS * PINNED_ROWS;
                            let cur = menu.selected / items;
                            if scroll.y > 0.0 && cur + 1 < total_pages {
                                menu.selected = ((cur + 1) * items).min(total_pinned - 1);
                            } else if scroll.y < 0.0 && cur > 0 {
                                menu.selected = (cur - 1) * items;
                            }
                        }
                    })
                    .child(container_column),
            ) as Box<dyn Widget>
        };

        (content_widget, header)
    };

    // 3. Recommended Section (Only in Pinned Mode)
    let recommended_section: Box<dyn Widget> =
        if query_raw.is_empty() && current_view == StartMenuViewMode::Pinned {
            Box::new(render_recommended_section(font, &tokens))
        } else {
            Box::new(Rectangle::new().width(Parent).height(0.0))
        };

    // 4. User Profile & Power Controls Footer
    let username = std::env::var("USER").unwrap_or_else(|_| "hypokoto".to_string());
    let display_name = if username == "developer" || username.is_empty() {
        "hypokoto"
    } else {
        &username
    };

    let user_info = Row::new(vec![
        Box::new(
            Rectangle::new()
                .width(36.0)
                .height(36.0)
                .radius(radius::PILL)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.accent)
                .align_child(Center, Center)
                .child(Text::new("").size(16.0).color(tokens.accent).font(font)),
        ),
        Box::new(
            Column::new(vec![
                Box::new(
                    Text::new(display_name)
                        .size(typography::BODY)
                        .color(tokens.text_primary)
                        .font(font),
                ),
                Box::new(
                    Text::new("Signed in • Wayland")
                        .size(typography::CAPTION)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
            ])
            .align(Align::Start)
            .gap(1.0),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let power_buttons = Row::new(vec![
        Box::new(render_power_icon(
            "󰌾",
            "Lock",
            tokens.steel,
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
            tokens.steel,
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
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let mut content_elements: Vec<Box<dyn Widget>> =
        vec![Box::new(search_bar), Box::new(header_row), content_section];

    if query_raw.is_empty() && current_view == StartMenuViewMode::Pinned {
        content_elements.push(Box::new(
            Rectangle::new()
                .width(Parent)
                .height(1.0)
                .fill(tokens.border_subtle),
        ));
        content_elements.push(recommended_section);
    }

    content_elements.push(Box::new(
        Rectangle::new()
            .width(Parent)
            .height(1.0)
            .fill(tokens.border_subtle),
    ));
    content_elements.push(Box::new(footer_row));

    let start_menu_content = Column::new(content_elements)
        .gap(spacing::GAP_MD)
        .width(Parent)
        .height(Parent)
        .justify(Justify::SpaceBetween);

    // 640px width x 604px height with 12px corner radius (radius::XL)
    let container = Rectangle::new()
        .width(640.0)
        .height(604.0)
        .fill(tokens.acrylic_bg)
        .border(1.0, tokens.acrylic_border)
        .radius(radius::XL)
        .padding(Padding {
            top: spacing::PAD_LG,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_LG,
            right: spacing::PAD_LG,
        })
        .child(start_menu_content);

    let bottom_margin = (dock_height + 10.0) as i32;

    LayerWindow::new()
        .width(640.0)
        .height(604.0)
        .anchor_horizontal(Horizontal::Middle)
        .anchor_vertical(Vertical::Bottom)
        .margin(Margin {
            top: 12,
            right: 0,
            bottom: bottom_margin,
            left: 0,
        })
        .layer(Layer::Overlay)
        .target_output(OutputTarget::Active)
        .namespace("dotfield-shell-start-menu")
        .space(Zone::Ignore)
        .keyboard(Keyboard::Exclusive)
        .on_key(|key| match key {
            Key::Escape => {
                let mut menu = StartMenuService::write();
                if !menu.query.is_empty() {
                    menu.query.clear();
                    TextInput::set_text("start-menu-search", "");
                } else if menu.view_mode == StartMenuViewMode::AllApps {
                    menu.view_mode = StartMenuViewMode::Pinned;
                    menu.selected = 0;
                    menu.scroll_row = 0;
                } else {
                    menu.open = false;
                }
            }
            Key::Enter => {
                handle_enter_submission();
            }
            Key::Right => {
                let apps = Apps::read();
                let mut menu = StartMenuService::write();
                let query_lower = menu.query.trim().to_lowercase();
                let is_pinned =
                    menu.query.is_empty() && menu.view_mode == StartMenuViewMode::Pinned;
                let count = if is_pinned {
                    menu.pinned_apps.len()
                } else {
                    apps.list()
                        .iter()
                        .filter(|&a| {
                            query_lower.is_empty()
                                || a.name().to_lowercase().contains(&query_lower)
                                || a.description()
                                    .is_some_and(|d| d.to_lowercase().contains(&query_lower))
                        })
                        .count()
                };
                if count > 0 {
                    menu.selected = (menu.selected + 1).min(count - 1);
                }
            }
            Key::Left => {
                let mut menu = StartMenuService::write();
                menu.selected = menu.selected.saturating_sub(1);
            }
            Key::Down => {
                let apps = Apps::read();
                let mut menu = StartMenuService::write();
                let query_lower = menu.query.trim().to_lowercase();
                let is_pinned =
                    menu.query.is_empty() && menu.view_mode == StartMenuViewMode::Pinned;
                let step = if is_pinned { PINNED_COLS } else { LIST_COLS };
                let count = if is_pinned {
                    menu.pinned_apps.len()
                } else {
                    apps.list()
                        .iter()
                        .filter(|&a| {
                            query_lower.is_empty()
                                || a.name().to_lowercase().contains(&query_lower)
                                || a.description()
                                    .is_some_and(|d| d.to_lowercase().contains(&query_lower))
                        })
                        .count()
                };
                if count > 0 {
                    menu.selected = (menu.selected + step).min(count - 1);
                    if !is_pinned {
                        let cur_row = menu.selected / LIST_COLS;
                        if cur_row >= menu.scroll_row + LIST_VISIBLE_ROWS {
                            menu.scroll_row = cur_row + 1 - LIST_VISIBLE_ROWS;
                        }
                    }
                }
            }
            Key::Up => {
                let mut menu = StartMenuService::write();
                let is_pinned =
                    menu.query.is_empty() && menu.view_mode == StartMenuViewMode::Pinned;
                let step = if is_pinned { PINNED_COLS } else { LIST_COLS };
                menu.selected = menu.selected.saturating_sub(step);
                if !is_pinned {
                    let cur_row = menu.selected / LIST_COLS;
                    if cur_row < menu.scroll_row {
                        menu.scroll_row = cur_row;
                    }
                }
            }
            Key::PageDown => {
                let mut menu = StartMenuService::write();
                let is_pinned =
                    menu.query.is_empty() && menu.view_mode == StartMenuViewMode::Pinned;
                if is_pinned {
                    let items_per_page = PINNED_COLS * PINNED_ROWS;
                    let total_pinned = menu.pinned_apps.len();
                    if total_pinned > 0 {
                        menu.selected = (menu.selected + items_per_page).min(total_pinned - 1);
                    }
                } else {
                    let apps = Apps::read();
                    let query_lower = menu.query.trim().to_lowercase();
                    let count = apps
                        .list()
                        .iter()
                        .filter(|&a| {
                            query_lower.is_empty()
                                || a.name().to_lowercase().contains(&query_lower)
                                || a.description()
                                    .is_some_and(|d| d.to_lowercase().contains(&query_lower))
                        })
                        .count();
                    let total_rows = count.div_ceil(LIST_COLS);
                    menu.scroll_row = (menu.scroll_row + LIST_VISIBLE_ROWS)
                        .min(total_rows.saturating_sub(LIST_VISIBLE_ROWS));
                    menu.selected = (menu.selected + LIST_VISIBLE_ROWS * LIST_COLS)
                        .min(count.saturating_sub(1));
                }
            }
            Key::PageUp => {
                let mut menu = StartMenuService::write();
                let is_pinned =
                    menu.query.is_empty() && menu.view_mode == StartMenuViewMode::Pinned;
                if is_pinned {
                    let items_per_page = PINNED_COLS * PINNED_ROWS;
                    menu.selected = menu.selected.saturating_sub(items_per_page);
                } else {
                    menu.scroll_row = menu.scroll_row.saturating_sub(LIST_VISIBLE_ROWS);
                    menu.selected = menu.selected.saturating_sub(LIST_VISIBLE_ROWS * LIST_COLS);
                }
            }
            _ => {}
        })
        .visible(true)
        .child(container)
}

fn render_recommended_section(font: &str, tokens: &ThemeTokens) -> Rectangle {
    let header = Text::new("Recommended")
        .size(typography::TITLE_SM)
        .color(tokens.text_primary)
        .font(font);

    let item_terminal = render_quick_action_item(
        "󰆍",
        "Ghostty Terminal",
        "Shell prompt",
        font,
        tokens,
        || {
            let _ = std::process::Command::new("ghostty").spawn();
            close();
        },
    );

    let item_files = render_quick_action_item(
        "󰉋",
        "Yazi Explorer",
        "Terminal file manager",
        font,
        tokens,
        || {
            let _ = std::process::Command::new("ghostty")
                .arg("-e")
                .arg("yazi")
                .spawn();
            close();
        },
    );

    let item_nvim = render_quick_action_item(
        "󰅩",
        "Neovim Editor",
        "Modal code editor",
        font,
        tokens,
        || {
            let _ = std::process::Command::new("ghostty")
                .arg("-e")
                .arg("nvim")
                .spawn();
            close();
        },
    );

    let item_btop = render_quick_action_item(
        "󰓅",
        "Btop Monitor",
        "System telemetry HUD",
        font,
        tokens,
        || {
            let _ = std::process::Command::new("ghostty")
                .arg("-e")
                .arg("btop")
                .spawn();
            close();
        },
    );

    let row1 = Row::new(vec![Box::new(item_terminal), Box::new(item_files)])
        .gap(spacing::GAP_MD)
        .width(Parent);

    let row2 = Row::new(vec![Box::new(item_nvim), Box::new(item_btop)])
        .gap(spacing::GAP_MD)
        .width(Parent);

    let content = Column::new(vec![Box::new(header), Box::new(row1), Box::new(row2)])
        .gap(spacing::GAP_SM)
        .width(Parent);

    Rectangle::new().width(Parent).height(124.0).child(content)
}

fn render_quick_action_item<F>(
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
    font: &str,
    tokens: &ThemeTokens,
    on_click: F,
) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    let icon_part = Text::new(icon).size(20.0).color(tokens.accent).font(font);

    let text_part = Column::new(vec![
        Box::new(
            Text::new(title)
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(font),
        ),
        Box::new(
            Text::new(desc)
                .size(typography::CAPTION)
                .color(tokens.text_secondary)
                .font(font),
        ),
    ])
    .align(Align::Start)
    .gap(1.0);

    let row = Row::new(vec![Box::new(icon_part), Box::new(text_part)])
        .gap(spacing::GAP_SM)
        .align(Align::Center);

    Rectangle::new()
        .width(292.0)
        .height(44.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .on_click(move |_| on_click())
        .child(row)
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
        .width(36.0)
        .height(36.0)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(Text::new(icon).size(16.0).color(color).font(font))
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

    #[test]
    fn test_format_tile_label() {
        let (l1, l2) = format_tile_label("Alacritty");
        assert_eq!(l1, "Alacritty");
        assert!(l2.is_none());

        let (l1, l2) = format_tile_label("Advanced Network Configuration");
        assert_eq!(l1, "Advanced");
        assert_eq!(l2.unwrap(), "Network Co…");

        let (l1, l2) = format_tile_label("ArtCraft Launcher");
        assert_eq!(l1, "ArtCraft");
        assert_eq!(l2.unwrap(), "Launcher");
    }

    #[test]
    fn test_get_shell_actions() {
        let actions = get_shell_actions();
        assert!(actions.iter().any(|a| a.id == "control-center"));
        assert!(actions.iter().any(|a| a.id == "power"));
        assert!(actions.iter().any(|a| a.id == "display"));
    }

    #[test]
    fn test_start_menu_grid_dimensions() {
        assert_eq!(LIST_VISIBLE_ROWS, 7);
        assert_eq!(PINNED_COLS, 6);
        assert_eq!(PINNED_ROWS, 3);
        assert_eq!(LIST_COLS, 2);
    }

    #[test]
    fn test_pin_and_unpin_apps_logic() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        // Clear pinned apps for clean test
        {
            let mut menu = StartMenuService::write();
            menu.pinned_apps.clear();
        }
        assert!(!is_pinned("TestApp"));

        pin_app("TestApp");
        assert!(is_pinned("TestApp"));
        assert!(is_pinned("testapp")); // Case-insensitive check

        // Duplicate pin should be idempotent
        pin_app("TestApp");
        assert_eq!(StartMenuService::read().pinned_apps.len(), 1);

        // Toggle pin
        toggle_pin_app("TestApp");
        assert!(!is_pinned("TestApp"));

        toggle_pin_app("AnotherApp");
        assert!(is_pinned("AnotherApp"));

        unpin_app("AnotherApp");
        assert!(!is_pinned("AnotherApp"));
    }

    #[test]
    fn test_pinned_apps_pagination_math() {
        let items_per_page = PINNED_COLS * PINNED_ROWS;
        assert_eq!(items_per_page, 18);

        // 0 pinned apps -> 1 page
        let total_pages_0 = 0usize.div_ceil(items_per_page).max(1);
        assert_eq!(total_pages_0, 1);

        // 18 pinned apps -> 1 page
        let total_pages_18 = 18usize.div_ceil(items_per_page).max(1);
        assert_eq!(total_pages_18, 1);

        // 19 pinned apps -> 2 pages
        let total_pages_19 = 19usize.div_ceil(items_per_page).max(1);
        assert_eq!(total_pages_19, 2);

        // 36 pinned apps -> 2 pages
        let total_pages_36 = 36usize.div_ceil(items_per_page).max(1);
        assert_eq!(total_pages_36, 2);

        // 37 pinned apps -> 3 pages
        let total_pages_37 = 37usize.div_ceil(items_per_page).max(1);
        assert_eq!(total_pages_37, 3);

        // Page indexing
        let selected_0 = 0;
        assert_eq!(selected_0 / items_per_page, 0);

        let selected_17 = 17;
        assert_eq!(selected_17 / items_per_page, 0);

        let selected_18 = 18;
        assert_eq!(selected_18 / items_per_page, 1);
    }
}
