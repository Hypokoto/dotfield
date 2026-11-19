use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use amane::{
    Align, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, OutputTarget, Padding,
    Parent, Pointer, Rectangle, Row, Scroll, Service, Text, TextInput, Vertical, Widget, Zone,
};

use crate::config::{ConfigService, ShellManifest};
use crate::services::niri::NiriService;
use crate::services::topology::{ProjectTarget, TopologyService};
use crate::ui::components::{action_card_interactive, modal_shell, status_badge};
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub const VISIBLE_PROJECT_ROWS: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagerTab {
    Projects,
    Terminals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchMode {
    Standard,
    Multiplexer,
    Scratchpad,
}

#[derive(Debug, Clone)]
pub struct TerminalPreset {
    pub id: &'static str,
    pub name: &'static str,
    pub glyph: &'static str,
    pub description: &'static str,
    pub tag: &'static str,
    pub bin: &'static str,
    pub standard_cmd: &'static str,
    pub mux_cmd: &'static str,
    pub scratchpad_cmd: &'static str,
}

pub static TERMINAL_PRESETS: &[TerminalPreset] = &[
    TerminalPreset {
        id: "ghostty",
        name: "Ghostty",
        glyph: "󰊠",
        description: "Modern GPU",
        tag: "Modern",
        bin: "ghostty",
        standard_cmd: "ghostty",
        mux_cmd: "ghostty -e tmux",
        scratchpad_cmd: "ghostty --class=floating-term",
    },
    TerminalPreset {
        id: "foot",
        name: "Foot",
        glyph: "󰞷",
        description: "Wayland client",
        tag: "Wayland",
        bin: "foot",
        standard_cmd: "foot",
        mux_cmd: "foot -e tmux",
        scratchpad_cmd: "foot --app-id=floating-term",
    },
    TerminalPreset {
        id: "kitty",
        name: "Kitty",
        glyph: "󰄛",
        description: "OpenGL GPU",
        tag: "OpenGL",
        bin: "kitty",
        standard_cmd: "kitty",
        mux_cmd: "kitty -e tmux",
        scratchpad_cmd: "kitty --class=floating-term",
    },
];

pub struct TerminalManagerService {
    pub open: bool,
    pub open_time: Option<Instant>,
    pub active_tab: ManagerTab,
    pub search_query: String,
    pub selected: usize,
    pub scroll_offset: usize,
    pub hover_index: Option<usize>,
    pub default_terminal: String,
    pub status_message: Option<String>,
}

impl Service for TerminalManagerService {
    fn new() -> Self {
        let default_term = detect_default_terminal();
        Self {
            open: false,
            open_time: None,
            active_tab: ManagerTab::Projects,
            search_query: String::new(),
            selected: 0,
            scroll_offset: 0,
            hover_index: None,
            default_terminal: default_term,
            status_message: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut svc = TerminalManagerService::write();
        svc.open = !svc.open;
        svc.open_time = if svc.open { Some(Instant::now()) } else { None };
        svc.active_tab = ManagerTab::Projects;
        svc.search_query.clear();
        svc.selected = 0;
        svc.scroll_offset = 0;
        svc.hover_index = None;
        svc.status_message = None;
        svc.default_terminal = detect_default_terminal();
        TextInput::set_text("terminal-manager-search", "");
        if svc.open {
            TopologyService::refresh_async();
        }
        svc.open
    };
    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::TerminalManager,
        );
    }
}

pub fn open() {
    let mut svc = TerminalManagerService::write();
    svc.open = true;
    svc.open_time = Some(Instant::now());
    svc.active_tab = ManagerTab::Projects;
    svc.search_query.clear();
    svc.selected = 0;
    svc.scroll_offset = 0;
    svc.hover_index = None;
    svc.status_message = None;
    svc.default_terminal = detect_default_terminal();
    TextInput::set_text("terminal-manager-search", "");
    TopologyService::refresh_async();
    drop(svc);

    crate::ui::modal_manager::ModalManager::open_exclusive(
        crate::ui::modal_manager::ModalType::TerminalManager,
    );
}

pub fn close() {
    let mut svc = TerminalManagerService::write();
    svc.open = false;
    svc.open_time = None;
    svc.search_query.clear();
    svc.selected = 0;
    svc.scroll_offset = 0;
    svc.hover_index = None;
    svc.status_message = None;
}

