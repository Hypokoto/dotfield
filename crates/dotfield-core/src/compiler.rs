use crate::error::Result;
use crate::profile::Profile;

pub struct CompiledFragment {
    pub target_path: String,
    pub content: String,
}

pub struct Compiler;

impl Compiler {
    pub fn compile(profile: &Profile) -> Result<Vec<CompiledFragment>> {
        let mut fragments = Vec::new();

        match profile.compositor.engine.as_str() {
            "sway" => {
                fragments.push(Self::compile_sway_bindings(profile)?);
            }
            "niri" => {
                fragments.push(Self::compile_niri_bindings(profile)?);
            }
            other => {
                return Err(crate::error::DotfieldError::Compilation(format!(
                    "Unsupported compositor engine: '{other}'"
                )));
            }
        }

        // Fuzzel launcher configuration
        fragments.push(Self::compile_fuzzel_colors(profile)?);

        // Terminals
        fragments.push(Self::compile_foot_colors(profile)?);
        fragments.push(Self::compile_kitty_colors(profile)?);
        fragments.push(Self::compile_alacritty_colors(profile)?);
        fragments.push(Self::compile_ghostty_colors(profile)?);

        // Notifications
        fragments.push(Self::compile_mako_config(profile)?);

        // Terminal Multiplexers
        fragments.push(Self::compile_tmux_colors(profile)?);
        fragments.push(Self::compile_zellij_theme(profile)?);

        Ok(fragments)
    }

    fn compile_sway_bindings(profile: &Profile) -> Result<CompiledFragment> {
        let mut content = String::from("# Dotfield Generated Sway Keybindings\n");
        content.push_str("# Atomically compiled by Dotfield. Do not edit directly.\n\n");

        for (action, keybind) in &profile.bindings {
            let sway_key = keybind.replace("Super+", "Mod4+");
            let cmd = match action.as_str() {
                "terminal" => Self::sway_exec(&profile.apps.terminal),
                "launcher" => Self::sway_exec(&profile.apps.launcher),
                "app_menu" => Self::sway_exec("rofi -show drun"),
                "hud" => Self::sway_exec("dotfield-gui --hud"),
                "profile_switcher" => Self::sway_exec("dotfield-gui --switcher"),
                "close_window" => "kill".to_string(),
                "fullscreen" => "fullscreen toggle".to_string(),
                "lock" => Self::sway_exec("~/.config/dotfield/scripts/lock.sh"),
                "screenshot_area" => {
                    Self::sway_exec("~/.config/dotfield/scripts/screenshot.sh --area")
                }
                "screenshot_full" => {
                    Self::sway_exec("~/.config/dotfield/scripts/screenshot.sh --full")
                }
                "volume_up" => Self::sway_exec("~/.config/dotfield/scripts/osd.sh volume_up"),
                "volume_down" => Self::sway_exec("~/.config/dotfield/scripts/osd.sh volume_down"),
                "volume_mute" => Self::sway_exec("~/.config/dotfield/scripts/osd.sh volume_mute"),
                "mic_mute" => Self::sway_exec("~/.config/dotfield/scripts/osd.sh mic_mute"),
                "brightness_up" => {
                    Self::sway_exec("~/.config/dotfield/scripts/osd.sh brightness_up")
                }
                "brightness_down" => {
                    Self::sway_exec("~/.config/dotfield/scripts/osd.sh brightness_down")
                }
                "media_play_pause" => {
                    Self::sway_exec("~/.config/dotfield/scripts/osd.sh media_play_pause")
                }
                "media_next" => Self::sway_exec("~/.config/dotfield/scripts/osd.sh media_next"),
                "media_prev" => Self::sway_exec("~/.config/dotfield/scripts/osd.sh media_prev"),
                action if action.starts_with("exec:") => Self::sway_exec(&action[5..]),
                other => {
                    return Err(crate::error::DotfieldError::Compilation(format!(
                        "Unknown keybinding action: '{other}'"
                    )));
                }
            };
            content.push_str(&format!("bindsym --no-warn {sway_key} {cmd}\n"));
        }

        Ok(CompiledFragment {
            target_path: "sway/00-generated-bindings.conf".to_string(),
            content,
        })
    }

