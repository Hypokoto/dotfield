use amane::Color;

use crate::config::ShellManifest;

/// Geometric spacing scale based on an 8pt modular rhythm
pub mod spacing {
    pub const GAP_XS: f32 = 4.0;
    pub const GAP_SM: f32 = 8.0;
    pub const GAP_MD: f32 = 12.0;
    pub const GAP_LG: f32 = 16.0;
    pub const GAP_XL: f32 = 20.0;

    pub const PAD_XS: f32 = 4.0;
    pub const PAD_SM: f32 = 8.0;
    pub const PAD_MD: f32 = 12.0;
    pub const PAD_LG: f32 = 14.0;
    pub const PAD_XL: f32 = 16.0;
    pub const PAD_PANEL: f32 = 16.0;
}

/// Border radius scale providing smooth claymorphic curvature (Generous, tactile, pillowy)
pub mod radius {
    pub const XS: f32 = 4.0; // Micro indicators, dot tags
    pub const SM: f32 = 10.0; // Keycaps, tags, badges, buttons
    pub const MD: f32 = 16.0; // Standard cards, buttons, input fields
    pub const LG: f32 = 24.0; // Windows, modal shells, popovers
    pub const XL: f32 = 30.0; // Large flyouts and Start Menu
    pub const PILL: f32 = 999.0; // Fully rounded pills, radar dots
}

/// Border thickness tokens (thin, subtle, technical)
pub mod border {
    pub const THIN: f32 = 1.0; // Minimal subtle perimeter outline
    pub const FOCUS: f32 = 1.5; // High-priority active / focused element border
}

/// Restrained animation durations (120-180ms tactical timing)
pub mod animation {
    use std::time::Duration;

    pub const DURATION_FAST: Duration = Duration::from_millis(120); // Quick state transitions, micro-interactions
    pub const DURATION_NORMAL: Duration = Duration::from_millis(150); // Workspace switch, column glide, telemetry fade
    pub const DURATION_MODAL: Duration = Duration::from_millis(180); // Restrained dashboard summon & dismissal
}

/// Typography size scale
pub mod typography {
    pub const CAPTION: f32 = 10.0; // Micro tags, shortcut keycaps
    pub const BADGE: f32 = 10.5; // Status badges, secondary chips
    pub const SECONDARY: f32 = 11.5; // Subtitles, metadata, hints
    pub const BODY: f32 = 13.0; // Standard reading text, list items
    pub const BODY_LG: f32 = 13.5; // Prominent body text, card headers
    pub const TITLE_SM: f32 = 14.5; // Section headers, dialog headers
    pub const TITLE_MD: f32 = 16.0; // Prominent titles
    pub const DISPLAY: f32 = 22.0; // Hero display, large clock
}

pub mod palette {
    // Canonical Controlled Chaos Palette (PALETTE.md)
    pub const VOID: &str = "#0A0A0C"; // Primary background
    pub const OBSIDIAN: &str = "#121216"; // Windows, panels, and cards
    pub const GRAPHITE: &str = "#1C1C23"; // Elevated surfaces
    pub const IRON: &str = "#34343D"; // Borders and separators
    pub const ASH: &str = "#A1A1AA"; // Secondary text
    pub const BONE: &str = "#E8E6E3"; // Primary text
    pub const CRIMSON: &str = "#E63946"; // Chaos and critical emphasis
    pub const BLOOD_RED: &str = "#9D0208"; // Critical warnings and dangerous operations
    pub const INDUSTRIAL_AMBER: &str = "#FFB000"; // Primary accent and active states
    pub const PALE_AMBER: &str = "#FFD166"; // Hover states and highlighted information
    pub const STEEL: &str = "#8B949E"; // Neutral states and secondary controls
    pub const SUCCESS_GREEN: &str = "#7BC47F"; // Successful operations

    // Tints & Translucency
    pub const AMBER_TINT: &str = "#FFB00026";
    pub const ACRYLIC_BG: &str = "#121216fa";
    pub const ACRYLIC_BORDER: &str = IRON;

    // Playful Clay Palette Foundations (Soft 3D malleable clay)
    pub const CLAY_CANVAS: &str = "#EBF2FE";
    pub const CLAY_SURFACE: &str = "#FFFFFF";
    pub const CLAY_SURFACE_HOVER: &str = "#F0F4FF";
    pub const CLAY_SURFACE_ACTIVE: &str = "#E0ECFF";
    pub const CLAY_SURFACE_SELECTED: &str = "#D8E7FF";
    pub const CLAY_SURFACE_TRACK: &str = "#E2ECFE";
    pub const CLAY_PRIMARY: &str = "#3B82F6";
    pub const CLAY_PRIMARY_HOVER: &str = "#2563EB";
    pub const CLAY_PRIMARY_SUBTLE: &str = "#EFF6FF";
    pub const CLAY_TEXT_PRIMARY: &str = "#1C398E";
    pub const CLAY_TEXT_SECONDARY: &str = "#5B72B4";
    pub const CLAY_TEXT_MUTED: &str = "#8EA0D0";
    pub const CLAY_SUCCESS: &str = "#16A34A";
    pub const CLAY_WARNING: &str = "#D97706";
    pub const CLAY_DANGER: &str = "#DC2626";
    pub const CLAY_DANGER_CRITICAL: &str = "#B91C1C";

