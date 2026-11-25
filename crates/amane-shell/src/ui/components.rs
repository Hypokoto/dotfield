use amane::{
    Align, Center, Color, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Scroll, Stack,
    Text, Widget,
};

use crate::ui::tokens::{ThemeTokens, border, radius, spacing, typography};

/// Standardized action button for closing overlays and modals (Smooth Clay Pill)
pub fn close_button(on_click_fn: impl Fn() + 'static, tokens: &ThemeTokens) -> Rectangle {
    Rectangle::new()
        .width(26.0)
        .height(26.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_click_fn())
        .child(
            Text::new("󰅖")
                .size(typography::BODY)
                .color(tokens.text_secondary)
                .font(&tokens.font_mono)
                .tight(),
        )
}

/// Standardized modal header bar across all popover windows (Smooth Clay styling)
pub fn modal_header(
    icon: &str,
    title: &str,
    tag: Option<&str>,
    tokens: &ThemeTokens,
    on_close: impl Fn() + 'static,
) -> Rectangle {
    let mut title_elements: Vec<Box<dyn Widget>> = vec![
        Box::new(
            Text::new(icon)
                .size(typography::TITLE_SM)
                .color(tokens.accent)
                .font(&tokens.font_mono),
        ),
        Box::new(
            Text::new(title)
                .size(typography::TITLE_SM)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ];

    if let Some(tag_text) = tag {
        title_elements.push(Box::new(
            Text::new(format!("• {tag_text}"))
                .size(typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_sans),
        ));
    }

    let title_row = Row::new(title_elements)
        .gap(spacing::GAP_SM)
        .align(Align::Center);

    let close_btn = close_button(on_close, tokens);

    let header_row = Row::new(vec![Box::new(title_row), Box::new(close_btn)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(28.0)
        .align_child(Align::Start, Align::Center)
        .child(header_row)
}

/// Standardized keyboard shortcut keycap badge (Smooth Clay Pill Badge)
pub fn keycap(key_text: &str, tokens: &ThemeTokens) -> Rectangle {
    let char_count = key_text.chars().count();
    let cap_width = (char_count as f32 * 7.5 + 14.0).max(22.0);
    Rectangle::new()
        .width(cap_width)
        .height(20.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 5.0,
            right: 5.0,
        })
        .align_child(Center, Center)
        .child(
            Text::new(key_text)
                .size(typography::CAPTION)
                .color(tokens.text_primary)
                .font(&tokens.font_mono)
                .tight(),
        )
}

/// Smooth Clay Modal Shell Frame
/// Continuous rounded ceramic surface with soft Clay Lip border and warm earthen bisque fill.
pub fn clay_modal_frame(
    width: f32,
    height: f32,
    content: impl Widget + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius::LG)
        .fill(tokens.surface)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(content)
}

/// Compatibility alias for legacy callers
pub fn victorian_panel_frame(
    width: f32,
    height: f32,
    content: impl Widget + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    clay_modal_frame(width, height, content, tokens)
}

/// Status badge with dot indicator (Tactical Telemetry Badge)
pub fn status_badge(label: &str, dot_color: Color, tokens: &ThemeTokens) -> Rectangle {
    let char_count = label.chars().count();
    let badge_width = (char_count as f32 * 7.5 + 24.0).max(64.0);
    let content = Row::new(vec![
        Box::new(
            Rectangle::new()
                .width(6.0)
                .height(6.0)
                .radius(radius::PILL)
                .fill(dot_color),
        ),
        Box::new(
            Text::new(label)
                .size(typography::BADGE)
                .color(tokens.text_primary)
                .font(&tokens.font_family)
                .tight(),
        ),
    ])
    .gap(5.0)
    .align(Align::Center);

    Rectangle::new()
        .width(badge_width)
        .height(20.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(border::THIN, tokens.border_subtle)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 8.0,
            right: 8.0,
        })
        .align_child(Center, Center)
        .child(content)
}

/// Clean progress bar with rounded ends and high-contrast track
pub fn progress_bar(
    progress_frac: f32,
    bar_width: f32,
    bar_height: f32,
    fill_color: Color,
    tokens: &ThemeTokens,
) -> Rectangle {
    let clamped_frac = progress_frac.clamp(0.0, 1.0);
    let fill_w = (bar_width * clamped_frac).max(bar_height);

    let fill = Rectangle::new()
        .width(fill_w)
        .height(bar_height)
        .radius(radius::PILL)
        .fill(fill_color);

    Rectangle::new()
        .width(bar_width)
        .height(bar_height)
        .radius(radius::PILL)
        .fill(tokens.surface_track)
        .child(fill)
}

/// Standardized hairline divider
pub fn divider(tokens: &ThemeTokens) -> Rectangle {
    Rectangle::new()
        .width(Parent)
        .height(1.0)
        .fill(tokens.border_subtle)
}

