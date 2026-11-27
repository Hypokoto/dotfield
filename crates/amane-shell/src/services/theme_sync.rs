use crate::services::palettes::PaletteSpec;
use std::fs;
use std::path::Path;
use std::process::Command;

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let s = hex.trim_start_matches('#');
    if s.len() < 6 {
        return (0, 0, 0);
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    (r, g, b)
}

fn strip_hash(hex: &str) -> &str {
    hex.trim_start_matches('#')
}

fn mix_hex(a: &str, b: &str, t: f32) -> String {
    let (ar, ag, ab) = parse_hex(a);
    let (br, bg, bb) = parse_hex(b);
    let mix = |x: u8, y: u8| -> u8 {
        ((x as f32) * (1.0 - t) + (y as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    format!("#{:02X}{:02X}{:02X}", mix(ar, br), mix(ag, bg), mix(ab, bb))
}

fn contrast_fg(bg_hex: &str) -> String {
    let (r, g, b) = parse_hex(bg_hex);
    let lum = 0.2126 * (r as f32) + 0.7152 * (g as f32) + 0.0722 * (b as f32);
    if lum > 140.0 {
        "#111111".to_string()
    } else {
        "#FFFFFF".to_string()
    }
}

struct Ansi16 {
    c0: String,
    c1: String,
    c2: String,
    c3: String,
    c4: String,
    c5: String,
    c6: String,
    c7: String,
    c8: String,
    c9: String,
    c10: String,
    c11: String,
    c12: String,
    c13: String,
    c14: String,
    c15: String,
    on_accent: String,
    on_danger: String,
}

impl Ansi16 {
    fn from_spec(spec: &PaletteSpec) -> Self {
        let boost = if spec.is_dark { "#FFFFFF" } else { "#000000" };
        let purple = mix_hex(spec.accent, spec.danger, 0.45);
        let cyan = mix_hex(spec.accent, spec.success, 0.35);

        let (c0, c7, c8, c15) = if spec.is_dark {
            (
                spec.surface.to_string(),
                spec.fg.to_string(),
                mix_hex(spec.fg_muted, spec.fg, 0.25),
                mix_hex(spec.fg, "#FFFFFF", 0.30),
            )
        } else {
            (
                spec.fg.to_string(),
                spec.overlay.to_string(),
                spec.fg_muted.to_string(),
                spec.surface.to_string(),
            )
        };

        Self {
            c0,
            c1: spec.danger.to_string(),
            c2: spec.success.to_string(),
            c3: spec.warning.to_string(),
            c4: spec.accent.to_string(),
            c5: purple.clone(),
            c6: cyan.clone(),
            c7,
            c8,
            c9: mix_hex(spec.danger, boost, 0.20),
            c10: mix_hex(spec.success, boost, 0.20),
            c11: mix_hex(spec.warning, boost, 0.20),
            c12: mix_hex(spec.accent, boost, 0.20),
            c13: mix_hex(&purple, boost, 0.20),
            c14: mix_hex(&cyan, boost, 0.20),
            c15,
            on_accent: contrast_fg(spec.accent),
            on_danger: contrast_fg(spec.danger),
        }
    }
}

/// Build the single canonical list of color variables for a palette.
pub fn build_vars(spec: &PaletteSpec) -> Vec<(&'static str, String)> {
    let a = Ansi16::from_spec(spec);
    let mut vars = vec![
        ("DOTFIELD_THEME_ID", spec.id.to_string()),
        (
            "DOTFIELD_MODE",
            if spec.is_dark { "dark" } else { "light" }.to_string(),
        ),
        (
            "DOTFIELD_FOOT_SECTION",
            if spec.is_dark {
                "colors-dark"
            } else {
                "colors-light"
            }
            .to_string(),
        ),
    ];

    let colors: &[(&'static str, &str)] = &[
        ("DOTFIELD_BG", spec.bg),
        ("DOTFIELD_SURFACE", spec.surface),
        ("DOTFIELD_OVERLAY", spec.overlay),
        ("DOTFIELD_BORDER", spec.border),
        ("DOTFIELD_FG", spec.fg),
        ("DOTFIELD_FG_SECONDARY", spec.fg_secondary),
        ("DOTFIELD_FG_MUTED", spec.fg_muted),
        ("DOTFIELD_ACCENT", spec.accent),
        ("DOTFIELD_SUCCESS", spec.success),
        ("DOTFIELD_WARNING", spec.warning),
        ("DOTFIELD_DANGER", spec.danger),
        ("DOTFIELD_ON_ACCENT", &a.on_accent),
        ("DOTFIELD_ON_DANGER", &a.on_danger),
        ("DOTFIELD_C0", &a.c0),
        ("DOTFIELD_C1", &a.c1),
        ("DOTFIELD_C2", &a.c2),
        ("DOTFIELD_C3", &a.c3),
        ("DOTFIELD_C4", &a.c4),
        ("DOTFIELD_C5", &a.c5),
        ("DOTFIELD_C6", &a.c6),
        ("DOTFIELD_C7", &a.c7),
        ("DOTFIELD_C8", &a.c8),
        ("DOTFIELD_C9", &a.c9),
        ("DOTFIELD_C10", &a.c10),
        ("DOTFIELD_C11", &a.c11),
        ("DOTFIELD_C12", &a.c12),
        ("DOTFIELD_C13", &a.c13),
        ("DOTFIELD_C14", &a.c14),
        ("DOTFIELD_C15", &a.c15),
    ];

    for &(k, v) in colors {
        vars.push((k, v.to_string()));
    }
    // Separate static-key table for *_HEX variants (no `#` prefix)
    let hex_keys: &[(&'static str, &str)] = &[
        ("DOTFIELD_BG_HEX", spec.bg),
        ("DOTFIELD_SURFACE_HEX", spec.surface),
        ("DOTFIELD_OVERLAY_HEX", spec.overlay),
        ("DOTFIELD_BORDER_HEX", spec.border),
        ("DOTFIELD_FG_HEX", spec.fg),
        ("DOTFIELD_FG_SECONDARY_HEX", spec.fg_secondary),
        ("DOTFIELD_FG_MUTED_HEX", spec.fg_muted),
        ("DOTFIELD_ACCENT_HEX", spec.accent),
        ("DOTFIELD_SUCCESS_HEX", spec.success),
        ("DOTFIELD_WARNING_HEX", spec.warning),
        ("DOTFIELD_DANGER_HEX", spec.danger),
        ("DOTFIELD_ON_ACCENT_HEX", &a.on_accent),
        ("DOTFIELD_ON_DANGER_HEX", &a.on_danger),
        ("DOTFIELD_C0_HEX", &a.c0),
        ("DOTFIELD_C1_HEX", &a.c1),
        ("DOTFIELD_C2_HEX", &a.c2),
        ("DOTFIELD_C3_HEX", &a.c3),
        ("DOTFIELD_C4_HEX", &a.c4),
        ("DOTFIELD_C5_HEX", &a.c5),
        ("DOTFIELD_C6_HEX", &a.c6),
        ("DOTFIELD_C7_HEX", &a.c7),
        ("DOTFIELD_C8_HEX", &a.c8),
        ("DOTFIELD_C9_HEX", &a.c9),
        ("DOTFIELD_C10_HEX", &a.c10),
        ("DOTFIELD_C11_HEX", &a.c11),
        ("DOTFIELD_C12_HEX", &a.c12),
        ("DOTFIELD_C13_HEX", &a.c13),
        ("DOTFIELD_C14_HEX", &a.c14),
        ("DOTFIELD_C15_HEX", &a.c15),
    ];
    for &(k, v) in hex_keys {
        vars.push((k, strip_hash(v).to_string()));
    }
    vars
}

/// Substitute `${VAR}` placeholders using the variable table.
pub fn render_template(template: &str, vars: &[(&'static str, String)]) -> String {
    let mut out = template.to_string();
    for (k, v) in vars {
        out = out.replace(&format!("${{{k}}}"), v);
    }
    out
}

pub fn render_colors_sh(vars: &[(&'static str, String)]) -> String {
    let mut out = String::from("# Auto-generated by Amane Settings — central color variables\n");
    for (k, v) in vars {
        out.push_str(&format!("{k}=\"{v}\"\n"));
    }
    out
}

pub fn render_colors_lua(spec: &PaletteSpec) -> String {
    let a = Ansi16::from_spec(spec);
    let mode = if spec.is_dark { "dark" } else { "light" };
    format!(
        "-- Auto-generated by Amane Settings ({id})\n\
         return {{\n\
           mode = \"{mode}\",\n\
           bg = \"{bg}\",\n\
           surface = \"{surface}\",\n\
           overlay = \"{overlay}\",\n\
           border = \"{border}\",\n\
           fg = \"{fg}\",\n\
           fg_muted = \"{fg_muted}\",\n\
           accent = \"{accent}\",\n\
           on_accent = \"{on_accent}\",\n\
         }}\n",
        id = spec.id,
        mode = mode,
        bg = spec.bg,
        surface = spec.surface,
        overlay = spec.overlay,
        border = spec.border,
        fg = spec.fg,
        fg_muted = spec.fg_muted,
        accent = spec.accent,
        on_accent = a.on_accent,
    )
}

pub fn render_colors_tmux(spec: &PaletteSpec) -> String {
    let a = Ansi16::from_spec(spec);
    format!(
        "# Auto-generated by Amane Settings ({id})\n\
         set -g @dotfield_bg \"{bg}\"\n\
         set -g @dotfield_surface \"{surface}\"\n\
         set -g @dotfield_overlay \"{overlay}\"\n\
         set -g @dotfield_border \"{border}\"\n\
         set -g @dotfield_fg \"{fg}\"\n\
         set -g @dotfield_fg_muted \"{fg_muted}\"\n\
         set -g @dotfield_accent \"{accent}\"\n\
         set -g @dotfield_warning \"{warning}\"\n\
         set -g @dotfield_on_accent \"{on_accent}\"\n",
        id = spec.id,
        bg = spec.bg,
        surface = spec.surface,
        overlay = spec.overlay,
        border = spec.border,
        fg = spec.fg,
        fg_muted = spec.fg_muted,
        accent = spec.accent,
        warning = spec.warning,
        on_accent = a.on_accent,
    )
}

pub fn render_gtk_css(spec: &PaletteSpec) -> String {
    let a = Ansi16::from_spec(spec);
    format!(
        "/* Auto-generated by Amane Settings ({id}) */\n\
         @define-color accent_color {accent};\n\
         @define-color accent_bg_color {accent};\n\
         @define-color accent_fg_color {on_accent};\n\
         @define-color window_bg_color {bg};\n\
         @define-color window_fg_color {fg};\n\
         @define-color view_bg_color {surface};\n\
         @define-color view_fg_color {fg};\n\
         @define-color headerbar_bg_color {surface};\n\
         @define-color headerbar_fg_color {fg};\n\
         @define-color headerbar_border_color {border};\n\
         @define-color headerbar_backdrop_color {bg};\n\
         @define-color card_bg_color {surface};\n\
         @define-color card_fg_color {fg};\n\
         @define-color popover_bg_color {overlay};\n\
         @define-color popover_fg_color {fg};\n\
         @define-color sidebar_bg_color {surface};\n\
         @define-color sidebar_fg_color {fg_secondary};\n\
         @define-color borders {border};\n\
         @define-color warning_color {warning};\n\
         @define-color error_color {danger};\n\
         @define-color success_color {success};\n",
        id = spec.id,
        accent = spec.accent,
        on_accent = a.on_accent,
        bg = spec.bg,
        fg = spec.fg,
        surface = spec.surface,
        border = spec.border,
        overlay = spec.overlay,
        fg_secondary = spec.fg_secondary,
        warning = spec.warning,
        danger = spec.danger,
        success = spec.success,
    )
}

fn load_template(templates_dir: &Path, filename: &str, fallback: &str) -> String {
    fs::read_to_string(templates_dir.join(filename)).unwrap_or_else(|_| fallback.to_string())
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, content);
}

fn spawn_detached(bin: &str, args: &[&str]) {
    let _ = Command::new(bin).args(args).spawn();
}

/// Update central color variable files and apply them across all configured app templates.
pub fn sync_all(spec: &PaletteSpec) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let h = Path::new(&home);
    let dotfield_dir = h.join(".config/dotfield");
    let themes_dir = dotfield_dir.join("themes");
    let templates_dir = dotfield_dir.join("templates");
    let vars = build_vars(spec);

    // 1. Native variable files (apps reference variables directly)
    write_file(&dotfield_dir.join("colors.sh"), &render_colors_sh(&vars));
    write_file(&dotfield_dir.join("colors.lua"), &render_colors_lua(spec));
    write_file(&dotfield_dir.join("colors.tmux"), &render_colors_tmux(spec));
    let gtk_css = render_gtk_css(spec);
    write_file(&h.join(".config/gtk-3.0/dotfield-live.css"), &gtk_css);
    write_file(&h.join(".config/gtk-4.0/dotfield-live.css"), &gtk_css);

    // 2. Template-substituted configs (for apps without native variable syntax)
    let targets: &[(&str, &str, std::path::PathBuf)] = &[
        (
            "ghostty.conf.tpl",
            include_str!("../../../../theme/templates/ghostty.conf.tpl"),
            themes_dir.join("ghostty.conf"),
        ),
        (
            "kitty.conf.tpl",
            include_str!("../../../../theme/templates/kitty.conf.tpl"),
            themes_dir.join("kitty.conf"),
        ),
        (
            "foot.ini.tpl",
            include_str!("../../../../theme/templates/foot.ini.tpl"),
            themes_dir.join("foot.ini"),
        ),
        (
            "zellij.kdl.tpl",
            include_str!("../../../../theme/templates/zellij.kdl.tpl"),
            h.join(".config/zellij/themes/dotfield-live.kdl"),
        ),
        (
            "yazi.toml.tpl",
            include_str!("../../../../theme/templates/yazi.toml.tpl"),
            h.join(".config/yazi/flavors/dotfield-live.yazi/flavor.toml"),
        ),
        (
            "btop.theme.tpl",
            include_str!("../../../../theme/templates/btop.theme.tpl"),
            h.join(".config/btop/themes/dotfield-live.theme"),
        ),
        (
            "cava.tpl",
            include_str!("../../../../theme/templates/cava.tpl"),
            h.join(".config/cava/themes/dotfield-live"),
        ),
        (
            "imv.conf.tpl",
            include_str!("../../../../theme/templates/imv.conf.tpl"),
            themes_dir.join("imv.conf"),
        ),
        (
            "mpv.conf.tpl",
            include_str!("../../../../theme/templates/mpv.conf.tpl"),
            themes_dir.join("mpv.conf"),
        ),
        (
            "starship.toml.tpl",
            include_str!("../../../../theme/templates/starship.toml.tpl"),
            themes_dir.join("starship.toml"),
        ),
        (
            "niri.kdl.tpl",
            include_str!("../../../../theme/templates/niri.kdl.tpl"),
            h.join(".config/niri/conf.d/99-amane-theme.kdl"),
        ),
        (
            "fuzzel.ini.tpl",
            include_str!("../../../../theme/templates/fuzzel.ini.tpl"),
            themes_dir.join("fuzzel.ini"),
        ),
        (
            "mako.conf.tpl",
            include_str!("../../../../theme/templates/mako.conf.tpl"),
            themes_dir.join("mako.conf"),
        ),
    ];

    for (tpl_name, fallback, dest) in targets {
        let tpl = load_template(&templates_dir, tpl_name, fallback);
        write_file(dest, &render_template(&tpl, &vars));
    }

    // 3. Non-blocking live reloads
    spawn_detached("pkill", &["-USR1", "kitty"]);
    spawn_detached("pkill", &["-USR1", "cava"]);
    spawn_detached("pkill", &["-USR2", "btop"]);
    spawn_detached("pkill", &["-USR1", "nvim"]);
    spawn_detached("makoctl", &["reload"]);
    spawn_detached(
        "tmux",
        &[
            "source-file",
            &dotfield_dir.join("colors.tmux").to_string_lossy(),
        ],
    );
    spawn_detached("niri", &["msg", "action", "load-config-file"]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::palettes::find_palette;

    #[test]
    fn test_central_vars_and_templates_dark_and_light() {
        let dark = find_palette("cyber-synthwave").expect("cyber-synthwave exists");
        let light = find_palette("catppuccin-latte").expect("catppuccin-latte exists");

        let dark_vars = build_vars(dark);
        let light_vars = build_vars(light);

        let sh = render_colors_sh(&dark_vars);
        assert!(sh.contains("DOTFIELD_BG=\"#000000\""));
        assert!(sh.contains("DOTFIELD_ACCENT=\"#04C5F0\""));

        let ghostty = render_template(
            include_str!("../../../../theme/templates/ghostty.conf.tpl"),
            &dark_vars,
        );
        assert!(ghostty.contains("background = 000000"));
        assert!(ghostty.contains("background-opacity = 1.0"));

        let fuzzel = render_template(
            include_str!("../../../../theme/templates/fuzzel.ini.tpl"),
            &light_vars,
        );
        assert!(fuzzel.contains(&format!("background={}f2", strip_hash(light.bg))));

        let mako = render_template(
            include_str!("../../../../theme/templates/mako.conf.tpl"),
            &dark_vars,
        );
        assert!(mako.contains("border-color=#04C5F0"));

        assert!(render_colors_lua(light).contains("mode = \"light\""));
        assert!(render_colors_tmux(dark).contains("set -g @dotfield_accent \"#04C5F0\""));
        assert!(render_gtk_css(dark).contains("@define-color accent_color #04C5F0;"));
    }
}
