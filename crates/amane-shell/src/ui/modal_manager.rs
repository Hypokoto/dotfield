//! Modal Manager for Amane desktop shell.
//! Provides centralized, mutual-exclusion modal state management,
//! eliminating N*(N-1) cross-modal close cascades.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    Launcher,
    ControlCenter,
    Calendar,
    DisplayConfig,
    Powermenu,
    ScreenRecord,
    StartMenu,
    Overview,
}

pub struct ModalManager;

impl ModalManager {
    /// Opens the specified modal exclusively, closing any other active modal.
    pub fn open_exclusive(target: ModalType) {
        if target != ModalType::StartMenu && target != ModalType::Launcher {
            crate::ui::start_menu::close();
        }
        if target != ModalType::ControlCenter {
            crate::ui::control_center::close();
        }
        if target != ModalType::Calendar {
            crate::ui::calendar::close();
        }
        if target != ModalType::DisplayConfig {
            crate::ui::display_config::close();
        }
        if target != ModalType::Powermenu {
            crate::ui::powermenu::close();
        }
        if target != ModalType::ScreenRecord {
            crate::ui::screen_record::close();
        }
        if target != ModalType::Overview {
            crate::ui::overview::close();
        }

        #[cfg(target_os = "linux")]
        unsafe {
            libc::malloc_trim(0);
        }
    }

    /// Closes all open modals and reclaims heap memory back to OS.
    pub fn close_all() {
        crate::ui::start_menu::close();
        crate::ui::control_center::close();
        crate::ui::calendar::close();
        crate::ui::display_config::close();
        crate::ui::powermenu::close();
        crate::ui::screen_record::close();
        crate::ui::overview::close();

        #[cfg(target_os = "linux")]
        unsafe {
            libc::malloc_trim(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_manager_close_all() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        ModalManager::close_all();
    }

    #[test]
    fn test_modal_manager_open_exclusive() {
        let _lock = crate::ui::TEST_UI_MUTEX.lock().unwrap();
        ModalManager::open_exclusive(ModalType::Launcher);
        ModalManager::close_all();
    }
}
