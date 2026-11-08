use std::process::Command;
use std::time::Duration;

use amane::{
    Align, Center, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};

use crate::config::ConfigService;

#[derive(Debug, Clone)]
pub struct PowerAction {
    pub id: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub key_hint: &'static str,
    pub command: &'static str,
    pub is_critical: bool,
}

pub static ACTIONS: &[PowerAction] = &[
    PowerAction {
        id: "lock",
        icon: "󰌾",
        title: "Lock",
        key_hint: "L",
        command: "command -v gtklock >/dev/null 2>&1 && gtklock || swaylock || true",
        is_critical: false,
    },
    PowerAction {
        id: "suspend",
        icon: "󰒲",
        title: "Sleep",
        key_hint: "S",
        command: "systemctl suspend",
        is_critical: false,
    },
    PowerAction {
        id: "reboot",
        icon: "󰑐",
        title: "Reboot",
        key_hint: "R",
        command: "systemctl reboot",
        is_critical: true,
    },
    PowerAction {
        id: "poweroff",
        icon: "󰐥",
        title: "Shutdown",
        key_hint: "P",
        command: "systemctl poweroff",
        is_critical: true,
    },
    PowerAction {
        id: "exit",
        icon: "󰈆",
        title: "Exit Niri",
        key_hint: "E",
        command: "niri msg action quit",
        is_critical: false,
    },
];

pub struct PowermenuService {
    pub open: bool,
    pub selected: usize,
}

impl Service for PowermenuService {
    fn new() -> Self {
        Self {
            open: false,
            selected: 0,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let mut menu = PowermenuService::write();
    menu.open = !menu.open;
    menu.selected = 0;
}

pub fn close() {
    let mut menu = PowermenuService::write();
    menu.open = false;
    menu.selected = 0;
}

pub fn select_prev() {
    let mut menu = PowermenuService::write();
    if menu.selected == 0 {
        menu.selected = ACTIONS.len().saturating_sub(1);
    } else {
        menu.selected -= 1;
    }
}

pub fn select_next() {
    let mut menu = PowermenuService::write();
    menu.selected = (menu.selected + 1) % ACTIONS.len();
}

fn spawn_power_action(id: &str) {
    match id {
        "lock" => {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let script = format!("{home}/.config/dotfield/scripts/lock.sh");
            let fallback = "scripts/lock.sh";
            let cmd = if std::path::Path::new(&script).exists() {
                &script
            } else {
                fallback
            };
            let _ = Command::new("bash").arg(cmd).spawn();
        }
        "suspend" => {
            let _ = Command::new("systemctl").arg("suspend").spawn();
        }
        "reboot" => {
            let _ = Command::new("systemctl").arg("reboot").spawn();
        }
        "poweroff" => {
            let _ = Command::new("systemctl").arg("poweroff").spawn();
        }
        "exit" => {
            let _ = Command::new("niri")
                .args(["msg", "action", "quit", "--skip-confirmation"])
                .spawn();
        }
        _ => {}
    }
}

pub fn execute_selected() {
    let selected_idx = PowermenuService::read().selected;
    close();
    if let Some(action) = ACTIONS.get(selected_idx) {
        spawn_power_action(action.id);
    }
}

pub fn execute_action(idx: usize) {
    close();
    if let Some(action) = ACTIONS.get(idx) {
        spawn_power_action(action.id);
    }
}

/// Renders the fullscreen centered Powermenu overlay modal.
pub fn powermenu_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let menu = PowermenuService::read();

    let is_visible = menu.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-powermenu");
    }

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let hover_color = Color::from(manifest.palette.hover.as_str()); // #FFD166
    let crimson_color = Color::from(manifest.palette.accent.as_str()); // Crimson #E63946
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let steel_color = Color::from(manifest.palette.steel.as_str());
    let font = manifest.theme.font_family.clone();

