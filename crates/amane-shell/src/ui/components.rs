use amane::{
    Align, Center, Color, Column, InputArea, Justify, Padding, Parent, Pointer, Rectangle, Row,
    Scroll, Stack, Text, Weight, Widget,
};

use crate::ui::animation::PanelMotion;
use crate::ui::tokens::{ThemeTokens, border, radius, spacing, typography};

pub const CONNECTED_PANEL_FILLET_PAD: f32 = 0.0;
pub const CONNECTED_PANEL_OUTER_PAD: f32 = 0.0;
pub const CONNECTED_PANEL_BAR_OVERLAP: f32 = 0.0;

/// Returns total LayerWindow dimensions `(width, height)` required for a popout panel.
#[inline]
pub fn bar_connected_window_size(body_width: f32, body_height: f32) -> (f32, f32) {
    (body_width, body_height)
}

/// Returns the interactive input rectangle for a popout panel.
#[inline]
pub fn bar_connected_input_region(
    body_width: f32,
    body_height: f32,
    _is_top_bar: bool,
) -> Vec<InputArea> {
    vec![InputArea {
        x: 0,
        y: 0,
        width: body_width.round() as i32,
        height: body_height.round() as i32,
    }]
}

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
                .font(&tokens.font_mono)
                .tight(),
        ),
        Box::new(
            Text::new(title)
                .size(typography::TITLE_SM)
                .weight(Weight::SemiBold)
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
        .fill(tokens.surface_track)
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
                .weight(Weight::Medium)
                .color(tokens.text_secondary)
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
        .radius(radius::IOS_WIDGET_LG)
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

/// Floating island popout panel with smooth rounded corners, full enclosed border, and spring slide motion.
/// Slides smoothly into place from the island/dock and parks cleanly at 0 Hz when settled.
#[allow(clippy::too_many_arguments)]
pub fn bar_connected_panel(
    body_width: f32,
    body_height: f32,
    is_top_bar: bool,
    motion: PanelMotion,
    padding: Padding,
    border_color: Color,
    content: impl Widget + 'static,
    tokens: &ThemeTokens,
) -> Rectangle {
    let slide_travel = 16.0;
    let slide_px = (1.0 - motion.progress) * slide_travel;
    let signed_slide = if is_top_bar { -slide_px } else { slide_px };

    let mut panel = Rectangle::new()
        .width(body_width)
        .height(body_height)
        .radius(radius::IOS_WIDGET_LG)
        .fill(tokens.surface)
        .border(1.0, border_color)
        .padding(padding)
        .child(content);

    if motion.animating || motion.progress < 1.0 {
        panel = panel.translate(0.0, signed_slide).opacity(motion.alpha);
    }

    panel
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
    // Outer container with squircle radius and glass specular outline
    Rectangle::new()
        .width(width)
        .height(height)
        .radius(corner_radius)
        .fill(tokens.surface_elevated)
        .border(border::THIN, tokens.border_specular)
        .padding(Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(content)
}

/// iOS System Small Bento Widget (~1×1 aspect, ~156×156px or custom dimensions).
/// Displays leading circular glyph badge, title, primary metric, and secondary footer without text collision.
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
        .width(24.0)
        .height(24.0)
        .radius(radius::PILL)
        .fill(accent_color)
        .align_child(Center, Center)
        .child(
            Text::new(icon)
                .size(13.0)
                .color(tokens.text_on_accent)
                .font(&tokens.font_mono)
                .tight(),
        );

    let header_row = Row::new(vec![
        Box::new(icon_badge),
        Box::new(
            Text::new(title)
                .size(typography::BADGE)
                .weight(Weight::Medium)
                .color(tokens.text_secondary)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(6.0)
    .align(Align::Center);

    let mut body_items: Vec<Box<dyn Widget>> = vec![
        Box::new(header_row),
        Box::new(
            Text::new(metric_text)
                .size(19.0)
                .weight(Weight::Bold)
                .color(tokens.text_primary)
                .font(&tokens.font_mono),
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

    let content = Column::new(body_items).gap(3.0).align(Align::Start);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius::IOS_WIDGET_SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 8.0,
            bottom: 8.0,
            left: 10.0,
            right: 10.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(content)
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
    let (circle_bg, icon_color, border_color, text_color) = if is_active {
        (
            tokens.accent,
            tokens.text_on_accent,
            tokens.border_focus,
            tokens.text_primary,
        )
    } else {
        (
            tokens.surface_track,
            tokens.text_secondary,
            tokens.border_subtle,
            tokens.text_secondary,
        )
    };

    let glyph_circle = Rectangle::new()
        .width(36.0)
        .height(36.0)
        .radius(radius::PILL)
        .fill(circle_bg)
        .align_child(Center, Center)
        .child(
            Text::new(icon)
                .size(17.0)
                .color(icon_color)
                .font(&tokens.font_mono)
                .tight(),
        );

    let content = Column::new(vec![
        Box::new(glyph_circle),
        Box::new(
            Text::new(label)
                .size(11.0)
                .weight(if is_active {
                    Weight::SemiBold
                } else {
                    Weight::Regular
                })
                .color(text_color)
                .font(&tokens.font_sans)
                .tight(),
        ),
    ])
    .gap(5.0)
    .align(Align::Center);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius::IOS_TOGGLE)
        .fill(if is_active {
            tokens.surface_active
        } else {
            tokens.surface_elevated
        })
        .border(1.0, border_color)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_toggle())
        .child(content)
}

/// iOS Bento Split Connectivity Widget (~172×68px):
/// Left side: illuminated 38×38 circular toggle button (toggles Wi-Fi or Bluetooth).
/// Right side: Title + Network/Device status + chevron, tapping opens detail sheet.
#[allow(clippy::too_many_arguments)]
pub fn ios_connectivity_widget(
    icon: &'static str,
    title: &str,
    status_text: &str,
    is_active: bool,
    width: f32,
    height: f32,
    tokens: &ThemeTokens,
    on_toggle: impl Fn() + Send + Sync + 'static,
    on_details: impl Fn() + Send + Sync + 'static,
) -> Rectangle {
    let (circle_bg, circle_icon_color) = if is_active {
        (tokens.accent, tokens.text_on_accent)
    } else {
        (tokens.surface_track, tokens.text_secondary)
    };

    let toggle_btn = Rectangle::new()
        .width(38.0)
        .height(38.0)
        .radius(radius::PILL)
        .fill(circle_bg)
        .cursor(Pointer)
        .align_child(Center, Center)
        .on_click(move |_| on_toggle())
        .child(
            Text::new(icon)
                .size(18.0)
                .color(circle_icon_color)
                .font(&tokens.font_mono)
                .tight(),
        );

    let details_col = Column::new(vec![
        Box::new(
            Text::new(title)
                .size(typography::BODY)
                .weight(Weight::SemiBold)
                .color(tokens.text_primary)
                .font(&tokens.font_sans),
        ),
        Box::new(
            Text::new(status_text)
                .size(typography::CAPTION)
                .color(if is_active {
                    tokens.text_secondary
                } else {
                    tokens.text_muted
                })
                .font(&tokens.font_sans),
        ),
    ])
    .gap(1.0)
    .align(Align::Start);

    let chevron = Text::new("󰅂")
        .size(12.0)
        .color(tokens.text_muted)
        .font(&tokens.font_mono)
        .tight();

    let details_row = Row::new(vec![Box::new(details_col), Box::new(chevron)])
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    let details_width = width - 38.0 - 8.0 - 22.0;
    let details_btn = Rectangle::new()
        .width(details_width.max(80.0))
        .height(height)
        .cursor(Pointer)
        .align_child(Align::Start, Align::Center)
        .on_click(move |_| on_details())
        .child(details_row);

    let card_content = Row::new(vec![Box::new(toggle_btn), Box::new(details_btn)])
        .width(Parent)
        .gap(8.0)
        .align(Align::Center);

    Rectangle::new()
        .width(width)
        .height(height)
        .radius(radius::IOS_WIDGET_SM)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: 4.0,
            bottom: 4.0,
            left: 10.0,
            right: 12.0,
        })
        .align_child(Align::Start, Align::Center)
        .child(card_content)
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

    let value_str = if is_muted {
        "Muted".to_string()
    } else {
        format!("{:.0}%", clamped_frac * 100.0)
    };

    // 1. Base track: recessed deep-violet pill track
    let bg_rect = Rectangle::new()
        .width(slider_width)
        .height(slider_height)
        .radius(radius::PILL)
        .fill(tokens.surface_track)
        .border(1.0, tokens.border_subtle);

    // 2. Dynamic progress fill: illuminated high-contrast pill
    let fill_rect = Rectangle::new()
        .width(fill_w)
        .height(slider_height)
        .radius(radius::PILL)
        .fill(if is_muted {
            tokens.danger
        } else {
            tokens.accent
        });

    let left_over_fill = clamped_frac >= 0.24;
    let (icon_color, label_color) = if !left_over_fill {
        (tokens.text_primary, tokens.text_primary)
    } else if is_muted {
        (tokens.text_on_danger, tokens.text_on_danger)
    } else {
        (tokens.text_on_accent, tokens.text_on_accent)
    };

    // 3. Embedded glyph icon
    let icon_element: Box<dyn Widget> = if let Some(on_click) = on_icon_click {
        Box::new(
            Rectangle::new()
                .width(28.0)
                .height(28.0)
                .radius(radius::PILL)
                .fill(Color::rgba(0, 0, 0, 42))
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(move |_| on_click())
                .child(
                    Text::new(icon)
                        .size(15.0)
                        .color(icon_color)
                        .font(&tokens.font_mono)
                        .tight(),
                ),
        )
    } else {
        Box::new(
            Text::new(icon)
                .size(15.0)
                .color(icon_color)
                .font(&tokens.font_mono)
                .tight(),
        )
    };

    let label_row = Row::new(vec![
        icon_element,
        Box::new(
            Text::new(label)
                .size(typography::SECONDARY)
                .weight(Weight::SemiBold)
                .color(label_color)
                .font(&tokens.font_sans),
        ),
    ])
    .gap(spacing::GAP_SM)
    .align(Align::Center);

    // Value badge wrapped in a crisp dark translucent pill so contrast stays >10:1
    // across 0%..100% and never splits half-on-fill half-on-track.
    let badge_w = (value_str.chars().count() as f32 * 7.5 + 16.0).max(46.0);
    let value_tag = Rectangle::new()
        .width(badge_w)
        .height(22.0)
        .radius(radius::PILL)
        .fill(Color::rgba(20, 14, 41, 215))
        .border(1.0, tokens.border_subtle)
        .align_child(Center, Center)
        .child(
            Text::new(value_str)
                .size(typography::BADGE)
                .weight(Weight::SemiBold)
                .color(if is_muted {
                    tokens.danger
                } else {
                    tokens.text_primary
                })
                .font(&tokens.font_mono)
                .tight(),
        );

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
            right: 12.0,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ShellManifest;

    fn test_tokens() -> ThemeTokens {
        ThemeTokens::from_manifest(&ShellManifest::default())
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
    fn test_ios_control_toggle_renders() {
        let tokens = test_tokens();
        let _toggle_on = ios_control_toggle("󰤨", "Wi-Fi", true, 72.0, 72.0, &tokens, || {});
        let _toggle_off = ios_control_toggle("󰤭", "Wi-Fi", false, 72.0, 72.0, &tokens, || {});
    }

    #[test]
    fn test_ios_connectivity_widget_renders() {
        let tokens = test_tokens();
        let _conn_on = ios_connectivity_widget(
            "󰤨",
            "Wi-Fi",
            "Connected",
            true,
            172.0,
            68.0,
            &tokens,
            || {},
            || {},
        );
        let _conn_off = ios_connectivity_widget(
            "󰤭",
            "Wi-Fi",
            "Off",
            false,
            172.0,
            68.0,
            &tokens,
            || {},
            || {},
        );
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
}
