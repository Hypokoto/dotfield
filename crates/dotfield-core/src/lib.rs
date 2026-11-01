pub mod compiler;
pub mod diff;
pub mod error;
pub mod generation;
pub mod profile;
pub mod reloader;
pub mod validator;

pub use compiler::{CompiledFragment, Compiler};
pub use diff::DiffEngine;
pub use error::{DotfieldError, Result};
pub use generation::GenerationManager;
pub use profile::Profile;
pub use reloader::Reloader;
pub use validator::Validator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_parsing() {
        let toml_data = r#"
[meta]
name = "test"
description = "A test profile"
version = "1.0.0"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
launcher = "Super+space"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse test profile");
        assert_eq!(profile.meta.name, "test");
        assert_eq!(profile.apps.terminal, "foot");
        assert_eq!(profile.bindings.get("terminal").unwrap(), "Super+Return");
    }

    #[test]
    fn test_compiler_output() {
        let toml_data = r#"
[meta]
name = "test"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile fragments");
        assert!(!fragments.is_empty());
        let sway_frag = fragments
            .iter()
            .find(|f| f.target_path.contains("sway"))
            .expect("Must have sway fragment");
        assert!(sway_frag
            .content
            .contains("bindsym --no-warn Mod4+Return exec \"foot\""));
    }

    #[test]
    fn test_compiler_output_niri() {
        let toml_data = r#"
[meta]
name = "test-niri"

[compositor]
engine = "niri"

[apps]
terminal = "foot --server"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
fullscreen = "Super+f"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile fragments");
        let niri_frag = fragments
            .iter()
            .find(|f| f.target_path.contains("niri"))
            .expect("Must have niri fragment");
        assert!(niri_frag
            .content
            .contains("Mod+Return { spawn \"foot\" \"--server\"; }"));
        assert!(niri_frag.content.contains("Mod+f { fullscreen-window; }"));
    }

    #[test]
    fn test_validator_rejects_unknown_binding_action() {
        let toml_data = r#"
[meta]
name = "test-typo"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminl = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let result = Validator::validate(&profile);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown keybinding action 'terminl'"));
    }

    #[test]
    fn test_validator_rejects_unsupported_palette() {
        let toml_data = r#"
[meta]
name = "test-palette"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "nord"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let result = Validator::validate(&profile);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported theme palette: 'nord'"));
    }

    #[test]
    fn test_compiler_exec_action() {
        let toml_data = r#"
[meta]
name = "test-exec"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
"exec:pkill waybar" = "Super+w"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        assert!(Validator::validate(&profile).is_ok());
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let sway = fragments
            .iter()
            .find(|f| f.target_path.contains("sway"))
            .unwrap();
        assert!(sway
            .content
            .contains("bindsym --no-warn Mod4+w exec \"pkill waybar\""));
    }

    #[test]
    fn test_foot_colors_fragment() {
        let toml_data = r#"
[meta]
name = "test-foot"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let foot = fragments
            .iter()
            .find(|f| f.target_path == "foot/colors.ini")
            .expect("Must have foot fragment");
        assert!(foot.content.contains("regular2=7BC47F"));
        assert!(foot.content.contains("bright2=9AE6A0"));
        assert!(foot.content.contains("cursor=0A0A0C FFB000"));
        assert!(foot.content.contains("selection-foreground=E8E6E3"));
        assert!(foot.content.contains("selection-background=1C1C23"));
    }

    #[test]
    fn test_kitty_colors_fragment() {
        let toml_data = r#"
[meta]
name = "test-kitty"

[compositor]
engine = "sway"

[apps]
terminal = "kitty"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let kitty = fragments
            .iter()
            .find(|f| f.target_path == "kitty/colors.conf")
            .expect("Must have kitty fragment");
        assert!(kitty.content.contains("background #0A0A0C"));
        assert!(kitty.content.contains("foreground #E8E6E3"));
        assert!(kitty.content.contains("cursor #FFB000"));
        assert!(kitty.content.contains("color1 #E63946"));
        assert!(kitty.content.contains("color2 #7BC47F"));
    }

    #[test]
    fn test_alacritty_colors_fragment() {
        let toml_data = r#"
[meta]
name = "test-alacritty"

[compositor]
engine = "sway"

[apps]
terminal = "alacritty"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let alacritty = fragments
            .iter()
            .find(|f| f.target_path == "alacritty/colors.toml")
            .expect("Must have alacritty fragment");
        assert!(alacritty.content.contains("background = \"#0A0A0C\""));
        assert!(alacritty.content.contains("cursor = \"#FFB000\""));
        assert!(alacritty.content.contains("red = \"#E63946\""));
        assert!(alacritty.content.contains("green = \"#7BC47F\""));
    }

    #[test]
    fn test_ghostty_colors_fragment() {
        let toml_data = r#"
[meta]
name = "test-ghostty"

[compositor]
engine = "sway"

[apps]
terminal = "ghostty"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let ghostty = fragments
            .iter()
            .find(|f| f.target_path == "ghostty/themes/dotfield-theme")
            .expect("Must have ghostty fragment");
        assert!(ghostty.content.contains("background = #0A0A0C"));
        assert!(ghostty.content.contains("cursor-color = #FFB000"));
        assert!(ghostty.content.contains("palette = 1=#E63946"));
        assert!(ghostty.content.contains("palette = 2=#7BC47F"));
    }

    #[test]
    fn test_media_and_osd_keybindings() {
        let toml_data = r#"
[meta]
name = "test-media"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
volume_up = "XF86AudioRaiseVolume"
volume_down = "XF86AudioLowerVolume"
profile_switcher = "Super+p"
lock = "Super+Escape"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        assert!(Validator::validate(&profile).is_ok());
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let sway = fragments
            .iter()
            .find(|f| f.target_path.contains("sway"))
            .unwrap();
        assert!(sway
            .content
            .contains("bindsym --no-warn XF86AudioRaiseVolume exec \"~/.config/dotfield/scripts/osd.sh volume_up\""));
        assert!(sway
            .content
            .contains("bindsym --no-warn Mod4+p exec \"dotfield-gui --switcher\""));
        assert!(sway
            .content
            .contains("bindsym --no-warn Mod4+Escape exec \"~/.config/dotfield/scripts/lock.sh\""));
    }

    #[test]
    fn test_tmux_colors_fragment() {
        let toml_data = r#"
[meta]
name = "test-tmux"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"
multiplexer = "tmux"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let tmux = fragments
            .iter()
            .find(|f| f.target_path == "tmux/colors.conf")
            .expect("Must have tmux fragment");
        assert!(tmux
            .content
            .contains("set -g status-style \"bg=#121216,fg=#A1A1AA\""));
        assert!(tmux
            .content
            .contains("set -g pane-active-border-style \"fg=#FFB000\""));
        assert!(tmux
            .content
            .contains("set -g pane-border-style \"fg=#34343D\""));
    }

    #[test]
    fn test_zellij_theme_fragment() {
        let toml_data = r#"
[meta]
name = "test-zellij"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"
multiplexer = "zellij"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let zellij = fragments
            .iter()
            .find(|f| f.target_path == "zellij/themes/dotfield.kdl")
            .expect("Must have zellij fragment");
        assert!(zellij.content.contains("fg \"#E8E6E3\""));
        assert!(zellij.content.contains("bg \"#0A0A0C\""));
        assert!(zellij.content.contains("yellow \"#FFB000\""));
        assert!(zellij.content.contains("red \"#E63946\""));
    }

    #[test]
    fn test_nvim_palette_fragment() {
        let toml_data = r#"
[meta]
name = "test-nvim"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"
editor = "nvim"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let nvim_pal = fragments
            .iter()
            .find(|f| f.target_path == "nvim/lua/palette_generated.lua")
            .expect("Must have nvim palette fragment");
        assert!(nvim_pal.content.contains("void     = \"#0A0A0C\""));
        assert!(nvim_pal.content.contains("amber    = \"#FFB000\""));

        let nvim_theme = fragments
            .iter()
            .find(|f| f.target_path == "nvim/lua/themes/dotfield.lua")
            .expect("Must have nvim base46 theme fragment");
        assert!(nvim_theme.content.contains("white         = \"#E8E6E3\""));
        assert!(nvim_theme.content.contains("red           = \"#E63946\""));
    }

    #[test]
    fn test_niri_quoted_exec_action() {
        let toml_data = r#"
[meta]
name = "test-niri-exec"

[compositor]
engine = "niri"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
"exec:notify-send \"hello world\"" = "Super+n"
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let fragments = Compiler::compile(&profile).expect("Must compile");
        let niri = fragments
            .iter()
            .find(|f| f.target_path.contains("niri"))
            .expect("Must have niri fragment");
        assert!(niri
            .content
            .contains("Mod+n { spawn \"sh\" \"-c\" \"notify-send \\\"hello world\\\"\"; }"));
    }

    #[test]
    fn test_validator_warns_on_security_options() {
        let toml_data = r#"
[meta]
name = "test-security-warn"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"

[security]
firejail = true
nftables = true
apparmor = true
"#;
        let profile: Profile = toml_data.parse().expect("Must parse");
        let issues = Validator::validate(&profile).expect("Validation succeeds with warnings");
        assert!(issues
            .iter()
            .any(|i| i.message.contains("Security option 'firejail' is declared")));
        assert!(issues
            .iter()
            .any(|i| i.message.contains("Security option 'nftables' is declared")));
        assert!(issues
            .iter()
            .any(|i| i.message.contains("Security option 'apparmor' is declared")));
    }

    #[test]
    fn test_resolve_profile_path_explicit_missing() {
        let explicit = std::path::Path::new("./custom/missing/work.toml");
        let resolved = Profile::resolve_profile_path(explicit);
        assert_eq!(resolved, explicit);
    }

    #[test]
    fn test_resolve_profile_path_nested_missing() {
        let nested = std::path::Path::new("profiles/team/default.toml");
        let resolved = Profile::resolve_profile_path(nested);
        assert_eq!(resolved, nested);
    }

    #[test]
    fn test_split_command_args_preserves_empty_arguments() {
        let cmd = "my_tool --name \"\" --flag 'value'";
        let args = Compiler::split_command_args(cmd);
        assert_eq!(args, vec!["my_tool", "--name", "", "--flag", "value"]);
    }

    #[test]
    fn test_get_state_self_heals_from_current_meta() {
        use tempfile::tempdir;
        let dir = tempdir().expect("tempdir");
        let mgr = GenerationManager::new(dir.path());

        // Create generation directory with meta.json and current symlink
        let gen_1 = dir.path().join("generations").join("1");
        std::fs::create_dir_all(&gen_1).expect("create gen dir");
        let meta = r#"{"generation":1,"profile_name":"default","timestamp":"2026-10-31T12:00:00Z","fragment_count":3}"#;
        std::fs::write(gen_1.join("meta.json"), meta).expect("write meta");

        let current = dir.path().join("current");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&gen_1, &current).expect("symlink current");

        // get_state with missing state.json should self-heal from current/meta.json
        let state = mgr
            .get_state()
            .expect("get_state")
            .expect("self-healed state");
        assert_eq!(state.current_generation, 1);
        assert_eq!(state.active_profile, "default");
    }

    #[test]
    fn test_profile_rejects_unknown_fields() {
        let toml_data = r#"
[meta]
name = "test"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"
font_szie = 12

[bindings]
terminal = "Super+Return"
"#;
        let res: std::result::Result<Profile, _> = toml_data.parse();
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("unknown field `font_szie`"));
    }

    #[test]
    fn test_validator_detects_duplicate_keybindings() {
        let toml_data = r#"
[meta]
name = "test-dup"

[compositor]
engine = "sway"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
terminal = "Super+Return"
launcher = "Mod4+Return"
"#;
        let profile: Profile = toml_data.parse().expect("parse");
        let res = Validator::validate(&profile);
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("Duplicate keybinding"));
    }

    #[test]
    fn test_niri_spawn_shell_metacharacters() {
        let toml_data = r#"
[meta]
name = "test-niri-meta"

[compositor]
engine = "niri"

[apps]
terminal = "foot"
launcher = "fuzzel"

[theme]
palette = "controlled-chaos"

[bindings]
"exec:FOO=bar foot" = "Super+1"
"exec:cat < file | grep foo > out" = "Super+2"
"#;
        let profile: Profile = toml_data.parse().expect("parse");
        let fragments = Compiler::compile(&profile).expect("compile");
        let niri = fragments
            .iter()
            .find(|f| f.target_path.contains("niri"))
            .unwrap();
        assert!(niri
            .content
            .contains("spawn \"sh\" \"-c\" \"FOO=bar foot\""));
        assert!(niri
            .content
            .contains("spawn \"sh\" \"-c\" \"cat < file | grep foo > out\""));
    }

    #[test]
    fn test_diff_detects_removed_fragments() {
        use tempfile::tempdir;
        let dir = tempdir().expect("tempdir");
        let sway_fragment = dir.path().join("sway").join("00-generated-bindings.conf");
        std::fs::create_dir_all(sway_fragment.parent().unwrap()).unwrap();
        std::fs::write(&sway_fragment, "bindsym Mod4+Return exec foot\n").unwrap();

        // Candidate profile has no sway fragment
        let diffs = DiffEngine::diff_fragments(Some(dir.path()), &[]).expect("diff");
        assert_eq!(diffs.len(), 1);
        assert!(diffs[0].path.contains("00-generated-bindings.conf"));
        assert!(diffs[0]
            .diff_text
            .contains("- bindsym Mod4+Return exec foot"));
    }
}