    // Header with title, system info, and close button
    let user_name = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let host_name = std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "linux".to_string())
        .trim()
        .to_string();

    let header_card = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(Text::new("󰐥").size(14.0).color(crimson_color).font(&font)),
                Box::new(
                    Text::new("SESSION & POWER CONTROL")
                        .size(11.0)
                        .color(fg_color)
                        .font(&font),
                ),
                Box::new(
                    Text::new(format!("• {user_name}@{host_name}"))
                        .size(10.0)
                        .color(steel_color)
                        .font(&font),
                ),
            ])
            .gap(6.0)
            .align(Align::Center),
        ),
        Box::new(
            Rectangle::new()
                .width(26.0)
                .height(26.0)
                .radius(4.0)
                .fill(overlay_color)
                .border(1.0, border_color)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| {
                    close();
                })
                .child(Text::new("󰅖").size(12.0).color(steel_color).font(&font)),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    // Action buttons: 5 cards in a horizontal row
    let mut action_cards: Vec<Box<dyn Widget>> = Vec::new();

    for (idx, action) in ACTIONS.iter().enumerate() {
        let is_selected = idx == menu.selected;

        let card_fill = if is_selected { overlay_color } else { bg_color };

        let card_border = if is_selected {
            if action.is_critical {
                crimson_color
            } else {
                amber_color
            }
        } else {
            border_color
        };

        let icon_color = if is_selected {
            if action.is_critical {
                crimson_color
            } else {
                hover_color
            }
        } else if action.is_critical {
            Color::from("#FF6B76")
        } else {
            amber_color
        };

        let key_chip = Rectangle::new()
            .width(20.0)
            .height(16.0)
            .radius(3.0)
            .fill(if is_selected {
                if action.is_critical {
                    Color::from("#3D2024")
                } else {
                    Color::from("#382E15")
                }
            } else {
                overlay_color
            })
            .border(
                1.0,
                if is_selected {
                    card_border
                } else {
                    border_color
                },
            )
            .align_child(Center, Center)
            .child(
                Text::new(action.key_hint)
                    .size(9.0)
                    .color(if is_selected { fg_color } else { steel_color })
                    .font(&font),
            );

        let card = Rectangle::new()
            .width(104.0)
            .height(116.0)
            .radius(8.0)
            .fill(card_fill)
            .border(if is_selected { 1.5 } else { 1.0 }, card_border)
            .cursor(Pointer)
            .padding(Padding {
                top: 14.0,
                bottom: 10.0,
                left: 8.0,
                right: 8.0,
            })
            .align_child(Center, Center)
            .on_hover(move |hovering| {
                if hovering {
                    let mut menu = PowermenuService::write();
                    menu.selected = idx;
                }
            })
            .on_click(move |_| {
                execute_action(idx);
            })
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new(action.icon)
                            .size(30.0)
                            .color(icon_color)
                            .font(&font),
                    ),
                    Box::new(
                        Text::new(action.title)
                            .size(11.0)
                            .color(if is_selected { fg_color } else { steel_color })
                            .font(&font),
                    ),
                    Box::new(key_chip),
                ])
                .gap(8.0)
                .align(Align::Center),
            );

        action_cards.push(Box::new(card));
    }

    let actions_row = Row::new(action_cards)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    // Footer with hotkey hints
    let footer_text = Row::new(vec![Box::new(
        Text::new("Use [←/→] or [h/l] to navigate • [Enter] to execute • [Esc] to cancel")
            .size(9.5)
            .color(steel_color)
            .font(&font),
    )])
    .width(Parent)
    .align(Align::Center);

    // Modal Dialog Box
    let dialog = Rectangle::new()
        .width(580.0)
        .height(210.0)
        .radius(12.0)
        .fill(bg_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 14.0,
            bottom: 12.0,
            left: 16.0,
            right: 16.0,
        })
        .child(
            Column::new(vec![
                Box::new(header_card),
                Box::new(actions_row),
                Box::new(footer_text),
            ])
            .width(Parent)
            .gap(12.0),
        );

    LayerWindow::new()
        .width(580.0)
        .height(210.0)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Middle)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(if is_visible {
            Keyboard::Exclusive
        } else {
            Keyboard::None
        })
        .visible(is_visible)
        .namespace("dotfield-shell-powermenu")
        .on_key(|key| match key {
            Key::Escape | Key::Character('q') | Key::Character('Q') => {
                close();
            }
            Key::Left | Key::Up => {
                select_prev();
            }
            Key::Right | Key::Down | Key::Tab => {
                select_next();
            }
            Key::Enter | Key::Space => {
                execute_selected();
            }
            Key::Character('l') | Key::Character('L') => {
                execute_action(0);
            }
            Key::Character('s') | Key::Character('S') => {
                execute_action(1);
            }
            Key::Character('r') | Key::Character('R') => {
                execute_action(2);
            }
            Key::Character('p') | Key::Character('P') => {
                execute_action(3);
            }
            Key::Character('e') | Key::Character('E') => {
                execute_action(4);
            }
            _ => {}
        })
        .child(dialog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powermenu_actions_present() {
        assert_eq!(ACTIONS.len(), 5);
        assert_eq!(ACTIONS[0].id, "lock");
        assert_eq!(ACTIONS[1].id, "suspend");
        assert_eq!(ACTIONS[2].id, "reboot");
        assert_eq!(ACTIONS[3].id, "poweroff");
        assert_eq!(ACTIONS[4].id, "exit");
    }

    #[test]
    fn test_powermenu_toggle_and_navigation() {
        PowermenuService::write().open = false;
        toggle();
        assert!(PowermenuService::read().open);
        assert_eq!(PowermenuService::read().selected, 0);

        select_next();
        assert_eq!(PowermenuService::read().selected, 1);

        select_prev();
        assert_eq!(PowermenuService::read().selected, 0);

        select_prev();
        assert_eq!(PowermenuService::read().selected, 4);

        close();
        assert!(!PowermenuService::read().open);
    }
}
