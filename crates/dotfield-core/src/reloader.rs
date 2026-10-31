use std::process::Command;

use crate::compiler::CompiledFragment;
use crate::error::Result;

pub struct Reloader;

impl Reloader {
    pub fn reload_changed_targets(fragments: &[CompiledFragment]) -> Result<Vec<String>> {
        let mut reloaded = Vec::new();
        let mut errors = Vec::new();

        let has_sway = fragments.iter().any(|f| f.target_path.starts_with("sway/"));
        let has_niri = fragments.iter().any(|f| f.target_path.starts_with("niri/"));
        let has_mako = fragments.iter().any(|f| f.target_path.starts_with("mako/"));
        let has_kitty = fragments
            .iter()
            .any(|f| f.target_path.starts_with("kitty/"));
        let has_tmux = fragments.iter().any(|f| f.target_path.starts_with("tmux/"));

        let env_set = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty());

        if has_sway && env_set("SWAYSOCK") {
            match Self::run_cmd("swaymsg", &["reload"]) {
                Ok(()) => reloaded.push("sway (swaymsg reload)".to_string()),
                Err(e) => errors.push(format!("sway: {e}")),
            }
        }

        if has_niri && env_set("NIRI_SOCKET") {
            match Self::run_cmd("niri", &["msg", "action", "load-config-file"]) {
                Ok(()) => reloaded.push("niri (niri msg action load-config-file)".to_string()),
                Err(e) => errors.push(format!("niri: {e}")),
            }
        }

        if has_mako && env_set("WAYLAND_DISPLAY") {
            // Only attempt to reload mako if the mako daemon is running
            let mako_running = Command::new("pgrep")
                .args(["-x", "mako"])
                .output()
                .map(|out| out.status.success())
                .unwrap_or(false);

            if mako_running {
                match Self::run_cmd("makoctl", &["reload"]) {
                    Ok(()) => reloaded.push("mako (makoctl reload)".to_string()),
                    Err(e) => errors.push(format!("mako: {e}")),
                }
            }
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

        if has_kitty {
            if let Some(runtime_dir) = std::env::var("XDG_RUNTIME_DIR")
                .ok()
                .filter(|path| !path.is_empty() && std::path::Path::new(path).is_absolute())
            {
                let mut candidate_sockets: Vec<std::path::PathBuf> = Vec::new();
                if let Ok(listen_on) = std::env::var("KITTY_LISTEN_ON") {
                    if let Some(path) = listen_on.strip_prefix("unix:") {
                        candidate_sockets.push(std::path::PathBuf::from(path));
                    }
                }

                // Add discovered PID-suffixed or default sockets
                if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .map(|name| {
                                name.starts_with("kitty-dotfield.sock-")
                                    || name == "kitty-dotfield.sock"
                            })
                            .unwrap_or(false)
                            && !candidate_sockets.contains(&path)
                        {
                            candidate_sockets.push(path);
                        }
                    }
                }

                let kitty_conf = format!("{home}/.config/kitty/colors.conf");
                let mut kitty_reloaded = false;
                let mut last_kitty_err = None;

                for socket_path in candidate_sockets {
                    if !socket_path.exists() {
                        continue;
                    }
                    let socket_addr = format!("unix:{}", socket_path.display());
                    match Self::run_cmd(
                        "kitty",
                        &[
                            "@",
                            "--to",
                            &socket_addr,
                            "set-colors",
                            "--all",
                            "--configured",
                            &kitty_conf,
                        ],
                    ) {
                        Ok(()) => {
                            kitty_reloaded = true;
                            break;
                        }
                        Err(e) => {
                            last_kitty_err = Some(e);
                        }
                    }
                }

                if kitty_reloaded {
                    reloaded.push("kitty (kitten @ set-colors)".to_string());
                } else if let Some(e) = last_kitty_err {
                    errors.push(format!("kitty: {e}"));
                }
            }
        }

        if has_tmux && env_set("TMUX") {
            let tmux_conf = format!("{home}/.config/tmux/colors.conf");
            match Self::run_cmd("tmux", &["source-file", &tmux_conf]) {
                Ok(()) => reloaded.push("tmux (tmux source-file)".to_string()),
                Err(e) => errors.push(format!("tmux: {e}")),
            }
        }

        if !errors.is_empty() {
            let msg = if reloaded.is_empty() {
                errors.join("; ")
            } else {
                format!(
                    "Errors reloading targets: {} (successfully reloaded: {})",
                    errors.join("; "),
                    reloaded.join(", ")
                )
            };
            return Err(crate::error::DotfieldError::Reload(msg));
        }

        Ok(reloaded)
    }

    fn run_cmd(program: &str, args: &[&str]) -> std::result::Result<(), String> {
        match Command::new(program).args(args).output() {
            Ok(out) => {
                if out.status.success() {
                    Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                    let msg = if stderr.is_empty() {
                        format!("exit code: {}", out.status.code().unwrap_or(-1))
                    } else {
                        stderr
                    };
                    Err(format!("'{program}' failed: {msg}"))
                }
            }
            Err(e) => Err(format!("failed to execute '{program}': {e}")),
        }
    }
}
