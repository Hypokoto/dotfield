pub mod config;
pub mod services;
pub mod ui;

#[global_allocator]
static GLOBAL: std::alloc::System = std::alloc::System;

use amane::Service;
use std::io::IsTerminal;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Debug, Default)]
struct Cli {
    foreground: bool,
    daemon: bool,
    config: Option<PathBuf>,
    no_bar: bool,
    target_action: Option<String>,
    action_args: Vec<String>,
}

fn print_help() {
    println!(
        r#"amane — Unified Wayland desktop shell for Niri powered by Amane

Usage:
  amane [FLAGS] [COMMAND]

Flags:
  -f, --foreground        Run in foreground without detaching
  -d, --daemon            Run in background
  -c, --config <PATH>     Path to custom shell manifest TOML
      --no-bar            Run desktop shell without top status bar
  -h, --help              Print help information

Commands / Fast-Path IPC Actions (Forwarded to running daemon):
  launcher                Toggle application launcher overlay
  display-config          Toggle display configurator overlay
  control-center          Toggle control center overlay
  network                 Open NetworkManager interactive widget
  audio                   Open PipeWire audio interactive widget
  bluetooth               Open BlueZ bluetooth interactive widget
  tray                    Open System Tray & background applications widget
  powermenu               Toggle session & power control menu
  calendar                Toggle interactive calendar overlay
  screen-recorder         Toggle screen recorder overlay
  presentation            Toggle zero-disruption presentation mode
  widgets                 Toggle left slide-in widgets board
  start-menu              Toggle centered Start Menu & application grid
  action-center           Toggle right slide-in action center & calendar hub
  overview                Toggle spatial ribbon workspace overview
  reload                  Reload shell configuration
  osd <action> [delta]    Trigger OSD action (volume-up, volume-down, etc.)
  ipc <action> [args...]  Send generic IPC action
"#
    );
}

fn parse_cli_args() -> Result<Cli, String> {
    let mut cli = Cli::default();
    let mut args = std::env::args().skip(1).peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => {
                print_help();
                std::process::exit(0);
            }
            "-f" | "--foreground" => {
                cli.foreground = true;
            }
            "-d" | "--daemon" => {
                cli.daemon = true;
            }
            "--no-bar" => {
                cli.no_bar = true;
            }
            "-c" | "--config" => {
                if let Some(path) = args.next() {
                    cli.config = Some(PathBuf::from(path));
                } else {
                    return Err("Missing argument for --config".to_string());
                }
            }
            s if s.starts_with("--config=") => {
                cli.config = Some(PathBuf::from(&s["--config=".len()..]));
            }
            "--osd" | "osd" => {
                cli.target_action = Some("osd".to_string());
                while let Some(next) = args.peek() {
                    if next.starts_with('-') {
                        break;
                    }
                    if let Some(arg) = args.next() {
                        cli.action_args.push(arg);
                    }
                }
            }
            "ipc" => {
                if let Some(action) = args.next() {
                    cli.target_action = Some(action);
                    cli.action_args.extend(args);
                    break;
                } else {
                    return Err("Missing action for ipc command".to_string());
                }
            }
            "run" => {
                // Legacy "run": fall through to starting shell
            }
            "--launcher" | "launcher" => cli.target_action = Some("launcher".to_string()),
            "--display-config" | "display-config" => {
                cli.target_action = Some("display-config".to_string())
            }
            "--settings" | "settings" | "--setting" | "setting" => {
                cli.target_action = Some("settings".to_string())
            }
            "--control-center" | "control-center" => {
                cli.target_action = Some("control-center".to_string())
            }
            "--network" | "network" => cli.target_action = Some("network".to_string()),
            "--audio" | "audio" => cli.target_action = Some("audio".to_string()),
            "--bluetooth" | "bluetooth" => cli.target_action = Some("bluetooth".to_string()),
            "--tray" | "tray" => cli.target_action = Some("tray".to_string()),
            "--powermenu" | "powermenu" => cli.target_action = Some("powermenu".to_string()),
            "--calendar" | "calendar" => cli.target_action = Some("calendar".to_string()),
            "--screen-recorder" | "screen-recorder" | "--screen-record" | "screen-record" => {
                cli.target_action = Some("screen-recorder".to_string())
            }
            "--presentation" | "presentation" => {
                cli.target_action = Some("presentation".to_string())
            }
            "--widgets" | "widgets" => cli.target_action = Some("widgets".to_string()),
            "--start-menu" | "start-menu" | "--start" | "start" => {
                cli.target_action = Some("start-menu".to_string())
            }
            "--terminal-manager" | "terminal-manager" | "--terminals" | "terminals"
            | "--topology" | "topology" => cli.target_action = Some("terminal-manager".to_string()),
            "--sidecar" | "sidecar" => cli.target_action = Some("sidecar".to_string()),
            "--action-center" | "action-center" => {
                cli.target_action = Some("action-center".to_string())
            }
            "--overview" | "overview" => cli.target_action = Some("overview".to_string()),
            "--reload" | "reload" => cli.target_action = Some("reload".to_string()),
            other => {
                return Err(format!("Unknown option or action: {other}"));
            }
        }
    }

    Ok(cli)
}

