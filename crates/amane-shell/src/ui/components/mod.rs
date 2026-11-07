use amane::{Align, Center, Color, Padding, Pointer, Rectangle, Row, Size, Text, Widget};

/// Standard container card for control center, dialogs, and overlays.
pub fn container_card(
    width: impl Into<Size>,
    height: impl Into<Size>,
    bg_color: Color,
    border_color: Color,
    radius: f32,
    padding: Padding,
    child: impl Widget + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius)
        .fill(bg_color)
        .border(1.0, border_color)
        .padding(padding)
        .child(child)
}

/// Standardized section header with title and icon/action.
pub fn section_header(
    title: &str,
    glyph: &str,
    accent_color: Color,
    fg_color: Color,
    font: &str,
) -> Row {
    Row::new(vec![
        Box::new(Text::new(glyph).size(12.0).color(accent_color).font(font)),
        Box::new(Text::new(title).size(10.5).color(fg_color).font(font)),
    ])
    .gap(6.0)
    .align(Align::Center)
}

#[derive(Debug, Clone, Copy)]
pub struct PillStyle<'a> {
    pub is_active: bool,
    pub bg_color: Color,
    pub border_color: Color,
    pub accent_color: Color,
    pub fg_color: Color,
    pub font: &'a str,
}

/// Interactive pill button used across settings and menus.
pub fn pill_button(
    width: f32,
    height: f32,
    label: &str,
    style: PillStyle<'_>,
    on_click: impl Fn(amane::Button) + 'static,
) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(6.0)
        .fill(style.bg_color)
        .border(
            1.0,
            if style.is_active {
                style.accent_color
            } else {
                style.border_color
            },
        )
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(on_click)
        .child(
            Text::new(label)
                .size(10.5)
                .color(if style.is_active {
                    style.accent_color
                } else {
                    style.fg_color
                })
                .font(style.font),
        )
}
