//! Adaptive Centered Application Dock for Amane desktop shell.
//! Provides Windows 11 style centered app icons, 3px running indicator pills,
//! Start Menu trigger, Search quick-pill, and Task View (Niri Overview) summon triggers.

use std::path::PathBuf;

use amane::{
    Align, Apps, Center, Color, Image, Padding, Pointer, Rectangle, Row, Service, Text, Widget,
};
use niri_ipc::Window;

use crate::config::ShellManifest;
use crate::services::niri::NiriService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

#[derive(Clone, Debug)]
pub struct PinnedApp {
    pub name: &'static str,
    pub icon: &'static str,
    pub app_id_pattern: &'static str,
    pub exec_cmd: &'static str,
    pub exec_args: &'static [&'static str],
}

pub const DEFAULT_PINNED_APPS: &[PinnedApp] = &[
    PinnedApp {
        name: "Terminal",
        icon: "󰆍",
        app_id_pattern: "ghostty",
        exec_cmd: "ghostty",
        exec_args: &[],
    },
    PinnedApp {
        name: "Browser",
        icon: "󰈹",
        app_id_pattern: "firefox",
        exec_cmd: "firefox",
        exec_args: &[],
    },
    PinnedApp {
        name: "Files",
        icon: "󰉋",
        app_id_pattern: "yazi",
        exec_cmd: "ghostty",
        exec_args: &["-e", "yazi"],
    },
    PinnedApp {
        name: "Editor",
        icon: "󰅩",
        app_id_pattern: "nvim",
        exec_cmd: "ghostty",
        exec_args: &["-e", "nvim"],
    },
];

pub fn render_dock(manifest: &ShellManifest, tokens: &ThemeTokens) -> (Box<dyn Widget>, f32) {
    let font = &manifest.theme.font_family;
    let windows = NiriService::cached_windows();
    let focused_win = NiriService::focused_window();

    let mut dock_items: Vec<Box<dyn Widget>> = Vec::new();
    let mut total_width = 0.0;

    // 1. Start Button (Arch Linux logo, bold & elevated in Fluent / Arch Blue)
    let start_btn = render_dock_trigger(
        "󰣇",
        22.0,
        tokens.accent,
        "Start Menu (Super)",
        font,
        tokens,
        || {
            crate::ui::start_menu::toggle();
        },
    );
    dock_items.push(Box::new(start_btn));
    total_width += 40.0 + spacing::GAP_SM;

    // 2. Search Quick-Launch Pill (Windows 11 style)
    let search_btn = render_search_pill(font, tokens, || {
        crate::ui::launcher::toggle();
    });
    dock_items.push(Box::new(search_btn));
    total_width += 112.0 + spacing::GAP_SM;

    // 3. Task View Button (Niri Overview)
    let task_view_btn = render_dock_trigger(
        "󱂬",
        19.0,
        tokens.text_secondary,
        "Task View (Super+Tab)",
        font,
        tokens,
        || {
            let _ = NiriService::toggle_overview();
        },
    );
    dock_items.push(Box::new(task_view_btn));
    total_width += 40.0 + spacing::GAP_MD;

    // Divider between system triggers and apps
    let divider = Rectangle::new()
        .width(1.0)
        .height(18.0)
        .fill(Color::from("#ffffff1f"));
    dock_items.push(Box::new(divider));
    total_width += 1.0 + spacing::GAP_MD;

    // 4. Pinned Apps with active/background indicators
    for pinned in DEFAULT_PINNED_APPS {
        let matching_wins: Vec<&Window> = windows
            .iter()
            .filter(|w| {
                w.app_id
                    .as_deref()
                    .is_some_and(|id| id.to_lowercase().contains(pinned.app_id_pattern))
                    || w.title
                        .as_deref()
                        .is_some_and(|t| t.to_lowercase().contains(pinned.app_id_pattern))
            })
            .collect();

        let is_running = !matching_wins.is_empty();
        let is_focused = matching_wins.iter().any(|w| w.id == focused_win.id);
        let first_win_id = matching_wins.first().map(|w| w.id);

        let exec_cmd = pinned.exec_cmd;
        let exec_args = pinned.exec_args.to_vec();
        let icon_path = find_app_icon_path(pinned.app_id_pattern);

        let app_tile = render_app_tile(
            pinned.icon,
            icon_path,
            is_running,
            is_focused,
            font,
            tokens,
            move || {
                if let Some(id) = first_win_id {
                    let _ = NiriService::focus_window(id);
                } else {
                    let mut cmd = std::process::Command::new(exec_cmd);
                    cmd.args(&exec_args);
                    let _ = cmd.spawn();
                }
            },
        );

        dock_items.push(Box::new(app_tile));
        total_width += 40.0 + spacing::GAP_SM;
    }

    // 5. Dynamic Unpinned Running Apps
    for win in windows.iter() {
        let app_id = win.app_id.as_deref().unwrap_or("window").to_lowercase();
        let is_pinned = DEFAULT_PINNED_APPS.iter().any(|p| {
            app_id.contains(p.app_id_pattern)
                || win
                    .title
                    .as_deref()
                    .is_some_and(|t| t.to_lowercase().contains(p.app_id_pattern))
        });

        if !is_pinned {
            let win_id = win.id;
            let is_focused = win_id == focused_win.id;
            let icon_path = find_app_icon_path(&app_id);
            let icon = resolve_app_icon(&app_id);

            let app_tile =
                render_app_tile(icon, icon_path, true, is_focused, font, tokens, move || {
                    let _ = NiriService::focus_window(win_id);
                });

            dock_items.push(Box::new(app_tile));
            total_width += 40.0 + spacing::GAP_SM;
        }
    }

    let dock_row = Row::new(dock_items)
        .gap(spacing::GAP_SM)
        .align(Align::Center);

    (Box::new(dock_row), total_width)
}

