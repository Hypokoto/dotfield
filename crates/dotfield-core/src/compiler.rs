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
            _ => {}
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
                "terminal" => format!("exec {}", profile.apps.terminal),
                "launcher" => format!("exec {}", profile.apps.launcher),
                "hud" => "exec dotfield-gui --hud".to_string(),
                "close_window" => "kill".to_string(),
                "fullscreen" => "fullscreen toggle".to_string(),
                "lock" => "exec swaylock -f".to_string(),
                "screenshot_area" => {
                    "exec ~/.config/dotfield/scripts/screenshot.sh --area".to_string()
                }
                "screenshot_full" => {
                    "exec ~/.config/dotfield/scripts/screenshot.sh --full".to_string()
                }
                other => format!("exec {other}"),
            };
            content.push_str(&format!("bindsym {sway_key} {cmd}\n"));
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
                "terminal" => format!("spawn \"{}\";", profile.apps.terminal),
                "launcher" => format!("spawn \"{}\";", profile.apps.launcher),
                "hud" => "spawn \"dotfield-gui\" \"--hud\";".to_string(),
                "close_window" => "close-window;".to_string(),
                "fullscreen" => "maximize-column;".to_string(),
                "lock" => "spawn \"swaylock\" \"-f\";".to_string(),
                "screenshot_area" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/screenshot.sh --area\";"
                        .to_string()
                }
                "screenshot_full" => {
                    "spawn \"sh\" \"-c\" \"~/.config/dotfield/scripts/screenshot.sh --full\";"
                        .to_string()
                }
                other => format!("spawn \"{other}\";"),
            };
            content.push_str(&format!("    {niri_key} {{ {statement} }}\n"));
        }

        content.push_str("}\n");

        Ok(CompiledFragment {
            target_path: "niri/00-generated-bindings.kdl".to_string(),
            content,
        })
    }

    fn compile_fuzzel_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos color palette fragment for Fuzzel
[colors]
background=0A0A0Cee
text=E8E6E3ff
match=FFB000ff
selection=1C1C23ff
selection-text=E8E6E3ff
border=FFB000ff
"#;
        Ok(CompiledFragment {
            target_path: "fuzzel/colors.ini".to_string(),
            content: content.to_string(),
        })
    }

    fn compile_foot_colors(_profile: &Profile) -> Result<CompiledFragment> {
        let content = r#"# Controlled Chaos color palette fragment for Foot
[colors]
background=0A0A0C
foreground=E8E6E3
regular0=121216
regular1=E63946
regular2=8B949E
regular3=FFB000
regular4=8B949E
regular5=A1A1AA
regular6=34343D
regular7=E8E6E3
bright0=34343D
bright1=E63946
bright2=8B949E
bright3=FFB000
bright4=8B949E
bright5=A1A1AA
bright6=A1A1AA
bright7=E8E6E3
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
group-by=summary

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
"#;
        Ok(CompiledFragment {
            target_path: "mako/config".to_string(),
            content: content.to_string(),
        })
    }
}