fn lock_path() -> PathBuf {
    amane::ipc_socket().with_extension("lock")
}

struct DaemonLock {
    file: std::fs::File,
}

impl DaemonLock {
    fn try_acquire() -> anyhow::Result<Option<Self>> {
        use std::os::unix::io::AsRawFd;
        let path = lock_path();
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;

        let fd = file.as_raw_fd();
        let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
        if ret == 0 {
            Ok(Some(Self { file }))
        } else {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EWOULDBLOCK) {
                Ok(None)
            } else {
                Err(err.into())
            }
        }
    }

    fn release(self) {
        use std::os::unix::io::AsRawFd;
        let fd = self.file.as_raw_fd();
        unsafe {
            libc::flock(fd, libc::LOCK_UN);
        }
    }

    fn is_locked() -> bool {
        match Self::try_acquire() {
            Ok(Some(lock)) => {
                lock.release();
                false
            }
            Ok(None) => true,
            Err(_) => false,
        }
    }
}

fn send_amane_ipc(call: &amane::IpcCall) -> anyhow::Result<String> {
    use std::io::Read;

    let socket_path = amane::ipc_socket();
    let mut stream = UnixStream::connect(&socket_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to connect to amane daemon at {}: {e}",
            socket_path.display()
        )
    })?;

    call.write(&mut stream)?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;
    Ok(reply)
}

fn ipc_osd_handler(args: &[String]) -> String {
    if args.is_empty() {
        return "osd: missing action".to_string();
    }
    let action = args[0].as_str();
    let delta = args
        .get(1)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(5.0);

    match action {
        "volume-up" => {
            let current = amane::Audio::read().volume() as i32;
            let target = (current + delta as i32).clamp(0, 100) as u8;
            amane::Audio::set_volume(target);
            ui::osd::trigger_osd(ui::osd::OsdType::Volume, target, false, 1200);
            format!("volume: {target}%")
        }
        "volume-down" => {
            let current = amane::Audio::read().volume() as i32;
            let target = (current - delta as i32).clamp(0, 100) as u8;
            amane::Audio::set_volume(target);
            ui::osd::trigger_osd(ui::osd::OsdType::Volume, target, false, 1200);
            format!("volume: {target}%")
        }
        "volume-mute" => {
            amane::Audio::toggle_mute();
            let audio = amane::Audio::read();
            ui::osd::trigger_osd(
                ui::osd::OsdType::Volume,
                audio.volume(),
                audio.muted(),
                1200,
            );
            format!("volume mute: {}", audio.muted())
        }
        "brightness-up" => {
            let current = amane::Brightness::read().percent() as i32;
            let target = (current + delta as i32).clamp(0, 100) as u8;
            amane::Brightness::set(target);
            ui::osd::trigger_osd(ui::osd::OsdType::Brightness, target, false, 1200);
            format!("brightness: {target}%")
        }
        "brightness-down" => {
            let current = amane::Brightness::read().percent() as i32;
            let target = (current - delta as i32).clamp(0, 100) as u8;
            amane::Brightness::set(target);
            ui::osd::trigger_osd(ui::osd::OsdType::Brightness, target, false, 1200);
            format!("brightness: {target}%")
        }
        _ => format!("osd: unknown action {action}"),
    }
}

