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

/// Border radius scale providing consistent curvature hierarchy (BLACKSITE spec: 4-6px)
pub mod radius {
    pub const XS: f32 = 2.0; // Micro indicators, dot tags
    pub const SM: f32 = 4.0; // Keycaps, tags, badges
    pub const MD: f32 = 5.0; // Standard cards, buttons, input fields
    pub const LG: f32 = 6.0; // Windows, modal shells, popovers
    pub const PILL: f32 = 999.0; // Fully rounded pills, radar dots
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

    pub success: Color,
    pub steel: Color,

    pub font_family: String,
}

impl ThemeTokens {
    pub fn from_manifest(manifest: &ShellManifest) -> Self {
        let palette = &manifest.palette;

        let bg = if !palette.background.is_empty() {
            Color::from(palette.background.as_str())
        } else {
            Color::from("#0A0A0C")
        };
        let surface = if !palette.surface.is_empty() {
            Color::from(palette.surface.as_str())
        } else {
            Color::from("#121216")
        };
        let surface_elevated = if !palette.overlay.is_empty() {
            Color::from(palette.overlay.as_str())
        } else {
            Color::from("#1C1C23")
        };
        let border_subtle = if !palette.border.is_empty() {
            Color::from(palette.border.as_str())
        } else {
            Color::from("#34343D")
        };
        let text_primary = if !palette.foreground.is_empty() {
            Color::from(palette.foreground.as_str())
        } else {
            Color::from("#E8E6E3")
        };
        let text_secondary = if !palette.muted.is_empty() {
            Color::from(palette.muted.as_str())
        } else {
            Color::from("#A1A1AA")
        };
        let text_muted = if !palette.steel.is_empty() {
            Color::from(palette.steel.as_str())
        } else {
            Color::from("#8B949E")
        };
        let accent = if !palette.highlight.is_empty() {
            Color::from(palette.highlight.as_str())
        } else {
            Color::from("#FFB000")
        };
        let accent_hover = if !palette.hover.is_empty() {
            Color::from(palette.hover.as_str())
        } else {
            Color::from("#FFD166")
        };
        let danger = if !palette.accent.is_empty() {
            Color::from(palette.accent.as_str())
        } else {
            Color::from("#E63946")
        };
        let danger_critical = if !palette.danger_critical.is_empty() {
            Color::from(palette.danger_critical.as_str())
        } else {
            Color::from("#9D0208")
        };
        let success = if !palette.success.is_empty() {
            Color::from(palette.success.as_str())
        } else {
            Color::from("#7BC47F")
        };
        let steel = if !palette.steel.is_empty() {
            Color::from(palette.steel.as_str())
        } else {
            Color::from("#8B949E")
        };

        // Derived surface & border states
        let surface_hover = Color::from("#24242D");
        let surface_active = Color::from("#2C2C37");
        let surface_selected = Color::from("#281F0D");
        let surface_track = Color::from("#16161C");

        let border_hover = Color::from("#484855");
        let border_focus = accent;
        let border_danger = danger;

        let accent_subtle = Color::from("#2A1F08");
        let danger_subtle = Color::from("#2D1215");

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
            success,
            steel,
            font_family: manifest.theme.font_family.clone(),
        }
    }
}
