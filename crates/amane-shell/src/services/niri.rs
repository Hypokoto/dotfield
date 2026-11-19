use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;

use amane::Service;
use arc_swap::ArcSwap;
use niri_ipc::socket::Socket;
use niri_ipc::{
    Action, Event, Output, OutputAction, Request, Response, Window, WorkspaceReferenceArg,
};

use crate::services::topology::ProjectTarget;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NiriFocusedWindow {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub is_full_width: bool,
}

static FOCUSED_WINDOW: LazyLock<ArcSwap<NiriFocusedWindow>> =
    LazyLock::new(|| ArcSwap::from_pointee(NiriFocusedWindow::default()));

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NiriWindowService {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub display_title: String,
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

    /// Returns the currently focused window info via lock-free atomic load.
    pub fn focused_window() -> NiriFocusedWindow {
        (**FOCUSED_WINDOW.load()).clone()
    }

    /// Returns an Arc pointer to the currently focused window info with zero allocations.
    pub fn focused_window_arc() -> Arc<NiriFocusedWindow> {
        FOCUSED_WINDOW.load_full()
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

    /// Focus a specific window ID on Niri's spatial ribbon.
    pub fn focus_window(id: u64) -> anyhow::Result<()> {
        Self::perform_action(Action::FocusWindow { id })
    }

    /// Focus a workspace by name (e.g. "01 DEV").
    pub fn focus_workspace(name: &str) -> anyhow::Result<()> {
        Self::perform_action(Action::FocusWorkspace {
            reference: WorkspaceReferenceArg::Name(name.to_string()),
        })
    }

    /// Query the current list of open windows from Niri IPC.
    pub fn query_windows() -> anyhow::Result<Vec<Window>> {
        let mut socket = Socket::connect()?;
        match socket.send(Request::Windows)? {
            Ok(Response::Windows(wins)) => Ok(wins),
            Ok(other) => anyhow::bail!("Unexpected response querying windows: {other:?}"),
            Err(e) => anyhow::bail!("Niri error: {e}"),
        }
    }

    /// Query connected outputs from Niri IPC.
    pub fn query_outputs() -> anyhow::Result<HashMap<String, Output>> {
        let mut socket = Socket::connect()?;
        match socket.send(Request::Outputs)? {
            Ok(Response::Outputs(outputs)) => Ok(outputs),
            Ok(other) => anyhow::bail!("Unexpected response querying outputs: {other:?}"),
            Err(e) => anyhow::bail!("Niri error: {e}"),
        }
    }

    /// Configure an output directly via Niri IPC without spawning `niri msg`.
    pub fn configure_output(output: String, action: OutputAction) -> anyhow::Result<()> {
        let mut socket = Socket::connect()?;
        match socket.send(Request::Output { output, action })? {
            Ok(Response::Handled) | Ok(Response::OutputConfigChanged(_)) => Ok(()),
            Ok(other) => anyhow::bail!("Unexpected response configuring output: {other:?}"),
            Err(e) => anyhow::bail!("Niri error: {e}"),
        }
    }

    /// Warp to an existing project window on the ribbon or spawn an atomic worktree cell pair.
    pub fn jump_or_spawn_project(target: &ProjectTarget) -> anyhow::Result<()> {
        let expected_app_id = format!("dev.{}", target.slug);

        // 1. Check if a window matching app_id == dev.<slug> already exists
        if let Ok(windows) = Self::query_windows() {
            for win in windows {
                if let Some(app_id) = win.app_id.as_deref()
                    && app_id == expected_app_id
                {
                    return Self::focus_window(win.id);
                }
            }
        }

        // 2. Unmaterialized target: ensure workspace "01 DEV" is active
        let _ = Self::focus_workspace("01 DEV");

        // Small delay to allow workspace focus before window placement
        thread::sleep(Duration::from_millis(50));

        let term = crate::ui::terminal_manager::detect_default_terminal();

        // Spawn primary terminal (2/3 width via window rule) with dev.<slug> app-id
        // attaching to or creating a tmux session with the slug name
        let tmux_args = ["tmux", "new-session", "-A", "-s", &target.slug];
        let _ = crate::ui::terminal_manager::spawn_terminal_window(
            &term,
            &expected_app_id,
            Some(&target.path),
            Some(&tmux_args),
        );

        // Spawn linked companion sidecar (1/3 width via window rule with open-focused false)
        let sidecar_app_id = format!("sidecar-{}", target.slug);
        let _ = crate::ui::terminal_manager::spawn_terminal_window(
            &term,
            &sidecar_app_id,
            Some(&target.path),
            None,
        );

        Ok(())
    }

    /// Inspect current focused window and spawn a companion sidecar if not already present.
    pub fn spawn_sidecar_for_focused() -> anyhow::Result<()> {
        let focused = Self::focused_window();
        let slug = if focused.app_id.starts_with("dev.") {
            focused.app_id.trim_start_matches("dev.").to_string()
        } else if !focused.app_id.is_empty() && focused.app_id != "amane" {
            ProjectTarget::slugify(&focused.app_id)
        } else {
            "scratch".to_string()
        };

        let sidecar_app_id = format!("sidecar-{slug}");

        // Check if sidecar is already open
        if let Ok(windows) = Self::query_windows() {
            for win in windows {
                if let Some(app_id) = win.app_id.as_deref()
                    && app_id == sidecar_app_id
                {
                    return Self::focus_window(win.id);
                }
            }
        }

        // Spawn companion sidecar
        let term = crate::ui::terminal_manager::detect_default_terminal();
        crate::ui::terminal_manager::spawn_terminal_window(&term, &sidecar_app_id, None, None)?;

        Ok(())
    }

    /// Quarantines external/secondary outputs for presentation mode.
    /// Focuses each external output and routes its active view to "05 PRESENTATION" (or "03 RESEARCH").
    /// Finally refocuses the primary internal display (eDP-1 if present).
    pub fn quarantine_external_outputs_for_presentation() -> anyhow::Result<()> {
        if !Self::is_available() {
            return Ok(());
        }

        let mut socket = Socket::connect()?;
        let outputs = match socket.send(Request::Outputs)? {
            Ok(Response::Outputs(outs)) => outs,
            Ok(other) => anyhow::bail!("Unexpected response querying outputs: {other:?}"),
            Err(e) => anyhow::bail!("Niri error: {e}"),
        };

        let external_outputs: Vec<String> = outputs
            .keys()
            .filter(|name| !name.starts_with("eDP"))
            .cloned()
            .collect();

        if external_outputs.is_empty() {
            return Ok(());
        }

        for ext_out in external_outputs {
            let _ = Self::perform_action(Action::FocusMonitor { output: ext_out });
            let _ = Self::focus_workspace("05 PRESENTATION")
                .or_else(|_| Self::focus_workspace("03 RESEARCH"));
        }

        if let Some(primary) = outputs.keys().find(|k| k.starts_with("eDP")) {
            let _ = Self::perform_action(Action::FocusMonitor {
                output: primary.clone(),
            });
        }

        Ok(())
    }

    pub fn update_active_window(
        id: u64,
        title: &str,
        app_id: &str,
        count: usize,
        is_full_width: bool,
    ) {
        FOCUSED_WINDOW.store(Arc::new(NiriFocusedWindow {
            id,
            title: title.to_string(),
            app_id: app_id.to_string(),
            is_full_width,
        }));

        let raw = if title.is_empty() {
            if app_id.is_empty() {
                "Dotfield"
            } else {
                app_id
            }
        } else {
            title
        };
        let display_title = if raw.chars().count() > 28 {
            format!("{}...", raw.chars().take(25).collect::<String>())
        } else {
            raw.to_string()
        };

        let mut svc = NiriWindowService::write();
        svc.id = id;
        svc.title = title.to_string();
        svc.app_id = app_id.to_string();
        svc.display_title = display_title;
        svc.window_count = count;
    }

    fn is_window_full_width(win: &Window, max_output_width: f64) -> bool {
        !win.is_floating && win.layout.tile_size.0 >= (0.90 * max_output_width).max(1200.0)
    }

    fn run_event_loop() -> std::io::Result<()> {
        let mut socket = Socket::connect()?;

        let mut max_output_width = 1920.0f64;
        if let Ok(Ok(Response::Outputs(outputs))) = socket.send(Request::Outputs) {
            for (_, out) in outputs {
                if let Some(logical) = out.logical
                    && logical.width as f64 > max_output_width
                {
                    max_output_width = logical.width as f64;
                }
            }
        }

        let mut windows: HashMap<u64, Window> = HashMap::new();

        // Seed initial focused window
        if let Ok(Ok(Response::FocusedWindow(Some(win)))) = socket.send(Request::FocusedWindow) {
            let title = win.title.clone().unwrap_or_default();
            let app_id = win.app_id.clone().unwrap_or_default();
            let full_w = Self::is_window_full_width(&win, max_output_width);
            Self::update_active_window(win.id, &title, &app_id, 1, full_w);
            windows.insert(win.id, win);
        }

        // Seed initial window list
        if let Ok(Ok(Response::Windows(wins))) = socket.send(Request::Windows) {
            let total = wins.len();
            for win in wins {
                if win.is_focused {
                    let title = win.title.clone().unwrap_or_default();
                    let app_id = win.app_id.clone().unwrap_or_default();
                    let full_w = Self::is_window_full_width(&win, max_output_width);
                    Self::update_active_window(win.id, &title, &app_id, total, full_w);
                }
                windows.insert(win.id, win);
            }
        }

        // Subscribe to event stream
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
                            let full_w = Self::is_window_full_width(&win, max_output_width);
                            Self::update_active_window(win.id, &title, &app_id, total, full_w);
                            found_focused = true;
                        }
                        windows.insert(win.id, win);
                    }
                    if !found_focused && windows.is_empty() {
                        Self::update_active_window(0, "", "", 0, false);
                    }
                }
                Event::WindowOpenedOrChanged { window } => {
                    let id = window.id;
                    let is_focused = window.is_focused;
                    let title = window.title.clone().unwrap_or_default();
                    let app_id = window.app_id.clone().unwrap_or_default();
                    let full_w = Self::is_window_full_width(&window, max_output_width);
                    windows.insert(id, window);
                    let total = windows.len();

                    if is_focused || FOCUSED_WINDOW.load().id == id {
                        Self::update_active_window(id, &title, &app_id, total, full_w);
                    }
                }
                Event::WindowClosed { id } => {
                    windows.remove(&id);
                    let total = windows.len();
                    if FOCUSED_WINDOW.load().id == id {
                        Self::update_active_window(0, "", "", total, false);
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
                                let full_w = Self::is_window_full_width(win, max_output_width);
                                Self::update_active_window(win.id, &title, &app_id, total, full_w);
                            } else {
                                Self::update_active_window(win_id, "", "", total, false);
                            }
                        }
                        None => {
                            Self::update_active_window(0, "", "", total, false);
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

    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_niri_service_availability() {
        let available = NiriService::is_available();
        let env_set = std::env::var_os("NIRI_SOCKET").is_some();
        assert_eq!(available, env_set);
    }

    #[test]
    fn test_niri_update_active_window() {
        let _lock = TEST_MUTEX.lock().unwrap();
        NiriService::update_active_window(42, "Foot Terminal", "foot", 3, true);
        let focused = NiriService::focused_window();
        assert_eq!(focused.id, 42);
        assert_eq!(focused.title, "Foot Terminal");
        assert_eq!(focused.app_id, "foot");
        assert!(focused.is_full_width);

        // Clean up
        NiriService::update_active_window(0, "", "", 0, false);
        let reset = NiriService::focused_window();
        assert_eq!(reset.id, 0);
        assert!(reset.title.is_empty());
        assert!(!reset.is_full_width);
    }

    #[test]
    fn test_niri_perform_action_offline_errors_cleanly() {
        // Never send destructive window actions to a live compositor during unit tests
        if !NiriService::is_available() {
            let result = NiriService::close_focused_window();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_niri_lock_free_concurrent_access() {
        let _lock = TEST_MUTEX.lock().unwrap();
        use std::sync::atomic::{AtomicBool, Ordering};

        let running = Arc::new(AtomicBool::new(true));
        let mut handles = vec![];

        // Spawn 4 reader threads
        for _ in 0..4 {
            let r = Arc::clone(&running);
            handles.push(thread::spawn(move || {
                let mut read_count = 0;
                while r.load(Ordering::Relaxed) {
                    let win = NiriService::focused_window();
                    let _ = win.id;
                    read_count += 1;
                }
                read_count
            }));
        }

        // Writer thread updates active window 1000 times
        let writer = thread::spawn(|| {
            for i in 1..=1000 {
                NiriService::update_active_window(i, "Ghostty", "ghostty", 1, false);
            }
        });

        writer.join().unwrap();
        running.store(false, Ordering::Relaxed);

        for handle in handles {
            let count = handle.join().unwrap();
            assert!(count > 0, "reader should have performed lock-free reads");
        }

        // Reset
        NiriService::update_active_window(0, "", "", 0, false);
    }

    #[test]
    fn test_app_id_slug_matching() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let target = ProjectTarget {
            name: "dotfield".to_string(),
            slug: "dotfield".to_string(),
            path: std::path::PathBuf::from(format!("{home}/dev/projects/dotfield")),
            branch: Some("main".to_string()),
            is_worktree: false,
            is_tmux_active: false,
            glyph: "󰘐",
        };

        let expected_app_id = format!("dev.{}", target.slug);
        let sidecar_app_id = format!("sidecar-{}", target.slug);
        assert_eq!(expected_app_id, "dev.dotfield");
        assert_eq!(sidecar_app_id, "sidecar-dotfield");
    }

    #[test]
    fn test_sidecar_slug_derivation() {
        let dev_app_id = "dev.hackathon";
        assert_eq!(dev_app_id.trim_start_matches("dev."), "hackathon");

        let custom_app_id = "foot";
        assert_eq!(ProjectTarget::slugify(custom_app_id), "foot");
    }

    #[test]
    fn test_quarantine_external_outputs_offline_graceful() {
        if !NiriService::is_available() {
            let res = NiriService::quarantine_external_outputs_for_presentation();
            assert!(res.is_ok(), "offline niri service should no-op cleanly");
        }
    }
}
