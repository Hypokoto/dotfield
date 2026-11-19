use std::process::Command;
use std::time::Duration;

use amane::{
    Align, Center, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};

use crate::config::ConfigService;

#[derive(Debug, Clone)]
pub struct PowerAction {
    pub id: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub op_code: &'static str,
    pub key_hint: &'static str,
    pub command: &'static str,
    pub is_critical: bool,
}

pub static ACTIONS: &[PowerAction] = &[
    PowerAction {
        id: "lock",
        icon: "󰌾",
        title: "LOCK",
        op_code: "SYS:01",
        key_hint: "L",
        command: "command -v gtklock >/dev/null 2>&1 && gtklock || swaylock || true",
        is_critical: false,
    },
    PowerAction {
        id: "suspend",
        icon: "󰒲",
        title: "SUSPEND",
        op_code: "SYS:02",
        key_hint: "S",
        command: "systemctl suspend",
        is_critical: false,
    },
    PowerAction {
        id: "reboot",
        icon: "󰑐",
        title: "REBOOT",
        op_code: "SYS:03",
        key_hint: "R",
        command: "systemctl reboot",
        is_critical: true,
    },
    PowerAction {
        id: "poweroff",
        icon: "󰐥",
        title: "HALT",
        op_code: "SYS:04",
        key_hint: "P",
        command: "systemctl poweroff",
        is_critical: true,
    },
    PowerAction {
        id: "exit",
        icon: "󰈆",
        title: "EXIT",
        op_code: "SYS:05",
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
    let is_open = {
        let mut menu = PowermenuService::write();
        menu.open = !menu.open;
        menu.selected = 0;
        menu.open
    };
    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::Powermenu,
        );
    }
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
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-powermenu");
    }

    let tokens = crate::ui::tokens::ThemeTokens::from_manifest(&manifest);

    // Header with title, system info, and close button
    let user_name = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let host_name = std::fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "linux".to_string())
        .trim()
        .to_string();

    let header_card = crate::ui::components::modal_header(
        "󰐥",
        "BLACKSITE // SESSION & POWER",
        Some(&format!("{user_name}@{host_name}")),
        &tokens,
        close,
    );

    // Action buttons: 5 cards in a horizontal row
    let mut action_cards: Vec<Box<dyn Widget>> = Vec::new();

    for (idx, action) in ACTIONS.iter().enumerate() {
        let is_selected = idx == menu.selected;

        let (card_fill, card_border, icon_color, title_color, op_color) = if is_selected {
            if action.is_critical {
                (
                    tokens.danger_subtle,
                    tokens.border_danger,
                    tokens.danger,
                    tokens.text_primary,
                    tokens.danger,
                )
            } else {
                (
                    tokens.surface_selected,
                    tokens.border_focus,
                    tokens.accent,
                    tokens.text_primary,
                    tokens.accent,
                )
            }
        } else if action.is_critical {
            (
                tokens.surface_elevated,
                tokens.border_subtle,
                tokens.danger,
                tokens.text_secondary,
                tokens.danger,
            )
        } else {
            (
                tokens.surface_elevated,
                tokens.border_subtle,
                tokens.text_secondary,
                tokens.text_secondary,
                tokens.text_muted,
            )
        };

        let key_chip = crate::ui::components::keycap(action.key_hint, &tokens);

        let card = Rectangle::new()
            .width(102.0)
            .height(116.0)
            .radius(crate::ui::tokens::radius::SM)
            .fill(card_fill)
            .border(if is_selected { 1.5 } else { 1.0 }, card_border)
            .cursor(Pointer)
            .padding(Padding {
                top: 8.0,
                bottom: 8.0,
                left: 6.0,
                right: 6.0,
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
                        Text::new(format!("[{}]", action.op_code))
                            .size(crate::ui::tokens::typography::CAPTION)
                            .color(op_color)
                            .font(&tokens.font_family),
                    ),
                    Box::new(
                        Text::new(action.icon)
                            .size(26.0)
                            .color(icon_color)
                            .font(&tokens.font_family),
                    ),
                    Box::new(
                        Text::new(action.title)
                            .size(crate::ui::tokens::typography::SECONDARY)
                            .color(title_color)
                            .font(&tokens.font_family),
                    ),
                    Box::new(key_chip),
                ])
                .gap(3.0)
                .align(Align::Center),
            );

        action_cards.push(Box::new(card));
    }

    let actions_row = Row::new(action_cards)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    // Footer with keycap hints
    let footer_row = Row::new(vec![
        Box::new(crate::ui::components::keycap("←/→", &tokens)),
        Box::new(
            Text::new("NAVIGATE")
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
        Box::new(crate::ui::components::keycap("ENTER", &tokens)),
        Box::new(
            Text::new("EXECUTE")
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
        Box::new(crate::ui::components::keycap("ESC", &tokens)),
        Box::new(
            Text::new("CANCEL")
                .size(crate::ui::tokens::typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
    ])
    .gap(6.0)
    .align(Align::Center);

    // Modal Dialog Box
    let dialog = Rectangle::new()
        .width(580.0)
        .height(218.0)
        .radius(crate::ui::tokens::radius::LG)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
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
                Box::new(footer_row),
            ])
            .width(Parent)
            .gap(12.0),
        );

    LayerWindow::new()
        .width(580.0)
        .height(218.0)
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
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
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
