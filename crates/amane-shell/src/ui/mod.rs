pub mod animation;
pub mod bar;
pub mod calendar;
pub mod components;
pub mod control_center;
pub mod display_config;
pub mod dock;
pub mod modal_manager;
pub mod notifications;
pub mod osd;
pub mod overview;
pub mod powermenu;
pub mod screen_record;
pub mod start_menu;
pub mod tokens;
pub mod tray;
pub mod widgets;

#[cfg(test)]
pub(crate) static TEST_UI_MUTEX: std::sync::LazyLock<std::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(()));