pub fn is_installed(bin: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let p = dir.join(bin);
            if p.is_file() {
                return true;
            }
        }
    }
    false
}

pub fn detect_default_terminal() -> String {
    let conf = ConfigService::read();
    if !conf.manifest.terminal.default.is_empty() {
        return conf.manifest.terminal.default.clone();
    }
    drop(conf);

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let shell_path = PathBuf::from(&home).join(".config/dotfield/shell.toml");
    if let Ok(content) = std::fs::read_to_string(shell_path)
        && let Ok(manifest) = toml::from_str::<ShellManifest>(&content)
        && !manifest.terminal.default.is_empty()
    {
        return manifest.terminal.default;
    }

    "ghostty".to_string()
}

/// Spawns a terminal window with specific app-id, optional working directory, and command arguments.
pub fn spawn_terminal_window(
    term_id: &str,
    app_id: &str,
    working_dir: Option<&std::path::Path>,
    command: Option<&[&str]>,
) -> anyhow::Result<()> {
    let mut cmd = match term_id {
        "foot" => {
            let mut c = Command::new("foot");
            c.arg(format!("--app-id={app_id}"));
            if let Some(dir) = working_dir {
                c.arg("-D").arg(dir);
            }
            if let Some(args) = command {
                c.args(args);
            }
            c
        }
        "kitty" => {
            let mut c = Command::new("kitty");
            c.arg(format!("--class={app_id}"));
            if let Some(dir) = working_dir {
                c.arg("-d").arg(dir);
            }
            if let Some(args) = command {
                c.arg("-e").args(args);
            }
            c
        }
        _ => {
            let mut c = Command::new("ghostty");
            c.arg(format!("--class={app_id}"));
            if let Some(dir) = working_dir {
                c.arg(format!("--working-directory={}", dir.display()));
            }
            if let Some(args) = command {
                c.arg("-e").args(args);
            }
            c
        }
    };
    cmd.spawn()?;
    Ok(())
}

pub fn set_default_terminal(index: usize) {
    if let Some(preset) = TERMINAL_PRESETS.get(index) {
        let term_id = preset.id;
        let mut svc = TerminalManagerService::write();
        svc.default_terminal = term_id.to_string();
        svc.status_message = Some(format!("Default set to {}", preset.name));
        drop(svc);

        ConfigService::write().manifest.terminal.default = term_id.to_string();

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let shell_path = PathBuf::from(&home).join(".config/dotfield/shell.toml");
        if shell_path.exists()
            && let Ok(content) = std::fs::read_to_string(&shell_path)
            && let Ok(mut manifest) = toml::from_str::<ShellManifest>(&content)
        {
            manifest.terminal.default = term_id.to_string();
            if let Ok(serialized) = toml::to_string_pretty(&manifest) {
                let _ = std::fs::write(&shell_path, serialized);
            }
        }

        if amane::Notifications::read().running() {
            amane::Notifications::notify(
                "dotfield",
                "Terminal Manager",
                format!("Default terminal updated to {}", preset.name),
                amane::Urgency::Normal,
            );
        } else {
            let _ = Command::new("notify-send")
                .args([
                    "-a",
                    "dotfield",
                    "-u",
                    "normal",
                    "Terminal Manager",
                    &format!("Default terminal updated to {}", preset.name),
                ])
                .spawn();
        }
    }
}

pub fn launch_preset(index: usize, mode: LaunchMode) {
    if let Some(preset) = TERMINAL_PRESETS.get(index) {
        if !is_installed(preset.bin) {
            let mut svc = TerminalManagerService::write();
            svc.status_message = Some(format!("{} is not installed!", preset.name));
            return;
        }

        let cmd = match mode {
            LaunchMode::Standard => preset.standard_cmd,
            LaunchMode::Multiplexer => preset.mux_cmd,
            LaunchMode::Scratchpad => preset.scratchpad_cmd,
        };

        let mode_desc = match mode {
            LaunchMode::Standard => "Standard",
            LaunchMode::Multiplexer => "Multiplexer",
            LaunchMode::Scratchpad => "Floating Scratchpad",
        };

        let mut parts = cmd.split_whitespace();
        if let Some(bin) = parts.next() {
            let args: Vec<&str> = parts.collect();
            let mut scope_cmd = Command::new("systemd-run");
            scope_cmd
                .args(["--user", "--scope", "--quiet", bin])
                .args(&args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());

            if scope_cmd.spawn().is_err() {
                use std::os::unix::process::CommandExt;
                let _ = Command::new(bin)
                    .args(&args)
                    .process_group(0)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn();
            }
        }

        if amane::Notifications::read().running() {
            amane::Notifications::notify(
                "dotfield",
                "Terminal Manager",
                format!("Launched {} ({mode_desc})", preset.name),
                amane::Urgency::Low,
            );
        } else {
            let _ = Command::new("notify-send")
                .args([
                    "-a",
                    "dotfield",
                    "-u",
                    "low",
                    "Terminal Manager",
                    &format!("Launched {} ({mode_desc})", preset.name),
                ])
                .spawn();
        }

        close();
    }
}

