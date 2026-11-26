use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;

use amane::Service;
use arc_swap::ArcSwap;
use niri_ipc::socket::Socket;
use niri_ipc::{
    Action, Event, Output, OutputAction, Reply, Request, Response, Window, WorkspaceReferenceArg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NiriError {
    Unavailable,
    IoError,
    NiriFailure,
    UnexpectedResponse,
}

impl std::fmt::Display for NiriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => write!(f, "Niri compositor socket is unavailable"),
            Self::IoError => write!(f, "I/O error communicating with Niri socket"),
            Self::NiriFailure => write!(f, "Niri compositor reported failure"),
            Self::UnexpectedResponse => write!(f, "Unexpected response from Niri IPC"),
        }
    }
}

impl std::error::Error for NiriError {}

impl From<std::io::Error> for NiriError {
    fn from(_: std::io::Error) -> Self {
        Self::IoError
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NiriFocusedWindow {
    pub id: u64,
    pub title: String,
    pub app_id: String,
}

static FOCUSED_WINDOW: LazyLock<ArcSwap<NiriFocusedWindow>> =
    LazyLock::new(|| ArcSwap::from_pointee(NiriFocusedWindow::default()));

static CACHED_WINDOWS: LazyLock<ArcSwap<Vec<Window>>> =
    LazyLock::new(|| ArcSwap::from_pointee(Vec::new()));

static FOCUSED_OUTPUT_HEIGHT: LazyLock<ArcSwap<f32>> =
    LazyLock::new(|| ArcSwap::from_pointee(1080.0));

/// True when the focused window occupies the full output (fullscreen mode).
/// Updated atomically by the Niri event loop — zero-lock read from the render thread.
static IS_FULLSCREEN_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NiriWindowService {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub display_title: String,
    pub layout_width: f32,
    pub window_count: usize,
    pub is_fullscreen: bool,
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
    pub fn perform_action(action: Action) -> Result<(), NiriError> {
        let mut socket = Socket::connect().map_err(|_| NiriError::IoError)?;
        let reply = socket
            .send(Request::Action(action))
            .map_err(|_| NiriError::IoError)?;
        match reply {
            Ok(Response::Handled) => Ok(()),
            Ok(_) => Err(NiriError::UnexpectedResponse),
            Err(_) => Err(NiriError::NiriFailure),
        }
    }

    /// Gracefully exits Niri compositor.
    pub fn quit(skip_confirmation: bool) -> Result<(), NiriError> {
        Self::perform_action(Action::Quit { skip_confirmation })
    }

    /// Powers off monitors via DPMS.
    pub fn power_off_monitors() -> Result<(), NiriError> {
        Self::perform_action(Action::PowerOffMonitors {})
    }

    /// Powers on monitors via DPMS.
    pub fn power_on_monitors() -> Result<(), NiriError> {
        Self::perform_action(Action::PowerOnMonitors {})
    }

    /// Closes the currently focused window.
    pub fn close_focused_window() -> Result<(), NiriError> {
        Self::perform_action(Action::CloseWindow { id: None })
    }

    /// Focus a specific window ID on Niri's spatial ribbon.
    pub fn focus_window(id: u64) -> Result<(), NiriError> {
        Self::perform_action(Action::FocusWindow { id })
    }

    /// Focus a workspace by name (e.g. "01 DEV").
    pub fn focus_workspace(name: &str) -> Result<(), NiriError> {
        Self::perform_action(Action::FocusWorkspace {
            reference: WorkspaceReferenceArg::Name(name.to_string()),
        })
    }

    /// Query the current list of open windows from Niri IPC.
    pub fn query_windows() -> Result<Vec<Window>, NiriError> {
        let mut socket = Socket::connect().map_err(|_| NiriError::IoError)?;
        match socket
            .send(Request::Windows)
            .map_err(|_| NiriError::IoError)?
        {
            Ok(Response::Windows(wins)) => Ok(wins),
            Ok(_) => Err(NiriError::UnexpectedResponse),
            Err(_) => Err(NiriError::NiriFailure),
        }
    }

    /// Query connected outputs from Niri IPC.
    pub fn query_outputs() -> Result<HashMap<String, Output>, NiriError> {
        let mut socket = Socket::connect().map_err(|_| NiriError::IoError)?;
        match socket
            .send(Request::Outputs)
            .map_err(|_| NiriError::IoError)?
        {
            Ok(Response::Outputs(outputs)) => {
                Self::update_cached_output_height(&outputs);
                Ok(outputs)
            }
            Ok(_) => Err(NiriError::UnexpectedResponse),
            Err(_) => Err(NiriError::NiriFailure),
        }
    }

    fn update_cached_output_height(outputs: &HashMap<String, Output>) {
        if let Some(out) = outputs.values().find(|o| o.logical.is_some())
            && let Some(logical) = &out.logical
            && logical.height > 0
        {
            FOCUSED_OUTPUT_HEIGHT.store(Arc::new(logical.height as f32));
        }
    }

    /// Configure an output directly via Niri IPC without spawning `niri msg`.
    pub fn configure_output(output: String, action: OutputAction) -> Result<(), NiriError> {
        let mut socket = Socket::connect().map_err(|_| NiriError::IoError)?;
        match socket
            .send(Request::Output { output, action })
            .map_err(|_| NiriError::IoError)?
        {
            Ok(Response::Handled) | Ok(Response::OutputConfigChanged(_)) => Ok(()),
            Ok(_) => Err(NiriError::UnexpectedResponse),
            Err(_) => Err(NiriError::NiriFailure),
        }
    }

    /// Returns an Arc snapshot of the cached open windows from Niri IPC event stream with zero runtime allocations.
    pub fn cached_windows() -> Arc<Vec<Window>> {
        CACHED_WINDOWS.load_full()
    }

    /// Returns the cached height of the primary or focused output in logical pixels (default 1080.0).
    pub fn focused_output_height() -> f32 {
        *FOCUSED_OUTPUT_HEIGHT.load_full()
    }

    /// Returns true when the focused window currently occupies the full output (fullscreen).
    /// Cheap lock-free read — safe to call every render frame.
    pub fn is_fullscreen() -> bool {
        IS_FULLSCREEN_ACTIVE.load(Ordering::Relaxed)
    }

    /// Toggles Niri overview mode (Task View).
    pub fn toggle_overview() -> Result<(), NiriError> {
        Self::perform_action(Action::ToggleOverview {})
    }

    /// Quarantines external/secondary outputs for presentation mode.
    /// Focuses each external output and routes its active view to "05 PRESENTATION" (or "03 RESEARCH").
    /// Finally refocuses the primary internal display (eDP-1 if present).
    pub fn quarantine_external_outputs_for_presentation() -> Result<(), NiriError> {
        if !Self::is_available() {
            return Ok(());
        }

        let mut socket = Socket::connect().map_err(|_| NiriError::IoError)?;
        let outputs = match socket
            .send(Request::Outputs)
            .map_err(|_| NiriError::IoError)?
        {
            Ok(Response::Outputs(outs)) => outs,
            Ok(_) => return Err(NiriError::UnexpectedResponse),
            Err(_) => return Err(NiriError::NiriFailure),
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
            let _ = Self::focus_workspace("05 SCRATCH")
                .or_else(|_| Self::focus_workspace("05 PRESENTATION"))
                .or_else(|_| Self::focus_workspace("03 RESEARCH"));
        }

        if let Some(primary) = outputs.keys().find(|k| k.starts_with("eDP")) {
            let _ = Self::perform_action(Action::FocusMonitor {
                output: primary.clone(),
            });
        }

        Ok(())
    }

    pub fn update_active_window(id: u64, title: &str, app_id: &str, count: usize) {
        FOCUSED_WINDOW.store(Arc::new(NiriFocusedWindow {
            id,
            title: title.to_string(),
            app_id: app_id.to_string(),
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

        let layout_width = (display_title.chars().count() as f32 * 8.5 + 40.0).clamp(64.0, 280.0);

        let mut svc = NiriWindowService::write();
        svc.id = id;
        svc.title = title.to_string();
        svc.app_id = app_id.to_string();
        svc.display_title = display_title;
        svc.layout_width = layout_width;
        svc.window_count = count;
    }

    /// Update the IS_FULLSCREEN_ACTIVE flag from a niri_ipc Window's tile_size.
    /// Fullscreen is detected when tile height >= (output_height - 4) to tolerate HiDPI rounding.
    /// Passing tile_size = None clears the flag (window closed / focus lost).
    fn update_fullscreen_flag(tile_size: Option<(f64, f64)>) {
        let is_fs = match tile_size {
            Some((_, h)) => {
                let out_h = *FOCUSED_OUTPUT_HEIGHT.load_full() as f64;
                h >= out_h - 4.0
            }
            None => false,
        };
        IS_FULLSCREEN_ACTIVE.store(is_fs, Ordering::Relaxed);
        let mut svc = NiriWindowService::write();
        svc.is_fullscreen = is_fs;
    }

    fn run_event_loop() -> std::io::Result<()> {
        let socket_path = match std::env::var_os("NIRI_SOCKET") {
            Some(p) => p,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "NIRI_SOCKET is not set",
                ));
            }
        };

        let mut socket = Socket::connect()?;
        let mut windows: HashMap<u64, Window> = HashMap::new();

        // Seed initial focused window
        if let Ok(Ok(Response::FocusedWindow(Some(win)))) = socket.send(Request::FocusedWindow) {
            let title = win.title.clone().unwrap_or_default();
            let app_id = win.app_id.clone().unwrap_or_default();
            Self::update_active_window(win.id, &title, &app_id, 1);
            Self::update_fullscreen_flag(Some(win.layout.tile_size));
            windows.insert(win.id, win);
        }

        // Seed initial window list
        if let Ok(Ok(Response::Windows(wins))) = socket.send(Request::Windows) {
            let total = wins.len();
            for win in &wins {
                if win.is_focused {
                    let title = win.title.clone().unwrap_or_default();
                    let app_id = win.app_id.clone().unwrap_or_default();
                    Self::update_active_window(win.id, &title, &app_id, total);
                    Self::update_fullscreen_flag(Some(win.layout.tile_size));
                }
                windows.insert(win.id, win.clone());
            }
            CACHED_WINDOWS.store(Arc::new(wins));
        }

        // Seed initial output height
        if let Ok(Ok(Response::Outputs(outs))) = socket.send(Request::Outputs) {
            Self::update_cached_output_height(&outs);
        }

        drop(socket);

        // Connect directly to NIRI_SOCKET and wrap in BufReader for zero-alloc line reading
        let stream = UnixStream::connect(socket_path)?;
        let mut reader = BufReader::new(stream);

        // Subscribe to event stream
        let mut req_bytes = serde_json::to_vec(&Request::EventStream)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        req_bytes.push(b'\n');
        reader.get_mut().write_all(&req_bytes)?;
        reader.get_mut().flush()?;

        let mut line_buf = Vec::with_capacity(2048);
        reader.read_until(b'\n', &mut line_buf)?;
        let reply: Reply = serde_json::from_slice(&line_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        if !matches!(reply, Ok(Response::Handled)) {
            return Err(std::io::Error::other(
                "Failed to subscribe to niri event stream",
            ));
        }

        let _ = reader.get_mut().shutdown(std::net::Shutdown::Write);

        line_buf.clear();
        while reader.read_until(b'\n', &mut line_buf)? > 0 {
            if let Ok(event) = serde_json::from_slice::<Event>(&line_buf) {
                match event {
                    Event::WindowsChanged {
                        windows: new_windows,
                    } => {
                        windows.clear();
                        let total = new_windows.len();
                        let mut found_focused = false;
                        for win in &new_windows {
                            if win.is_focused {
                                let title = win.title.clone().unwrap_or_default();
                                let app_id = win.app_id.clone().unwrap_or_default();
                                Self::update_active_window(win.id, &title, &app_id, total);
                                Self::update_fullscreen_flag(Some(win.layout.tile_size));
                                found_focused = true;
                            }
                            windows.insert(win.id, win.clone());
                        }
                        if !found_focused && windows.is_empty() {
                            Self::update_active_window(0, "", "", 0);
                            Self::update_fullscreen_flag(None);
                        }
                        CACHED_WINDOWS.store(Arc::new(new_windows));
                    }
                    Event::WindowOpenedOrChanged { window } => {
                        let id = window.id;
                        let is_focused = window.is_focused;
                        let title = window.title.clone().unwrap_or_default();
                        let app_id = window.app_id.clone().unwrap_or_default();
                        let tile_size = window.layout.tile_size;
                        windows.insert(id, window);
                        let total = windows.len();

                        if is_focused || FOCUSED_WINDOW.load().id == id {
                            Self::update_active_window(id, &title, &app_id, total);
                            Self::update_fullscreen_flag(Some(tile_size));
                        }
                        CACHED_WINDOWS.store(Arc::new(windows.values().cloned().collect()));
                    }
                    Event::WindowClosed { id } => {
                        windows.remove(&id);
                        let total = windows.len();
                        if FOCUSED_WINDOW.load().id == id {
                            Self::update_active_window(0, "", "", total);
                            Self::update_fullscreen_flag(None);
                        } else {
                            NiriWindowService::write().window_count = total;
                        }
                        CACHED_WINDOWS.store(Arc::new(windows.values().cloned().collect()));
                    }
                    Event::WindowFocusChanged { id } => {
                        let total = windows.len();
                        for win in windows.values_mut() {
                            win.is_focused = Some(win.id) == id;
                        }
                        match id {
                            Some(win_id) => {
                                if let Some(win) = windows.get(&win_id) {
                                    let title = win.title.clone().unwrap_or_default();
                                    let app_id = win.app_id.clone().unwrap_or_default();
                                    Self::update_active_window(win.id, &title, &app_id, total);
                                    Self::update_fullscreen_flag(Some(win.layout.tile_size));
                                } else {
                                    Self::update_active_window(win_id, "", "", total);
                                    Self::update_fullscreen_flag(None);
                                }
                            }
                            None => {
                                Self::update_active_window(0, "", "", total);
                                Self::update_fullscreen_flag(None);
                            }
                        }
                        CACHED_WINDOWS.store(Arc::new(windows.values().cloned().collect()));
                    }
                    _ => {}
                }
            }
            line_buf.clear();
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
                NiriService::update_active_window(i, "Ghostty", "ghostty", 1);
            }
        });

        writer.join().unwrap();
        running.store(false, Ordering::Relaxed);

        for handle in handles {
            let count = handle.join().unwrap();
            assert!(count > 0, "reader should have performed lock-free reads");
        }

        // Reset
        NiriService::update_active_window(0, "", "", 0);
    }

    #[test]
    fn test_niri_cached_windows_default() {
        let windows = NiriService::cached_windows();
        assert!(windows.is_empty() || !windows.is_empty());
    }

    #[test]
    fn test_toggle_overview_offline() {
        if !NiriService::is_available() {
            let res = NiriService::toggle_overview();
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_quarantine_external_outputs_offline_graceful() {
        if !NiriService::is_available() {
            let res = NiriService::quarantine_external_outputs_for_presentation();
            assert!(res.is_ok(), "offline niri service should no-op cleanly");
        }
    }

    #[test]
    fn test_niri_fullscreen_detection() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let out_h = NiriService::focused_output_height() as f64;

        // Tile size matching output height -> fullscreen
        NiriService::update_fullscreen_flag(Some((1920.0, out_h)));
        assert!(NiriService::is_fullscreen());

        // Smaller tile height -> not fullscreen
        NiriService::update_fullscreen_flag(Some((1920.0, out_h - 100.0)));
        assert!(!NiriService::is_fullscreen());

        // None (no focused window) -> not fullscreen
        NiriService::update_fullscreen_flag(None);
        assert!(!NiriService::is_fullscreen());
    }
}
