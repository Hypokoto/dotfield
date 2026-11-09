use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use amane::{
    Align, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};

use crate::config::{ConfigService, ShellManifest};

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
        ConfigService::write().manifest.terminal.default = term_id.to_string();

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
    let manifest = ConfigService::read().manifest.clone();
    let svc = TerminalManagerService::read();

    let is_visible = svc.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-terminal-manager");
    }
    let selected_idx = svc.selected;
    let default_id = svc.default_terminal.clone();
    let total_presets = TERMINAL_PRESETS.len();

    let tokens = crate::ui::tokens::ThemeTokens::from_manifest(&manifest);

    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // Standardized Modal Header
    let default_tag = format!("Default: {default_id}");
    let header = crate::ui::components::modal_header(
        "󰞷",
        "Terminal Manager",
        Some(&default_tag),
        &tokens,
        close,
    );
    rows.push(Box::new(header));

    // Terminal preset cards in a 2x2 grid
    let mut cards: Vec<Box<dyn Widget>> = Vec::new();

    for (idx, preset) in TERMINAL_PRESETS.iter().enumerate() {
        let is_selected = idx == selected_idx;
        let is_hovered = svc.hover_index == Some(idx);
        let is_default = preset.id == default_id;
        let installed = is_installed(preset.bin);

        let (card_bg, border_color, title_color) = if is_selected {
            (tokens.surface_selected, tokens.border_focus, tokens.accent)
        } else if is_hovered {
            (
                tokens.surface_hover,
                tokens.border_hover,
                tokens.text_primary,
            )
        } else {
            (
                tokens.surface_elevated,
                tokens.border_subtle,
                if installed {
                    tokens.text_primary
                } else {
                    tokens.text_muted
                },
            )
        };

        let icon_color = if is_selected {
            tokens.accent
        } else if is_hovered {
            tokens.text_primary
        } else if installed {
            tokens.text_secondary
        } else {
            tokens.text_muted
        };

        // Top line: Glyph + Title on left, quick number keycap on right
        let title_left = Row::new(vec![
            Box::new(
                Text::new(preset.glyph)
                    .size(crate::ui::tokens::typography::TITLE_SM)
                    .color(icon_color)
                    .font(&tokens.font_family),
            ),
            Box::new(
                Text::new(preset.name)
                    .size(crate::ui::tokens::typography::SECONDARY)
                    .color(title_color)
                    .font(&tokens.font_family),
            ),
        ])
        .gap(crate::ui::tokens::spacing::GAP_SM)
        .align(Align::Center);

        let num_keycap = crate::ui::components::keycap(&format!("{}", idx + 1), &tokens);

        let top_row = Row::new(vec![Box::new(title_left), Box::new(num_keycap)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);

        // Middle line: Description
        let desc_text = Text::new(preset.description)
            .size(crate::ui::tokens::typography::CAPTION)
            .color(if installed {
                tokens.text_secondary
            } else {
                tokens.text_muted
            })
            .font(&tokens.font_family);

        // Bottom line: Badges (Default / Not Found / Tag) on left, Launch keycap on right
        let mut left_badges: Vec<Box<dyn Widget>> = Vec::new();
        if is_default {
            left_badges.push(Box::new(crate::ui::components::status_badge(
                "Default",
                tokens.accent,
                &tokens,
            )));
        } else if !installed {
            left_badges.push(Box::new(crate::ui::components::status_badge(
                "Missing",
                tokens.danger,
                &tokens,
            )));
        } else {
            left_badges.push(Box::new(crate::ui::components::status_badge(
                preset.tag,
                tokens.text_muted,
                &tokens,
            )));
        }

        let mut right_badges: Vec<Box<dyn Widget>> = Vec::new();
        if is_selected {
            right_badges.push(Box::new(crate::ui::components::keycap("↵ Launch", &tokens)));
        }

        let bottom_row = Row::new(vec![
            Box::new(
                Row::new(left_badges)
                    .gap(crate::ui::tokens::spacing::GAP_XS)
                    .align(Align::Center),
            ),
            Box::new(
                Row::new(right_badges)
                    .gap(crate::ui::tokens::spacing::GAP_XS)
                    .align(Align::Center),
            ),
        ])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

        let indicator_pill =
            Rectangle::new()
                .width(3.0)
                .height(48.0)
                .radius(1.5)
                .fill(if is_selected {
                    tokens.accent
                } else if is_hovered {
                    tokens.border_hover
                } else {
                    Color::TRANSPARENT
                });

        let card_body = Column::new(vec![
            Box::new(top_row),
            Box::new(desc_text),
            Box::new(bottom_row),
        ])
        .gap(crate::ui::tokens::spacing::GAP_XS)
        .width(Parent)
        .align(Align::Start);

        let card_content = Row::new(vec![Box::new(indicator_pill), Box::new(card_body)])
            .width(Parent)
            .gap(crate::ui::tokens::spacing::GAP_SM)
            .align(Align::Center);

        let card = Rectangle::new()
            .width(240.0)
            .height(78.0)
            .radius(crate::ui::tokens::radius::MD)
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
            .gap(crate::ui::tokens::spacing::GAP_MD);

        rows.push(Box::new(grid_container));
    } else {
        let grid_container = Column::new(cards)
            .width(Parent)
            .gap(crate::ui::tokens::spacing::GAP_SM);
        rows.push(Box::new(grid_container));
    }

    // Status message banner
    if let Some(ref msg) = svc.status_message {
        let status_box = Rectangle::new()
            .width(Parent)
            .height(22.0)
            .radius(crate::ui::tokens::radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_focus)
            .align_child(Align::Center, Align::Center)
            .child(
                Text::new(msg)
                    .size(crate::ui::tokens::typography::CAPTION)
                    .color(tokens.accent)
                    .font(&tokens.font_family),
            );
        rows.push(Box::new(status_box));
    }

    // Quick actions tray (Multiplexer, Scratchpad, Set Default)
    let sel_preset = TERMINAL_PRESETS.get(selected_idx);
    let sel_name = sel_preset.map(|p| p.name).unwrap_or("Terminal");

    let mux_btn = Rectangle::new()
        .width(156.0)
        .height(28.0)
        .radius(crate::ui::tokens::radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(move |_| {
            launch_preset(selected_idx, LaunchMode::Multiplexer);
        })
        .child(
            Text::new(format!("󰒋 {sel_name} + Tmux"))
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_primary)
                .font(&tokens.font_family),
        );

    let scratch_btn = Rectangle::new()
        .width(156.0)
        .height(28.0)
        .radius(crate::ui::tokens::radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(move |_| {
            launch_preset(selected_idx, LaunchMode::Scratchpad);
        })
        .child(
            Text::new(format!("󰖲 {sel_name} Floating"))
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_primary)
                .font(&tokens.font_family),
        );

    let default_btn = Rectangle::new()
        .width(156.0)
        .height(28.0)
        .radius(crate::ui::tokens::radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Align::Center, Align::Center)
        .on_click(move |_| {
            set_default_terminal(selected_idx);
        })
        .child(
            Text::new(format!("★ Make {sel_name} Default"))
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.accent)
                .font(&tokens.font_family),
        );

    let actions_row = Rectangle::new().width(Parent).height(28.0).child(
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

    // Footer navigation key hints
    let footer_hints = Row::new(vec![
        Box::new(crate::ui::components::keycap("1-4", &tokens)),
        Box::new(
            Text::new("Pick")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new("•")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_muted)
                .font(&tokens.font_family),
        ),
        Box::new(crate::ui::components::keycap("↵", &tokens)),
        Box::new(
            Text::new("Launch")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new("•")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_muted)
                .font(&tokens.font_family),
        ),
        Box::new(crate::ui::components::keycap("m", &tokens)),
        Box::new(
            Text::new("Tmux")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new("•")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_muted)
                .font(&tokens.font_family),
        ),
        Box::new(crate::ui::components::keycap("s", &tokens)),
        Box::new(
            Text::new("Float")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new("•")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_muted)
                .font(&tokens.font_family),
        ),
        Box::new(crate::ui::components::keycap("d", &tokens)),
        Box::new(
            Text::new("Default")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new("•")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_muted)
                .font(&tokens.font_family),
        ),
        Box::new(crate::ui::components::keycap("Esc", &tokens)),
        Box::new(
            Text::new("Dismiss")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
    ])
    .gap(5.0)
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
        .radius(crate::ui::tokens::radius::LG)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
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
        .target_output(OutputTarget::Active)
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