pub fn launch_in_dir(index: usize, dir: &str) {
    if let Some(preset) = TERMINAL_PRESETS.get(index) {
        if !is_installed(preset.bin) {
            let mut svc = TerminalManagerService::write();
            svc.status_message = Some(format!("{} is not installed!", preset.name));
            return;
        }

        let mut scope_cmd = Command::new("systemd-run");
        scope_cmd
            .args(["--user", "--scope", "--quiet", preset.bin])
            .current_dir(dir)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        if scope_cmd.spawn().is_err() {
            use std::os::unix::process::CommandExt;
            let _ = Command::new(preset.bin)
                .current_dir(dir)
                .process_group(0)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }

        close();
    }
}

pub fn get_filtered_projects(query: &str) -> Vec<ProjectTarget> {
    let all = TopologyService::list();
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return all.to_vec();
    }

    all.iter()
        .filter(|t| {
            t.name.to_lowercase().contains(&q)
                || t.slug.to_lowercase().contains(&q)
                || t.branch
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
                || t.path.to_string_lossy().to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

pub fn terminal_manager_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let svc = TerminalManagerService::read();

    let is_visible = svc.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-terminal-manager");
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let active_tab = svc.active_tab;
    let selected_idx = svc.selected;
    let default_id = svc.default_terminal.clone();

    // Tab switcher row
    let tab_projects = Rectangle::new()
        .width(190.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(if active_tab == ManagerTab::Projects {
            tokens.surface_selected
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if active_tab == ManagerTab::Projects {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(|_| {
            let mut svc = TerminalManagerService::write();
            svc.active_tab = ManagerTab::Projects;
            svc.selected = 0;
            svc.scroll_offset = 0;
            svc.hover_index = None;
        })
        .child(
            Text::new("󰘬 [ 01: PROJECTS & WORKTREES ]")
                .size(typography::BADGE)
                .color(if active_tab == ManagerTab::Projects {
                    tokens.accent
                } else {
                    tokens.text_secondary
                })
                .font(&tokens.font_family),
        );

    let tab_terminals = Rectangle::new()
        .width(170.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(if active_tab == ManagerTab::Terminals {
            tokens.surface_selected
        } else {
            tokens.surface_elevated
        })
        .border(
            1.0,
            if active_tab == ManagerTab::Terminals {
                tokens.border_focus
            } else {
                tokens.border_subtle
            },
        )
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(|_| {
            let mut svc = TerminalManagerService::write();
            svc.active_tab = ManagerTab::Terminals;
            svc.selected = 0;
            svc.hover_index = None;
        })
        .child(
            Text::new("󰆍 [ 02: QUICK TERMINALS ]")
                .size(typography::BADGE)
                .color(if active_tab == ManagerTab::Terminals {
                    tokens.accent
                } else {
                    tokens.text_secondary
                })
                .font(&tokens.font_family),
        );

    let tabs_row = Row::new(vec![Box::new(tab_projects), Box::new(tab_terminals)])
        .gap(spacing::GAP_SM)
        .align(Align::Center);

    let mut body_rows: Vec<Box<dyn Widget>> = vec![Box::new(tabs_row)];

    let total_items = match active_tab {
        ManagerTab::Projects => {
            let filtered = get_filtered_projects(&svc.search_query);
            let count = filtered.len();

            let count_badge = status_badge(
                &if count > 0 {
                    format!("{}/{} projects", (selected_idx + 1).min(count), count)
                } else {
                    "0 projects".to_string()
                },
                tokens.accent,
                &tokens,
            );

            let search_icon = Text::new("󰍉")
                .size(typography::TITLE_SM)
                .color(tokens.accent)
                .font(&tokens.font_family)
                .tight();

            let search_input = TextInput::new("terminal-manager-search")
                .size(typography::BODY)
                .color(tokens.text_primary)
                .placeholder("Search project, branch, or worktree...")
                .focused()
                .on_change(|text| {
                    let mut svc = TerminalManagerService::write();
                    svc.search_query = text;
                    svc.selected = 0;
                    svc.scroll_offset = 0;
                })
                .on_submit(|_| {
                    submit_selection();
                });

            let search_row = Row::new(vec![
                Box::new(search_icon),
                Box::new(search_input),
                Box::new(count_badge),
            ])
            .gap(spacing::GAP_MD)
            .align(Align::Center);

            let search_box = Rectangle::new()
                .width(Parent)
                .height(40.0)
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

            body_rows.push(Box::new(search_box));

            let scroll_offset = svc.scroll_offset;
            let mut project_cards: Vec<Box<dyn Widget>> = Vec::new();

            if count == 0 {
                let empty_card = Rectangle::new()
                    .width(Parent)
                    .height(80.0)
                    .radius(radius::MD)
                    .fill(tokens.surface_elevated)
                    .border(1.0, tokens.border_subtle)
                    .align_child(Align::Center, Align::Center)
                    .child(
                        Text::new(if svc.search_query.is_empty() {
                            "No project targets discovered in ~/dev/projects/".to_string()
                        } else {
                            format!("No projects matching \"{}\"", svc.search_query)
                        })
                        .size(typography::SECONDARY)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_family),
                    );
                project_cards.push(Box::new(empty_card));
            } else {
                for (rel_idx, target) in filtered
                    .iter()
                    .skip(scroll_offset)
                    .take(VISIBLE_PROJECT_ROWS)
                    .enumerate()
                {
                    let abs_idx = scroll_offset + rel_idx;
                    let is_selected = abs_idx == selected_idx;
                    let is_hovered = svc.hover_index == Some(abs_idx);

                    let glyph = if target.is_worktree {
                        "󰘬"
                    } else if target.is_tmux_active {
                        "󰓩"
                    } else {
                        "󰲋"
                    };

                    let icon_box = Box::new(
                        Rectangle::new()
                            .width(28.0)
                            .height(28.0)
                            .radius(radius::SM)
                            .fill(tokens.surface)
                            .border(1.0, tokens.border_subtle)
                            .align_child(Align::Center, Align::Center)
                            .child(
                                Text::new(glyph)
                                    .size(typography::TITLE_SM)
                                    .color(if is_selected {
                                        tokens.accent
                                    } else {
                                        tokens.text_secondary
                                    })
                                    .font(&tokens.font_family),
                            ),
                    );

                    let badge_text = if target.is_worktree {
                        target.branch.as_deref().unwrap_or("worktree")
                    } else if target.is_tmux_active {
                        "tmux"
                    } else {
                        "dev"
                    };

                    let path_str = target.path.to_string_lossy().to_string();
                    let target_clone = target.clone();

                    let card = action_card_interactive(
                        548.0,
                        46.0,
                        icon_box,
                        &target.name,
                        Some(&path_str),
                        Some(badge_text),
                        is_selected,
                        is_hovered,
                        move |is_h| {
                            let mut svc = TerminalManagerService::write();
                            if is_h {
                                svc.hover_index = Some(abs_idx);
                            } else if svc.hover_index == Some(abs_idx) {
                                svc.hover_index = None;
                            }
                        },
                        move || {
                            let _ = NiriService::jump_or_spawn_project(&target_clone);
                            close();
                        },
                        &tokens,
                    );
                    project_cards.push(Box::new(card));
                }
            }

            let list_col = Column::new(project_cards)
                .width(Parent)
                .gap(spacing::GAP_SM);
            let scroll_container = Rectangle::new()
                .width(Parent)
                .height(260.0)
                .cursor(Pointer)
                .on_scroll(move |scroll: Scroll| {
                    let mut svc = TerminalManagerService::write();
                    if scroll.y < 0.0 {
                        if svc.scroll_offset + VISIBLE_PROJECT_ROWS < count {
                            svc.scroll_offset += 1;
                            if svc.selected < svc.scroll_offset {
                                svc.selected = svc.scroll_offset;
                            }
                        }
                    } else if svc.scroll_offset > 0 {
                        svc.scroll_offset = svc.scroll_offset.saturating_sub(1);
                        if svc.selected >= svc.scroll_offset + VISIBLE_PROJECT_ROWS {
                            svc.selected = svc.scroll_offset + VISIBLE_PROJECT_ROWS - 1;
                        }
                    }
                })
                .child(list_col);

            body_rows.push(Box::new(scroll_container));
            count
        }
        ManagerTab::Terminals => {
            let total_presets = TERMINAL_PRESETS.len();
            let mut cards: Vec<Box<dyn Widget>> = Vec::new();
            for (idx, preset) in TERMINAL_PRESETS.iter().enumerate() {
                let is_selected = idx == selected_idx;
                let is_hovered = svc.hover_index == Some(idx);
                let is_default = preset.id == default_id;
                let installed = is_installed(preset.bin);

                let icon_box = Box::new(
                    Rectangle::new()
                        .width(28.0)
                        .height(28.0)
                        .radius(radius::SM)
                        .fill(tokens.surface)
                        .border(1.0, tokens.border_subtle)
                        .align_child(Align::Center, Align::Center)
                        .child(
                            Text::new(preset.glyph)
                                .size(typography::TITLE_SM)
                                .color(if is_selected {
                                    tokens.accent
                                } else if installed {
                                    tokens.text_secondary
                                } else {
                                    tokens.text_muted
                                })
                                .font(&tokens.font_family),
                        ),
                );

                let badge_text = if is_default {
                    "Default"
                } else if !installed {
                    "Missing"
                } else {
                    preset.tag
                };

                let card = action_card_interactive(
                    176.0,
                    66.0,
                    icon_box,
                    preset.name,
                    Some(preset.description),
                    Some(badge_text),
                    is_selected,
                    is_hovered,
                    move |is_h| {
                        let mut svc = TerminalManagerService::write();
                        if is_h {
                            svc.hover_index = Some(idx);
                        } else if svc.hover_index == Some(idx) {
                            svc.hover_index = None;
                        }
                    },
                    move || {
                        launch_preset(idx, LaunchMode::Standard);
                    },
                    &tokens,
                );
                cards.push(Box::new(card));
            }

            let cards_row = Row::new(cards)
                .width(Parent)
                .justify(Justify::SpaceBetween)
                .align(Align::Center);

            body_rows.push(Box::new(cards_row));

            let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
            let dotfield_dir = format!("{}/dev/projects/dotfield", home);
            let home_dir = home.clone();

            let bookmark_home = Rectangle::new()
                .width(110.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Align::Center, Align::Center)
                .on_click(move |_| {
                    launch_in_dir(selected_idx, &home_dir);
                })
                .child(
                    Text::new("󰋜 Home (~)")
                        .size(typography::BADGE)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_family),
                );

            let bookmark_dotfield = Rectangle::new()
                .width(130.0)
                .height(26.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Align::Center, Align::Center)
                .on_click(move |_| {
                    launch_in_dir(selected_idx, &dotfield_dir);
                })
                .child(
                    Text::new("󰲋 Dotfield")
                        .size(typography::BADGE)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_family),
                );

            let bookmarks_row = Row::new(vec![
                Box::new(
                    Text::new("Open in:")
                        .size(typography::BADGE)
                        .color(tokens.text_muted)
                        .font(&tokens.font_family),
                ),
                Box::new(bookmark_home),
                Box::new(bookmark_dotfield),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center);

            body_rows.push(Box::new(bookmarks_row));

            let sel_preset = TERMINAL_PRESETS.get(selected_idx);
            let sel_name = sel_preset.map(|p| p.name).unwrap_or("Terminal");

            let mux_btn = Rectangle::new()
                .width(176.0)
                .height(28.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Align::Center, Align::Center)
                .on_click(move |_| {
                    launch_preset(selected_idx, LaunchMode::Multiplexer);
                })
                .child(
                    Text::new(format!("󰒋 {} + Tmux", sel_name))
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(&tokens.font_family),
                );

            let scratch_btn = Rectangle::new()
                .width(176.0)
                .height(28.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Align::Center, Align::Center)
                .on_click(move |_| {
                    launch_preset(selected_idx, LaunchMode::Scratchpad);
                })
                .child(
                    Text::new(format!("󰖲 {} Float", sel_name))
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(&tokens.font_family),
                );

            let default_btn = Rectangle::new()
                .width(176.0)
                .height(28.0)
                .radius(radius::SM)
                .fill(tokens.surface_elevated)
                .border(1.0, tokens.border_subtle)
                .cursor(Pointer)
                .align_child(Align::Center, Align::Center)
                .on_click(move |_| {
                    set_default_terminal(selected_idx);
                })
                .child(
                    Text::new(format!("★ Make {} Default", sel_name))
                        .size(typography::BADGE)
                        .color(tokens.accent)
                        .font(&tokens.font_family),
                );

            let actions_row = Row::new(vec![
                Box::new(mux_btn),
                Box::new(scratch_btn),
                Box::new(default_btn),
            ])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

            body_rows.push(Box::new(actions_row));
            total_presets
        }
    };

    if let Some(ref msg) = svc.status_message {
        let status_box = Rectangle::new()
            .width(Parent)
            .height(24.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_focus)
            .align_child(Align::Center, Align::Center)
            .child(
                Text::new(msg)
                    .size(typography::CAPTION)
                    .color(tokens.accent)
                    .font(&tokens.font_family),
            );
        body_rows.push(Box::new(status_box));
    }

    let body_column = Column::new(body_rows).width(Parent).gap(spacing::GAP_MD);

    let header_tag = match active_tab {
        ManagerTab::Projects => "01 DEV Ribbon",
        ManagerTab::Terminals => &format!("Default: {}", default_id),
    };

    let footer_hints = match active_tab {
        ManagerTab::Projects => vec![
            ("TAB", "SWITCH VIEW"),
            ("↑↓", "NAVIGATE"),
            ("ENTER", "JUMP / OPEN"),
            ("ESC", "DISMISS"),
        ],
        ManagerTab::Terminals => vec![
            ("TAB", "VIEW"),
            ("1-3", "SELECT"),
            ("ENTER", "LAUNCH"),
            ("M", "TMUX"),
            ("S", "FLOAT"),
            ("D", "DEFAULT"),
            ("ESC", "CLOSE"),
        ],
    };

    let dialog = modal_shell(
        580.0,
        450.0,
        "󰞷",
        "[ DEV // TOPOLOGY ROUTER ]",
        Some(header_tag),
        Box::new(body_column),
        footer_hints,
        close,
        &tokens,
    );

    let transition = crate::ui::animation::ModalTransition::compute(
        svc.open_time,
        crate::ui::animation::DEFAULT_MODAL_ANIMATION_DURATION,
    );
    let dialog = transition.apply(dialog);

    LayerWindow::new()
        .width(580.0)
        .height(450.0)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .layer(Layer::Overlay)
        .keyboard(Keyboard::OnDemand)
        .on_key(move |key| {
            handle_key(key, total_items);
        })
        .space(Zone::Ignore)
        .visible(is_visible)
        .namespace("dotfield-shell-terminal-manager")
        .child(dialog)
}

fn handle_key(key: Key, count: usize) {
    let active_tab = TerminalManagerService::read().active_tab;

    match key {
        Key::Tab => {
            let mut svc = TerminalManagerService::write();
            svc.active_tab = match svc.active_tab {
                ManagerTab::Projects => ManagerTab::Terminals,
                ManagerTab::Terminals => ManagerTab::Projects,
            };
            svc.selected = 0;
            svc.scroll_offset = 0;
            svc.hover_index = None;
        }
        Key::Escape => {
            close();
        }
        Key::Enter => {
            submit_selection();
        }
        _ => match active_tab {
            ManagerTab::Projects => handle_key_projects(key, count),
            ManagerTab::Terminals => handle_key_terminals(key, count),
        },
    }
}

fn handle_key_projects(key: Key, count: usize) {
    match key {
        Key::Down => {
            let mut svc = TerminalManagerService::write();
            if count > 0 && svc.selected + 1 < count {
                svc.selected += 1;
                if svc.selected >= svc.scroll_offset + VISIBLE_PROJECT_ROWS {
                    svc.scroll_offset = svc.selected + 1 - VISIBLE_PROJECT_ROWS;
                }
            }
        }
        Key::Up => {
            let mut svc = TerminalManagerService::write();
            if svc.selected > 0 {
                svc.selected -= 1;
                if svc.selected < svc.scroll_offset {
                    svc.scroll_offset = svc.selected;
                }
            }
        }
        Key::Home => {
            let mut svc = TerminalManagerService::write();
            svc.selected = 0;
            svc.scroll_offset = 0;
        }
        Key::End => {
            let mut svc = TerminalManagerService::write();
            if count > 0 {
                svc.selected = count - 1;
                if svc.selected >= svc.scroll_offset + VISIBLE_PROJECT_ROWS {
                    svc.scroll_offset = svc.selected + 1 - VISIBLE_PROJECT_ROWS;
                }
            }
        }
        _ => {}
    }
}

fn handle_key_terminals(key: Key, count: usize) {
    match key {
        Key::Right | Key::Down => {
            let mut svc = TerminalManagerService::write();
            if svc.selected + 1 < count {
                svc.selected += 1;
            }
        }
        Key::Left | Key::Up => {
            let mut svc = TerminalManagerService::write();
            if svc.selected > 0 {
                svc.selected -= 1;
            }
        }
        Key::Home => {
            let mut svc = TerminalManagerService::write();
            svc.selected = 0;
        }
        Key::End => {
            let mut svc = TerminalManagerService::write();
            if count > 0 {
                svc.selected = count - 1;
            }
        }
        Key::Character('1') => {
            launch_preset(0, LaunchMode::Standard);
        }
        Key::Character('2') => {
            launch_preset(1, LaunchMode::Standard);
        }
        Key::Character('3') => {
            launch_preset(2, LaunchMode::Standard);
        }
        Key::Character('m') | Key::Character('M') => {
            let svc = TerminalManagerService::read();
            let sel = svc.selected;
            drop(svc);
            launch_preset(sel, LaunchMode::Multiplexer);
        }
        Key::Character('s') | Key::Character('S') => {
            let svc = TerminalManagerService::read();
            let sel = svc.selected;
            drop(svc);
            launch_preset(sel, LaunchMode::Scratchpad);
        }
        Key::Character('d') | Key::Character('D') => {
            let svc = TerminalManagerService::read();
            let sel = svc.selected;
            drop(svc);
            set_default_terminal(sel);
        }
        _ => {}
    }
}

fn submit_selection() {
    let svc = TerminalManagerService::read();
    let tab = svc.active_tab;
    let selected = svc.selected;
    let query = svc.search_query.clone();
    drop(svc);

    match tab {
        ManagerTab::Projects => {
            let filtered = get_filtered_projects(&query);
            if let Some(target) = filtered.get(selected) {
                let _ = NiriService::jump_or_spawn_project(target);
                close();
            }
        }
        ManagerTab::Terminals => {
            launch_preset(selected, LaunchMode::Standard);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_presets_present() {
        assert_eq!(TERMINAL_PRESETS.len(), 3);
        assert_eq!(TERMINAL_PRESETS[0].id, "ghostty");
        assert_eq!(TERMINAL_PRESETS[1].id, "foot");
        assert_eq!(TERMINAL_PRESETS[2].id, "kitty");
    }

    #[test]
    fn test_terminal_service_default() {
        let svc = TerminalManagerService::new();
        assert!(!svc.open);
        assert_eq!(svc.selected, 0);
        assert_eq!(svc.active_tab, ManagerTab::Projects);
        assert!(svc.search_query.is_empty());
        assert!(svc.hover_index.is_none());
    }

    #[test]
    fn test_terminal_toggle_and_close() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        assert!(!TerminalManagerService::read().open);
        assert!(TerminalManagerService::read().open_time.is_none());

        toggle();
        assert!(TerminalManagerService::read().open);
        assert!(TerminalManagerService::read().open_time.is_some());

        close();
        assert!(!TerminalManagerService::read().open);
        assert!(TerminalManagerService::read().open_time.is_none());
    }

    #[test]
    fn test_get_filtered_projects_empty() {
        let projects = get_filtered_projects("nonexistent_random_pattern_xyz_123");
        assert!(projects.is_empty());
    }
}
