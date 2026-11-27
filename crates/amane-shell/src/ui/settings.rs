//! Settings window for Amane desktop shell.
//! Provides a unified, GPU-accelerated Settings experience with 10 categories,
//! live in-memory updates via `ConfigService::update_and_save()`,
//! and 0 Hz frame parking when closed.

use std::time::Duration;

use amane::{
    Align, Color, Column, Horizontal, Key, Keyboard, Layer, LayerWindow, Margin, OutputTarget,
    Padding, Parent, Pointer, Rectangle, Row, ScrollArea, Service, Text, Vertical, Weight, Widget,
    Zone,
};

use crate::config::ConfigService;
use crate::services::appearance::{AppearanceService, PROFILES};
use crate::services::display::DisplayTopologyService;
use crate::services::night_light::NightLightService;
use crate::services::niri_config::NiriConfigService;
use crate::services::telemetry::TelemetryService;
use crate::ui::animation::panel_motion;
use crate::ui::components;
use crate::ui::modal_manager::{ModalManager, ModalType};
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub const SETTINGS_WIDTH: f32 = 880.0;
pub const SETTINGS_HEIGHT: f32 = 620.0;
pub const SIDEBAR_WIDTH: f32 = 220.0;
pub const DETAIL_WIDTH: f32 = 624.0; // 880 - 220 - padding/borders

#[inline]
fn pad(top: f32, right: f32, bottom: f32, left: f32) -> Padding {
    Padding {
        top,
        right,
        bottom,
        left,
    }
}

#[inline]
fn pad_all(v: f32) -> Padding {
    pad(v, v, v, v)
}