    fn compile_niri_bindings(profile: &Profile) -> Result<CompiledFragment> {
        let mut content = String::from("// Dotfield Generated Niri Keybindings\n");
        content.push_str("// Atomically compiled by Dotfield. Do not edit directly.\n\n");
        content.push_str("binds {\n");

        for (action, keybind) in &profile.bindings {
            let niri_key = keybind.replace("Super+", "Mod+");
            let statement = match action.as_str() {
                "terminal" => Self::niri_spawn(&profile.apps.terminal),
                "launcher" => Self::niri_spawn(&profile.apps.launcher),
                "app_menu" => "spawn \"rofi\" \"-show\" \"drun\";".to_string(),
                "hud" => "spawn \"dotfield-gui\" \"--hud\";".to_string(),
                "profile_switcher" => "spawn \"dotfield-gui\" \"--switcher\";".to_string(),
                "close_window" => "close-window;".to_string(),
                "fullscreen" => "fullscreen-window;".to_string(),
                "lock" => "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/lock.sh\";".to_string(),
                "screenshot_area" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/screenshot.sh --area\";"
                        .to_string()
                }
                "screenshot_full" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/screenshot.sh --full\";"
                        .to_string()
                }
                "volume_up" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh volume_up\";"
                        .to_string()
                }
                "volume_down" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh volume_down\";"
                        .to_string()
                }
                "volume_mute" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh volume_mute\";"
                        .to_string()
                }
                "mic_mute" => "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh mic_mute\";"
                    .to_string(),
                "brightness_up" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh brightness_up\";"
                        .to_string()
                }
                "brightness_down" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh brightness_down\";"
                        .to_string()
                }
                "media_play_pause" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh media_play_pause\";"
                        .to_string()
                }
                "media_next" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh media_next\";"
                        .to_string()
                }
                "media_prev" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/osd.sh media_prev\";"
                        .to_string()
                }
                action if action.starts_with("exec:") => {
                    let cmd = &action[5..];
                    format!(
                        "spawn \"sh\" \"-c\" \"{}\";",
                        cmd.replace('\\', "\\\\").replace('"', "\\\"")
                    )
                }
                other => {
                    return Err(crate::error::DotfieldError::Compilation(format!(
                        "Unknown keybinding action: '{other}'"
                    )));
                }
            };
            content.push_str(&format!("    {niri_key} {{ {statement} }}\n"));
        }

        content.push_str("}\n");

        Ok(CompiledFragment {
            target_path: "niri/00-generated-bindings.kdl".to_string(),
            content,
        })
    }

    pub(crate) fn split_command_args(cmd: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_single = false;
        let mut in_double = false;
        let mut escaped = false;
        let mut token_active = false;

        for c in cmd.chars() {
            if escaped {
                current.push(c);
                token_active = true;
                escaped = false;
                continue;
            }

            match c {
                '\\' if !in_single => {
                    escaped = true;
                    token_active = true;
                }
                '\'' if !in_double => {
                    in_single = !in_single;
                    token_active = true;
                }
                '"' if !in_single => {
                    in_double = !in_double;
                    token_active = true;
                }
                c if c.is_whitespace() && !in_single && !in_double => {
                    if token_active {
                        tokens.push(std::mem::take(&mut current));
                        token_active = false;
                    }
                }
                _ => {
                    current.push(c);
                    token_active = true;
                }
            }
        }

        if escaped {
            current.push('\\');
        }
        if token_active {
            tokens.push(current);
        }

        tokens
    }

    fn niri_spawn(cmd: &str) -> String {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return "spawn \"\";".to_string();
        }

        // If the command contains shell syntax, metacharacters, environment variables ($),
        // tilde expansion (~), or leading VAR=value environment assignments,
        // route it through a shell wrapper so Niri expands them rather than passing them literally.
        const SHELL_META: &[char] = &[
            '$', '~', '&', '|', ';', '<', '>', '*', '?', '`', '(', ')', '{', '}', '\n',
        ];

        let has_leading_env = trimmed
            .split_whitespace()
            .next()
            .is_some_and(|first| first.contains('=') && !first.starts_with('-'));

        if trimmed.contains(SHELL_META) || has_leading_env {
            format!(
                "spawn \"sh\" \"-c\" \"{}\";",
                trimmed.replace('\\', "\\\\").replace('"', "\\\"")
            )
        } else {
            let parts = Self::split_command_args(trimmed);
            let quoted = parts
                .iter()
                .map(|p| format!("\"{}\"", p.replace('\\', "\\\\").replace('"', "\\\"")))
                .collect::<Vec<_>>()
                .join(" ");
            format!("spawn {quoted};")
        }
    }

    fn compile_fuzzel_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos color palette fragment for Fuzzel
