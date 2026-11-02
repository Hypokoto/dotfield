use std::collections::HashMap;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

use amane::Service;
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Event, Request, Response, Window};
use parking_lot::RwLock;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NiriFocusedWindow {
    pub id: u64,
    pub title: String,
    pub app_id: String,
}

static FOCUSED_WINDOW: LazyLock<RwLock<NiriFocusedWindow>> =
    LazyLock::new(|| RwLock::new(NiriFocusedWindow::default()));

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NiriWindowService {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub window_count: usize,
}

impl Service for NiriWindowService {
    fn new() -> Self {
        Self::default()
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub struct NiriService;

impl NiriService {
    /// Starts the background Niri event listener.
    pub fn start() {
        if !Self::is_available() {
            return;
        }

        thread::spawn(|| {
            loop {
                if let Err(_e) = Self::run_event_loop() {
                    // Backoff before reconnecting
                    thread::sleep(Duration::from_millis(1000));
                }
            }
        });
    }

    /// Checks if running inside an active Niri compositor session.
    pub fn is_available() -> bool {
        std::env::var_os("NIRI_SOCKET").is_some()
    }

    /// Returns the currently focused window info.
    pub fn focused_window() -> NiriFocusedWindow {
        FOCUSED_WINDOW.read().clone()
    }

    /// Sends an Action request to Niri over IPC.
    pub fn perform_action(action: Action) -> anyhow::Result<()> {
        let mut socket = Socket::connect()?;
        let reply = socket.send(Request::Action(action))?;
        match reply {
            Ok(Response::Handled) => Ok(()),
            Ok(other) => anyhow::bail!("Unexpected response from niri: {other:?}"),
            Err(err) => anyhow::bail!("Niri error: {err}"),
        }
    }

    /// Gracefully exits Niri compositor.
    pub fn quit(skip_confirmation: bool) -> anyhow::Result<()> {
        Self::perform_action(Action::Quit { skip_confirmation })
    }

    /// Powers off monitors via DPMS.
    pub fn power_off_monitors() -> anyhow::Result<()> {
        Self::perform_action(Action::PowerOffMonitors {})
    }

    /// Powers on monitors via DPMS.
    pub fn power_on_monitors() -> anyhow::Result<()> {
        Self::perform_action(Action::PowerOnMonitors {})
    }

    /// Closes the currently focused window.
    pub fn close_focused_window() -> anyhow::Result<()> {
        Self::perform_action(Action::CloseWindow { id: None })
    }

    fn update_active_window(id: u64, title: &str, app_id: &str, count: usize) {
        *FOCUSED_WINDOW.write() = NiriFocusedWindow {
            id,
            title: title.to_string(),
            app_id: app_id.to_string(),
        };

        let mut svc = NiriWindowService::write();
        svc.id = id;
        svc.title = title.to_string();
        svc.app_id = app_id.to_string();
        svc.window_count = count;
    }

    fn run_event_loop() -> std::io::Result<()> {
        let mut socket = Socket::connect()?;

        let mut windows: HashMap<u64, Window> = HashMap::new();

        // 1. Seed initial focused window
        if let Ok(Ok(Response::FocusedWindow(Some(win)))) = socket.send(Request::FocusedWindow) {
            let title = win.title.clone().unwrap_or_default();
            let app_id = win.app_id.clone().unwrap_or_default();
            Self::update_active_window(win.id, &title, &app_id, 1);
            windows.insert(win.id, win);
        }

        // 2. Seed initial window list
        if let Ok(Ok(Response::Windows(wins))) = socket.send(Request::Windows) {
            let total = wins.len();
            for win in wins {
                if win.is_focused {
                    let title = win.title.clone().unwrap_or_default();
                    let app_id = win.app_id.clone().unwrap_or_default();
                    Self::update_active_window(win.id, &title, &app_id, total);
                }
                windows.insert(win.id, win);
            }
        }

        // 3. Subscribe to event stream
        let reply = socket.send(Request::EventStream)?;
        if !matches!(reply, Ok(Response::Handled)) {
            return Err(std::io::Error::other(
                "Failed to subscribe to niri event stream",
            ));
        }

        let mut read_event = socket.read_events();
        while let Ok(event) = read_event() {
            match event {
                Event::WindowsChanged {
                    windows: new_windows,
                } => {
                    windows.clear();
                    let total = new_windows.len();
                    let mut found_focused = false;
                    for win in new_windows {
                        if win.is_focused {
                            let title = win.title.clone().unwrap_or_default();
                            let app_id = win.app_id.clone().unwrap_or_default();
                            Self::update_active_window(win.id, &title, &app_id, total);
                            found_focused = true;
                        }
                        windows.insert(win.id, win);
                    }
                    if !found_focused && windows.is_empty() {
                        Self::update_active_window(0, "", "", 0);
                    }
                }
                Event::WindowOpenedOrChanged { window } => {
                    let id = window.id;
                    let is_focused = window.is_focused;
                    let title = window.title.clone().unwrap_or_default();
                    let app_id = window.app_id.clone().unwrap_or_default();
                    windows.insert(id, window);
                    let total = windows.len();

                    if is_focused || FOCUSED_WINDOW.read().id == id {
                        Self::update_active_window(id, &title, &app_id, total);
                    }
                }
                Event::WindowClosed { id } => {
                    windows.remove(&id);
                    let total = windows.len();
                    if FOCUSED_WINDOW.read().id == id {
                        Self::update_active_window(0, "", "", total);
                    } else {
                        NiriWindowService::write().window_count = total;
                    }
                }
                Event::WindowFocusChanged { id } => {
                    let total = windows.len();
                    match id {
                        Some(win_id) => {
                            if let Some(win) = windows.get(&win_id) {
                                let title = win.title.clone().unwrap_or_default();
                                let app_id = win.app_id.clone().unwrap_or_default();
                                Self::update_active_window(win.id, &title, &app_id, total);
                            } else {
                                Self::update_active_window(win_id, "", "", total);
                            }
                        }
                        None => {
                            Self::update_active_window(0, "", "", total);
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_niri_focused_window_default() {
        let default_win = NiriFocusedWindow::default();
        assert_eq!(default_win.id, 0);
        assert!(default_win.title.is_empty());
        assert!(default_win.app_id.is_empty());
    }

    #[test]
    fn test_niri_window_service_default() {
        let ws = NiriWindowService::default();
        assert_eq!(ws.id, 0);
        assert_eq!(ws.window_count, 0);
        assert!(ws.title.is_empty());
        assert!(ws.app_id.is_empty());
    }

    #[test]
    fn test_niri_service_availability() {
        let available = NiriService::is_available();
        let env_set = std::env::var_os("NIRI_SOCKET").is_some();
        assert_eq!(available, env_set);
    }

    #[test]
    fn test_niri_update_active_window() {
        NiriService::update_active_window(42, "Foot Terminal", "foot", 3);
        let focused = NiriService::focused_window();
        assert_eq!(focused.id, 42);
        assert_eq!(focused.title, "Foot Terminal");
        assert_eq!(focused.app_id, "foot");

        // Clean up
        NiriService::update_active_window(0, "", "", 0);
        let reset = NiriService::focused_window();
        assert_eq!(reset.id, 0);
        assert!(reset.title.is_empty());
    }

    #[test]
    fn test_niri_perform_action_offline_errors_cleanly() {
        // Never send destructive window actions to a live compositor during unit tests
        if !NiriService::is_available() {
            let result = NiriService::close_focused_window();
            assert!(result.is_err());
        }
    }
}