fn ipc_volume_up_handler(args: &[String]) -> String {
    let mut osd_args = vec!["volume-up".to_string()];
    osd_args.extend_from_slice(args);
    ipc_osd_handler(&osd_args)
}

fn ipc_volume_down_handler(args: &[String]) -> String {
    let mut osd_args = vec!["volume-down".to_string()];
    osd_args.extend_from_slice(args);
    ipc_osd_handler(&osd_args)
}

fn ipc_volume_mute_handler(_args: &[String]) -> String {
    ipc_osd_handler(&["volume-mute".to_string()])
}

fn ipc_brightness_up_handler(args: &[String]) -> String {
    let mut osd_args = vec!["brightness-up".to_string()];
    osd_args.extend_from_slice(args);
    ipc_osd_handler(&osd_args)
}

fn ipc_brightness_down_handler(args: &[String]) -> String {
    let mut osd_args = vec!["brightness-down".to_string()];
    osd_args.extend_from_slice(args);
    ipc_osd_handler(&osd_args)
}

fn ipc_reload_handler(_args: &[String]) -> String {
    let path = config::ShellManifest::default_path();
    match config::ShellManifest::load_from_path(&path) {
        Ok(manifest) => {
            config::ConfigService::write().manifest = manifest.clone();
            services::appearance::AppearanceService::reconcile_theme(&manifest);
            "reloaded".to_string()
        }
        Err(e) => format!("reload failed: {e}"),
    }
}

fn ipc_launcher_handler(_args: &[String]) -> String {
    ui::start_menu::toggle();
    "start-menu toggled".to_string()
}

fn ipc_control_center_handler(_args: &[String]) -> String {
    ui::control_center::toggle();
    "control-center toggled".to_string()
}

fn ipc_network_handler(_args: &[String]) -> String {
    ui::control_center::open_view(ui::control_center::ActiveView::Network);
    "network widget opened".to_string()
}

fn ipc_audio_handler(_args: &[String]) -> String {
    ui::control_center::open_view(ui::control_center::ActiveView::Audio);
    "audio widget opened".to_string()
}

fn ipc_bluetooth_handler(_args: &[String]) -> String {
    ui::control_center::open_view(ui::control_center::ActiveView::Bluetooth);
    "bluetooth widget opened".to_string()
}

fn ipc_tray_handler(_args: &[String]) -> String {
    ui::control_center::open_view(ui::control_center::ActiveView::Tray);
    "tray widget opened".to_string()
}

fn ipc_notifications_handler(_args: &[String]) -> String {
    ui::control_center::open_view(ui::control_center::ActiveView::Notifications);
    "notification center opened".to_string()
}

fn ipc_display_config_handler(_args: &[String]) -> String {
    ui::display_config::toggle();
    "display-config toggled".to_string()
}

fn ipc_settings_handler(args: &[String]) -> String {
    if let Some(cat) = args
        .first()
        .and_then(|a| ui::settings::SettingsCategory::from_str_loose(a))
    {
        ui::settings::open_category(cat);
        return format!("settings opened at category '{}'", cat.name());
    }
    ui::settings::toggle();
    "settings toggled".to_string()
}

fn ipc_powermenu_handler(_args: &[String]) -> String {
    ui::powermenu::toggle();
    "powermenu toggled".to_string()
}

fn ipc_calendar_handler(_args: &[String]) -> String {
    ui::calendar::toggle();
    "calendar toggled".to_string()
}

fn ipc_screen_record_handler(_args: &[String]) -> String {
    ui::screen_record::toggle();
    "screen-recorder toggled".to_string()
}

