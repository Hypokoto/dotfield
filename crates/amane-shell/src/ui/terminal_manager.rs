use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;
use std::time::Duration;

use amane::{
    Align, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin, Padding,
    Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};
use parking_lot::RwLock;

use crate::config::ShellManifest;

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
        id: "foot",
        name: "Foot",
        glyph: "󰞷",
        description: "Ultra-fast Wayland client with minimal latency & memory footprint",
        tag: "Wayland Native",
        bin: "foot",
        standard_cmd: "foot",
        mux_cmd: "foot -e tmux",
        scratchpad_cmd: "foot --app-id=floating-term",
    },
    TerminalPreset {
        id: "kitty",
        name: "Kitty",
        glyph: "󰄛",
        description: "GPU-accelerated terminal with kitty graphics protocol & ligatures",
        tag: "OpenGL GPU",
        bin: "kitty",
        standard_cmd: "kitty",
        mux_cmd: "kitty -e tmux",
        scratchpad_cmd: "kitty --class=floating-term",
    },
    TerminalPreset {
        id: "alacritty",
        name: "Alacritty",
        glyph: "󰆍",
        description: "Hardware-accelerated terminal focused on raw rendering throughput",
        tag: "Cross-Platform GPU",
        bin: "alacritty",
        standard_cmd: "alacritty",
        mux_cmd: "alacritty -e tmux",
        scratchpad_cmd: "alacritty --class floating-term",
    },
    TerminalPreset {
        id: "ghostty",
        name: "Ghostty",
        glyph: "󰊠",
        description: "Next-gen GPU terminal with modern Wayland tabs & shader support",
        tag: "Modern GPU",
        bin: "ghostty",
        standard_cmd: "ghostty",
        mux_cmd: "ghostty -e tmux",
        scratchpad_cmd: "ghostty --class=floating-term",
    },
];

pub struct TerminalManagerService {
    pub open: bool,
    pub selected: usize,
    pub hover_index: Option<usize>,
    pub default_terminal: String,
    pub status_message: Option<String>,
}

