//! Legacy Launcher Compatibility Bridge.
//! The legacy floating modal launcher has been retired in favor of the improved
//! Windows 11 Fluent Start Menu & Desktop Launcher (`ui::start_menu`).
//! All calls to `launcher::toggle()` and `launcher::close()` seamlessly delegate
//! to `ui::start_menu`.

use std::time::{Duration, Instant};

use amane::{Layer, LayerWindow, OutputTarget, Service};

pub use crate::ui::start_menu::{ShellAction, execute_shell_action, get_shell_actions};

pub struct LauncherService {
    pub open: bool,
    pub open_time: Option<Instant>,
    pub query: String,
    pub selected: usize,
    pub hover_index: Option<usize>,
    pub scroll_row: usize,
}

impl Service for LauncherService {
    fn new() -> Self {
        Self {
            open: false,
            open_time: None,
            query: String::new(),
            selected: 0,
            hover_index: None,
            scroll_row: 0,
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub fn toggle() {
    crate::ui::start_menu::toggle();
}

pub fn close() {
    crate::ui::start_menu::close();
    let mut launcher = LauncherService::write();
    launcher.open = false;
    launcher.open_time = None;
    launcher.query.clear();
    launcher.selected = 0;
    launcher.scroll_row = 0;
    launcher.hover_index = None;
}

/// Returns a hidden null overlay window for backwards compatibility if referenced by runner.
pub fn launcher_overlay() -> LayerWindow {
    LayerWindow::new()
        .width(1.0)
        .height(1.0)
        .visible(false)
        .layer(Layer::Overlay)
        .target_output(OutputTarget::Active)
        .namespace("dotfield-shell-launcher-retired")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_service_default() {
        let launcher = LauncherService::read();
        assert!(!launcher.open);
        assert_eq!(launcher.selected, 0);
        assert_eq!(launcher.scroll_row, 0);
        assert!(launcher.query.is_empty());
    }

    #[test]
    fn test_launcher_toggle_and_close() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        close();
        toggle();
        assert!(crate::ui::start_menu::StartMenuService::read().open);
        close();
        assert!(!crate::ui::start_menu::StartMenuService::read().open);
    }
}