fn ipc_presentation_handler(_args: &[String]) -> String {
    let (now_active, drained) =
        services::presentation::PresentationService::global().toggle_with_drained();
    if now_active {
        "presentation mode active".to_string()
    } else {
        format!("presentation mode disabled (drained {drained} notifications)")
    }
}

fn ipc_widgets_handler(_args: &[String]) -> String {
    ui::control_center::toggle();
    "control-center toggled".to_string()
}

fn ipc_start_menu_handler(_args: &[String]) -> String {
    ui::start_menu::toggle();
    "start-menu toggled".to_string()
}

fn ipc_action_center_handler(_args: &[String]) -> String {
    ui::calendar::toggle();
    "calendar toggled".to_string()
}

fn ipc_overview_handler(_args: &[String]) -> String {
    ui::overview::toggle();
    "overview toggled".to_string()
}

fn ipc_terminal_manager_handler(args: &[String]) -> String {
    if !args.is_empty() {
        let query = format!(": {}", args.join(" "));
        ui::start_menu::open_with_query(&query);
        "terminal command queued".to_string()
    } else {
        let is_open = ui::start_menu::StartMenuService::read().open;
        if is_open {
            ui::start_menu::close();
            "terminal-manager closed".to_string()
        } else {
            ui::start_menu::open_terminal_mode();
            "terminal-manager opened".to_string()
        }
    }
}

fn ipc_sidecar_handler(args: &[String]) -> String {
    let has_ghostty = std::process::Command::new("which")
        .arg("ghostty")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let has_foot = !has_ghostty
        && std::process::Command::new("which")
            .arg("foot")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
    let has_kitty = !has_ghostty
        && !has_foot
        && std::process::Command::new("which")
            .arg("kitty")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    let mut cmd = if has_ghostty {
        let mut c = std::process::Command::new("ghostty");
        c.arg("--class=sidecar-term");
        c
    } else if has_foot {
        let mut c = std::process::Command::new("foot");
        c.arg("--app-id=sidecar-term");
        c
    } else if has_kitty {
        let mut c = std::process::Command::new("kitty");
        c.arg("--class=sidecar-term");
        c
    } else {
        std::process::Command::new("xterm")
    };

    if !args.is_empty() {
        cmd.arg("-e");
        cmd.args(args);
    }

    match cmd.spawn() {
        Ok(_) => "sidecar spawned".to_string(),
        Err(e) => format!("failed to spawn sidecar: {e}"),
    }
}