[colors]
background=121216fa
text=e8e6e3ff
match=e63946ff
selection=1c1c23ff
selection-text=ffd166ff
selection-match=ffb000ff
border=ffb000ff
"#;
        Ok(CompiledFragment {
            target_path: "fuzzel/colors.ini".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_foot_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos color palette fragment for Foot
[colors-dark]
background=0A0A0C
foreground=E8E6E3
regular0=1C1C23
regular1=E63946
regular2=7BC47F
regular3=FFB000
regular4=82AAFF
regular5=C792EA
regular6=8B949E
regular7=E8E6E3
bright0=53535D
bright1=FF6B76
bright2=9AE6A0
bright3=FFD166
bright4=A4C0FF
bright5=D9ABF4
bright6=B0B8C1
bright7=FFFFFF

# Selection & cursor
cursor=0A0A0C FFB000
selection-foreground=E8E6E3
selection-background=1C1C23
"#;
        Ok(CompiledFragment {
            target_path: "foot/colors.ini".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_mako_config(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos styling for Mako
font=JetBrainsMono Nerd Font 10
width=380
height=180
margin=16
padding=12,16
border-size=2
border-radius=8
layer=overlay
max-visible=5
default-timeout=6000
group-by=app-name,summary,urgency

format=<b>%s</b>\n%b
markup=1
actions=1

icons=1
max-icon-size=48
icon-location=left

background-color=#0A0A0C
text-color=#E8E6E3
border-color=#FFB000
progress-color=source #FFB000

[urgency=low]
background-color=#121216
text-color=#A1A1AA
border-color=#34343D
progress-color=source #8B949E
default-timeout=4000

[urgency=critical]
background-color=#121216
text-color=#E8E6E3
border-color=#E63946
progress-color=source #E63946
default-timeout=0
ignore-timeout=1

[category=volume]
history=0
default-timeout=2000

[category=brightness]
history=0
default-timeout=2000

[category=osd]
history=0
default-timeout=2000

[app-name=dotfield]
border-color=#FFB000

[app-name=Screenshot]
border-color=#FFD166

[mode=do-not-disturb]
invisible=1
"#;
        Ok(CompiledFragment {
            target_path: "mako/config".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_kitty_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos color palette fragment for Kitty
background #0A0A0C
foreground #E8E6E3
cursor #FFB000
cursor_text_color #0A0A0C
selection_background #1C1C23
selection_foreground #E8E6E3

# Normal colors
color0 #1C1C23
color1 #E63946
color2 #7BC47F
color3 #FFB000
color4 #82AAFF
color5 #C792EA
color6 #8B949E
color7 #E8E6E3

# Bright colors
color8 #53535D
color9 #FF6B76
color10 #9AE6A0
color11 #FFD166
color12 #A4C0FF
color13 #D9ABF4
color14 #B0B8C1
color15 #FFFFFF
"#;
        Ok(CompiledFragment {
            target_path: "kitty/colors.conf".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_alacritty_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r##"# Controlled Chaos color palette fragment for Alacritty
[colors.primary]
background = "#0A0A0C"
foreground = "#E8E6E3"

[colors.cursor]
text = "#0A0A0C"
cursor = "#FFB000"

[colors.selection]
text = "#E8E6E3"
background = "#1C1C23"

[colors.normal]
black = "#1C1C23"
red = "#E63946"
green = "#7BC47F"
yellow = "#FFB000"
blue = "#82AAFF"
magenta = "#C792EA"
cyan = "#8B949E"
white = "#E8E6E3"

[colors.bright]
black = "#53535D"
red = "#FF6B76"
green = "#9AE6A0"
yellow = "#FFD166"
blue = "#A4C0FF"
magenta = "#D9ABF4"
cyan = "#B0B8C1"
white = "#FFFFFF"
"##;
        Ok(CompiledFragment {
            target_path: "alacritty/colors.toml".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_ghostty_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos color palette fragment for Ghostty
palette = 0=#1C1C23
palette = 1=#E63946
palette = 2=#7BC47F
palette = 3=#FFB000
palette = 4=#82AAFF
palette = 5=#C792EA
palette = 6=#8B949E
palette = 7=#E8E6E3
palette = 8=#53535D
palette = 9=#FF6B76
palette = 10=#9AE6A0
palette = 11=#FFD166
palette = 12=#A4C0FF
palette = 13=#D9ABF4
palette = 14=#B0B8C1
palette = 15=#FFFFFF
background = #0A0A0C
foreground = #E8E6E3
cursor-color = #FFB000
cursor-text = #0A0A0C
selection-background = #1C1C23
selection-foreground = #E8E6E3
"#;
        Ok(CompiledFragment {
            target_path: "ghostty/themes/dotfield-theme".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_tmux_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r##"# Controlled Chaos styling for Tmux
# Generated by Dotfield — Void Black · Crimson · Industrial Amber · Steel

set -g status-position top
set -g status-justify left
set -g status-style "bg=#121216,fg=#A1A1AA"
set -g status-left-length 40
set -g status-right-length 80

set -g status-left "#[bg=#FFB000,fg=#0A0A0C,bold]  #S #[bg=#121216,fg=#FFB000,nobold] "
set -g status-right "#[fg=#34343D]│ #[fg=#8B949E] %H:%M #[fg=#34343D]│ #[fg=#E8E6E3] %d-%b-%y "

setw -g window-status-style "fg=#A1A1AA,bg=#121216"
setw -g window-status-current-style "fg=#E8E6E3,bg=#1C1C23,bold"
setw -g window-status-format " #[fg=#8B949E]#I:#[fg=#A1A1AA]#W "
setw -g window-status-current-format " #[fg=#FFB000]#I:#[fg=#E8E6E3]#W  "
setw -g window-status-separator ""

set -g pane-border-style "fg=#34343D"
set -g pane-active-border-style "fg=#FFB000"

set -g message-style "bg=#1C1C23,fg=#FFB000,bold"
set -g message-command-style "bg=#1C1C23,fg=#FFD166,bold"
set -g mode-style "bg=#1C1C23,fg=#FFD166,bold"
setw -g clock-mode-colour "#FFB000"
"##;
        Ok(CompiledFragment {
            target_path: "tmux/colors.conf".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_zellij_theme(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r##"// Controlled Chaos color theme for Zellij
// Generated by Dotfield — Void Black · Crimson · Industrial Amber · Steel

themes {
    dotfield {
        fg "#E8E6E3"
        bg "#0A0A0C"
        black "#1C1C23"
        red "#E63946"
        green "#7BC47F"
        yellow "#FFB000"
        blue "#82AAFF"
        magenta "#C792EA"
        cyan "#8B949E"
        white "#E8E6E3"
        orange "#FFB000"
    }
}
"##;
        Ok(CompiledFragment {
            target_path: "zellij/themes/dotfield.kdl".to_string(),
            content: content.to_string(),
        })
    }

    fn sway_exec(cmd: &str) -> String {
        format!(
            "exec \"{}\"",
            cmd.trim().replace('\\', "\\\\").replace('"', "\\\"")
        )
    }
}
