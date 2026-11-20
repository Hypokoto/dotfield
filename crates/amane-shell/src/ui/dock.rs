//! Adaptive Centered Application Dock for Amane desktop shell.
//! Provides Windows 11 / Material You style centered app icons, running indicator pips,
//! and Start Menu / Widgets summon triggers.

use amane::{Align, Center, Color, Padding, Pointer, Rectangle, Row, Text, Widget};
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
        exec_cmd: "terminal-select.sh",
        exec_args: &[],
    },
    PinnedApp {
        name: "Editor",
        icon: "󰅩",
        app_id_pattern: "nvim",
        exec_cmd: "terminal-select.sh",
        exec_args: &["-e", "nvim"],
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
        exec_cmd: "terminal-select.sh",
        exec_args: &["-e", "yazi"],
    },
    PinnedApp {
        name: "Dev Tools",
        icon: "󰘬",
        app_id_pattern: "dev.",
        exec_cmd: "terminal-select.sh",
        exec_args: &[],
    },
];

pub fn render_dock(manifest: &ShellManifest, tokens: &ThemeTokens) -> (Box<dyn Widget>, f32) {
    let font = &manifest.theme.font_family;
    let windows = NiriService::query_windows().unwrap_or_default();
    let focused_win = NiriService::focused_window();

    let mut dock_items: Vec<Box<dyn Widget>> = Vec::new();
    let mut total_width = 0.0;

    // 1. Start Menu Button
    let start_btn = render_dock_trigger(
        "󰣇",
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

    // 2. Widgets Board Trigger Button
    let widgets_btn = render_dock_trigger(
        "󱂬",
        tokens.text_secondary,
        "Widgets (Mod+W)",
        font,
        tokens,
        || {
            crate::ui::widgets_board::toggle();
        },
    );
    dock_items.push(Box::new(widgets_btn));
    total_width += 40.0 + spacing::GAP_MD;

    // Divider between triggers and apps
    let divider = Rectangle::new()
        .width(1.0)
        .height(20.0)
        .fill(tokens.border_subtle);
    dock_items.push(Box::new(divider));
    total_width += 1.0 + spacing::GAP_MD;

    // 3. Pinned Apps
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

        let app_tile = render_app_tile(
            pinned.icon,
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

    // 4. Dynamic Unpinned Running Apps
    for win in &windows {
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
            let icon = resolve_app_icon(&app_id);

            let app_tile = render_app_tile(icon, true, is_focused, font, tokens, move || {
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
    icon_color: Color,
    _tooltip: &'static str,
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
        .radius(radius::SM)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(
            Text::new(icon)
                .size(typography::BODY + 1.0)
                .color(icon_color)
                .font(font),
        )
}

fn render_app_tile<F>(
    icon: &'static str,
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
        tokens.surface_elevated
    } else {
        tokens.surface
    };

    let border_color = if is_focused {
        tokens.accent
    } else {
        tokens.border_subtle
    };

    let icon_color = if is_focused {
        tokens.accent
    } else if is_running {
        tokens.text_primary
    } else {
        tokens.text_secondary
    };

    // Pip indicator below or inside tile
    let pip_color = if is_focused {
        tokens.accent
    } else if is_running {
        tokens.steel
    } else {
        Color::TRANSPARENT
    };

    let pip = Rectangle::new()
        .width(if is_focused { 12.0 } else { 4.0 })
        .height(3.0)
        .radius(radius::PILL)
        .fill(pip_color);

    let icon_text = Text::new(icon)
        .size(typography::BODY + 1.0)
        .color(icon_color)
        .font(font);

    let content_col = amane::Column::new(vec![Box::new(icon_text), Box::new(pip)])
        .align(Center)
        .gap(2.0);

    Rectangle::new()
        .width(36.0)
        .height(36.0)
        .radius(radius::SM)
        .fill(bg_color)
        .border(1.0, border_color)
        .cursor(Pointer)
        .padding(Padding {
            top: 4.0,
            bottom: 2.0,
            left: 2.0,
            right: 2.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| on_click())
        .child(content_col)
}

fn resolve_app_icon(app_id: &str) -> &'static str {
    if app_id.contains("term")
        || app_id.contains("foot")
        || app_id.contains("ghostty")
        || app_id.contains("kitty")
    {
        "󰆍"
    } else if app_id.contains("fire")
        || app_id.contains("zen")
        || app_id.contains("chrome")
        || app_id.contains("web")
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
