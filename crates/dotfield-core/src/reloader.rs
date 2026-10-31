use std::process::Command;

use crate::compiler::CompiledFragment;
use crate::error::Result;

pub struct Reloader;

impl Reloader {
    pub fn reload_changed_targets(fragments: &[CompiledFragment]) -> Result<Vec<String>> {
        let mut reloaded = Vec::new();

        let has_sway = fragments.iter().any(|f| f.target_path.starts_with("sway/"));
        let has_niri = fragments.iter().any(|f| f.target_path.starts_with("niri/"));
        let has_mako = fragments.iter().any(|f| f.target_path.starts_with("mako/"));
        let has_kitty = fragments
            .iter()
            .any(|f| f.target_path.starts_with("kitty/"));
        let has_tmux = fragments.iter().any(|f| f.target_path.starts_with("tmux/"));

        if has_sway && Self::run_cmd("swaymsg", &["reload"]) {
            reloaded.push("sway (swaymsg reload)".to_string());
        }

        if has_niri && Self::run_cmd("niri", &["msg", "action", "reload-config"]) {
            reloaded.push("niri (niri msg action reload-config)".to_string());
        }

        if has_mako && Self::run_cmd("makoctl", &["reload"]) {
            reloaded.push("mako (makoctl reload)".to_string());
        }

        if has_kitty
            && Self::run_cmd(
                "kitty",
                &["@", "set-colors", "--all", "~/.config/kitty/colors.conf"],
            )
        {
            reloaded.push("kitty (kitten @ set-colors)".to_string());
        }

        if has_tmux && Self::run_cmd("tmux", &["source-file", "~/.config/tmux/colors.conf"]) {
            reloaded.push("tmux (tmux source-file)".to_string());
        }

        Ok(reloaded)
    }

    fn run_cmd(program: &str, args: &[&str]) -> bool {
        Command::new(program)
            .args(args)
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }
}
