//! Legacy Launcher Compatibility Bridge.
//! The legacy floating modal launcher has been retired in favor of the improved
//! Windows 11 Fluent Start Menu & Desktop Launcher (`ui::start_menu`).
//! All calls to `launcher::toggle()` and `launcher::close()` seamlessly delegate
//! to `ui::start_menu`.

pub use crate::ui::start_menu::{ShellAction, execute_shell_action, get_shell_actions};

pub fn toggle() {
    crate::ui::start_menu::toggle();
}

pub fn close() {
    crate::ui::start_menu::close();
}

#[cfg(test)]
mod tests {
    use super::*;
    use amane::Service;

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
