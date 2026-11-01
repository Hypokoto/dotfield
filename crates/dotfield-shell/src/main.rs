pub mod config;
pub mod services;
pub mod ui;

use amane::Service;
use clap::Parser;
use std::io::IsTerminal;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

#[derive(Parser, Debug, Default)]
#[command(name = "dotfield-shell")]
#[command(about = "Unified Wayland desktop shell for Dotfield powered by Amane")]
struct Cli {
    /// Run in the foreground without detaching to background
    #[arg(short, long)]
    foreground: bool,

    /// Explicitly run in background (default when invoked from an interactive terminal)
    #[arg(short, long)]
    daemon: bool,

    /// Path to custom shell manifest TOML
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Toggle application launcher overlay
    #[arg(long)]
    launcher: bool,

    /// Toggle terminal manager overlay
    #[arg(long)]
    terminal_manager: bool,

    /// Toggle display configurator overlay
    #[arg(long)]
    display_config: bool,

    /// Toggle control center overlay
    #[arg(long)]
    control_center: bool,

    /// Reload shell configuration
    #[arg(long)]
    reload: bool,

    /// Trigger OSD action: volume-up, volume-down, volume-mute, brightness-up, brightness-down
    #[arg(long, num_args = 1..=2)]
    osd: Option<Vec<String>>,

    /// Positional action or legacy subcommand (e.g. 'run', 'ipc', 'launcher', 'terminal-manager')
    #[arg(value_name = "ACTION")]
    action: Option<String>,

    /// Additional arguments passed to action (e.g. for osd)
    #[arg(value_name = "ARGS", trailing_var_arg = true)]
    extra_args: Vec<String>,
}

fn send_amane_ipc(call: &amane::IpcCall) -> anyhow::Result<String> {
    use std::io::Read;

    let socket_path = amane::ipc_socket();
    let mut stream = UnixStream::connect(socket_path)?;
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

fn ipc_reload_handler(_args: &[String]) -> String {
    let path = config::ShellManifest::default_path();
    match config::ShellManifest::load_from_path(&path) {
        Ok(manifest) => {
            ui::bar::set_config(manifest.clone());
            ui::osd::set_config(manifest.clone());
            ui::notifications::set_config(manifest.clone());
            ui::launcher::set_config(manifest.clone());
            ui::control_center::set_config(manifest.clone());
            ui::display_config::set_config(manifest.clone());
            ui::terminal_manager::set_config(manifest);
            "reloaded".to_string()
        }
        Err(e) => format!("reload failed: {e}"),
    }
}

fn ipc_launcher_handler(_args: &[String]) -> String {
    ui::launcher::toggle();
    "launcher toggled".to_string()
}

fn ipc_control_center_handler(_args: &[String]) -> String {
    ui::control_center::toggle();
    "control-center toggled".to_string()
}

fn ipc_display_config_handler(_args: &[String]) -> String {
    ui::display_config::toggle();
    "display-config toggled".to_string()
}

fn ipc_terminal_manager_handler(_args: &[String]) -> String {
    ui::terminal_manager::toggle();
    "terminal-manager toggled".to_string()
}

fn run_shell(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = config_path.unwrap_or_else(config::ShellManifest::default_path);
    let manifest = config::ShellManifest::load_from_path(&path)?;
    ui::bar::set_config(manifest.clone());
    ui::osd::set_config(manifest.clone());
    ui::notifications::set_config(manifest.clone());
    ui::launcher::set_config(manifest.clone());
    ui::control_center::set_config(manifest.clone());
    ui::display_config::set_config(manifest.clone());
    ui::terminal_manager::set_config(manifest);

    // Start background event watchers (e.g. Niri socket)
    services::niri::NiriService::start();

    // Run Amane application with bar, osd, notifications, launcher, control-center, display-config, and terminal-manager
    amane::App::new()
        .ipc("osd", ipc_osd_handler)
        .ipc("reload", ipc_reload_handler)
        .ipc("launcher", ipc_launcher_handler)
        .ipc("control-center", ipc_control_center_handler)
        .ipc("display-config", ipc_display_config_handler)
        .ipc("terminal-manager", ipc_terminal_manager_handler)
        .window(ui::bar::top_bar)
        .window(ui::osd::osd_overlay)
        .window(ui::notifications::notification_popups)
        .window(ui::launcher::launcher_overlay)
        .window(ui::control_center::control_center_panel)
        .window(ui::display_config::display_config_overlay)
        .window(ui::terminal_manager::terminal_manager_overlay)
        .run();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Determine if an action was requested via flags or positional argument
    let mut target_action: Option<String> = None;
    let mut action_args: Vec<String> = Vec::new();

    if cli.launcher {
        target_action = Some("launcher".to_string());
    } else if cli.terminal_manager {
        target_action = Some("terminal-manager".to_string());
    } else if cli.display_config {
        target_action = Some("display-config".to_string());
    } else if cli.control_center {
        target_action = Some("control-center".to_string());
    } else if cli.reload {
        target_action = Some("reload".to_string());
    } else if let Some(osd_args) = cli.osd {
        target_action = Some("osd".to_string());
        action_args = osd_args;
    } else if let Some(act) = cli.action.as_deref() {
        match act {
            "run" => {
                // Legacy "run" argument: fall through to starting the shell
            }
            "ipc" => {
                if let Some(sub) = cli.extra_args.first() {
                    target_action = Some(sub.clone());
                    action_args = cli.extra_args[1..].to_vec();
                }
            }
            "launcher" | "terminal-manager" | "display-config" | "control-center" | "reload" => {
                target_action = Some(act.to_string());
                action_args = cli.extra_args;
            }
            "osd" => {
                target_action = Some("osd".to_string());
                action_args = cli.extra_args;
            }
            other => {
                eprintln!("Unknown option or action: {other}");
                std::process::exit(1);
            }
        }
    }

    if let Some(act) = target_action {
        let call = amane::IpcCall::new(&act, &action_args);
        let res = send_amane_ipc(&call)?;
        if !res.is_empty() {
            println!("{res}");
        }
        return Ok(());
    }

    // No action requested: run the shell!
    // When invoked from an interactive terminal and --foreground is not set,
    // automatically fork/detach into the background.
    if (std::io::stdout().is_terminal() || cli.daemon) && !cli.foreground {
        if UnixStream::connect(amane::ipc_socket()).is_ok() {
            println!("dotfield-shell is already running in background");
            return Ok(());
        }

        let exe = std::env::current_exe()?;
        let mut cmd = std::process::Command::new(exe);
        cmd.arg("--foreground");
        if let Some(cfg) = &cli.config {
            cmd.arg("--config").arg(cfg);
        }

        use std::os::unix::process::CommandExt;
        cmd.process_group(0)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        let child = cmd.spawn()?;
        println!("dotfield-shell started in background (PID {})", child.id());
        return Ok(());
    }

    // Running directly in foreground (systemd, Niri spawn-at-startup, or explicit --foreground)
    run_shell(cli.config)
}
