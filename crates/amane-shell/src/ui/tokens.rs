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

/// Border radius scale providing consistent curvature hierarchy
pub mod radius {
    pub const XS: f32 = 3.0; // Micro indicators, indicators
    pub const SM: f32 = 4.0; // Keycaps, tags, badges
    pub const MD: f32 = 8.0; // Standard cards, buttons, input fields
    pub const LG: f32 = 12.0; // Windows, popovers, control center
    pub const PILL: f32 = 999.0; // Fully rounded pills, radar dots
}

/// Typography size scale
pub mod typography {
    pub const CAPTION: f32 = 9.0; // Micro tags, shortcut keycaps
    pub const BADGE: f32 = 10.0; // Status badges, secondary chips
    pub const SECONDARY: f32 = 11.0; // Subtitles, metadata, hints
    pub const BODY: f32 = 12.0; // Standard reading text, list items
    pub const BODY_LG: f32 = 13.0; // Prominent body text, card headers
    pub const TITLE_SM: f32 = 14.0; // Section headers, dialog headers
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
    pub danger_subtle: Color,

    pub font_family: String,
}

impl ThemeTokens {
    pub fn from_manifest(manifest: &ShellManifest) -> Self {
        let palette = &manifest.palette;

        let bg = Color::from(palette.background.as_str());
        let surface = Color::from(palette.surface.as_str());
        let surface_elevated = Color::from(palette.overlay.as_str());
        let border_subtle = Color::from(palette.border.as_str());
        let text_primary = Color::from(palette.foreground.as_str());
        let text_muted = Color::from(palette.muted.as_str());
        let text_secondary = Color::from(palette.steel.as_str());
        let accent = Color::from(palette.highlight.as_str());
        let accent_hover = Color::from(palette.hover.as_str());
        let danger = Color::from(palette.accent.as_str());

        // Derived harmonious surface states
        let surface_hover = Color::from("#23232C");
        let surface_active = Color::from("#2A2A35");
        let surface_selected = Color::from("#282215");
        let surface_track = Color::from("#202028");

        let border_hover = Color::from("#454554");
        let border_focus = accent;
        let border_danger = danger;

        let accent_subtle = Color::from("#2E2514");
        let danger_subtle = Color::from("#2E1417");

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
            danger_subtle,
            font_family: manifest.theme.font_family.clone(),
        }
    }
}
