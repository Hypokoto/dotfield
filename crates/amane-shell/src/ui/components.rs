use amane::{
    Align, Center, Color, Justify, Padding, Parent, Pointer, Rectangle, Row, Text, Widget,
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
                .font(&tokens.font_family),
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
        .child(header_row)
}

/// Standardized keyboard shortcut keycap badge
pub fn keycap(key_text: &str, tokens: &ThemeTokens) -> Rectangle {
    let cap_width = (key_text.len() as f32 * 8.0 + 12.0).max(22.0);
    Rectangle::new()
        .width(cap_width)
        .height(18.0)
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
                .font(&tokens.font_family),
        )
}

/// Status badge pill with dot indicator
pub fn status_badge(label: &str, dot_color: Color, tokens: &ThemeTokens) -> Rectangle {
    let badge_width = (label.len() as f32 * 7.5 + 24.0).max(64.0);
    let content = Row::new(vec![
        Box::new(
            Text::new("●")
                .size(7.0)
                .color(dot_color)
                .font(&tokens.font_family),
        ),
        Box::new(
            Text::new(label)
                .size(typography::BADGE)
                .color(tokens.text_secondary)
                .font(&tokens.font_family),
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
