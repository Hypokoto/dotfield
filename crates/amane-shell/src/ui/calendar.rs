use chrono::{Datelike, Local, NaiveDate};
use std::time::Duration;

use amane::{
    Align, Center, Color, Column, Horizontal, Justify, Key, Keyboard, Layer, LayerWindow, Margin,
    OutputTarget, Padding, Parent, Pointer, Rectangle, Row, Service, Text, Vertical, Widget, Zone,
};

use crate::config::ConfigService;

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
    let mut cal = CalendarService::write();
    cal.open = !cal.open;
    if cal.open {
        let now = Local::now();
        cal.view_year = now.year();
        cal.view_month = now.month();
        cal.selected_day = Some(now.day());
    }
}

pub fn open() {
    let mut cal = CalendarService::write();
    cal.open = true;
    let now = Local::now();
    cal.view_year = now.year();
    cal.view_month = now.month();
    cal.selected_day = Some(now.day());
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

    let is_visible = cal.open;
    if !is_visible {
        return LayerWindow::new()
            .width(1.0)
            .height(1.0)
            .target_output(OutputTarget::Active)
            .visible(false)
            .namespace("dotfield-shell-calendar");
    }

    let bg_color = Color::from(manifest.palette.surface.as_str()); // Obsidian #121216
    let overlay_color = Color::from(manifest.palette.overlay.as_str()); // Graphite #1C1C23
    let amber_color = Color::from(manifest.palette.highlight.as_str()); // Amber #FFB000
    let hover_color = Color::from(manifest.palette.hover.as_str()); // #FFD166
    let fg_color = Color::from(manifest.palette.foreground.as_str());
    let border_color = Color::from(manifest.palette.border.as_str()); // Iron #34343D
    let steel_color = Color::from(manifest.palette.steel.as_str());
    let font = manifest.theme.font_family.clone();

    let now = Local::now();
    let today_year = now.year();
    let today_month = now.month();
    let today_day = now.day();

    // Digital Clock and Date Summary Header
    let time_str = now.format("%H:%M:%S").to_string();
    let full_date_str = now.format("%A, %B %d, %Y").to_string();

    let header_clock = Rectangle::new()
        .width(Parent)
        .height(58.0)
        .radius(8.0)
        .fill(overlay_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 6.0,
            bottom: 6.0,
            left: 12.0,
            right: 12.0,
        })
        .child(
            Row::new(vec![
                Box::new(
                    Column::new(vec![
                        Box::new(
                            Text::new(time_str)
                                .size(18.0)
                                .color(amber_color)
                                .font(&font),
                        ),
                        Box::new(
                            Text::new(full_date_str)
                                .size(10.0)
                                .color(steel_color)
                                .font(&font),
                        ),
                    ])
                    .gap(2.0),
                ),
                Box::new(
                    Rectangle::new()
                        .width(26.0)
                        .height(26.0)
                        .radius(4.0)
                        .fill(bg_color)
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
                        .radius(4.0)
                        .fill(overlay_color)
                        .border(1.0, border_color)
                        .cursor(Pointer)
                        .align_child(Center, Center)
                        .on_click(|_| {
                            prev_month();
                        })
                        .child(Text::new("❮").size(11.0).color(amber_color).font(&font)),
                ),
                Box::new(
                    Text::new(month_label)
                        .size(12.0)
                        .color(fg_color)
                        .font(&font),
                ),
                Box::new(
                    Rectangle::new()
                        .width(28.0)
                        .height(26.0)
                        .radius(4.0)
                        .fill(overlay_color)
                        .border(1.0, border_color)
                        .cursor(Pointer)
                        .align_child(Center, Center)
                        .on_click(|_| {
                            next_month();
                        })
                        .child(Text::new("❯").size(11.0).color(amber_color).font(&font)),
                ),
            ])
            .gap(8.0)
            .align(Align::Center),
        ),
        Box::new(
            Rectangle::new()
                .width(52.0)
                .height(26.0)
                .radius(4.0)
                .fill(overlay_color)
                .border(1.0, border_color)
                .cursor(Pointer)
                .align_child(Center, Center)
                .on_click(|_| {
                    reset_today();
                })
                .child(Text::new("Today").size(10.0).color(amber_color).font(&font)),
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
            .child(Text::new(label).size(10.0).color(steel_color).font(&font));
        weekday_pills.push(Box::new(pill));
    }
    let weekday_row = Row::new(weekday_pills)
        .width(Parent)
        .justify(Justify::SpaceBetween)
        .align(Align::Center);

    // Calendar days grid: 6 rows x 7 days
    let first_day = NaiveDate::from_ymd_opt(cal.view_year, cal.view_month, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    let start_col = first_day.weekday().num_days_from_monday() as usize; // 0 for Monday

    let (next_y, next_m) = if cal.view_month == 12 {
        (cal.view_year + 1, 1)
    } else {
        (cal.view_year, cal.view_month + 1)
    };
    let next_first = NaiveDate::from_ymd_opt(next_y, next_m, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2026, 2, 1).unwrap());
    let days_in_month = (next_first - first_day).num_days() as u32;

    let (prev_y, prev_m) = if cal.view_month == 1 {
        (cal.view_year - 1, 12)
    } else {
        (cal.view_year, cal.view_month - 1)
    };
    let prev_first = NaiveDate::from_ymd_opt(prev_y, prev_m, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
    let days_in_prev_month = (first_day - prev_first).num_days() as u32;

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

            let cell_fill = if is_today {
                amber_color
            } else if is_selected {
                overlay_color
            } else {
                Color::from("#00000000") // Transparent
            };

            let cell_border = if is_selected && !is_today {
                amber_color
            } else {
                Color::from("#00000000")
            };

            let text_color = if is_today {
                Color::BLACK
            } else if is_selected {
                hover_color
            } else if is_current_month {
                fg_color
            } else {
                Color::from("#484852") // Muted inactive day
            };

            let cell = Rectangle::new()
                .width(38.0)
                .height(28.0)
                .radius(5.0)
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
                        .font(&font),
                );

            row_cells.push(Box::new(cell));
        }

        let week_row = Row::new(row_cells)
            .width(Parent)
            .justify(Justify::SpaceBetween)
            .align(Align::Center);
        week_rows.push(Box::new(week_row));
    }

    let calendar_panel = Rectangle::new()
        .width(320.0)
        .height(350.0)
        .radius(12.0)
        .fill(bg_color)
        .border(1.0, border_color)
        .padding(Padding {
            top: 12.0,
            bottom: 12.0,
            left: 12.0,
            right: 12.0,
        })
        .child(
            Column::new(vec![
                Box::new(header_clock),
                Box::new(nav_row),
                Box::new(weekday_row),
                Box::new(Column::new(week_rows).width(Parent).gap(3.0)),
            ])
            .width(Parent)
            .gap(8.0),
        );

    LayerWindow::new()
        .width(320.0)
        .height(350.0)
        .target_output(OutputTarget::Active)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 38,
            right: 12,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Top)
        .space(Zone::Ignore)
        .keyboard(if is_visible {
            Keyboard::OnDemand
        } else {
            Keyboard::None
        })
        .visible(is_visible)
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
}