    // Backward-compatibility & semantic aliases mapping directly to Controlled Chaos
    pub const MATTE_DARK_CLAY: &str = VOID;
    pub const WARM_BISQUE: &str = OBSIDIAN;
    pub const TERRACOTTA_PUTTY: &str = GRAPHITE;
    pub const CLAY_INSET: &str = "#23232C";
    pub const CLAY_ACTIVE: &str = "#2A2A35";
    pub const CLAY_LIP: &str = IRON;
    pub const CLAY_LIP_LIGHT: &str = "#454552";
    pub const CLAY_SEAM: &str = "#18181F";
    pub const CLAY_BORDER_SUBTLE: &str = IRON;
    pub const GLAZED_TERRACOTTA: &str = INDUSTRIAL_AMBER;
    pub const LIGHT_TERRACOTTA: &str = PALE_AMBER;
    pub const TERRACOTTA_TINT: &str = AMBER_TINT;
    pub const WARM_SAND: &str = BONE;
    pub const MUTED_PUTTY: &str = ASH;
    pub const STONE_GREY: &str = STEEL;
    pub const CRIMSON_GLAZE: &str = CRIMSON;
    pub const CRIMSON_GLAZE_HOVER: &str = "#FF6B76";
    pub const CRIMSON_GLAZE_BORDER: &str = BLOOD_RED;
    pub const SAGE_CLAY: &str = SUCCESS_GREEN;
}

/// Semantic theme tokens resolved from the active shell manifest
#[derive(Debug, Clone)]
pub struct ThemeTokens {
    pub bg: Color,
    pub surface: Color,
    pub surface_elevated: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub surface_selected: Color,
    pub surface_track: Color,

    pub border_subtle: Color,
    pub border_hover: Color,
    pub border_focus: Color,
    pub border_danger: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,

    pub accent: Color,
    pub accent_hover: Color,
    pub accent_subtle: Color,

    pub danger: Color,
    pub danger_critical: Color,
    pub danger_subtle: Color,

    pub warning: Color,
    pub warning_subtle: Color,

    pub success: Color,
    pub steel: Color,

    pub acrylic_bg: Color,
    pub acrylic_border: Color,
    pub card_bg: Color,
    pub card_border: Color,
    pub indicator_active: Color,

    pub font_family: String,
    pub font_sans: String,
    pub font_mono: String,
    pub accent_tint: Color,
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self::from_manifest(&ShellManifest::default())
    }
}

impl ThemeTokens {
    pub fn dark_controlled_chaos() -> Self {
        let bg = Color::from(palette::VOID);
        let surface = Color::from(palette::OBSIDIAN);
        let surface_elevated = Color::from(palette::GRAPHITE);
        let border_subtle = Color::from(palette::IRON);
        let text_primary = Color::from(palette::BONE);
        let text_secondary = Color::from(palette::ASH);
        let text_muted = Color::from(palette::STEEL);
        let accent = Color::from(palette::INDUSTRIAL_AMBER);
        let accent_hover = Color::from(palette::PALE_AMBER);
        let danger = Color::from(palette::CRIMSON);
        let danger_critical = Color::from(palette::BLOOD_RED);
        let warning = Color::from(palette::INDUSTRIAL_AMBER);
        let warning_subtle = Color::from("#2E2010");
        let success = Color::from(palette::SUCCESS_GREEN);
        let steel = Color::from(palette::STEEL);

        let acrylic_bg = Color::from(palette::ACRYLIC_BG);
        let acrylic_border = Color::from(palette::IRON);
        let card_bg = surface_elevated;
        let card_border = Color::from(palette::IRON);
        let indicator_active = accent;

        let surface_hover = Color::from("#24242D");
        let surface_active = Color::from("#2C2C37");
        let surface_selected = Color::from("#2A2A35");
        let surface_track = Color::from(palette::VOID);

        let border_hover = Color::from("#454552");
        let border_focus = Color::rgba(0xFF, 0xB0, 0x00, 0x88); // Soft warm amber clay highlight
        let border_danger = Color::rgba(0xE6, 0x39, 0x46, 0x88); // Soft crimson clay highlight

        let accent_subtle = Color::from("#332400");
        let danger_subtle = Color::from("#361114");

        Self {
            bg,
            surface,
            surface_elevated,
            surface_hover,
            surface_active,
            surface_selected,
            surface_track,
            border_subtle,
            border_hover,
            border_focus,
            border_danger,
            text_primary,
            text_secondary,
            text_muted,
            accent,
            accent_hover,
            accent_subtle,
            danger,
            danger_critical,
            danger_subtle,
            warning,
            warning_subtle,
            success,
            steel,
            acrylic_bg,
            acrylic_border,
            card_bg,
            card_border,
            indicator_active,
            font_family: "Outfit".to_string(),
            font_sans: "Outfit".to_string(),
            font_mono: "JetBrainsMono Nerd Font".to_string(),
            accent_tint: Color::from(palette::AMBER_TINT),
        }
    }

