//! amane-msg — Ultra-lightweight Zero-Spawn IPC CLI for Amane desktop shell.
//! Dispatches commands to `$XDG_RUNTIME_DIR/amane.sock` using single-byte opcodes (<0.2ms).

use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use amane::name_to_opcode;

fn socket_path() -> Result<PathBuf, &'static str> {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR").ok_or("XDG_RUNTIME_DIR is not set")?;
    Ok(PathBuf::from(runtime_dir).join("amane.sock"))
}

fn print_help() {
    eprintln!(
        r#"amane-msg — Ultra-fast Zero-Spawn IPC client for Amane desktop shell

Usage:
  amane-msg <command> [args...]

Fast-Path Opcodes (0-alloc byte frames, <0.2ms):
  launcher              Toggle application launcher modal
  terminal-manager      Toggle terminal selector
  topology              Toggle project topology & worktree switcher (alias)
  presentation          Toggle zero-disruption presentation mode shield
  sidecar               Spawn or focus companion sidecar for active window
  widgets               Toggle left slide-in widgets board
  start-menu            Toggle centered Start Menu & application grid
  action-center         Toggle right slide-in action center & calendar hub
  overview              Toggle spatial ribbon workspace overview
  display-config        Toggle interactive display configurator
  control-center        Toggle control center
  network               Toggle network settings
  audio                 Toggle audio settings
  bluetooth             Toggle bluetooth settings
  tray                  Toggle system tray drawer
  powermenu             Toggle power menu
  calendar              Toggle calendar & clock
  screen-recorder       Toggle screen recorder HUD
  reload                Reload desktop shell configuration
  volume-up             OSD volume up
  volume-down           OSD volume down
  volume-mute           OSD toggle mute
  brightness-up         OSD brightness up
  brightness-down       OSD brightness down
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        std::process::exit(1);
    }

    let cmd = &args[0];
    if cmd == "-h" || cmd == "--help" || cmd == "help" {
        print_help();
        return;
    }

    // Normalize command names (e.g. remove leading '--')
    let normalized_cmd = cmd.strip_prefix("--").unwrap_or(cmd);

    let sock = match socket_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("amane-msg: error: {e}");
            std::process::exit(1);
        }
    };

    let mut stream = match UnixStream::connect(&sock) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("amane-msg: cannot connect to {}: {e}", sock.display());
            std::process::exit(2);
        }
    };

    // Check if single command maps to a fast-path 1-byte opcode
    if args.len() == 1
        && let Some(opcode) = name_to_opcode(normalized_cmd)
    {
        // Set brief timeout for socket operations
        let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
        let _ = stream.set_read_timeout(Some(Duration::from_millis(150)));

        if stream.write_all(&[opcode]).is_err() || stream.flush().is_err() {
            eprintln!("amane-msg: failed to send opcode");
            std::process::exit(3);
        }

        // Optional 1-byte ACK
        let mut ack = [0u8; 1];
        let _ = stream.read_exact(&mut ack);
        return;
    }

    // Fallback: send text protocol (line-based)
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));

    let mut payload = normalized_cmd.to_string();
    payload.push('\n');
    for arg in &args[1..] {
        payload.push_str(arg);
        payload.push('\n');
    }

    if stream.write_all(payload.as_bytes()).is_err() || stream.flush().is_err() {
        eprintln!("amane-msg: failed to send text command");
        std::process::exit(3);
    }

    // Read and print any response
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    if !response.trim().is_empty() {
        println!("{}", response.trim());
    }
}
