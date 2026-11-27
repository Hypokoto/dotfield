use chrono::{Datelike, Local, NaiveDate};
use std::time::Duration;

use amane::{
    Align, Center, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Widget, Zone,
};

use crate::config::ConfigService;
use crate::ui::tokens::{ThemeTokens, radius, spacing, typography};

pub struct CalendarService {
    pub open: bool,
    pub view_year: i32,
    pub view_month: u32,
    pub selected_day: Option<u32>,
}

impl Service for CalendarService {
    fn new() -> Self {
        let now = Local::now();
        Self {
            open: false,
            view_year: now.year(),
            view_month: now.month(),
            selected_day: Some(now.day()),
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    let is_open = {
        let mut cal = CalendarService::write();
        cal.open = !cal.open;
        if cal.open {
            let now = Local::now();
            cal.view_year = now.year();
            cal.view_month = now.month();
            cal.selected_day = Some(now.day());
        }
        cal.open
    };
    if is_open {
        crate::ui::modal_manager::ModalManager::open_exclusive(
            crate::ui::modal_manager::ModalType::Calendar,
        );
    }
}

pub fn open() {
    let mut cal = CalendarService::write();
    cal.open = true;
    let now = Local::now();
    cal.view_year = now.year();
    cal.view_month = now.month();
    cal.selected_day = Some(now.day());
    drop(cal);

    crate::ui::modal_manager::ModalManager::open_exclusive(
        crate::ui::modal_manager::ModalType::Calendar,
    );
}

pub fn close() {
    let mut cal = CalendarService::write();
    cal.open = false;
}

pub fn prev_month() {
    let mut cal = CalendarService::write();
    if cal.view_month == 1 {
        cal.view_month = 12;
        cal.view_year -= 1;
    } else {
        cal.view_month -= 1;
    }
    cal.selected_day = None;
}

pub fn next_month() {
    let mut cal = CalendarService::write();
    if cal.view_month == 12 {
        cal.view_month = 1;
        cal.view_year += 1;
    } else {
        cal.view_month += 1;
    }
    cal.selected_day = None;
}

pub fn reset_today() {
    let mut cal = CalendarService::write();
    let now = Local::now();
    cal.view_year = now.year();
    cal.view_month = now.month();
    cal.selected_day = Some(now.day());
}

/// Renders the interactive monthly Calendar drawer overlay.
pub fn calendar_overlay() -> LayerWindow {
    let manifest = ConfigService::read().manifest.clone();
    let cal = CalendarService::read();

    let is_open = cal.open;
    let motion = crate::ui::animation::panel_motion("calendar", is_open);
    if !motion.visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .layer(Layer::Overlay)
            .visible(false)
            .namespace("dotfield-shell-calendar")
            .child(Rectangle::new().width(Parent).height(Parent));
    }

    let tokens = ThemeTokens::from_manifest(&manifest);

    let now = Local::now();
    let today_year = now.year();
    let today_month = now.month();
    let today_day = now.day();

    // Digital Clock and Date Summary Header with Horology Pocket Watch aesthetic
    let time_str = now.format("%H:%M:%S").to_string();
    let full_date_str = now.format("%A, %B %d, %Y").to_string();

    let header_clock = Rectangle::new()
        .width(Parent)
        .height(58.0)
        .radius(radius::MD)
        .fill(tokens.surface_elevated)
        .border(1.0, tokens.border_subtle)
        .padding(Padding {
            top: spacing::PAD_XS,
            bottom: spacing::PAD_XS,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        })
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new(time_str)
                                .size(typography::DISPLAY)
                                .color(tokens.accent)
                                .font(&tokens.font_mono),
                        ),
                        Box::new(
                            Text::new(full_date_str)
                                .size(typography::BADGE)
                                .color(tokens.text_primary)
                                .font(&tokens.font_sans),
                        ),
                    ])
                    .gap(2.0),
                ),
                Box::new(crate::ui::components::close_button(close, &tokens)),
            ])
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center),
        );

    // Month navigation bar
    let month_name = match cal.view_month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    };
    let month_label = format!("{} {}", month_name, cal.view_year);

    let nav_row = Row::new(vec![
        Box::new(
            Row::new(vec![
                Box::new(
                    Rectangle::new()
                        .width(28.0)
                        .height(26.0)
                        .radius(radius::PILL)
                        .fill(tokens.surface_elevated)
                        .border(1.0, tokens.border_subtle)
                        .cursor(Pointer)
                        .align_child(Center, Center)
                        .on_click(|_| {
                            prev_month();
                        })
                        .child(
                            Text::new("󰅁")
                                .size(13.0)
                                .color(tokens.text_primary)
                                .font(&tokens.font_mono),
                        ),
                ),
                Box::new(
                    Text::new(month_label)
                        .size(typography::BODY)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
                Box::new(
                    Rectangle::new()
                        .width(28.0)
                        .height(26.0)
                        .radius(radius::PILL)
                        .fill(tokens.surface_elevated)
                        .border(1.0, tokens.border_subtle)
                        .cursor(Pointer)
                        .align_child(Center, Center)
                        .on_click(|_| {
                            next_month();
                        })
                        .child(
                            Text::new("󰅂")
                                .size(13.0)
                                .color(tokens.text_primary)
                                .font(&tokens.font_mono),
                        ),
                ),
            ])
            .gap(spacing::GAP_MD)
            .align(Align::Center),
        ),
        Box::new(
            Rectangle::new()
                .width(58.0)
                .height(26.0)
                .radius(radius::PILL)
                .fill(tokens.accent_tint)
                .border(1.0, tokens.border_focus)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| {
                    reset_today();
                })
                .child(
                    Text::new("Today")
                        .size(typography::BADGE)
                        .color(tokens.text_primary)
                        .font(&tokens.font_sans),
                ),
        ),
    ])
    .width(Parent)
    .justify(Justify::SpaceBetween)
    .align(Align::Center);

    // Weekday column headers
    let weekday_labels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let mut weekday_pills: Vec<Box<dyn Widget>> = Vec::new();
    for label in weekday_labels {
        let pill = Rectangle::new()
            .width(38.0)
            .height(20.0)
            .align_child(Center, Center)
            .child(
                Text::new(label)
                    .size(typography::BADGE)
                    .color(tokens.text_secondary)
                    .font(&tokens.font_sans),
            );
        weekday_pills.push(Box::new(pill));
    }
    let weekday_row = Row::new(weekday_pills)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    // Calendar days grid: 6 rows x 7 days
    let (first_day, days_in_month, days_in_prev_month) =
        calculate_calendar_month_metrics(cal.view_year, cal.view_month);
    let start_col = first_day.weekday().num_days_from_monday() as usize; // 0 for Monday

    let mut grid_cells: Vec<(u32, bool, bool)> = Vec::new(); // (day_number, is_current_month, is_today)

    // Previous month filler days
    for i in (0..start_col).rev() {
        let day_num = days_in_prev_month - i as u32;
        grid_cells.push((day_num, false, false));
    }

    // Current month days
    for day in 1..=days_in_month {
        let is_today =
            cal.view_year == today_year && cal.view_month == today_month && day == today_day;
        grid_cells.push((day, true, is_today));
    }

    // Next month filler days to complete 35 or 42 cells (5 or 6 weeks)
    let total_cells = if grid_cells.len() > 35 { 42 } else { 35 };
    let mut next_day = 1;
    while grid_cells.len() < total_cells {
        grid_cells.push((next_day, false, false));
        next_day += 1;
    }

    // Build the 5 or 6 week rows
    let mut week_rows: Vec<Box<dyn Widget>> = Vec::new();
    for chunk in grid_cells.chunks(7) {
        let mut row_cells: Vec<Box<dyn Widget>> = Vec::new();

        for &(day_num, is_current_month, is_today) in chunk {
            let is_selected = is_current_month && cal.selected_day == Some(day_num);

            let (cell_fill, cell_border, text_color) = if is_today {
                (tokens.accent, tokens.accent, tokens.text_on_accent)
            } else if is_selected {
                (
                    tokens.surface_active,
                    tokens.border_focus,
                    tokens.text_primary,
                )
            } else {
                (
                    Color::TRANSPARENT,
                    Color::TRANSPARENT,
                    if is_current_month {
                        tokens.text_primary
                    } else {
                        tokens.text_muted
                    },
                )
            };

            let cell = Rectangle::new()
                .width(38.0)
                .height(28.0)
                .radius(radius::PILL)
                .fill(cell_fill)
                .border(1.0, cell_border)
                .cursor(if is_current_month {
                    Pointer
                } else {
                    amane::Cursor::Default
                })
                .align_child(Center, Center)
                .on_click(move |_| {
                    if is_current_month {
                        let mut cal = CalendarService::write();
                        cal.selected_day = Some(day_num);
                    }
                })
                .child(
                    Text::new(format!("{day_num}"))
                        .size(10.5)
                        .color(text_color)
                        .font(&tokens.font_mono),
                );

            row_cells.push(Box::new(cell));
        }

        let week_row = Row::new(row_cells)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);
        week_rows.push(Box::new(week_row));
    }

    let calendar_content = Column::new(vec![
        Box::new(header_clock),
        Box::new(nav_row),
        Box::new(weekday_row),
        Box::new(Column::new(week_rows).width(Parent).gap(3.0)),
    ])
    .width(Parent)
    .gap(spacing::GAP_MD);

    let is_top_bar = true;
    let calendar_panel = crate::ui::components::bar_connected_panel(
        330.0,
        360.0,
        is_top_bar,
        motion,
        Padding {
            top: spacing::PAD_MD,
            bottom: spacing::PAD_MD,
            left: spacing::PAD_MD,
            right: spacing::PAD_MD,
        },
        tokens.border_subtle,
        calendar_content,
        &tokens,
    );

    let (win_w, win_h) = crate::ui::components::bar_connected_window_size(330.0, 360.0);
    let top_margin = crate::ui::bar::top_bar_clearance(8.0);

    LayerWindow::new()
        .width(win_w)
        .height(win_h)
        .target_output(OutputTarget::Active)
        .anchor_vertical(amane::Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: top_margin,
            right: 12,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .keyboard(if is_open {
            Keyboard::OnDemand
        } else {
            Keyboard::None
        })
        .input_region(crate::ui::components::bar_connected_input_region(
            330.0, 360.0, is_top_bar,
        ))
        .visible(motion.visible)
        .namespace("dotfield-shell-calendar")
        .on_key(|key| match key {
            Key::Escape | Key::Character('q') | Key::Character('Q') => {
                close();
            }
            Key::Left => {
                prev_month();
            }
            Key::Right => {
                next_month();
            }
            Key::Character('t') | Key::Character('T') => {
                reset_today();
            }
            _ => {}
        })
        .child(calendar_panel)
}
/// Pure function computing first day NaiveDate, days in current month, and days in previous month.
pub fn calculate_calendar_month_metrics(year: i32, month: u32) -> (NaiveDate, u32, u32) {
    let month = month.clamp(1, 12);
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_else(|| {
        let today = Local::now().date_naive();
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    });

    let (next_y, next_m) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let days_in_month = NaiveDate::from_ymd_opt(next_y, next_m, 1)
        .map(|next| (next - first_day).num_days() as u32)
        .unwrap_or(30);

    let (prev_y, prev_m) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    let days_in_prev_month = NaiveDate::from_ymd_opt(prev_y, prev_m, 1)
        .map(|prev| (first_day - prev).num_days() as u32)
        .unwrap_or(30);

    (first_day, days_in_month, days_in_prev_month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_service_defaults() {
        let cal = CalendarService::new();
        assert!(!cal.open);
        assert!(cal.view_year >= 2026);
        assert!(cal.view_month >= 1 && cal.view_month <= 12);
        assert!(cal.selected_day.is_some());
    }

    #[test]
    fn test_calendar_navigation() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        CalendarService::write().open = false;
        toggle();
        assert!(CalendarService::read().open);

        let initial_month = CalendarService::read().view_month;
        next_month();
        let next_m = CalendarService::read().view_month;
        if initial_month == 12 {
            assert_eq!(next_m, 1);
        } else {
            assert_eq!(next_m, initial_month + 1);
        }

        prev_month();
        assert_eq!(CalendarService::read().view_month, initial_month);

        close();
        assert!(!CalendarService::read().open);
    }

    #[test]
    fn test_calculate_calendar_month_metrics() {
        // Leap year February (2024 has 29 days, January has 31)
        let (first, days, prev_days) = calculate_calendar_month_metrics(2024, 2);
        assert_eq!(first, NaiveDate::from_ymd_opt(2024, 2, 1).unwrap());
        assert_eq!(days, 29);
        assert_eq!(prev_days, 31);

        // Non-leap year February (2025 has 28 days)
        let (first, days, _) = calculate_calendar_month_metrics(2025, 2);
        assert_eq!(first, NaiveDate::from_ymd_opt(2025, 2, 1).unwrap());
        assert_eq!(days, 28);

        // Year boundary transition: January 2027 (previous month December 2026 has 31 days)
        let (first, days, prev_days) = calculate_calendar_month_metrics(2027, 1);
        assert_eq!(first, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
        assert_eq!(days, 31);
        assert_eq!(prev_days, 31);
    }
}