    pub fn playful_clay() -> Self {
        let bg = Color::from(palette::CLAY_CANVAS);
        let surface = Color::from(palette::CLAY_SURFACE);
        let surface_elevated = Color::from(palette::CLAY_SURFACE);
        let surface_hover = Color::from(palette::CLAY_SURFACE_HOVER);
        let surface_active = Color::from(palette::CLAY_SURFACE_ACTIVE);
        let surface_selected = Color::from(palette::CLAY_SURFACE_SELECTED);
        let surface_track = Color::from(palette::CLAY_SURFACE_TRACK);

        let border_subtle = Color::rgba(28, 57, 142, 28);
        let border_hover = Color::rgba(59, 130, 246, 120);
        let border_focus = Color::rgba(59, 130, 246, 180);
        let border_danger = Color::rgba(220, 38, 38, 180);

        let text_primary = Color::from(palette::CLAY_TEXT_PRIMARY);
        let text_secondary = Color::from(palette::CLAY_TEXT_SECONDARY);
        let text_muted = Color::from(palette::CLAY_TEXT_MUTED);

        let accent = Color::from(palette::CLAY_PRIMARY);
        let accent_hover = Color::from(palette::CLAY_PRIMARY_HOVER);
        let accent_subtle = Color::from(palette::CLAY_PRIMARY_SUBTLE);
        let accent_tint = Color::rgba(59, 130, 246, 36);

        let danger = Color::from(palette::CLAY_DANGER);
        let danger_critical = Color::from(palette::CLAY_DANGER_CRITICAL);
        let danger_subtle = Color::rgba(220, 38, 38, 36);

        let warning = Color::from(palette::CLAY_WARNING);
        let warning_subtle = Color::rgba(217, 119, 6, 36);

        let success = Color::from(palette::CLAY_SUCCESS);
        let steel = Color::from(palette::CLAY_TEXT_SECONDARY);

        let acrylic_bg = Color::rgba(255, 255, 255, 245);
        let acrylic_border = Color::rgba(28, 57, 142, 28);
        let card_bg = Color::from(palette::CLAY_SURFACE);
        let card_border = Color::rgba(28, 57, 142, 28);
        let indicator_active = accent;

        Self {
            bg,
            surface,
            surface_elevated,
            surface_hover,
            surface_active,
            surface_selected,
            surface_track,
            border_subtle,
            border_hover,
            border_focus,
            border_danger,
            text_primary,
            text_secondary,
            text_muted,
            accent,
            accent_hover,
            accent_subtle,
            danger,
            danger_critical,
            danger_subtle,
            warning,
            warning_subtle,
            success,
            steel,
            acrylic_bg,
            acrylic_border,
            card_bg,
            card_border,
            indicator_active,
            font_family: "Inter".to_string(),
            font_sans: "Inter".to_string(),
            font_mono: "JetBrainsMono Nerd Font".to_string(),
            accent_tint,
        }
    }

    pub fn from_manifest(manifest: &ShellManifest) -> Self {
        let style = manifest.theme.style.to_lowercase();
        let mut tokens = if style.contains("dark") || style.contains("controlled") {
            Self::dark_controlled_chaos()
        } else {
            Self::playful_clay()
        };

        if !manifest.theme.font_family.is_empty() && manifest.theme.font_family != "monospace" {
            tokens.font_family = manifest.theme.font_family.clone();
            tokens.font_sans = manifest.theme.font_family.clone();
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokens_from_manifest_defaults_to_playful_clay() {
        let manifest = ShellManifest::default();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.text_primary, Color::from(palette::CLAY_TEXT_PRIMARY));
        assert_eq!(tokens.accent, Color::from(palette::CLAY_PRIMARY));
        assert_eq!(tokens.font_sans, "Inter");
    }

    #[test]
    fn test_tokens_from_manifest_dark_style() {
        let mut manifest = ShellManifest::default();
        manifest.theme.style = "dark".to_string();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.text_primary, Color::from(palette::BONE));
        assert_eq!(tokens.accent, Color::from(palette::INDUSTRIAL_AMBER));
    }
}
