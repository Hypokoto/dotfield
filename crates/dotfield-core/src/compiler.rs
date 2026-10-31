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

        // Notifications
        fragments.push(Self::compile_mako_config(profile)?);

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
                "close_window" => "kill".to_string(),
                "fullscreen" => "fullscreen toggle".to_string(),
                "lock" => Self::sway_exec("swaylock -f"),
                "screenshot_area" => {
                    Self::sway_exec("~/.config/dotfield/scripts/screenshot.sh --area")
                }
                "screenshot_full" => {
                    Self::sway_exec("~/.config/dotfield/scripts/screenshot.sh --full")
                }
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
                "close_window" => "close-window;".to_string(),
                "fullscreen" => "fullscreen-window;".to_string(),
                "lock" => "spawn \"swaylock\" \"-f\";".to_string(),
                "screenshot_area" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/screenshot.sh --area\";"
                        .to_string()
                }
                "screenshot_full" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/screenshot.sh --full\";"
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
font=monospace 10
width=360
height=160
margin=16
padding=12
border-size=2
border-radius=4
default-timeout=6000
group-by=app-name,summary,urgency

background-color=#0A0A0C
text-color=#E8E6E3
border-color=#FFB000

[urgency=low]
background-color=#121216
text-color=#A1A1AA
border-color=#34343D
default-timeout=4000

[urgency=critical]
background-color=#121216
text-color=#E8E6E3
border-color=#E63946
default-timeout=0
ignore-timeout=1
"#;
        Ok(CompiledFragment {
            target_path: "mako/config".to_string(),
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