/// Full modal shell container wrapping header, body, and keyboard shortcuts footer
#[allow(clippy::too_many_arguments)]
pub fn modal_shell(
    width: f32,
    height: f32,
    icon: &str,
    title: &str,
    tag: Option<&str>,
    body: Box<dyn Widget>,
    footer_hints: Vec<(&str, &str)>,
    on_close: impl Fn() + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let header = modal_header(icon, title, tag, tokens, on_close);

    let mut shell_children: Vec<Box<dyn Widget>> =
        vec![Box::new(header), Box::new(divider(tokens)), body];

    if !footer_hints.is_empty() {
        shell_children.push(Box::new(divider(tokens)));

        let mut hint_widgets: Vec<Box<dyn Widget>> = Vec::new();
        for (key, desc) in footer_hints {
            let item = Row::new(vec![
                Box::new(keycap(key, tokens)),
                Box::new(
                    Text::new(desc)
                        .size(typography::CAPTION)
                        .color(tokens.text_muted)
                        .font(&tokens.font_family),
                ),
            ])
            .gap(spacing::GAP_XS)
            .align(Align::Center);

            hint_widgets.push(Box::new(item));
        }

        let footer_row = Row::new(hint_widgets)
            .gap(spacing::GAP_MD)
            .align(Align::Center);

        shell_children.push(Box::new(footer_row));
    }

    let shell_column = Column::new(shell_children)
        .gap(spacing::GAP_MD)
        .width(Parent)
        .height(Parent);

    clay_modal_frame(width, height, shell_column, tokens)
}