impl Service for TerminalManagerService {
    fn new() -> Self {
        let default_term = detect_default_terminal();
        Self {
            open: false,
            selected: 0,
            hover_index: None,
            default_terminal: default_term,
            status_message: None,
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
    let mut svc = TerminalManagerService::write();
    svc.open = !svc.open;
    svc.selected = 0;
    svc.hover_index = None;
    svc.status_message = None;
    svc.default_terminal = detect_default_terminal();
}

pub fn open() {
    let mut svc = TerminalManagerService::write();
    svc.open = true;
    svc.selected = 0;
    svc.hover_index = None;
    svc.status_message = None;
    svc.default_terminal = detect_default_terminal();
}

pub fn close() {
    let mut svc = TerminalManagerService::write();
    svc.open = false;
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

fn detect_default_terminal() -> String {
    let conf = CONFIG.read();
    if !conf.terminal.default.is_empty() {
        return conf.terminal.default.clone();
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

    "foot".to_string()
}

pub fn set_default_terminal(index: usize) {
    if let Some(preset) = TERMINAL_PRESETS.get(index) {
        let term_id = preset.id;
        let mut svc = TerminalManagerService::write();
        svc.default_terminal = term_id.to_string();
        svc.status_message = Some(format!("Default set to {}", preset.name));
        drop(svc);

        // Update active in-memory config
        CONFIG.write().terminal.default = term_id.to_string();

        // Update ~/.config/dotfield/shell.toml
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

        // Detached spawn using discrete binary and arguments
        let mut parts = cmd.split_whitespace();
        if let Some(bin) = parts.next() {
            use std::os::unix::process::CommandExt;
            let args: Vec<&str> = parts.collect();
            let _ = Command::new(bin)
                .args(&args)
                .process_group(0)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }

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

        close();
    }
}

pub fn terminal_manager_overlay() -> LayerWindow {
    let manifest = CONFIG.read().clone();
    let svc = TerminalManagerService::read();

    let is_visible = svc.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .visible(false)
            .namespace("dotfield-shell-terminal-manager");
    }
    let selected_idx = svc.selected;
    let default_id = svc.default_terminal.clone();

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let fg_color = Color::from(manifest.palette.foreground.as_str()); // Bone #E8E6E3
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let muted_color = Color::from(manifest.palette.muted.as_str()); // Ash #A1A1AA
    let font = manifest.theme.font_family.clone();

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Header Box
    let header_icon = Text::new("󰞷").size(16.0).color(amber_color).font(&font);

    let header_title = Text::new("Terminal Manager")
        .size(13.0)
        .color(fg_color)
        .font(&font);

    let default_badge = Rectangle::new()
        .width(130.0)
        .height(18.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, Color::from(manifest.palette.border.as_str()))
        .padding(Padding {
            top: 1.0,
            bottom: 1.0,
            left: 6.0,
            right: 6.0,
        })
        .align_child(Align::Center, Align::Center)
        .child(
            Text::new(format!("Default: {default_id}"))
                .size(9.0)
                .color(amber_color)
                .font(&font),
        );

    let header_left = Row::new(vec![Box::new(header_icon), Box::new(header_title)])
        .gap(8.0)
        .align(Align::Center);

    let close_btn = Rectangle::new()
        .width(22.0)
        .height(22.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, Color::from(manifest.palette.border.as_str()))
        .align_child(Align::Center, Align::Center)
        .cursor(Pointer)
        .on_click(|_| {
            close();
        })
        .child(Text::new("✕").size(10.0).color(muted_color).font(&font));

    let header_row = Rectangle::new()
        .width(Parent)
        .height(28.0)
        .align_child(Align::Start, Align::Center)
        .child(
            Row::new(vec![
                Box::new(header_left),
                Box::new(
                    Row::new(vec![Box::new(default_badge), Box::new(close_btn)])
                        .gap(6.0)
                        .align(Align::Center),
                ),
            ])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
        );

    rows.push(Box::new(header_row));

    // 2. Terminal Preset Cards in a 2x2 Grid Box
    let total_presets = TERMINAL_PRESETS.len();
    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    let hover_color = Color::from(manifest.palette.hover.as_str()); // Pale Amber #FFD166
    let steel_color = Color::from(manifest.palette.steel.as_str()); // Steel #8B949E
    let border_color_token = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let green_color = Color::from("#7BC47F"); // Success Green

    for (idx, preset) in TERMINAL_PRESETS.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let is_hovered = svc.hover_index == Some(idx);
        let is_default = preset.id == default_id;
        let installed = is_installed(preset.bin);

        let card_bg = if is_selected {
            Color::from("#262632") // Elevated Graphite
        } else if is_hovered {
            Color::from("#202028") // Subtle hover surface
        } else {
            Color::from("#17171E") // Structured Obsidian card surface
        };

        let border_color = if is_selected {
            amber_color // Industrial Amber #FFB000
        } else if is_hovered {
            Color::from("#4E4E5C") // Focused steel-iron border
        } else {
            border_color_token // Iron #34343D
        };

        let icon_color = if is_selected {
            amber_color
        } else if is_hovered {
            hover_color
        } else if installed {
            steel_color
        } else {
            Color::from("#52525B")
        };

        let title_color = if is_selected {
            amber_color
        } else if is_hovered {
            hover_color
        } else if installed {
            fg_color
        } else {
            Color::from("#71717A")
        };

        // Top line: Glyph + Title on left, quick number badge on right
        let title_left = Row::new(vec![
            Box::new(
                Text::new(preset.glyph)
                    .size(16.0)
                    .color(icon_color)
                    .font(&font),
            ),
            Box::new(
                Text::new(preset.name)
                    .size(12.5)
                    .color(title_color)
                    .font(&font),
            ),
        ])
        .gap(6.0)
        .align(Align::Center);

        let num_badge = Rectangle::new()
            .width(20.0)
            .height(16.0)
            .radius(3.0)
            .fill(overlay_color)
            .border(
                1.0,
                if is_selected {
                    amber_color
                } else {
                    border_color_token
                },
            )
            .align_child(Align::Center, Align::Center)
            .child(
                Text::new(format!("[{}]", idx + 1))
                    .size(8.5)
                    .color(if is_selected {
                        amber_color
                    } else {
                        steel_color
                    })
                    .font(&font),
            );

        let top_row = Row::new(vec![Box::new(title_left), Box::new(num_badge)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        // Middle line: Description
        let desc_text = Text::new(preset.description)
            .size(8.5)
            .color(if installed {
                muted_color
            } else {
                Color::from("#52525B")
            })
            .font(&font);

        // Bottom line: Badges (Default / Not Found / Tag) on left, Launch on right
        let mut left_badges: Vec<Box<dyn Widget>> = Vec::new();
        if is_default {
            left_badges.push(Box::new(
                Rectangle::new()
                    .width(58.0)
                    .height(16.0)
                    .radius(3.0)
                    .fill(Color::from("#152818"))
                    .border(1.0, green_color)
                    .align_child(Align::Center, Align::Center)
                    .child(
                        Text::new("★ Default")
                            .size(8.5)
                            .color(green_color)
                            .font(&font),
                    ),
            ));
        } else if !installed {
            left_badges.push(Box::new(
                Rectangle::new()
                    .width(62.0)
                    .height(16.0)
                    .radius(3.0)
                    .fill(overlay_color)
                    .border(1.0, Color::from("#452A2D"))
                    .align_child(Align::Center, Align::Center)
                    .child(
                        Text::new("Not Found")
                            .size(8.5)
                            .color(Color::from("#E63946"))
                            .font(&font),
                    ),
            ));
        } else {
            left_badges.push(Box::new(
                Text::new(preset.tag)
                    .size(8.5)
                    .color(if is_selected {
                        hover_color
                    } else {
                        steel_color
                    })
                    .font(&font),
            ));
        }

        let mut right_badges: Vec<Box<dyn Widget>> = Vec::new();
        if is_selected {
            right_badges.push(Box::new(
                Rectangle::new()
                    .width(52.0)
                    .height(16.0)
                    .radius(3.0)
                    .fill(Color::BLACK)
                    .border(1.0, amber_color)
                    .align_child(Align::Center, Align::Center)
                    .child(
                        Text::new("↵ Launch")
                            .size(8.5)
                            .color(amber_color)
                            .font(&font),
                    ),
            ));
        }

        let bottom_row = Row::new(vec![
            Box::new(Row::new(left_badges).gap(4.0).align(Align::Center)),
            Box::new(Row::new(right_badges).gap(4.0).align(Align::Center)),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let indicator_pill =
            Rectangle::new()
                .width(3.0)
                .height(50.0)
                .radius(1.5)
                .fill(if is_selected {
                    amber_color
                } else if is_hovered {
                    hover_color
                } else {
                    Color::TRANSPARENT
                });

        let card_body = Column::new(vec![
            Box::new(top_row),
            Box::new(desc_text),
            Box::new(bottom_row),
        ])
        .gap(3.0)
        .width(Parent)
        .align(Align::Start);

        let card_content = Row::new(vec![Box::new(indicator_pill), Box::new(card_body)])
            .width(Parent)
            .gap(8.0)
            .align(Align::Center);

        let card = Rectangle::new()
            .width(240.0)
            .height(78.0)
            .radius(8.0)
            .fill(card_bg)
            .border(if is_selected { 1.5 } else { 1.0 }, border_color)
            .padding(Padding {
                top: 7.0,
                bottom: 7.0,
                left: 8.0,
                right: 8.0,
            })
            .cursor(Pointer)
            .on_hover(move |is_h| {
                let mut svc = TerminalManagerService::write();
                if is_h {
                    svc.hover_index = Some(idx);
                } else if svc.hover_index == Some(idx) {
                    svc.hover_index = None;
                }
            })
            .on_click(move |_| {
                launch_preset(idx, LaunchMode::Standard);
            })
            .child(card_content);

        cards.push(Box::new(card));
    }

    // Assemble 2x2 Grid Box
    if cards.len() >= 4 {
        let card3 = cards.pop().unwrap();
        let card2 = cards.pop().unwrap();
        let card1 = cards.pop().unwrap();
        let card0 = cards.pop().unwrap();

        let grid_row_0 = Row::new(vec![card0, card1])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        let grid_row_1 = Row::new(vec![card2, card3])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        let grid_container = Column::new(vec![Box::new(grid_row_0), Box::new(grid_row_1)])
            .width(Parent)
            .gap(10.0);

        rows.push(Box::new(grid_container));
    } else {
        let grid_container = Column::new(cards).width(Parent).gap(8.0);
        rows.push(Box::new(grid_container));
    }

    // 3. Status Message Banner
    if let Some(ref msg) = svc.status_message {
        let status_box = Rectangle::new()
            .width(Parent)
            .height(20.0)
            .radius(4.0)
            .fill(Color::from("#181822"))
            .border(1.0, hover_color)
            .align_child(Align::Center, Align::Center)
            .child(Text::new(msg).size(9.0).color(hover_color).font(&font));
        rows.push(Box::new(status_box));
    }

    // 4. Quick Actions Tray (Multiplexer, Scratchpad, Set Default)
    let sel_preset = TERMINAL_PRESETS.get(selected_idx);
    let sel_name = sel_preset.map(|p| p.name).unwrap_or("Terminal");

    let mux_btn = Rectangle::new()
        .width(156.0)
        .height(26.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color_token)
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(move |_| {
            launch_preset(selected_idx, LaunchMode::Multiplexer);
        })
        .child(
            Text::new(format!("󰒋 {sel_name} + Tmux"))
                .size(9.0)
                .color(fg_color)
                .font(&font),
        );

    let scratch_btn = Rectangle::new()
        .width(156.0)
        .height(26.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color_token)
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(move |_| {
            launch_preset(selected_idx, LaunchMode::Scratchpad);
        })
        .child(
            Text::new(format!("󰖲 {sel_name} Floating"))
                .size(9.0)
                .color(fg_color)
                .font(&font),
        );

    let default_btn = Rectangle::new()
        .width(156.0)
        .height(26.0)
        .radius(4.0)
        .fill(overlay_color)
        .border(1.0, border_color_token)
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(move |_| {
            set_default_terminal(selected_idx);
        })
        .child(
            Text::new(format!("★ Make {sel_name} Default"))
                .size(9.0)
                .color(amber_color)
                .font(&font),
        );

    let actions_row = Rectangle::new().width(Parent).height(26.0).child(
        Row::new(vec![
            Box::new(mux_btn),
            Box::new(scratch_btn),
            Box::new(default_btn),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center),
    );

    rows.push(Box::new(actions_row));

    // 5. Footer Navigation Key Hints
    let footer_hints = Row::new(vec![
        Box::new(
            Text::new("[Arrows]")
                .size(9.0)
                .color(hover_color)
                .font(&font),
        ),
        Box::new(
            Text::new("Navigate")
                .size(9.0)
                .color(steel_color)
                .font(&font),
        ),
        Box::new(Text::new("[1-4]").size(9.0).color(hover_color).font(&font)),
        Box::new(
            Text::new("Quick Pick")
                .size(9.0)
                .color(steel_color)
                .font(&font),
        ),
        Box::new(Text::new("[↵]").size(9.0).color(hover_color).font(&font)),
        Box::new(Text::new("Launch").size(9.0).color(steel_color).font(&font)),
        Box::new(Text::new("[m]").size(9.0).color(hover_color).font(&font)),
        Box::new(Text::new("Tmux").size(9.0).color(steel_color).font(&font)),
        Box::new(Text::new("[s]").size(9.0).color(hover_color).font(&font)),
        Box::new(
            Text::new("Floating")
                .size(9.0)
                .color(steel_color)
                .font(&font),
        ),
        Box::new(Text::new("[d]").size(9.0).color(hover_color).font(&font)),
        Box::new(
            Text::new("Default")
                .size(9.0)
                .color(steel_color)
                .font(&font),
        ),
        Box::new(Text::new("[Esc]").size(9.0).color(hover_color).font(&font)),
        Box::new(
            Text::new("Dismiss")
                .size(9.0)
                .color(steel_color)
                .font(&font),
        ),
    ])
    .gap(4.0)
    .align(Align::Center);

    let footer_row = Rectangle::new()
        .width(Parent)
        .height(18.0)
        .align_child(Align::Center, Align::Center)
        .child(footer_hints);

    rows.push(Box::new(footer_row));

    // Main Modal Dialog Container (520x340)
    let dialog = Rectangle::new()
        .width(520.0)
        .height(340.0)
        .radius(10.0)
        .fill(bg_color)
        .border(1.0, border_color_token)
        .padding(Padding {
            top: 14.0,
            bottom: 12.0,
            left: 14.0,
            right: 14.0,
        })
        .child(Column::new(rows).width(Parent).gap(8.0));

    LayerWindow::new()
        .width(520.0)
        .height(340.0)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        })
        .layer(Layer::Overlay)
        .keyboard(Keyboard::OnDemand)
        .on_key(move |key| {
            handle_key(key, total_presets);
        })
        .space(Zone::Ignore)
        .visible(is_visible)
        .namespace("dotfield-shell-terminal-manager")
        .child(dialog)
}

fn handle_key(key: Key, count: usize) {
    match key {
        Key::Down => {
            let mut svc = TerminalManagerService::write();
            if svc.selected + 2 < count {
                svc.selected += 2;
            }
        }
        Key::Up => {
            let mut svc = TerminalManagerService::write();
            if svc.selected >= 2 {
                svc.selected -= 2;
            }
        }
        Key::Right => {
            let mut svc = TerminalManagerService::write();
            if svc.selected % 2 == 0 && svc.selected + 1 < count {
                svc.selected += 1;
            }
        }
        Key::Left => {
            let mut svc = TerminalManagerService::write();
            if svc.selected % 2 == 1 {
                svc.selected -= 1;
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
        Key::Character('4') => {
            launch_preset(3, LaunchMode::Standard);
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
        Key::Escape => {
            close();
        }
        Key::Enter => {
            let svc = TerminalManagerService::read();
            let sel = svc.selected;
            drop(svc);
            launch_preset(sel, LaunchMode::Standard);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_presets_present() {
        assert_eq!(TERMINAL_PRESETS.len(), 4);
        assert_eq!(TERMINAL_PRESETS[0].id, "foot");
        assert_eq!(TERMINAL_PRESETS[1].id, "kitty");
        assert_eq!(TERMINAL_PRESETS[2].id, "alacritty");
        assert_eq!(TERMINAL_PRESETS[3].id, "ghostty");
    }

    #[test]
    fn test_terminal_service_default() {
        let svc = TerminalManagerService::new();
        assert!(!svc.open);
        assert_eq!(svc.selected, 0);
        assert!(svc.hover_index.is_none());
    }

    #[test]
    fn test_terminal_toggle_and_close() {
        close();
        assert!(!TerminalManagerService::read().open);

        toggle();
        assert!(TerminalManagerService::read().open);

        close();
        assert!(!TerminalManagerService::read().open);
    }
}