#[inline]
fn pad_sym(h: f32, v: f32) -> Padding {
    pad(v, h, v, h)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsCategory {
    #[default]
    Appearance,
    Display,
    ShellBar,
    Workspaces,
    Animations,
    WindowRules,
    Shortcuts,
    Input,
    AudioBluetooth,
    SystemPower,
}

impl SettingsCategory {
    pub const ALL: &[SettingsCategory] = &[
        SettingsCategory::Appearance,
        SettingsCategory::Display,
        SettingsCategory::ShellBar,
        SettingsCategory::Workspaces,
        SettingsCategory::Animations,
        SettingsCategory::WindowRules,
        SettingsCategory::Shortcuts,
        SettingsCategory::Input,
        SettingsCategory::AudioBluetooth,
        SettingsCategory::SystemPower,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::Appearance => "appearance",
            Self::Display => "display",
            Self::ShellBar => "shell-bar",
            Self::Workspaces => "workspaces",
            Self::Animations => "animations",
            Self::WindowRules => "window-rules",
            Self::Shortcuts => "shortcuts",
            Self::Input => "input",
            Self::AudioBluetooth => "audio-bluetooth",
            Self::SystemPower => "system-power",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Display => "Display & Night Light",
            Self::ShellBar => "Shell & Bar",
            Self::Workspaces => "Workspaces",
            Self::Animations => "Animations",
            Self::WindowRules => "Window Rules",
            Self::Shortcuts => "Keyboard Shortcuts",
            Self::Input => "Input & Peripherals",
            Self::AudioBluetooth => "Audio & Bluetooth",
            Self::SystemPower => "System & Power",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Appearance => "󰔎",
            Self::Display => "󰍹",
            Self::ShellBar => "󰒓",
            Self::Workspaces => "󰨇",
            Self::Animations => "󰑐",
            Self::WindowRules => "󰖲",
            Self::Shortcuts => "󰌌",
            Self::Input => "󰍽",
            Self::AudioBluetooth => "󰕾",
            Self::SystemPower => "󰐥",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            Self::Appearance => "Themes, colors, shell style, and fonts",
            Self::Display => "Monitors, resolution, scaling, and night light",
            Self::ShellBar => "Bar position, height, dock, and timeouts",
            Self::Workspaces => "Columns, workspace labels, and gaps",
            Self::Animations => "Spring curves, speeds, and window shaders",
            Self::WindowRules => "Tiling defaults, floating, and opacity rules",
            Self::Shortcuts => "Niri compositor and desktop hotkeys",
            Self::Input => "Keyboard layouts, touchpad gestures, and mouse",
            Self::AudioBluetooth => "Default sinks, sources, and paired devices",
            Self::SystemPower => "Power profiles, idle locks, and system info",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let clean = s.trim().to_lowercase();
        match clean.as_str() {
            "appearance" | "theme" | "colors" | "style" | "wallpaper" => Some(Self::Appearance),
            "display" | "monitors" | "outputs" | "nightlight" | "night-light" => {
                Some(Self::Display)
            }
            "shell" | "bar" | "shell-bar" | "dock" => Some(Self::ShellBar),
            "workspaces" | "workspace" => Some(Self::Workspaces),
            "animations" | "animation" => Some(Self::Animations),
            "rules" | "window-rules" | "windows" => Some(Self::WindowRules),
            "shortcuts" | "binds" | "keys" | "keybinds" => Some(Self::Shortcuts),
            "input" | "keyboard" | "touchpad" | "mouse" => Some(Self::Input),
            "audio" | "bluetooth" | "sound" => Some(Self::AudioBluetooth),
            "system" | "power" | "info" | "about" => Some(Self::SystemPower),
            _ => None,
        }
    }
}

pub struct SettingsService {
    pub open: bool,
    pub selected: SettingsCategory,
    pub search_query: String,
    pub status_message: Option<(String, bool)>,
}

impl Service for SettingsService {
    fn new() -> Self {
        Self {
            open: false,
            selected: SettingsCategory::Appearance,
            search_query: String::new(),
            status_message: None,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn open() {
    let mut svc = SettingsService::write();
    svc.open = true;
    drop(svc);
    ModalManager::open_exclusive(ModalType::Settings);
}

pub fn open_category(cat: SettingsCategory) {
    let mut svc = SettingsService::write();
    svc.open = true;
    svc.selected = cat;
    drop(svc);
    ModalManager::open_exclusive(ModalType::Settings);
}

pub fn close() {
    let mut svc = SettingsService::write();
    svc.open = false;
    svc.status_message = None;
}

pub fn toggle() {
    let is_open = {
        let mut svc = SettingsService::write();
        svc.open = !svc.open;
        svc.open
    };
    if is_open {
        ModalManager::open_exclusive(ModalType::Settings);
    }
}

/// Renders the centered Layer::Overlay Settings window.
pub fn settings_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let svc = SettingsService::read();

    let is_open = svc.open;
    let motion = panel_motion("settings", is_open);
    if !motion.visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-settings")
            .child(Rectangle::new().width(Parent).height(Parent));
    }

    let tokens = ThemeTokens::from_manifest(&manifest);
    let selected_cat = svc.selected;
    let search = svc.search_query.trim().to_lowercase();

    // 1. Sidebar Navigation
    let mut nav_items: Vec<Box<dyn Widget>> = Vec::new();

    // Search input display badge
    let search_badge = Rectangle::new()
        .width(Parent)
        .height(34.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(pad_sym(10.0, 6.0))
        .child(
            Row::new(vec![
                Box::new(
                    Text::new("󰍉")
                        .size(13.0)
                        .color(tokens.text_muted)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
                Box::new(
                    Text::new(if search.is_empty() {
                        "Filter settings...".to_string()
                    } else {
                        search.clone()
                    })
                    .size(typography::SECONDARY)
                    .color(if search.is_empty() {
                        tokens.text_muted
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
                ),
            ])
            .gap(spacing::GAP_SM)
            .align(Align::Center),
        );
    nav_items.push(Box::new(search_badge));

    // Category pills
    for &cat in SettingsCategory::ALL {
        if !search.is_empty()
            && !cat.name().to_lowercase().contains(&search)
            && !cat.subtitle().to_lowercase().contains(&search)
        {
            continue;
        }

        let is_active = cat == selected_cat;
        let (pill_bg, pill_fg, pill_border) = if is_active {
            (tokens.accent, tokens.text_on_accent, tokens.accent)
        } else {
            (
                Color::TRANSPARENT,
                tokens.text_secondary,
                Color::TRANSPARENT,
            )
        };

        let cat_row = Rectangle::new()
            .width(Parent)
            .height(36.0)
            .radius(radius::SM)
            .fill(pill_bg)
            .border(1.0, pill_border)
            .cursor(Pointer)
            .padding(pad_sym(12.0, 6.0))
            .on_click(move |_| {
                let mut s = SettingsService::write();
                s.selected = cat;
            })
            .child(
                Row::new(vec![
                    Box::new(
                        Text::new(cat.icon())
                            .size(14.0)
                            .color(pill_fg)
                            .font(&tokens.font_mono)
                            .tight(),
                    ),
                    Box::new(
                        Text::new(cat.name())
                            .size(typography::BODY)
                            .weight(if is_active {
                                Weight::SemiBold
                            } else {
                                Weight::Regular
                            })
                            .color(pill_fg)
                            .font(&tokens.font_sans),
                    ),
                ])
                .gap(spacing::GAP_SM)
                .align(Align::Center),
            );

        nav_items.push(Box::new(cat_row));
    }

    let sidebar = Rectangle::new()
        .width(SIDEBAR_WIDTH)
        .height(Parent)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new("󰒓")
                                .size(16.0)
                                .color(tokens.accent)
                                .font(&tokens.font_mono)
                                .tight(),
                        ),
                        Box::new(
                            Text::new("Settings")
                                .size(typography::TITLE_SM)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
                ),
                Box::new(
                    Rectangle::new()
                        .width(Parent)
                        .height(1.0)
                        .fill(tokens.border_subtle),
                ),
                Box::new(Column::new(nav_items).gap(6.0).width(Parent)),
            ])
            .gap(12.0)
            .width(Parent),
        );

    // 2. Right Detail Pane
    let header_card = components::modal_header(
        selected_cat.icon(),
        selected_cat.name(),
        Some(selected_cat.subtitle()),
        &tokens,
        close,
    );

    let detail_content = match selected_cat {
        SettingsCategory::Appearance => render_appearance_tab(&manifest, &tokens),
        SettingsCategory::Display => render_display_tab(&tokens),
        SettingsCategory::ShellBar => render_shell_bar_tab(&manifest, &tokens),
        SettingsCategory::Workspaces => render_workspaces_tab(&tokens),
        SettingsCategory::Animations => render_animations_tab(&tokens),
        SettingsCategory::WindowRules => render_window_rules_tab(&tokens),
        SettingsCategory::Shortcuts => render_shortcuts_tab(&tokens),
        SettingsCategory::Input => render_input_tab(&tokens),
        SettingsCategory::AudioBluetooth => render_audio_bluetooth_tab(&tokens),
        SettingsCategory::SystemPower => render_system_power_tab(&manifest, &tokens),
    };

    let detail_pane = Rectangle::new()
        .width(DETAIL_WIDTH)
        .height(Parent)
        .fill(tokens.bg)
        .padding(pad_all(18.0))
        .child(
            Column::new(vec![
                Box::new(header_card),
                Box::new(
                    Rectangle::new()
                        .width(Parent)
                        .height(1.0)
                        .fill(tokens.border_subtle),
                ),
                Box::new(
                    ScrollArea::new("settings_detail_scroll", Column::new(vec![detail_content]))
                        .width(Parent)
                        .height(Parent),
                ),
            ])
            .gap(14.0)
            .width(Parent)
            .height(Parent),
        );

    let body = Row::new(vec![Box::new(sidebar), Box::new(detail_pane)])
        .width(Parent)
        .height(Parent);

    let window_shell = Rectangle::new()
        .width(SETTINGS_WIDTH)
        .height(SETTINGS_HEIGHT)
        .radius(radius::LG)
        .fill(tokens.bg)
        .border(1.0, tokens.border_subtle)
        .child(body);

    let motion_shell = motion.apply_centered_modal(window_shell);

    LayerWindow::new()
        .width(SETTINGS_WIDTH)
        .height(SETTINGS_HEIGHT)
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
        .keyboard(if is_open {
            Keyboard::OnDemand
        } else {
            Keyboard::None
        })
        .on_key(|key| {
            if key == Key::Escape {
                close();
            }
        })
        .space(Zone::Ignore)
        .visible(motion.visible)
        .namespace("dotfield-shell-settings")
        .child(motion_shell)
}

// ---------------------------------------------------------------------------
// TAB RENDERERS (Appearance, Shell & Bar, Placeholders)
// ---------------------------------------------------------------------------

fn render_appearance_tab(
    manifest: &crate::config::ShellManifest,
    tokens: &ThemeTokens,
) -> Box<dyn Widget> {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Shell Style Presets
    let current_style = &manifest.theme.style;
    let styles = &["cyber-synthwave", "win11-dark", "nordic-slate", "minimal"];
    let mut style_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &s in styles {
        let is_cur = current_style == s;
        let btn = Rectangle::new()
            .width(130.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(12.0, 4.0))
            .on_click(move |_| {
                let _ = ConfigService::update_and_save(|m| {
                    m.theme.style = s.to_string();
                });
            })
            .child(
                Text::new(s)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        style_buttons.push(Box::new(btn));
    }

    let style_card = Rectangle::new()
        .width(Parent)
        .height(96.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Shell Theme Style")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new("Immediate GPU style preset applied to taskbar, popups, and HUDs")
                        .size(typography::SECONDARY)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(style_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(style_card));

    // 2. Accent Color Swatches
    let accents = &[
        ("#04C5F0", "Cyan"),
        ("#00F5D4", "Mint"),
        ("#FF7F41", "Amber"),
        ("#FF3366", "Neon Pink"),
        ("#A78BFA", "Violet"),
    ];
    let cur_accent = &manifest.palette.accent;
    let mut swatch_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(hex, label) in accents {
        let is_cur = cur_accent.eq_ignore_ascii_case(hex);
        let color = Color::from(hex);
        let swatch = Rectangle::new()
            .width(96.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(color)
            .border(
                if is_cur { 2.0 } else { 1.0 },
                if is_cur {
                    Color::WHITE
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = ConfigService::update_and_save(|m| {
                    m.palette.accent = hex.to_string();
                });
            })
            .child(
                Text::new(if is_cur {
                    format!("✓ {label}")
                } else {
                    label.to_string()
                })
                .size(typography::SECONDARY)
                .weight(Weight::SemiBold)
                .color(Color::BLACK)
                .font(&tokens.font_sans),
            );
        swatch_buttons.push(Box::new(swatch));
    }

    let accent_card = Rectangle::new()
        .width(Parent)
        .height(96.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Accent Color")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new("Primary highlights, active pills, indicators, and focus rings")
                        .size(typography::SECONDARY)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(swatch_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(accent_card));

    // 3. One-Click Rice Profiles
    let mut profile_cards: Vec<Box<dyn Widget>> = Vec::new();
    for p in PROFILES {
        let is_cur = current_style == p.shell_style;
        let card = Rectangle::new()
            .width(186.0)
            .height(84.0)
            .radius(radius::MD)
            .fill(if is_cur {
                tokens.surface_elevated
            } else {
                tokens.surface
            })
            .border(
                if is_cur { 1.5 } else { 1.0 },
                if is_cur {
                    Color::from(p.accent)
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_all(10.0))
            .on_click({
                let p = p.clone();
                move |_| {
                    AppearanceService::apply_rice_profile(&p);
                }
            })
            .child(
                Column::new(vec![
                    Box::new(
                        Row::new(vec![
                            Box::new(
                                Rectangle::new()
                                    .width(10.0)
                                    .height(10.0)
                                    .radius(radius::PILL)
                                    .fill(Color::from(p.accent)),
                            ),
                            Box::new(
                                Text::new(p.name)
                                    .size(typography::BODY)
                                    .weight(Weight::SemiBold)
                                    .color(tokens.text_primary)
                                    .font(&tokens.font_sans),
                            ),
                        ])
                        .gap(6.0)
                        .align(Align::Center),
                    ),
                    Box::new(
                        Text::new(p.description)
                            .size(typography::CAPTION)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                    ),
                ])
                .gap(6.0),
            );
        profile_cards.push(Box::new(card));
    }

    let profiles_card = Rectangle::new()
        .width(Parent)
        .height(220.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("One-Click Rice Profiles")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new("Synchronized theme suites across Shell, GTK, Icons, Cursor, and Accent colors")
                        .size(typography::SECONDARY)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Column::new(vec![
                        Box::new(Row::new(profile_cards.drain(0..3).collect()).gap(8.0)),
                        Box::new(Row::new(profile_cards).gap(8.0)),
                    ])
                    .gap(8.0),
                ),
            ])
            .gap(10.0),
        );
    rows.push(Box::new(profiles_card));

    // 4. Wallpaper Browser (awww)
    let app_svc = AppearanceService::read();
    let current_wp = app_svc.current_wallpaper.clone();
    let wallpapers = app_svc.wallpapers.clone();
    let gtk_themes = app_svc.gtk_themes.clone();
    let cur_gtk = app_svc.current_gtk_theme.clone();
    let icon_themes = app_svc.icon_themes.clone();
    let cur_icon = app_svc.current_icon_theme.clone();
    let cursor_themes = app_svc.cursor_themes.clone();
    let cur_cursor = app_svc.current_cursor_theme.clone();
    let cur_cursor_size = app_svc.cursor_size;
    let font_families = app_svc.font_families.clone();
    let cur_font = app_svc.current_font.clone();
    drop(app_svc);

    let mut wp_items: Vec<Box<dyn Widget>> = Vec::new();
    for wp in wallpapers.iter().take(6) {
        let is_cur = current_wp.as_ref() == Some(wp);
        let wp_name = wp
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("wallpaper")
            .to_string();
        let wp_path = wp.clone();

        let btn = Rectangle::new()
            .width(186.0)
            .height(44.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 6.0))
            .on_click(move |_| {
                AppearanceService::apply_wallpaper(&wp_path);
            })
            .child(
                Row::new(vec![
                    Box::new(
                        Text::new("󰸉")
                            .size(16.0)
                            .color(if is_cur {
                                tokens.surface
                            } else {
                                tokens.accent
                            })
                            .font(&tokens.font_mono)
                            .tight(),
                    ),
                    Box::new(
                        Text::new(wp_name)
                            .size(typography::CAPTION)
                            .weight(if is_cur {
                                Weight::SemiBold
                            } else {
                                Weight::Regular
                            })
                            .color(if is_cur {
                                tokens.surface
                            } else {
                                tokens.text_primary
                            })
                            .font(&tokens.font_sans),
                    ),
                ])
                .gap(6.0)
                .align(Align::Center),
            );
        wp_items.push(Box::new(btn));
    }

    if !wp_items.is_empty() {
        let wp_card = Rectangle::new()
            .width(Parent)
            .height(130.0)
            .radius(radius::MD)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .padding(pad_all(14.0))
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new("Wallpaper Browser (awww)")
                            .size(typography::BODY)
                            .weight(Weight::SemiBold)
                            .color(tokens.text_primary)
                            .font(&tokens.font_sans),
                    ),
                    Box::new(
                        Text::new(
                            "Wallpapers located in ~/Pictures/Wallpapers or /usr/share/backgrounds",
                        )
                        .size(typography::SECONDARY)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                    ),
                    Box::new(Row::new(wp_items).gap(8.0)),
                ])
                .gap(10.0),
            );
        rows.push(Box::new(wp_card));
    }

    // 5. GTK & Icon Theme Selectors
    let mut gtk_buttons: Vec<Box<dyn Widget>> = Vec::new();
    for g in gtk_themes.iter().take(4) {
        let is_cur = cur_gtk == *g;
        let g_clone = g.clone();
        let btn = Rectangle::new()
            .width(135.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                AppearanceService::apply_gtk_theme(&g_clone);
            })
            .child(
                Text::new(g)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.surface
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
            );
        gtk_buttons.push(Box::new(btn));
    }

    let mut icon_buttons: Vec<Box<dyn Widget>> = Vec::new();
    for ic in icon_themes.iter().take(4) {
        let is_cur = cur_icon == *ic;
        let ic_clone = ic.clone();
        let btn = Rectangle::new()
            .width(135.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                AppearanceService::apply_icon_theme(&ic_clone);
            })
            .child(
                Text::new(ic)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.surface
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
            );
        icon_buttons.push(Box::new(btn));
    }

    let gtk_card = Rectangle::new()
        .width(Parent)
        .height(140.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("GTK & Icon Themes")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(vec![Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("GTK Theme")
                                .size(typography::CAPTION)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(Row::new(gtk_buttons).gap(6.0)),
                    ])
                    .gap(4.0),
                )])),
                Box::new(Row::new(vec![Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("Icon Theme")
                                .size(typography::CAPTION)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(Row::new(icon_buttons).gap(6.0)),
                    ])
                    .gap(4.0),
                )])),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(gtk_card));

    // 6. Cursor Theme & Size Controls
    let mut cursor_buttons: Vec<Box<dyn Widget>> = Vec::new();
    for cur in cursor_themes.iter().take(4) {
        let is_cur = cur_cursor == *cur;
        let c_clone = cur.clone();
        let btn = Rectangle::new()
            .width(150.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                AppearanceService::apply_cursor_theme(&c_clone);
            })
            .child(
                Text::new(cur)
                    .size(typography::CAPTION)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.surface
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
            );
        cursor_buttons.push(Box::new(btn));
    }

    let sizes = &[24u32, 32, 48];
    let mut size_btns: Vec<Box<dyn Widget>> = Vec::new();
    for &sz in sizes {
        let is_cur = cur_cursor_size == sz;
        let btn = Rectangle::new()
            .width(60.0)
            .height(30.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(8.0, 4.0))
            .on_click(move |_| {
                AppearanceService::apply_cursor_size(sz);
            })
            .child(
                Text::new(format!("{sz}px"))
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.surface
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
            );
        size_btns.push(Box::new(btn));
    }

    let cursor_card = Rectangle::new()
        .width(Parent)
        .height(115.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Cursor Theme & Size")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Column::new(vec![
                                Box::new(
                                    Text::new("Cursor Theme")
                                        .size(typography::CAPTION)
                                        .color(tokens.text_muted)
                                        .font(&tokens.font_sans),
                                ),
                                Box::new(Row::new(cursor_buttons).gap(6.0)),
                            ])
                            .gap(4.0),
                        ),
                        Box::new(
                            Column::new(vec![
                                Box::new(
                                    Text::new("Size")
                                        .size(typography::CAPTION)
                                        .color(tokens.text_muted)
                                        .font(&tokens.font_sans),
                                ),
                                Box::new(Row::new(size_btns).gap(6.0)),
                            ])
                            .gap(4.0),
                        ),
                    ])
                    .gap(16.0),
                ),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(cursor_card));

    // 7. Font Size & Family Controls
    let font_sizes = &[10.0, 11.5, 13.0, 14.5];
    let cur_size = manifest.theme.font_size;
    let mut size_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &sz in font_sizes {
        let is_cur = (cur_size - sz).abs() < 0.1;
        let btn = Rectangle::new()
            .width(76.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = ConfigService::update_and_save(|m| {
                    m.theme.font_size = sz;
                });
            })
            .child(
                Text::new(format!("{sz} px"))
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        size_buttons.push(Box::new(btn));
    }

    let mut font_pills: Vec<Box<dyn Widget>> = Vec::new();
    for f in font_families.iter().take(4) {
        let is_cur = cur_font.starts_with(f) || manifest.theme.font_family.starts_with(f);
        let f_clone = f.clone();
        let btn = Rectangle::new()
            .width(135.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(8.0, 4.0))
            .on_click(move |_| {
                AppearanceService::apply_font(&f_clone);
            })
            .child(
                Text::new(f)
                    .size(typography::CAPTION)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.surface
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
            );
        font_pills.push(Box::new(btn));
    }

    let font_card = Rectangle::new()
        .width(Parent)
        .height(130.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new("UI Base Font & Size")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(format!("• {}", manifest.theme.font_family))
                                .size(typography::SECONDARY)
                                .color(tokens.text_muted)
                                .font(&tokens.font_mono),
                        ),
                    ])
                    .gap(spacing::GAP_SM)
                    .align(Align::Center),
                ),
                Box::new(Row::new(size_buttons).gap(spacing::GAP_SM)),
                Box::new(Row::new(font_pills).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(font_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

fn render_shell_bar_tab(
    manifest: &crate::config::ShellManifest,
    tokens: &ThemeTokens,
) -> Box<dyn Widget> {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Bar Position (bottom vs top)
    let is_bottom = manifest.bar.position == "bottom";
    let pos_card = Rectangle::new()
        .width(Parent)
        .height(106.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Bar Position")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new("Dock & status line placement along screen edges")
                        .size(typography::SECONDARY)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Rectangle::new()
                                .width(180.0)
                                .height(30.0)
                                .radius(radius::SM)
                                .fill(if is_bottom {
                                    tokens.accent
                                } else {
                                    tokens.surface_elevated
                                })
                                .border(
                                    1.0,
                                    if is_bottom {
                                        tokens.accent
                                    } else {
                                        tokens.border_subtle
                                    },
                                )
                                .cursor(Pointer)
                                .padding(pad_sym(14.0, 4.0))
                                .on_click(|_| {
                                    let _ = ConfigService::update_and_save(|m| {
                                        m.bar.position = "bottom".to_string();
                                    });
                                })
                                .child(
                                    Text::new("󰘳 Bottom (Taskbar)")
                                        .size(typography::SECONDARY)
                                        .weight(if is_bottom {
                                            Weight::SemiBold
                                        } else {
                                            Weight::Regular
                                        })
                                        .color(if is_bottom {
                                            tokens.text_on_accent
                                        } else {
                                            tokens.text_secondary
                                        })
                                        .font(&tokens.font_sans),
                                ),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(180.0)
                                .height(30.0)
                                .radius(radius::SM)
                                .fill(if !is_bottom {
                                    tokens.accent
                                } else {
                                    tokens.surface_elevated
                                })
                                .border(
                                    1.0,
                                    if !is_bottom {
                                        tokens.accent
                                    } else {
                                        tokens.border_subtle
                                    },
                                )
                                .cursor(Pointer)
                                .padding(pad_sym(14.0, 4.0))
                                .on_click(|_| {
                                    let _ = ConfigService::update_and_save(|m| {
                                        m.bar.position = "top".to_string();
                                    });
                                })
                                .child(
                                    Text::new("󰘲 Top (Panel)")
                                        .size(typography::SECONDARY)
                                        .weight(if !is_bottom {
                                            Weight::SemiBold
                                        } else {
                                            Weight::Regular
                                        })
                                        .color(if !is_bottom {
                                            tokens.text_on_accent
                                        } else {
                                            tokens.text_secondary
                                        })
                                        .font(&tokens.font_sans),
                                ),
                        ),
                    ])
                    .gap(spacing::GAP_SM),
                ),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(pos_card));

    // 2. Bar Height
    let heights = &[40.0, 44.0, 48.0, 54.0];
    let cur_h = manifest.bar.height;
    let mut height_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &h in heights {
        let is_cur = (cur_h - h).abs() < 0.5;
        let btn = Rectangle::new()
            .width(80.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(12.0, 4.0))
            .on_click(move |_| {
                let _ = ConfigService::update_and_save(|m| {
                    m.bar.height = h;
                });
            })
            .child(
                Text::new(format!("{h} px"))
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        height_buttons.push(Box::new(btn));
    }

    let height_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Bar Height")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(height_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(height_card));

    // 3. Timeouts (OSD & Notifications)
    let timeouts_card = Rectangle::new()
        .width(Parent)
        .height(84.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Display Timeouts")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new(format!("OSD Timeout: {} ms", manifest.osd.timeout_ms))
                                .size(typography::SECONDARY)
                                .color(tokens.text_secondary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(format!(
                                "• Notifications: {} ms",
                                manifest.notifications.timeout_ms
                            ))
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(spacing::GAP_SM),
                ),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(timeouts_card));

    // 4. Default Terminal
    let terms = &["ghostty", "foot", "kitty"];
    let cur_term = &manifest.terminal.default;
    let mut term_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &t in terms {
        let is_cur = cur_term == t;
        let btn = Rectangle::new()
            .width(96.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(14.0, 4.0))
            .on_click(move |_| {
                let _ = ConfigService::update_and_save(|m| {
                    m.terminal.default = t.to_string();
                });
            })
            .child(
                Text::new(t)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        term_buttons.push(Box::new(btn));
    }

    let term_card = Rectangle::new()
        .width(Parent)
        .height(96.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Default Terminal Emulator")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Text::new("Used for sidecar terminal and CLI shortcut spawning")
                        .size(typography::SECONDARY)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(term_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(term_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

fn render_display_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Safe Mode Warning Banner (if countdown active)
    let safe_mode = DisplayTopologyService::read().safe_mode.clone();
    if let Some(safe) = safe_mode {
        let warning_card = Rectangle::new()
            .width(Parent)
            .height(72.0)
            .radius(radius::MD)
            .fill(Color::rgba(240, 60, 60, 40))
            .border(1.5, Color::rgba(240, 80, 80, 200))
            .padding(pad_all(12.0))
            .child(
                Row::new(vec![
                    Box::new(
                        Column::new(vec![
                            Box::new(
                                Text::new(format!(
                                    "󰀦 Display settings applied to {} — Reverting in {}s",
                                    safe.output_name, safe.remaining_secs
                                ))
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(Color::rgb(255, 200, 200))
                                .font(&tokens.font_sans),
                            ),
                            Box::new(
                                Text::new(
                                    "Confirm to keep new resolution and refresh rate, or revert now.",
                                )
                                .size(typography::SECONDARY)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                            ),
                        ])
                        .gap(4.0)
                        .width(360.0),
                    ),
                    Box::new(
                        Row::new(vec![
                            Box::new(
                                Rectangle::new()
                                    .width(100.0)
                                    .height(34.0)
                                    .radius(radius::SM)
                                    .fill(tokens.accent)
                                    .cursor(Pointer)
                                    .padding(pad_sym(16.0, 6.0))
                                    .on_click(|_| {
                                        DisplayTopologyService::confirm_safe_mode();
                                    })
                                    .child(
                                        Text::new("Keep")
                                            .size(typography::SECONDARY)
                                            .weight(Weight::SemiBold)
                                            .color(tokens.surface)
                                            .font(&tokens.font_sans),
                                    ),
                            ),
                            Box::new(
                                Rectangle::new()
                                    .width(90.0)
                                    .height(34.0)
                                    .radius(radius::SM)
                                    .fill(tokens.surface)
                                    .border(1.0, tokens.border_subtle)
                                    .cursor(Pointer)
                                    .padding(pad_sym(16.0, 6.0))
                                    .on_click(|_| {
                                        DisplayTopologyService::revert_safe_mode();
                                    })
                                    .child(
                                        Text::new("Revert")
                                            .size(typography::SECONDARY)
                                            .weight(Weight::SemiBold)
                                            .color(tokens.text_primary)
                                            .font(&tokens.font_sans),
                                    ),
                            ),
                        ])
                        .gap(8.0)
                        .align(Align::Center),
                    ),
                ])
                .gap(12.0)
                .align(Align::Center)
                .width(Parent),
            );
        rows.push(Box::new(warning_card));
    }

    // 2. Connected Outputs Row (Displays Canvas)
    let display_svc = DisplayTopologyService::read();
    let selected_name = display_svc.selected_output.clone();
    let outputs = display_svc.outputs.clone();
    drop(display_svc);

    let mut output_cards: Vec<Box<dyn Widget>> = Vec::new();
    for out in &outputs {
        let is_selected = out.name == selected_name;
        let border_color = if is_selected {
            tokens.accent
        } else {
            tokens.border_subtle
        };
        let bg_color = if is_selected {
            tokens.surface_elevated
        } else {
            tokens.surface
        };
        let out_name = out.name.clone();

        let card = Rectangle::new()
            .width(180.0)
            .height(110.0)
            .radius(radius::MD)
            .fill(bg_color)
            .border(if is_selected { 2.0 } else { 1.0 }, border_color)
            .cursor(Pointer)
            .padding(pad_all(12.0))
            .on_click(move |_| {
                DisplayTopologyService::select_output(&out_name);
            })
            .child(
                Column::new(vec![
                    Box::new(
                        Row::new(vec![
                            Box::new(
                                Text::new("󰍹")
                                    .size(16.0)
                                    .color(if is_selected {
                                        tokens.accent
                                    } else {
                                        tokens.text_muted
                                    })
                                    .font(&tokens.font_mono)
                                    .tight(),
                            ),
                            Box::new(
                                Text::new(&out.name)
                                    .size(typography::BODY)
                                    .weight(Weight::SemiBold)
                                    .color(tokens.text_primary)
                                    .font(&tokens.font_sans),
                            ),
                        ])
                        .gap(6.0)
                        .align(Align::Center),
                    ),
                    Box::new(
                        Text::new(format!("{} {}", out.make, out.model))
                            .size(typography::CAPTION)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                    ),
                    Box::new(
                        Text::new(format!(
                            "{}×{} @ {:.0}Hz",
                            out.width, out.height, out.refresh_hz
                        ))
                        .size(typography::SECONDARY)
                        .weight(Weight::Medium)
                        .color(if is_selected {
                            tokens.accent
                        } else {
                            tokens.text_secondary
                        })
                        .font(&tokens.font_sans),
                    ),
                    Box::new(
                        Text::new(format!(
                            "Scale: {:.2}× | VRR: {}",
                            out.scale,
                            if out.vrr_enabled { "On" } else { "Off" }
                        ))
                        .size(typography::CAPTION)
                        .color(tokens.text_muted)
                        .font(&tokens.font_sans),
                    ),
                ])
                .gap(4.0)
                .width(Parent),
            );
        output_cards.push(Box::new(card));
    }

    if output_cards.is_empty() {
        let empty = Rectangle::new()
            .width(Parent)
            .height(80.0)
            .radius(radius::MD)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .padding(pad_all(20.0))
            .child(
                Text::new("No active displays detected via Niri IPC")
                    .size(typography::BODY)
                    .color(tokens.text_muted)
                    .font(&tokens.font_sans),
            );
        rows.push(Box::new(empty));
    } else {
        rows.push(Box::new(Row::new(output_cards).gap(12.0)));
    }

    // 3. Selected Display Settings (Resolution, Refresh, Scaling, VRR)
    let cur_out = outputs.iter().find(|o| o.name == selected_name).cloned();
    if let Some(cur) = cur_out {
        // Mode & Refresh Rate selection
        let mut mode_pills: Vec<Box<dyn Widget>> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for m in &cur.modes {
            let key = (m.width, m.height, m.refresh_hz.round() as u32);
            if seen.insert(key) && mode_pills.len() < 6 {
                let is_cur = cur.width == m.width && (cur.refresh_hz - m.refresh_hz).abs() < 1.0;
                let sel_out = cur.name.clone();
                let mw = m.width;
                let mh = m.height;
                let mhz = m.refresh_hz;

                let pill = Rectangle::new()
                    .width(135.0)
                    .height(34.0)
                    .radius(radius::SM)
                    .fill(if is_cur {
                        tokens.accent
                    } else {
                        tokens.surface
                    })
                    .border(
                        1.0,
                        if is_cur {
                            tokens.accent
                        } else {
                            tokens.border_subtle
                        },
                    )
                    .cursor(Pointer)
                    .padding(pad_sym(10.0, 6.0))
                    .on_click(move |_| {
                        DisplayTopologyService::apply_mode(&sel_out, mw, mh, Some(mhz));
                    })
                    .child(
                        Text::new(format!("{}×{} {:.0}Hz", m.width, m.height, m.refresh_hz))
                            .size(typography::CAPTION)
                            .weight(if is_cur {
                                Weight::SemiBold
                            } else {
                                Weight::Regular
                            })
                            .color(if is_cur {
                                tokens.surface
                            } else {
                                tokens.text_primary
                            })
                            .font(&tokens.font_sans),
                    );
                mode_pills.push(Box::new(pill));
            }
        }

        let mode_card = Rectangle::new()
            .width(Parent)
            .height(115.0)
            .radius(radius::MD)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .padding(pad_all(14.0))
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new(format!("Resolution & Refresh Rate ({})", cur.name))
                            .size(typography::BODY)
                            .weight(Weight::SemiBold)
                            .color(tokens.text_primary)
                            .font(&tokens.font_sans),
                    ),
                    Box::new(
                        Text::new("Selecting a new mode triggers a 12s safe mode rollback timer")
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                    ),
                    Box::new(Row::new(mode_pills).gap(8.0)),
                ])
                .gap(8.0),
            );
        rows.push(Box::new(mode_card));

        // Display Scaling
        let scales: &[f64] = &[1.0, 1.25, 1.5, 1.75, 2.0];
        let mut scale_buttons: Vec<Box<dyn Widget>> = Vec::new();
        for &s in scales {
            let is_cur = (cur.scale - s).abs() < 0.05;
            let sel_out = cur.name.clone();
            let btn = Rectangle::new()
                .width(80.0)
                .height(34.0)
                .radius(radius::SM)
                .fill(if is_cur {
                    tokens.accent
                } else {
                    tokens.surface
                })
                .border(
                    1.0,
                    if is_cur {
                        tokens.accent
                    } else {
                        tokens.border_subtle
                    },
                )
                .cursor(Pointer)
                .padding(pad_sym(14.0, 6.0))
                .on_click(move |_| {
                    DisplayTopologyService::apply_scale(&sel_out, s);
                })
                .child(
                    Text::new(format!("{:.2}×", s))
                        .size(typography::SECONDARY)
                        .weight(if is_cur {
                            Weight::SemiBold
                        } else {
                            Weight::Regular
                        })
                        .color(if is_cur {
                            tokens.surface
                        } else {
                            tokens.text_primary
                        })
                        .font(&tokens.font_sans),
                );
            scale_buttons.push(Box::new(btn));
        }

        let scale_card = Rectangle::new()
            .width(Parent)
            .height(95.0)
            .radius(radius::MD)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .padding(pad_all(14.0))
            .child(
                Column::new(vec![
                    Box::new(
                        Text::new("Display Scaling")
                            .size(typography::BODY)
                            .weight(Weight::SemiBold)
                            .color(tokens.text_primary)
                            .font(&tokens.font_sans),
                    ),
                    Box::new(
                        Text::new("Fractional and integer scaling applied live via Niri IPC")
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                    ),
                    Box::new(Row::new(scale_buttons).gap(8.0)),
                ])
                .gap(8.0),
            );
        rows.push(Box::new(scale_card));

        // VRR / Adaptive Sync
        let sel_out = cur.name.clone();
        let vrr_on = cur.vrr_enabled;
        let vrr_card = Rectangle::new()
            .width(Parent)
            .height(72.0)
            .radius(radius::MD)
            .fill(tokens.surface)
            .border(1.0, tokens.border_subtle)
            .padding(pad_all(14.0))
            .child(
                Row::new(vec![
                    Box::new(
                        Column::new(vec![
                            Box::new(
                                Text::new("Variable Refresh Rate (VRR / FreeSync)")
                                    .size(typography::BODY)
                                    .weight(Weight::SemiBold)
                                    .color(tokens.text_primary)
                                    .font(&tokens.font_sans),
                            ),
                            Box::new(
                                Text::new(if cur.vrr_supported {
                                    "Dynamic refresh rate pacing to prevent screen tearing"
                                } else {
                                    "VRR unsupported or unadvertised on this display connection"
                                })
                                .size(typography::SECONDARY)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                            ),
                        ])
                        .gap(4.0)
                        .width(420.0),
                    ),
                    Box::new(
                        Rectangle::new()
                            .width(84.0)
                            .height(34.0)
                            .radius(radius::SM)
                            .fill(if vrr_on {
                                tokens.accent
                            } else {
                                tokens.surface
                            })
                            .border(
                                1.0,
                                if vrr_on {
                                    tokens.accent
                                } else {
                                    tokens.border_subtle
                                },
                            )
                            .cursor(Pointer)
                            .padding(pad_sym(12.0, 6.0))
                            .on_click(move |_| {
                                DisplayTopologyService::apply_vrr(&sel_out, !vrr_on);
                            })
                            .child(
                                Text::new(if vrr_on { "Enabled" } else { "Disabled" })
                                    .size(typography::SECONDARY)
                                    .weight(Weight::SemiBold)
                                    .color(if vrr_on {
                                        tokens.surface
                                    } else {
                                        tokens.text_primary
                                    })
                                    .font(&tokens.font_sans),
                            ),
                    ),
                ])
                .gap(12.0)
                .align(Align::Center)
                .width(Parent),
            );
        rows.push(Box::new(vrr_card));
    }

    // 4. Night Light Controls (wlsunset)
    let nl_enabled = NightLightService::is_active();
    let nl_temp = NightLightService::current_temp();
    let manifest = ConfigService::read().manifest.clone();
    let nl_mode = manifest.night_light.mode;

    let temps: &[(u32, &str)] = &[
        (3500, "3500K Candle"),
        (4000, "4000K Warm"),
        (4500, "4500K Soft"),
        (5500, "5500K Neutral"),
    ];

    let mut temp_buttons: Vec<Box<dyn Widget>> = Vec::new();
    for &(t, lbl) in temps {
        let is_cur = nl_temp == t;
        let btn = Rectangle::new()
            .width(115.0)
            .height(34.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 6.0))
            .on_click(move |_| {
                NightLightService::set_temperature(t);
            })
            .child(
                Text::new(lbl)
                    .size(typography::CAPTION)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.surface
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans),
            );
        temp_buttons.push(Box::new(btn));
    }

    let night_light_card = Rectangle::new()
        .width(Parent)
        .height(130.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Column::new(vec![
                                Box::new(
                                    Text::new("Night Light (wlsunset)")
                                        .size(typography::BODY)
                                        .weight(Weight::SemiBold)
                                        .color(tokens.text_primary)
                                        .font(&tokens.font_sans),
                                ),
                                Box::new(
                                    Text::new(format!(
                                        "Filters blue light to reduce eye strain — Mode: {}",
                                        if nl_mode == "location" {
                                            "Automatic (Sunset)"
                                        } else {
                                            "Manual"
                                        }
                                    ))
                                    .size(typography::SECONDARY)
                                    .color(tokens.text_muted)
                                    .font(&tokens.font_sans),
                                ),
                            ])
                            .gap(4.0)
                            .width(420.0),
                        ),
                        Box::new(
                            Rectangle::new()
                                .width(84.0)
                                .height(34.0)
                                .radius(radius::SM)
                                .fill(if nl_enabled {
                                    tokens.accent
                                } else {
                                    tokens.surface
                                })
                                .border(
                                    1.0,
                                    if nl_enabled {
                                        tokens.accent
                                    } else {
                                        tokens.border_subtle
                                    },
                                )
                                .cursor(Pointer)
                                .padding(pad_sym(12.0, 6.0))
                                .on_click(|_| {
                                    NightLightService::toggle();
                                })
                                .child(
                                    Text::new(if nl_enabled { "Active" } else { "Turn On" })
                                        .size(typography::SECONDARY)
                                        .weight(Weight::SemiBold)
                                        .color(if nl_enabled {
                                            tokens.surface
                                        } else {
                                            tokens.text_primary
                                        })
                                        .font(&tokens.font_sans),
                                ),
                        ),
                    ])
                    .gap(12.0)
                    .align(Align::Center)
                    .width(Parent),
                ),
                Box::new(
                    Text::new("Target Color Temperature")
                        .size(typography::CAPTION)
                        .weight(Weight::Medium)
                        .color(tokens.text_secondary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(temp_buttons).gap(8.0)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(night_light_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 4. WORKSPACES TAB
// ---------------------------------------------------------------------------

fn render_workspaces_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let state = NiriConfigService::read().state.clone();
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // Status Banner if any
    if let Some(ref st) = state.last_status {
        let is_err = st.contains("Validation failed") || st.contains("failed");
        let banner = Rectangle::new()
            .width(Parent)
            .height(38.0)
            .radius(radius::SM)
            .fill(if is_err {
                Color::rgba(240, 60, 60, 30)
            } else {
                Color::rgba(40, 200, 100, 30)
            })
            .border(
                1.0,
                if is_err {
                    Color::rgba(240, 80, 80, 150)
                } else {
                    Color::rgba(60, 220, 120, 150)
                },
            )
            .padding(pad_sym(12.0, 6.0))
            .child(
                Text::new(format!("󰋽 {st}"))
                    .size(typography::SECONDARY)
                    .color(if is_err {
                        Color::rgb(255, 180, 180)
                    } else {
                        Color::rgb(180, 255, 200)
                    })
                    .font(&tokens.font_sans),
            );
        rows.push(Box::new(banner));
    }

    // 1. Gaps
    let gaps_opts = &[0.0, 4.0, 8.0, 12.0, 16.0, 24.0];
    let cur_gap = state.gaps;
    let mut gap_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &g in gaps_opts {
        let is_cur = (cur_gap - g).abs() < 0.5;
        let btn = Rectangle::new()
            .width(68.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.gaps = g;
                });
            })
            .child(
                Text::new(format!("{g} px"))
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        gap_buttons.push(Box::new(btn));
    }

    let gap_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Inner & Outer Workspace Gaps")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(gap_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(gap_card));

    // 2. Default Column Width
    let widths = &[
        (0.33333, "1/3 Screen"),
        (0.5, "1/2 Screen"),
        (0.66667, "2/3 Screen"),
        (1.0, "Full Screen"),
    ];
    let cur_col = state.default_column_width;
    let mut col_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(w, label) in widths {
        let is_cur = (cur_col - w).abs() < 0.05;
        let btn = Rectangle::new()
            .width(115.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.default_column_width = w;
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        col_buttons.push(Box::new(btn));
    }

    let col_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Default Window Column Width")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(col_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(col_card));

    // 3. Center Focused Column
    let centers = &[
        ("never", "Never"),
        ("always", "Always"),
        ("on-overflow", "On Overflow"),
    ];
    let cur_center = &state.center_focused_column;
    let mut center_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(mode, label) in centers {
        let is_cur = cur_center == mode;
        let btn = Rectangle::new()
            .width(115.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.center_focused_column = mode.to_string();
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        center_buttons.push(Box::new(btn));
    }

    let center_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Center Focused Window Column")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(center_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(center_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 5. ANIMATIONS TAB
// ---------------------------------------------------------------------------

fn render_animations_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let state = NiriConfigService::read().state.clone();
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Animations Toggle Card
    let anim_enabled = state.animations_enabled;
    let toggle_card = Rectangle::new()
        .width(Parent)
        .height(80.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("Window & Workspace Animations")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(
                                "Smooth spring physics and transition animations across Niri.",
                            )
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(4.0)
                    .width(420.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(100.0)
                        .height(32.0)
                        .radius(radius::SM)
                        .fill(if anim_enabled {
                            tokens.accent
                        } else {
                            tokens.surface_elevated
                        })
                        .border(
                            1.0,
                            if anim_enabled {
                                tokens.accent
                            } else {
                                tokens.border_subtle
                            },
                        )
                        .cursor(Pointer)
                        .padding(pad_sym(12.0, 6.0))
                        .on_click(move |_| {
                            let _ = NiriConfigService::update_and_validate(|s| {
                                s.animations_enabled = !anim_enabled;
                            });
                        })
                        .child(
                            Text::new(if anim_enabled { "Enabled" } else { "Disabled" })
                                .size(typography::SECONDARY)
                                .weight(Weight::SemiBold)
                                .color(if anim_enabled {
                                    tokens.text_on_accent
                                } else {
                                    tokens.text_secondary
                                })
                                .font(&tokens.font_sans),
                        ),
                ),
            ])
            .gap(12.0)
            .align(Align::Center)
            .width(Parent),
        );
    rows.push(Box::new(toggle_card));

    // 2. Animation Duration Presets
    let durations = &[
        (100, "100 ms (Snappy)"),
        (200, "200 ms (Standard)"),
        (350, "350 ms (Smooth)"),
        (500, "500 ms (Cinematic)"),
    ];
    let cur_dur = state.animation_duration_ms;
    let mut dur_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(d, label) in durations {
        let is_cur = cur_dur == d;
        let btn = Rectangle::new()
            .width(140.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.animation_duration_ms = d;
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        dur_buttons.push(Box::new(btn));
    }

    let dur_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Animation Transition Duration")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(dur_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(dur_card));

    // 3. Easing Curve Presets
    let curves = &[
        ("ease-out-cubic", "Ease-Out Cubic"),
        ("ease-out-quad", "Ease-Out Quad"),
        ("linear", "Linear"),
    ];
    let cur_curve = &state.animation_curve;
    let mut curve_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(c, label) in curves {
        let is_cur = cur_curve == c;
        let btn = Rectangle::new()
            .width(135.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.animation_curve = c.to_string();
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        curve_buttons.push(Box::new(btn));
    }

    let curve_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Animation Easing Curve")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(curve_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(curve_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 6. WINDOW RULES TAB
// ---------------------------------------------------------------------------

fn render_window_rules_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let state = NiriConfigService::read().state.clone();
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Corner Radius
    let radii = &[0.0, 6.0, 8.0, 12.0, 16.0];
    let cur_r = state.corner_radius;
    let mut rad_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &r in radii {
        let is_cur = (cur_r - r).abs() < 0.5;
        let btn = Rectangle::new()
            .width(76.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.corner_radius = r;
                });
            })
            .child(
                Text::new(format!("{r} px"))
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        rad_buttons.push(Box::new(btn));
    }

    let rad_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Window Corner Radius")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(rad_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(rad_card));

    // 2. Active Window Border Width
    let borders = &[0.0, 1.0, 2.0, 3.0, 4.0];
    let cur_bw = state.border_width;
    let mut bw_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &b in borders {
        let is_cur = (cur_bw - b).abs() < 0.5;
        let btn = Rectangle::new()
            .width(76.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.border_width = b;
                });
            })
            .child(
                Text::new(format!("{b} px"))
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        bw_buttons.push(Box::new(btn));
    }

    let bw_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Active Window Border Width")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(bw_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(bw_card));

    // 3. Active Window Opacity
    let opacities = &[
        (0.80, "80%"),
        (0.85, "85%"),
        (0.90, "90%"),
        (1.0, "100% (Solid)"),
    ];
    let cur_op = state.active_window_opacity;
    let mut op_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(op, label) in opacities {
        let is_cur = (cur_op - op).abs() < 0.03;
        let btn = Rectangle::new()
            .width(105.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.active_window_opacity = op;
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        op_buttons.push(Box::new(btn));
    }

    let op_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Window Transparency & Opacity")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(op_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(op_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 7. KEYBOARD SHORTCUTS TAB
// ---------------------------------------------------------------------------

fn render_shortcuts_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    let shortcut_groups = &[
        (
            "Super / Mod+D / Mod+Space",
            "Start Menu & App Launcher",
            "󰀻",
        ),
        (
            "Mod+W / Mod+C / Mod+A",
            "Control Center & Quick Settings",
            "󰒓",
        ),
        ("Mod+N", "Notification Center HUD", "󰂚"),
        ("Mod+I / Mod+Shift+Comma", "Amane Settings Window", "󰒓"),
        ("Mod+Return", "Default Terminal Emulator", "󰞷"),
        (
            "Mod+Shift+Return",
            "Terminal in Focused Window Directory",
            "󰞷",
        ),
        ("Mod+Alt+Return", "Companion Terminal Sidecar", "󰞷"),
        ("Mod+E", "Run-or-Raise File Explorer", "󰉋"),
        ("Mod+B", "Run-or-Raise Web Browser", "󰈹"),
        ("Mod+Q / Alt+F4", "Close Active Window", "󰅖"),
        (
            "Mod+Tab / Alt+Tab",
            "Interactive Task Overview & Switcher",
            "󰘲",
        ),
        (
            "Ctrl+Mod+Up / Down",
            "Navigate Workspaces (Vertical Axis)",
            "󰘳",
        ),
        (
            "Mod+Shift+S / Shift+Print",
            "Interactive Screen Area Snip",
            "󰹑",
        ),
        ("Ctrl+Shift+Escape", "Launch Task Manager (btop)", "󰘚"),
        ("Ctrl+Alt+Delete / Mod+Shift+E", "Power & Session Menu", "󰐥"),
    ];

    let header_card = Rectangle::new()
        .width(Parent)
        .height(60.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Text::new("Compositor & Shell Keybindings (Configured in 08-binds.kdl)")
                .size(typography::BODY)
                .weight(Weight::SemiBold)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        );
    rows.push(Box::new(header_card));

    for &(bind, desc, icon) in shortcut_groups {
        let item = Rectangle::new()
            .width(Parent)
            .height(44.0)
            .radius(radius::SM)
            .fill(tokens.surface_elevated)
            .border(1.0, tokens.border_subtle)
            .padding(pad_sym(14.0, 8.0))
            .child(
                Row::new(vec![
                    Box::new(
                        Row::new(vec![
                            Box::new(
                                Text::new(icon)
                                    .size(16.0)
                                    .color(tokens.accent)
                                    .font(&tokens.font_mono),
                            ),
                            Box::new(
                                Text::new(desc)
                                    .size(typography::BODY)
                                    .weight(Weight::Medium)
                                    .color(tokens.text_primary)
                                    .font(&tokens.font_sans),
                            ),
                        ])
                        .gap(10.0)
                        .align(Align::Center)
                        .width(280.0),
                    ),
                    Box::new(
                        Rectangle::new()
                            .width(220.0)
                            .height(26.0)
                            .radius(radius::SM)
                            .fill(tokens.surface)
                            .border(1.0, tokens.border_subtle)
                            .padding(pad_sym(10.0, 4.0))
                            .child(
                                Text::new(bind)
                                    .size(typography::BADGE)
                                    .weight(Weight::SemiBold)
                                    .color(tokens.accent)
                                    .font(&tokens.font_mono),
                            ),
                    ),
                ])
                .gap(12.0)
                .align(Align::Center)
                .width(Parent),
            );
        rows.push(Box::new(item));
    }

    Box::new(Column::new(rows).gap(8.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 8. INPUT & PERIPHERALS TAB
// ---------------------------------------------------------------------------

fn render_input_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let state = NiriConfigService::read().state.clone();
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Touchpad Natural Scroll
    let nat_scroll = state.natural_scroll;
    let nat_card = Rectangle::new()
        .width(Parent)
        .height(72.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("Natural Scrolling")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new("Reverse scrolling direction on touchpad and wheel.")
                                .size(typography::SECONDARY)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(4.0)
                    .width(420.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(96.0)
                        .height(30.0)
                        .radius(radius::SM)
                        .fill(if nat_scroll {
                            tokens.accent
                        } else {
                            tokens.surface_elevated
                        })
                        .border(
                            1.0,
                            if nat_scroll {
                                tokens.accent
                            } else {
                                tokens.border_subtle
                            },
                        )
                        .cursor(Pointer)
                        .padding(pad_sym(12.0, 6.0))
                        .on_click(move |_| {
                            let _ = NiriConfigService::update_and_validate(|s| {
                                s.natural_scroll = !nat_scroll;
                            });
                        })
                        .child(
                            Text::new(if nat_scroll { "Enabled" } else { "Disabled" })
                                .size(typography::SECONDARY)
                                .weight(Weight::SemiBold)
                                .color(if nat_scroll {
                                    tokens.text_on_accent
                                } else {
                                    tokens.text_secondary
                                })
                                .font(&tokens.font_sans),
                        ),
                ),
            ])
            .gap(12.0)
            .align(Align::Center)
            .width(Parent),
        );
    rows.push(Box::new(nat_card));

    // 2. Tap-to-Click
    let tap = state.tap_to_click;
    let tap_card = Rectangle::new()
        .width(Parent)
        .height(72.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("Tap to Click")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new("Tap touchpad surface for left-click events.")
                                .size(typography::SECONDARY)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(4.0)
                    .width(420.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(96.0)
                        .height(30.0)
                        .radius(radius::SM)
                        .fill(if tap {
                            tokens.accent
                        } else {
                            tokens.surface_elevated
                        })
                        .border(
                            1.0,
                            if tap {
                                tokens.accent
                            } else {
                                tokens.border_subtle
                            },
                        )
                        .cursor(Pointer)
                        .padding(pad_sym(12.0, 6.0))
                        .on_click(move |_| {
                            let _ = NiriConfigService::update_and_validate(|s| {
                                s.tap_to_click = !tap;
                            });
                        })
                        .child(
                            Text::new(if tap { "Enabled" } else { "Disabled" })
                                .size(typography::SECONDARY)
                                .weight(Weight::SemiBold)
                                .color(if tap {
                                    tokens.text_on_accent
                                } else {
                                    tokens.text_secondary
                                })
                                .font(&tokens.font_sans),
                        ),
                ),
            ])
            .gap(12.0)
            .align(Align::Center)
            .width(Parent),
        );
    rows.push(Box::new(tap_card));

    // 3. Keyboard Repeat Delay & Rate
    let delays = &[(150, "150 ms (Fast)"), (200, "200 ms"), (300, "300 ms")];
    let cur_del = state.repeat_delay;
    let mut del_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(d, label) in delays {
        let is_cur = cur_del == d;
        let btn = Rectangle::new()
            .width(115.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = NiriConfigService::update_and_validate(|s| {
                    s.repeat_delay = d;
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        del_buttons.push(Box::new(btn));
    }

    let del_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Keyboard Repeat Delay")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(del_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(del_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 9. AUDIO & BLUETOOTH TAB
// ---------------------------------------------------------------------------

fn render_audio_bluetooth_tab(tokens: &ThemeTokens) -> Box<dyn Widget> {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();

    // 1. Audio Control Center Trigger
    let audio_card = Rectangle::new()
        .width(Parent)
        .height(84.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("󰕾 PipeWire / WirePlumber Audio")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(
                                "Manage active playback endpoints and mic inputs via Control Center HUD.",
                            )
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(4.0)
                    .width(420.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(110.0)
                        .height(32.0)
                        .radius(radius::SM)
                        .fill(tokens.accent)
                        .cursor(Pointer)
                        .padding(pad_sym(12.0, 6.0))
                        .on_click(|_| {
                            crate::ui::control_center::toggle();
                        })
                        .child(
                            Text::new("Open HUD")
                                .size(typography::SECONDARY)
                                .weight(Weight::SemiBold)
                                .color(tokens.surface)
                                .font(&tokens.font_sans),
                        ),
                ),
            ])
            .gap(12.0)
            .align(Align::Center)
            .width(Parent),
        );
    rows.push(Box::new(audio_card));

    // 2. Bluetooth Manager
    let bt_card = Rectangle::new()
        .width(Parent)
        .height(84.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("󰂯 BlueZ Bluetooth Manager")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(
                                "Pair, disconnect, and monitor peripherals using Bluetooth manager.",
                            )
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(4.0)
                    .width(420.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(110.0)
                        .height(32.0)
                        .radius(radius::SM)
                        .fill(tokens.surface_elevated)
                        .border(1.0, tokens.border_subtle)
                        .cursor(Pointer)
                        .padding(pad_sym(12.0, 6.0))
                        .on_click(|_| {
                            let _ = std::process::Command::new("sh")
                                .args(["-c", "which blueman-manager >/dev/null && blueman-manager &"])
                                .spawn();
                        })
                        .child(
                            Text::new("Pair Devices")
                                .size(typography::SECONDARY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                ),
            ])
            .gap(12.0)
            .align(Align::Center)
            .width(Parent),
        );
    rows.push(Box::new(bt_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

// ---------------------------------------------------------------------------
// 10. SYSTEM & POWER TAB
// ---------------------------------------------------------------------------

fn render_system_power_tab(
    manifest: &crate::config::ShellManifest,
    tokens: &ThemeTokens,
) -> Box<dyn Widget> {
    let mut rows: Vec<Box<dyn Widget>> = Vec::new();
    let telem = TelemetryService::read();

    // 1. Live System Telemetry Card
    let telem_card = Rectangle::new()
        .width(Parent)
        .height(100.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Live System Telemetry")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Row::new(vec![
                        Box::new(
                            Text::new(format!("󰍛 CPU: {}%", telem.cpu_percent))
                                .size(typography::SECONDARY)
                                .weight(Weight::Medium)
                                .color(tokens.accent)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(format!(
                                "• 󰘚 RAM: {}% ({}/{} MB)",
                                telem.mem_percent, telem.mem_used_mb, telem.mem_total_mb
                            ))
                            .size(typography::SECONDARY)
                            .color(tokens.text_secondary)
                            .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new(if telem.battery_present {
                                format!(
                                    "• 󰁹 Battery: {}%{}",
                                    telem.battery_percent.unwrap_or(0),
                                    if telem.battery_charging {
                                        " (Charging)"
                                    } else {
                                        ""
                                    }
                                )
                            } else {
                                "• 󰁹 AC Connected".to_string()
                            })
                            .size(typography::SECONDARY)
                            .color(tokens.text_muted)
                            .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(12.0),
                ),
            ])
            .gap(10.0),
        );
    rows.push(Box::new(telem_card));

    // 2. Idle Inactivity Timeout Presets
    let timeouts = &[
        (300, "5 min"),
        (600, "10 min"),
        (900, "15 min"),
        (1800, "30 min"),
    ];
    let cur_to = manifest.osd.timeout_ms / 1000;
    let mut to_buttons: Vec<Box<dyn Widget>> = Vec::new();

    for &(to_sec, label) in timeouts {
        let is_cur = cur_to == to_sec;
        let btn = Rectangle::new()
            .width(85.0)
            .height(28.0)
            .radius(radius::SM)
            .fill(if is_cur {
                tokens.accent
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if is_cur {
                    tokens.accent
                } else {
                    tokens.border_subtle
                },
            )
            .cursor(Pointer)
            .padding(pad_sym(10.0, 4.0))
            .on_click(move |_| {
                let _ = ConfigService::update_and_save(|m| {
                    m.osd.timeout_ms = to_sec * 1000;
                });
            })
            .child(
                Text::new(label)
                    .size(typography::SECONDARY)
                    .weight(if is_cur {
                        Weight::SemiBold
                    } else {
                        Weight::Regular
                    })
                    .color(if is_cur {
                        tokens.text_on_accent
                    } else {
                        tokens.text_secondary
                    })
                    .font(&tokens.font_sans),
            );
        to_buttons.push(Box::new(btn));
    }

    let to_card = Rectangle::new()
        .width(Parent)
        .height(90.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Column::new(vec![
                Box::new(
                    Text::new("Screen Idle Lock & Sleep Timeout")
                        .size(typography::BODY)
                        .weight(Weight::SemiBold)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(Row::new(to_buttons).gap(spacing::GAP_SM)),
            ])
            .gap(8.0),
        );
    rows.push(Box::new(to_card));

    // 3. Power Session Actions
    let power_card = Rectangle::new()
        .width(Parent)
        .height(84.0)
        .radius(radius::MD)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(pad_all(14.0))
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new("󰐥 Power & Session Management")
                                .size(typography::BODY)
                                .weight(Weight::SemiBold)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                        Box::new(
                            Text::new("Lock session, restart, or power off system via Powermenu.")
                                .size(typography::SECONDARY)
                                .color(tokens.text_muted)
                                .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(4.0)
                    .width(420.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(130.0)
                        .height(32.0)
                        .radius(radius::SM)
                        .fill(tokens.accent)
                        .cursor(Pointer)
                        .padding(pad_sym(12.0, 6.0))
                        .on_click(|_| {
                            crate::ui::powermenu::toggle();
                        })
                        .child(
                            Text::new("Open Powermenu")
                                .size(typography::SECONDARY)
                                .weight(Weight::SemiBold)
                                .color(tokens.surface)
                                .font(&tokens.font_sans),
                        ),
                ),
            ])
            .gap(12.0)
            .align(Align::Center)
            .width(Parent),
        );
    rows.push(Box::new(power_card));

    Box::new(Column::new(rows).gap(12.0).width(Parent))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_category_parsing() {
        assert_eq!(
            SettingsCategory::from_str_loose("appearance"),
            Some(SettingsCategory::Appearance)
        );
        assert_eq!(
            SettingsCategory::from_str_loose("theme"),
            Some(SettingsCategory::Appearance)
        );
        assert_eq!(
            SettingsCategory::from_str_loose("display"),
            Some(SettingsCategory::Display)
        );
        assert_eq!(
            SettingsCategory::from_str_loose("shell"),
            Some(SettingsCategory::ShellBar)
        );
        assert_eq!(
            SettingsCategory::from_str_loose("workspaces"),
            Some(SettingsCategory::Workspaces)
        );
        assert_eq!(
            SettingsCategory::from_str_loose("shortcuts"),
            Some(SettingsCategory::Shortcuts)
        );
        assert_eq!(SettingsCategory::from_str_loose("unknown-section"), None);
    }

    #[test]
    fn test_settings_toggle_and_close() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        assert!(!SettingsService::read().open);
        toggle();
        assert!(SettingsService::read().open);
        close();
        assert!(!SettingsService::read().open);
    }

    #[test]
    fn test_settings_open_category() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        open_category(SettingsCategory::Display);
        let svc = SettingsService::read();
        assert!(svc.open);
        assert_eq!(svc.selected, SettingsCategory::Display);
        drop(svc);
        close();
    }
}
