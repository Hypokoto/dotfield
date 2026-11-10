use amane::{
    Align, Center, Color, Column, Justify, Padding, Parent, Pointer, Rectangle, Row, Text, Widget,
};

use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

/// Standardized action button for closing overlays and modals
pub fn close_button(on_click_fn: impl Fn() + 'static, tokens: &ThemeTokens) -> Rectangle {
    Rectangle::new()
        .width(26.0)
        .height(26.0)
        .radius(radius::SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_click_fn())
        .child(
            Text::new("󰅖")
                .size(typography::BODY)
                .color(tokens.text_primary)
                .font(&tokens.font_family)
                .tight(),
        )
}

/// Standardized modal header bar across all popover windows
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
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new(title)
                .size(typography::SECONDARY)
                .color(tokens.text_primary)
                .font(&tokens.font_family),
        ),
    ];

    if let Some(tag_text) = tag {
        title_elements.push(Box::new(
            Text::new(format!("• {tag_text}"))
                .size(typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
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

/// Standardized keyboard shortcut keycap badge
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
                .color(tokens.text_secondary)
                .font(&tokens.font_family)
                .tight(),
        )
}

/// Status badge pill with dot indicator
pub fn status_badge(label: &str, dot_color: Color, tokens: &ThemeTokens) -> Rectangle {
    let char_count = label.chars().count();
    let badge_width = (char_count as f32 * 7.5 + 24.0).max(64.0);
    let content = Row::new(vec![
        Box::new(
            Text::new("●")
                .size(7.0)
                .color(dot_color)
                .font(&tokens.font_family)
                .tight(),
        ),
        Box::new(
            Text::new(label)
                .size(typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family)
                .tight(),
        ),
    ])
    .gap(5.0)
    .align(Align::Center);

    Rectangle::new()
        .width(badge_width)
        .height(20.0)
        .radius(radius::PILL)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
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
        .child(shell_column)
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

    let mut text_elements: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new(title)
            .size(typography::BODY)
            .color(title_color)
            .font(&tokens.font_family),
    )];

    if let Some(sub) = subtitle {
        text_elements.push(Box::new(
            Text::new(sub)
                .size(typography::SECONDARY)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
        ));
    }

    let text_column = Column::new(text_elements)
        .gap(spacing::GAP_XS)
        .align(Align::Start);

    let left_items: Vec<Box<dyn Widget>> = vec![icon, Box::new(text_column)];

    let card_row = Row::new(left_items)
        .gap(spacing::GAP_MD)
        .align(Align::Center);

    let outer_row = if let Some(badge_text) = badge {
        let badge_widget = keycap(badge_text, tokens);
        Row::new(vec![Box::new(card_row), Box::new(badge_widget)])
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
        .border(if selected { 1.5 } else { 1.0 }, border)
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
        .border(1.0, border_color)
        .padding(Padding {
            top: 2.0,
            bottom: 2.0,
            left: spacing::PAD_SM,
            right: spacing::PAD_SM,
        })
        .align_child(Center, Center)
        .child(row)
}