fn run_shell(config_path: Option<PathBuf>, no_bar: bool) -> anyhow::Result<()> {
    // Acquire exclusive daemon lock
    let lock = match DaemonLock::try_acquire()? {
        Some(l) => l,
        None => {
            eprintln!("Error: another amane daemon instance is already active and locked.");
            std::process::exit(1);
        }
    };

    let path = config_path.unwrap_or_else(config::ShellManifest::default_path);
    let manifest = match config::ShellManifest::load_from_path(&path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "[amane:config:warn] failed to load {}: {e}; using defaults",
                path.display()
            );
            config::ShellManifest::default()
        }
    };
    config::ConfigService::write().manifest = manifest.clone();
    services::appearance::AppearanceService::reconcile_theme(&manifest);

    // Clean up stale IPC socket if present (safe since we hold the lock)
    let sock = amane::ipc_socket();
    if sock.exists() {
        let _ = std::fs::remove_file(&sock);
    }

    // Start background event watchers (e.g. Niri socket, StatusNotifierWatcher tray, battery monitor, telemetry, topology, cava)
    services::niri::NiriService::start();
    services::tray::TrayService::start();
    services::battery::BatteryMonitor::start();
    services::telemetry::TelemetryService::start();
    services::cava::CavaService::start();

    // Start notification auto-dismiss timer worker
    ui::notifications::start_expiry_worker();

    let show_bar = !no_bar && manifest.bar.enabled;

    // Run Amane application with bar (if enabled), osd, notifications, launcher, control-center, display-config, powermenu, calendar, screen-recorder, widgets, start-menu, action-center, and overview
    let mut app = amane::App::new()
        .ipc("osd", ipc_osd_handler)
        .ipc("volume-up", ipc_volume_up_handler)
        .ipc("volume-down", ipc_volume_down_handler)
        .ipc("volume-mute", ipc_volume_mute_handler)
        .ipc("brightness-up", ipc_brightness_up_handler)
        .ipc("brightness-down", ipc_brightness_down_handler)
        .ipc("reload", ipc_reload_handler)
        .ipc("launcher", ipc_launcher_handler)
        .ipc("control-center", ipc_control_center_handler)
        .ipc("network", ipc_network_handler)
        .ipc("audio", ipc_audio_handler)
        .ipc("bluetooth", ipc_bluetooth_handler)
        .ipc("tray", ipc_tray_handler)
        .ipc("notifications", ipc_notifications_handler)
        .ipc("notification-center", ipc_notifications_handler)
        .ipc("display-config", ipc_display_config_handler)
        .ipc("settings", ipc_settings_handler)
        .ipc("setting", ipc_settings_handler)
        .ipc("presentation", ipc_presentation_handler)
        .ipc("widgets", ipc_widgets_handler)
        .ipc("start-menu", ipc_start_menu_handler)
        .ipc("start", ipc_start_menu_handler)
        .ipc("terminal-manager", ipc_terminal_manager_handler)
        .ipc("terminals", ipc_terminal_manager_handler)
        .ipc("topology", ipc_terminal_manager_handler)
        .ipc("sidecar", ipc_sidecar_handler)
        .ipc("action-center", ipc_action_center_handler)
        .ipc("overview", ipc_overview_handler)
        .ipc("powermenu", ipc_powermenu_handler)
        .ipc("calendar", ipc_calendar_handler)
        .ipc("screen-recorder", ipc_screen_record_handler);

    if show_bar {
        app = app
            .window_per_monitor(ui::bar::top_bar_monitor)
            .window_per_monitor(ui::bar::top_island_monitor);
    }

    app.window(ui::osd::osd_overlay)
        .window(ui::notifications::notification_popups)
        .window(ui::control_center::control_center_panel)
        .window(ui::display_config::display_config_overlay)
        .window(ui::powermenu::powermenu_overlay)
        .window(ui::calendar::calendar_overlay)
        .window(ui::screen_record::screen_record_overlay)
        .window(ui::start_menu::start_menu_overlay)
        .window(ui::overview::overview_overlay)
        .window(ui::settings::settings_overlay)
        .run();

    // If event loop returns unexpectedly, exit with code 1 so supervisor can restart
    drop(lock);
    eprintln!("[amane:warn] event loop exited unexpectedly");
    std::process::exit(1);
}

fn main() -> anyhow::Result<()> {
    let cli = match parse_cli_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    if let Some(act) = cli.target_action {
        let call = amane::IpcCall::new(&act, &cli.action_args);
        let res = send_amane_ipc(&call)?;
        if !res.is_empty() {
            println!("{res}");
        }
        return Ok(());
    }

    // Check if another daemon instance is already active and responding or locked
    if UnixStream::connect(amane::ipc_socket()).is_ok() || DaemonLock::is_locked() {
        println!("amane is already running");
        return Ok(());
    }

    // No action requested: run the shell!
    // When invoked from an interactive terminal and --foreground is not set,
    // automatically fork/detach into the background.
    if (std::io::stdout().is_terminal() || cli.daemon) && !cli.foreground {
        let exe = std::env::current_exe()?;
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("--foreground");
        if cli.no_bar {
            cmd.arg("--no-bar");
        }
        if let Some(cfg) = &cli.config {
            cmd.arg("--config").arg(cfg);
        }

        use std::os::unix::process::CommandExt;
        cmd.process_group(0)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let child = cmd.spawn()?;
        println!("amane started in background (PID {})", child.id());
        return Ok(());
    }

    // Running directly in foreground (systemd, Niri spawn-at-startup, or explicit --foreground)
    run_shell(cli.config, cli.no_bar)
}