fn render_dock_trigger<F>(
    icon: &'static str,
    icon_size: f32,
    icon_color: Color,
    _tooltip: &'static str,
    font: &str,
    _tokens: &ThemeTokens,
    on_click: F,
) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    Rectangle::new()
        .width(40.0)
        .height(40.0)
        .radius(radius::MD)
        .fill(Color::TRANSPARENT)
        .border(1.0, Color::TRANSPARENT)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(Text::new(icon).size(icon_size).color(icon_color).font(font))
}

fn render_search_pill<F>(font: &str, tokens: &ThemeTokens, on_click: F) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    Rectangle::new()
        .width(112.0)
        .height(34.0)
        .radius(radius::PILL)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .on_click(move |_| on_click())
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("")
                        .size(14.0)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
                Box::new(
                    Text::new("Search")
                        .size(typography::BODY)
                        .color(tokens.text_secondary)
                        .font(font),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        )
}

fn render_app_tile<F>(
    fallback_icon: &'static str,
    icon_path: Option<PathBuf>,
    is_running: bool,
    is_focused: bool,
    font: &str,
    tokens: &ThemeTokens,
    on_click: F,
) -> Rectangle
where
    F: Fn() + Send + Sync + 'static,
{
    let bg_color = if is_focused {
        Color::from("#ffffff14")
    } else {
        Color::TRANSPARENT
    };

    let border_color = Color::TRANSPARENT;

    let icon_color = if is_focused || is_running {
        tokens.text_primary
    } else {
        tokens.text_secondary
    };

    // Windows 11 Active indicator:
    // 3px height pill: 16px wide when active/focused, 6px dot when running in background, transparent when closed
    let pip_width = if is_focused {
        16.0
    } else if is_running {
        6.0
    } else {
        0.0
    };

    let pip_color = if is_focused {
        tokens.accent
    } else if is_running {
        tokens.steel
    } else {
        Color::TRANSPARENT
    };

    let pip = Rectangle::new()
        .width(pip_width)
        .height(3.0)
        .radius(radius::PILL)
        .fill(pip_color);

    let icon_widget: Box<dyn Widget> = if let Some(path) = icon_path {
        Box::new(
            Rectangle::new()
                .width(22.0)
                .height(22.0)
                .fill(Image::contain(path)),
        )
    } else {
        Box::new(
            Text::new(fallback_icon)
                .size(20.0)
                .color(icon_color)
                .font(font),
        )
    };

    let content_col = amane::Column::new(vec![icon_widget, Box::new(pip)])
        .align(Center)
        .gap(2.0);

    Rectangle::new()
        .width(40.0)
        .height(40.0)
        .radius(radius::MD)
        .fill(bg_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .padding(Padding {
            top: 3.0,
            bottom: 2.0,
            left: 2.0,
            right: 2.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(content_col)
}

fn find_app_icon_path(app_id_or_name: &str) -> Option<PathBuf> {
    let lower = app_id_or_name.to_lowercase();

    // 1. Direct system icons check in Fluent theme (Windows 11 vector SVGs)
    if lower.contains("ghostty")
        || lower.contains("term")
        || lower.contains("foot")
        || lower.contains("kitty")
    {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/com.mitchellh.ghostty.svg");
        if p.exists() {
            return Some(p);
        }
        let p2 = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/utilities-terminal.svg");
        if p2.exists() {
            return Some(p2);
        }
    }
    if lower.contains("fire") || lower.contains("browser") || lower.contains("zen") {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/firefox.svg");
        if p.exists() {
            return Some(p);
        }
    }
    if lower.contains("yazi")
        || lower.contains("file")
        || lower.contains("thunar")
        || lower.contains("nautilus")
    {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/file-manager.svg");
        if p.exists() {
            return Some(p);
        }
    }
    if lower.contains("nvim") || lower.contains("vim") || lower.contains("edit") {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/nvim.svg");
        if p.exists() {
            return Some(p);
        }
    }
    if lower.contains("code") {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/code.svg");
        if p.exists() {
            return Some(p);
        }
    }
    if lower.contains("disc") || lower.contains("vesktop") {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/discord.svg");
        if p.exists() {
            return Some(p);
        }
    }
    if lower.contains("spotify") || lower.contains("music") {
        let p = PathBuf::from("/usr/share/icons/Fluent/scalable/apps/spotify.svg");
        if p.exists() {
            return Some(p);
        }
    }

    // 2. Apps Service dynamic desktop icon lookup
    let apps = Apps::read();
    if let Some(path) = apps.list().iter().find_map(|app| {
        let app_name = app.name().to_lowercase();
        let app_exec = app.exec().to_lowercase();
        let app_icon = app.icon().unwrap_or("").to_lowercase();

        let is_match = app_name.contains(&lower)
            || app_exec.contains(&lower)
            || app_icon.contains(&lower)
            || lower.contains(&app_name);

        if is_match {
            app.icon_path()
                .filter(|p| p.exists())
                .map(|p| p.to_path_buf())
        } else {
            None
        }
    }) {
        return Some(path);
    }

    // 3. Common hicolor / pixmaps fallback paths
    let fallbacks = [
        format!("/usr/share/icons/hicolor/scalable/apps/{lower}.svg"),
        format!("/usr/share/pixmaps/{lower}.png"),
        format!("/usr/share/pixmaps/{lower}.svg"),
    ];
    for fb in &fallbacks {
        let p = PathBuf::from(fb);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

fn resolve_app_icon(app_id: &str) -> &'static str {
    if app_id.contains("term")
        || app_id.contains("foot")
        || app_id.contains("ghostty")
        || app_id.contains("kitty")
        || app_id.contains("alacritty")
    {
        "󰆍"
    } else if app_id.contains("fire")
        || app_id.contains("zen")
        || app_id.contains("chrome")
        || app_id.contains("web")
        || app_id.contains("browser")
    {
        "󰈹"
    } else if app_id.contains("code") || app_id.contains("nvim") || app_id.contains("edit") {
        "󰅩"
    } else if app_id.contains("file") || app_id.contains("yazi") || app_id.contains("thunar") {
        "󰉋"
    } else if app_id.contains("music") || app_id.contains("spotify") || app_id.contains("audio") {
        "󰝚"
    } else if app_id.contains("disc") || app_id.contains("elem") || app_id.contains("slack") {
        "󰙯"
    } else if app_id.contains("virt") || app_id.contains("box") || app_id.contains("qemu") {
        "󰢹"
    } else {
        "󰵆"
    }
}
