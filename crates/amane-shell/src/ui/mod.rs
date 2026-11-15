pub mod animation;
pub mod bar;
pub mod calendar;
pub mod components;
pub mod control_center;
pub mod display_config;
pub mod launcher;
pub mod notifications;
pub mod osd;
pub mod powermenu;
pub mod screen_record;
pub mod terminal_manager;
pub mod tokens;
pub mod tray;
pub mod widgets;

#[cfg(test)]
pub(crate) static TEST_UI_MUTEX: std::sync::LazyLock<std::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(()));