/// Universal action card tile for App Launcher, Terminal Manager, and Display Presets
#[allow(clippy::too_many_arguments)]
pub fn action_card(
    width: f32,
    height: f32,
    icon: Box<dyn Widget>,
    title: &str,
    subtitle: Option<&str>,
    badge: Option<&str>,
    selected: bool,
    hovered: bool,
    on_click: impl Fn() + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let (bg, border) = if selected {
        (tokens.surface_selected, tokens.border_focus)
    } else if hovered {
        (tokens.surface_hover, tokens.border_hover)
    } else {
        (tokens.surface_elevated, tokens.border_subtle)
    };

    let title_color = if selected {
        tokens.accent
    } else {
        tokens.text_primary
    };

    let avail_width = (width - spacing::PAD_MD * 2.0).max(60.0);
    let (badge_widget, badge_space) = if let Some(badge_text) = badge {
        let badge_w = (badge_text.chars().count() as f32 * 7.5 + 14.0).max(22.0);
        let pill_badge = Rectangle::new()
            .width(badge_w)
            .height(20.0)
            .radius(radius::PILL)
            .fill(if selected {
                tokens.accent_tint
            } else {
                tokens.surface_elevated
            })
            .border(
                1.0,
                if selected {
                    tokens.border_focus
                } else {
                    tokens.border_subtle
                },
            )
            .padding(Padding {
                top: 2.0,
                bottom: 2.0,
                left: 6.0,
                right: 6.0,
            })
            .align_child(Center, Center)
            .child(
                Text::new(badge_text)
                    .size(typography::CAPTION)
                    .color(if selected {
                        tokens.accent
                    } else {
                        tokens.text_primary
                    })
                    .font(&tokens.font_sans)
                    .tight(),
            );
        (Some(pill_badge), badge_w + spacing::GAP_MD)
    } else {
        (None, 0.0)
    };

    let text_max_w = (avail_width - 30.0 - spacing::GAP_MD - badge_space).max(40.0);

    let mut text_elements: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(title)
            .size(typography::BODY)
            .color(title_color)
            .font(&tokens.font_sans)
            .elide(),
    )];

    if let Some(sub) = subtitle {
        let home = std::env::var("HOME").unwrap_or_default();
        let formatted_sub = if !home.is_empty() && sub.starts_with(&home) {
            format!("~{}", &sub[home.len()..])
        } else {
            sub.to_string()
        };

        text_elements.push(Box::new(
            Text::new(formatted_sub)
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_sans)
                .elide(),
        ));
    }

    let text_column = Column::new(text_elements)
        .width(text_max_w)
        .gap(spacing::GAP_XS)
        .align(Align::Start);

    let left_items: Vec<Box<dyn Widget>> = vec![icon, Box::new(text_column)];

    let card_row = Row::new(left_items)
        .width(avail_width - badge_space)
        .gap(spacing::GAP_MD)
        .align(Align::Center);

    let outer_row = if let Some(badge_w) = badge_widget {
        Row::new(vec![Box::new(card_row), Box::new(badge_w)])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center)
    } else {
        Row::new(vec![Box::new(card_row)])
            .width(Parent)
            .align(Align::Center)
    };

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius::MD)
        .fill(bg)
        .border(
            if selected {
                border::FOCUS
            } else {
                border::THIN
            },
            border,
        )
        .cursor(Pointer)
        .padding(Padding {
            top: spacing::PAD_SM,
            bottom: spacing::PAD_SM,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .align_child(Align::Start, Align::Center)
        .on_click(move |_| on_click())
        .child(outer_row)
}

/// Action card with hover state tracking
#[allow(clippy::too_many_arguments)]
pub fn action_card_interactive(
    width: f32,
    height: f32,
    icon: Box<dyn Widget>,
    title: &str,
    subtitle: Option<&str>,
    badge: Option<&str>,
    selected: bool,
    hovered: bool,
    on_hover: impl Fn(bool) + 'static,
    on_click: impl Fn() + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let card = action_card(
        width, height, icon, title, subtitle, badge, selected, hovered, on_click, tokens,
    );
    card.on_hover(on_hover)
}

/// Modular telemetry capsule block for the top status bar
pub fn telemetry_module(icon: &str, text: &str, warn: bool, tokens: &ThemeTokens) -> Rectangle {
    let (icon_color, text_color, border_color) = if warn {
        (tokens.accent, tokens.accent, tokens.border_focus)
    } else {
        (tokens.accent, tokens.text_primary, tokens.border_subtle)
    };

    let char_count = text.chars().count();
    let width = (char_count as f32 * 7.5 + 32.0).max(56.0);

    let row = Row::new(vec![
        Box::new(
            Text::new(icon)
                .size(typography::BODY)
                .color(icon_color)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new(text)
                .size(typography::SECONDARY)
                .color(text_color)
                .font(&tokens.font_family),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    Rectangle::new()
        .width(width)
        .height(24.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(border::THIN, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: spacing::PAD_SM,
            right: spacing::PAD_SM,
        })
        .align_child(Center, Center)
        .child(row)
}

/// WinUI 3 ToggleSwitch primitive: 40x20 pill track with sliding thumb and binary state toggle
pub fn toggle_switch(
    checked: bool,
    on_toggle: impl Fn(bool) + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let track_w = 40.0;
    let track_h = 20.0;
    let thumb_size = 12.0;

    let thumb_left = if checked { 24.0 } else { 4.0 };
    let thumb_color = if checked { Color::BLACK } else { tokens.steel };
    let track_fill = if checked {
        tokens.accent
    } else {
        tokens.surface_track
    };
    let track_border = if checked {
        tokens.accent
    } else {
        tokens.border_subtle
    };

    let thumb = Rectangle::new()
        .width(thumb_size)
        .height(thumb_size)
        .radius(radius::PILL)
        .fill(thumb_color);

    let thumb_wrapper = Rectangle::new()
        .width(track_w)
        .height(track_h)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: thumb_left,
            right: 4.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(thumb);

    Rectangle::new()
        .width(track_w)
        .height(track_h)
        .radius(radius::PILL)
        .fill(track_fill)
        .border(border::THIN, track_border)
        .cursor(Pointer)
        .on_click(move |_| on_toggle(!checked))
        .child(thumb_wrapper)
}

/// Smooth Clay Settings / Quick Settings tile row with title, optional subtitle, and trailing ToggleSwitch
pub fn toggle_switch_card(
    checked: bool,
    icon: &str,
    title: &str,
    subtitle: Option<&str>,
    on_toggle: impl Fn(bool) + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let mut text_elements: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(title)
            .size(typography::BODY)
            .color(tokens.text_primary)
            .font(&tokens.font_sans),
    )];

    if let Some(sub) = subtitle {
        text_elements.push(Box::new(
            Text::new(sub)
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_sans),
        ));
    }

    let text_col = Column::new(text_elements).gap(2.0).align(Align::Start);

    let left_row = Row::new(vec![
        Box::new(
            Text::new(icon)
                .size(typography::BODY_LG)
                .color(if checked {
                    tokens.accent
                } else {
                    tokens.text_muted
                })
                .font(&tokens.font_mono),
        ),
        Box::new(text_col),
    ])
    .gap(spacing::GAP_MD)
    .align(Align::Center);

    let switch = toggle_switch(checked, on_toggle, tokens);

    let content_row = Row::new(vec![Box::new(left_row), Box::new(switch)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let (card_fill, card_border) = if checked {
        (tokens.surface_active, tokens.border_focus)
    } else {
        (tokens.surface_elevated, tokens.border_subtle)
    };

    Rectangle::new()
        .width(Parent)
        .height(if subtitle.is_some() { 48.0 } else { 40.0 })
        .radius(radius::MD)
        .fill(card_fill)
        .border(1.0, card_border)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .align_child(Align::Start, Align::Center)
        .child(content_row)
}

/// Smooth Clay Tactile Gauge Slider with recessed clay track,
/// glazed terracotta progress fill, circular pill icon button, and tactile scroll.
#[allow(clippy::too_many_arguments)]
pub fn fluent_slider<F1, F2>(
    progress_frac: f32,
    slider_width: f32,
    slider_height: f32,
    icon: &str,
    label: &str,
    is_muted: bool,
    tokens: &ThemeTokens,
    on_scroll_fn: F1,
    on_icon_click: Option<F2>,
) -> Rectangle
where
    F1: Fn(i32) + 'static,
    F2: Fn() + 'static,
{
    let clamped_frac = progress_frac.clamp(0.0, 1.0);
    let fill_w = (slider_width * clamped_frac).clamp(0.0, slider_width);

    let (icon_color, value_text) = if is_muted {
        (tokens.danger, "Muted".to_string())
    } else {
        (tokens.accent, format!("{:.0}%", clamped_frac * 100.0))
    };

    // 1. Base container track: recessed clay surface with subtle border
    let bg_rect = Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .radius(radius::MD)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle);

    // 2. Filled progress track with smooth clay active fill (no harsh outer border)
    let fill_rect = Rectangle::new()
        .width(fill_w)
        .height(slider_height)
        .radius(radius::MD)
        .fill(if is_muted {
            tokens.surface_track
        } else {
            tokens.surface_active
        });

    // 3. Leading icon: smooth circular pill button
    let icon_btn: Box<dyn Widget> = if let Some(on_click) = on_icon_click {
        Box::new(
            Rectangle::new()
                .width(26.0)
                .height(26.0)
                .radius(radius::PILL)
                .fill(if is_muted {
                    tokens.danger_subtle
                } else {
                    tokens.surface_elevated
                })
                .border(
                    1.0,
                    if is_muted {
                        tokens.border_danger
                    } else {
                        tokens.border_subtle
                    },
                )
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| on_click())
                .child(
                    Text::new(icon)
                        .size(typography::BODY_LG)
                        .color(if is_muted { tokens.danger } else { icon_color })
                        .font(&tokens.font_mono),
                ),
        )
    } else {
        Box::new(
            Text::new(icon)
                .size(typography::BODY_LG)
                .color(icon_color)
                .font(&tokens.font_mono),
        )
    };

    // 4. Content Row: Left (icon + label) | Right (value percentage)
    let left_items = Row::new(vec![
        icon_btn,
        Box::new(
            Text::new(label)
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let right_badge = Text::new(value_text)
        .size(typography::BADGE)
        .color(if is_muted {
            tokens.danger
        } else {
            tokens.accent
        })
        .font(&tokens.font_mono);

    let label_row = Row::new(vec![Box::new(left_items), Box::new(right_badge)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let content_rect = Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 12.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(label_row);

    let slider_stack = Stack::new(vec![
        Box::new(bg_rect),
        Box::new(fill_rect),
        Box::new(content_rect),
    ]);

    Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .cursor(Pointer)
        .on_scroll(move |scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            on_scroll_fn(step);
        })
        .child(slider_stack)
}

/// WinUI 3 InfoBadge: numeric count or status dot indicator
pub fn info_badge(
    count: Option<usize>,
    dot_only: bool,
    badge_color: Color,
    tokens: &ThemeTokens,
) -> Rectangle {
    if dot_only {
        return Rectangle::new()
            .width(8.0)
            .height(8.0)
            .radius(radius::PILL)
            .fill(badge_color);
    }

    let text_str = if let Some(c) = count {
        if c > 99 {
            "99+".to_string()
        } else {
            c.to_string()
        }
    } else {
        String::new()
    };

    let w = (text_str.chars().count() as f32 * 6.5 + 10.0).max(18.0);
    Rectangle::new()
        .width(w)
        .height(16.0)
        .radius(radius::PILL)
        .fill(badge_color)
        .align_child(Center, Center)
        .child(
            Text::new(text_str)
                .size(typography::CAPTION)
                .color(Color::BLACK)
                .font(&tokens.font_family)
                .tight(),
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoBarSeverity {
    Informational,
    Success,
    Warning,
    Danger,
}

/// WinUI 3 InfoBar banner: inline dismissible notification with severity styling
pub fn info_bar(
    severity: InfoBarSeverity,
    title: &str,
    message: &str,
    on_close: Option<Box<dyn Fn() + 'static>>,
    tokens: &ThemeTokens,
) -> Rectangle {
    let (icon, color) = match severity {
        InfoBarSeverity::Informational => ("󰋼", tokens.accent),
        InfoBarSeverity::Success => ("󰄬", tokens.success),
        InfoBarSeverity::Warning => ("󰀦", tokens.warning),
        InfoBarSeverity::Danger => ("󰅚", tokens.danger),
    };

    let icon_w = Text::new(icon)
        .size(typography::BODY_LG)
        .color(color)
        .font(&tokens.font_family);

    let text_col = Column::new(vec![
        Box::new(
            Text::new(title)
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new(message)
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ),
    ])
    .gap(1.0);

    let mut row_children: Vec<Box<dyn Widget>> = vec![Box::new(icon_w), Box::new(text_col)];

    if let Some(close_fn) = on_close {
        let close_btn = Rectangle::new()
            .width(22.0)
            .height(22.0)
            .radius(radius::SM)
            .cursor(Pointer)
            .align_child(Center, Center)
            .on_click(move |_| close_fn())
            .child(
                Text::new("󰅖")
                    .size(typography::SECONDARY)
                    .color(tokens.text_muted)
                    .font(&tokens.font_family),
            );
        row_children.push(Box::new(close_btn));
    }

    let row = Row::new(row_children)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    Rectangle::new()
        .width(Parent)
        .height(if message.is_empty() { 36.0 } else { 48.0 })
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(border::THIN, color)
        .padding(Padding {
            top: spacing::PAD_SM,
            bottom: spacing::PAD_SM,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(row)
}

/// WinUI 3 PipsPager: horizontal workspace/tab indicator pips
pub fn pips_pager(
    total: usize,
    active: usize,
    on_select: impl Fn(usize) + 'static + Copy,
    tokens: &ThemeTokens,
) -> Rectangle {
    let mut pips: Vec<Box<dyn Widget>> = Vec::with_capacity(total);

    for i in 0..total {
        let is_active = i == active;
        let (pip_w, pip_fill) = if is_active {
            (18.0, tokens.accent)
        } else {
            (6.0, tokens.steel)
        };

        let pip = Rectangle::new()
            .width(pip_w)
            .height(6.0)
            .radius(radius::PILL)
            .fill(pip_fill)
            .cursor(Pointer)
            .on_click(move |_| on_select(i));

        pips.push(Box::new(pip));
    }

    let pips_row = Row::new(pips).gap(4.0).align(Align::Center);
    let total_w = (total as f32 * 10.0 + 12.0).max(24.0);

    Rectangle::new()
        .width(total_w)
        .height(16.0)
        .align_child(Center, Center)
        .child(pips_row)
}

/// WinUI 3 Expander: Collapsible disclosure card
#[allow(clippy::too_many_arguments)]
pub fn expander(
    icon: &str,
    title: &str,
    subtitle: Option<&str>,
    is_expanded: bool,
    trailing_action: Option<Box<dyn Widget>>,
    content: Box<dyn Widget>,
    on_toggle: impl Fn() + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let chevron_glyph = if is_expanded { "󰅃" } else { "󰅀" };

    let mut text_items: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(title)
            .size(typography::BODY)
            .color(tokens.text_primary)
            .font(&tokens.font_family),
    )];

    if let Some(sub) = subtitle {
        text_items.push(Box::new(
            Text::new(sub)
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ));
    }

    let title_block = Row::new(vec![
        Box::new(
            Text::new(icon)
                .size(typography::BODY_LG)
                .color(tokens.accent)
                .font(&tokens.font_family),
        ),
        Box::new(Column::new(text_items).gap(1.0).align(Align::Start)),
    ])
    .gap(spacing::GAP_MD)
    .align(Align::Center);

    let mut right_items: Vec<Box<dyn Widget>> = Vec::new();
    if let Some(action) = trailing_action {
        right_items.push(action);
    }

    right_items.push(Box::new(
        Text::new(chevron_glyph)
            .size(typography::BODY)
            .color(tokens.text_secondary)
            .font(&tokens.font_family),
    ));

    let header_row = Row::new(vec![
        Box::new(title_block),
        Box::new(
            Row::new(right_items)
                .gap(spacing::GAP_SM)
                .align(Align::Center),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    let header = Rectangle::new()
        .width(Parent)
        .height(if subtitle.is_some() { 48.0 } else { 40.0 })
        .cursor(Pointer)
        .on_click(move |_| on_toggle())
        .child(header_row);

    let mut card_children: Vec<Box<dyn Widget>> = vec![Box::new(header)];

    if is_expanded {
        card_children.push(Box::new(divider(tokens)));
        card_children.push(content);
    }

    let card_height = if is_expanded {
        120.0
    } else if subtitle.is_some() {
        56.0
    } else {
        46.0
    };

    Rectangle::new()
        .width(Parent)
        .height(card_height)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(border::THIN, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_SM,
            bottom: spacing::PAD_SM,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Column::new(card_children)
                .gap(spacing::GAP_SM)
                .width(Parent),
        )
}

/// Smooth Clay acrylic framed panel
pub fn acrylic_card(
    width: f32,
    height: f32,
    child: Box<dyn Widget>,
    tokens: &ThemeTokens,
) -> Rectangle {
    clay_modal_frame(
        width,
        height,
        Column::new(vec![child]).width(Parent).height(Parent),
        tokens,
    )
}

/// WinUI 3 SplitButton: primary click action on the left + dropdown arrow on the right
pub fn split_button(
    primary_label: &str,
    primary_icon: Option<&str>,
    on_primary: impl Fn() + 'static,
    on_flyout: impl Fn() + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let char_count = primary_label.chars().count();
    let left_w = (char_count as f32 * 7.5 + 32.0).max(60.0);

    let mut left_items: Vec<Box<dyn Widget>> = Vec::new();
    if let Some(ic) = primary_icon {
        left_items.push(Box::new(
            Text::new(ic)
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(&tokens.font_family),
        ));
    }
    left_items.push(Box::new(
        Text::new(primary_label)
            .size(typography::SECONDARY)
            .color(tokens.text_primary)
            .font(&tokens.font_family),
    ));

    let left_btn = Rectangle::new()
        .width(left_w)
        .height(32.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(border::THIN, tokens.border_subtle)
        .cursor(Pointer)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Center, Center)
        .on_click(move |_| on_primary())
        .child(
            Row::new(left_items)
                .gap(spacing::GAP_XS)
                .align(Align::Center),
        );

    let right_btn = Rectangle::new()
        .width(26.0)
        .height(32.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(border::THIN, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_flyout())
        .child(
            Text::new("󰅀")
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        );

    let total_w = left_w + 26.0 + 1.0;
    Rectangle::new().width(total_w).height(32.0).child(
        Row::new(vec![Box::new(left_btn), Box::new(right_btn)])
            .gap(1.0)
            .align(Align::Center),
    )
}

// =========================================================================
// Apple HIG iOS Bento Widget & Component Primitives
// =========================================================================

/// Base iOS Bento container frame featuring continuous squircle corners, translucent vibrancy fill,
/// and subtle inner specular border highlight for depth.
pub fn ios_widget_frame(
    width: f32,
    height: f32,
    corner_radius: f32,
    content: impl Widget + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let specular_highlight = Color::rgba(255, 255, 255, 18);

    // Outer container with squircle radius and glass specular outline
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(corner_radius)
        .fill(tokens.surface_elevated)
        .border(border::THIN, specular_highlight)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(content)
}

/// iOS System Small Bento Widget (~1×1 aspect, ~156×156px or custom dimensions).
/// Displays leading squircle glyph badge, title, primary metric/content, and optional secondary footer.
#[allow(clippy::too_many_arguments)]
pub fn ios_widget_small(
    title: &str,
    icon: &str,
    accent_color: Color,
    metric_text: &str,
    footer_text: Option<&str>,
    width: f32,
    height: f32,
    tokens: &ThemeTokens,
) -> Rectangle {
    let icon_badge = Rectangle::new()
        .width(32.0)
        .height(32.0)
        .radius(radius::SM)
        .fill(tokens.surface_active)
        .border(1.0, Color::rgba(255, 255, 255, 14))
        .align_child(Center, Center)
        .child(
            Text::new(icon)
                .size(typography::BODY_LG)
                .color(accent_color)
                .font(&tokens.font_mono),
        );

    let header_row = Row::new(vec![
        Box::new(icon_badge),
        Box::new(
            Text::new(title)
                .size(typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let mut body_items: Vec<Box<dyn Widget>> = vec![
        Box::new(header_row),
        Box::new(
            Text::new(metric_text)
                .size(typography::DISPLAY)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ];

    if let Some(footer) = footer_text {
        body_items.push(Box::new(
            Text::new(footer)
                .size(typography::CAPTION)
                .color(tokens.text_muted)
                .font(&tokens.font_sans),
        ));
    }

    let content = Column::new(body_items)
        .gap(spacing::GAP_SM)
        .align(Align::Start);

    ios_widget_frame(width, height, radius::IOS_WIDGET_SM, content, tokens)
}

/// iOS System Medium Bento Widget (~2×1 aspect, ~328×156px or custom dimensions).
/// Ideal for multimedia playback, calendar, or combined system telemetry overview.
pub fn ios_widget_medium(
    title: &str,
    icon: &str,
    leading_content: impl Widget + 'static,
    trailing_content: Option<Box<dyn Widget>>,
    width: f32,
    height: f32,
    tokens: &ThemeTokens,
) -> Rectangle {
    let icon_badge = Rectangle::new()
        .width(28.0)
        .height(28.0)
        .radius(radius::SM)
        .fill(tokens.surface_active)
        .border(1.0, Color::rgba(255, 255, 255, 14))
        .align_child(Center, Center)
        .child(
            Text::new(icon)
                .size(typography::BODY)
                .color(tokens.accent)
                .font(&tokens.font_mono),
        );

    let title_block = Row::new(vec![
        Box::new(icon_badge),
        Box::new(
            Text::new(title)
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let mut header_widgets: Vec<Box<dyn Widget>> = vec![Box::new(title_block)];
    if let Some(trailing) = trailing_content {
        header_widgets.push(trailing);
    }

    let header_bar = Row::new(header_widgets)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let content_col = Column::new(vec![Box::new(header_bar), Box::new(leading_content)])
        .width(Parent)
        .gap(spacing::GAP_SM)
        .align(Align::Start);

    ios_widget_frame(width, height, radius::IOS_WIDGET_MD, content_col, tokens)
}

/// iOS Control Center Squircle Quick Action Toggle (~68×68px or 72×72px).
/// Features continuous squircle curvature (`radius::IOS_TOGGLE: 16.0`), high-contrast active state,
/// embedded glyph, and tactile label.
pub fn ios_control_toggle(
    icon: &'static str,
    label: &str,
    is_active: bool,
    width: f32,
    height: f32,
    tokens: &ThemeTokens,
    on_toggle: impl Fn() + Send + Sync + 'static,
) -> Rectangle {
    let (bg_fill, icon_color, text_color, border_color) = if is_active {
        (
            tokens.accent,
            tokens.bg,
            tokens.text_primary,
            Color::rgba(255, 255, 255, 40),
        )
    } else {
        (
            tokens.surface_elevated,
            tokens.text_secondary,
            tokens.text_muted,
            Color::rgba(255, 255, 255, 12),
        )
    };

    let glyph_circle = Rectangle::new()
        .width(36.0)
        .height(36.0)
        .radius(radius::PILL)
        .fill(if is_active {
            Color::rgba(0, 0, 0, 24)
        } else {
            tokens.surface_active
        })
        .align_child(Center, Center)
        .child(
            Text::new(icon)
                .size(18.0)
                .color(icon_color)
                .font(&tokens.font_mono),
        );

    let content = Column::new(vec![
        Box::new(glyph_circle),
        Box::new(
            Text::new(label)
                .size(typography::CAPTION)
                .color(text_color)
                .font(&tokens.font_sans)
                .tight(),
        ),
    ])
    .gap(4.0)
    .align(Align::Center);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius::IOS_TOGGLE)
        .fill(bg_fill)
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_toggle())
        .child(content)
}

/// iOS Control Center Thick Pill Slider (~44-54px height) with embedded glyph and tactile fill.
/// Unlike thin desktop trackbars, iOS sliders present a chunky tactile rounded pill with embedded icon
/// and scroll/click interaction.
#[allow(clippy::too_many_arguments)]
pub fn ios_pill_slider<F1, F2>(
    progress_frac: f32,
    slider_width: f32,
    slider_height: f32,
    icon: &str,
    label: &str,
    is_muted: bool,
    tokens: &ThemeTokens,
    on_scroll_fn: F1,
    on_icon_click: Option<F2>,
) -> Rectangle
where
    F1: Fn(i32) + 'static,
    F2: Fn() + 'static,
{
    let clamped_frac = progress_frac.clamp(0.0, 1.0);
    let fill_w = (slider_width * clamped_frac).clamp(0.0, slider_width);

    let (icon_color, value_str) = if is_muted {
        (tokens.danger, "Muted".to_string())
    } else {
        (tokens.accent, format!("{:.0}%", clamped_frac * 100.0))
    };

    // 1. Base track: deep elevated pill with specular highlight
    let bg_rect = Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .radius(radius::PILL)
        .fill(tokens.surface_track)
        .border(1.0, Color::rgba(255, 255, 255, 14));

    // 2. Dynamic progress fill: illuminated pill
    let fill_rect = Rectangle::new()
        .width(fill_w)
        .height(slider_height)
        .radius(radius::PILL)
        .fill(if is_muted {
            tokens.danger_subtle
        } else {
            tokens.surface_active
        });

    // 3. Embedded glyph icon
    let icon_element: Box<dyn Widget> = if let Some(on_click) = on_icon_click {
        Box::new(
            Rectangle::new()
                .width(32.0)
                .height(32.0)
                .radius(radius::PILL)
                .fill(Color::rgba(0, 0, 0, 20))
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| on_click())
                .child(
                    Text::new(icon)
                        .size(17.0)
                        .color(icon_color)
                        .font(&tokens.font_mono),
                ),
        )
    } else {
        Box::new(
            Text::new(icon)
                .size(17.0)
                .color(icon_color)
                .font(&tokens.font_mono),
        )
    };

    let label_row = Row::new(vec![
        icon_element,
        Box::new(
            Text::new(label)
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    let value_tag = Text::new(value_str)
        .size(typography::BADGE)
        .color(if is_muted {
            tokens.danger
        } else {
            tokens.text_secondary
        })
        .font(&tokens.font_mono);

    let overlay_row = Row::new(vec![Box::new(label_row), Box::new(value_tag)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let overlay_container = Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 12.0,
            right: 14.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(overlay_row);

    let layers: Vec<Box<dyn Widget>> = vec![
        Box::new(bg_rect),
        Box::new(fill_rect),
        Box::new(overlay_container),
    ];

    Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .radius(radius::PILL)
        .cursor(Pointer)
        .on_scroll(move |scroll: Scroll| {
            let step = if scroll.y < 0.0 { 5 } else { -5 };
            on_scroll_fn(step);
        })
        .child(Stack::new(layers))
}

/// iOS Dynamic Island / Capsule Pill widget.
/// Floating morphic pill displaying contextual indicators (e.g., active recording, audio playing, battery charging).
pub fn ios_dynamic_capsule(
    icon: &str,
    title: &str,
    subtitle: Option<&str>,
    accent_color: Color,
    tokens: &ThemeTokens,
) -> Rectangle {
    let glyph = Text::new(icon)
        .size(typography::BODY)
        .color(accent_color)
        .font(&tokens.font_mono);

    let mut text_elements: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(title)
            .size(typography::BADGE)
            .color(tokens.text_primary)
            .font(&tokens.font_sans),
    )];

    if let Some(sub) = subtitle {
        text_elements.push(Box::new(
            Text::new(sub)
                .size(typography::CAPTION)
                .color(tokens.text_muted)
                .font(&tokens.font_sans),
        ));
    }

    let text_stack = Column::new(text_elements).gap(1.0).align(Align::Start);

    let row = Row::new(vec![Box::new(glyph), Box::new(text_stack)])
        .gap(spacing::GAP_SM)
        .align(Align::Center);

    let pill_w = 190.0;
    let pill_h = 36.0;

    Rectangle::new()
        .width(pill_w)
        .height(pill_h)
        .radius(radius::IOS_CAPSULE)
        .fill(Color::rgba(0, 0, 0, 180))
        .border(1.0, Color::rgba(255, 255, 255, 25))
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 12.0,
            right: 14.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ShellManifest;

    fn test_tokens() -> ThemeTokens {
        ThemeTokens::from_manifest(&ShellManifest::default())
    }

    #[test]
    fn test_toggle_switch_renders() {
        let tokens = test_tokens();
        let _sw_on = toggle_switch(true, |_| {}, &tokens);
        let _sw_off = toggle_switch(false, |_| {}, &tokens);
    }

    #[test]
    fn test_toggle_switch_card_renders() {
        let tokens = test_tokens();
        let _card = toggle_switch_card(true, "󰤨", "Wi-Fi", Some("Connected"), |_| {}, &tokens);
    }

    #[test]
    fn test_fluent_slider_renders() {
        let tokens = test_tokens();
        let _slider = fluent_slider(
            0.75,
            200.0,
            32.0,
            "󰕾",
            "Volume 75%",
            false,
            &tokens,
            |_| {},
            None::<fn()>,
        );
    }

    #[test]
    fn test_info_badge_renders() {
        let tokens = test_tokens();
        let _dot = info_badge(None, true, tokens.accent, &tokens);
        let _num = info_badge(Some(5), false, tokens.accent, &tokens);
        let _overflow = info_badge(Some(120), false, tokens.accent, &tokens);
    }

    #[test]
    fn test_info_bar_renders() {
        let tokens = test_tokens();
        let _bar = info_bar(
            InfoBarSeverity::Success,
            "Connected",
            "Network active",
            Some(Box::new(|| {})),
            &tokens,
        );
    }

    #[test]
    fn test_pips_pager_renders() {
        let tokens = test_tokens();
        let _pager = pips_pager(5, 2, |_| {}, &tokens);
    }

    #[test]
    fn test_expander_renders() {
        let tokens = test_tokens();
        let content = Box::new(Rectangle::new().width(100.0).height(20.0));
        let _exp_closed = expander(
            "󰂯",
            "Bluetooth",
            Some("Enabled"),
            false,
            None,
            content,
            || {},
            &tokens,
        );
    }

    #[test]
    fn test_acrylic_card_renders() {
        let tokens = test_tokens();
        let content = Box::new(Text::new("Test").size(12.0).color(tokens.text_primary));
        let _card = acrylic_card(200.0, 100.0, content, &tokens);
    }

    #[test]
    fn test_split_button_renders() {
        let tokens = test_tokens();
        let _split = split_button("Wi-Fi", Some("󰤨"), || {}, || {}, &tokens);
    }

    #[test]
    fn test_ios_widget_frame_renders() {
        let tokens = test_tokens();
        let content = Rectangle::new().width(50.0).height(50.0);
        let _frame = ios_widget_frame(156.0, 156.0, radius::IOS_WIDGET_SM, content, &tokens);
    }

    #[test]
    fn test_ios_widget_small_renders() {
        let tokens = test_tokens();
        let _sm = ios_widget_small(
            "CPU",
            "󰍛",
            tokens.accent,
            "24%",
            Some("4.8 GHz"),
            156.0,
            156.0,
            &tokens,
        );
    }

    #[test]
    fn test_ios_widget_medium_renders() {
        let tokens = test_tokens();
        let body = Text::new("Song Title")
            .size(12.0)
            .color(tokens.text_primary);
        let trailing = Box::new(Text::new("󰐊").size(14.0).color(tokens.accent));
        let _md = ios_widget_medium(
            "Now Playing",
            "󰎆",
            body,
            Some(trailing),
            328.0,
            156.0,
            &tokens,
        );
    }

    #[test]
    fn test_ios_control_toggle_renders() {
        let tokens = test_tokens();
        let _toggle_on = ios_control_toggle("󰤨", "Wi-Fi", true, 72.0, 72.0, &tokens, || {});
        let _toggle_off = ios_control_toggle("󰤭", "Wi-Fi", false, 72.0, 72.0, &tokens, || {});
    }

    #[test]
    fn test_ios_pill_slider_renders() {
        let tokens = test_tokens();
        let _slider = ios_pill_slider(
            0.65,
            328.0,
            48.0,
            "󰕾",
            "Volume",
            false,
            &tokens,
            |_| {},
            Some(|| {}),
        );
    }

    #[test]
    fn test_ios_dynamic_capsule_renders() {
        let tokens = test_tokens();
        let _capsule = ios_dynamic_capsule(
            "󰑋",
            "Recording Screen",
            Some("00:42"),
            tokens.danger,
            &tokens,
        );
    }
}
