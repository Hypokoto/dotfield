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

/// Border radius scale providing mechanical tactical curvature (BLACKSITE Industrial Noir: 1-3px)
pub mod radius {
    pub const XS: f32 = 2.0; // Micro indicators, dot tags
    pub const SM: f32 = 4.0; // Keycaps, tags, badges, buttons
    pub const MD: f32 = 6.0; // Standard cards, buttons, input fields
    pub const LG: f32 = 8.0; // Windows, modal shells, popovers
    pub const XL: f32 = 12.0; // Large Windows 11 flyouts and Start Menu
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

/// Hardcoded design tokens strictly defined in PALETTE.md (Controlled Chaos)
pub mod palette {
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
    pub const ACCENT_BLUE: &str = "#60CDFF"; // Fluent Blue / WinUI accent
    pub const ACRYLIC_BG: &str = "#1c1c1ce6"; // Translucent Mica/Acrylic window background
    pub const ACRYLIC_BORDER: &str = "#ffffff14"; // Subtle translucent 1px border outline
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
}

impl Default for ThemeTokens {
    fn default() -> Self {
        Self::from_manifest(&ShellManifest::default())
    }
}

impl ThemeTokens {
    pub fn from_manifest(manifest: &ShellManifest) -> Self {
        let p = &manifest.palette;

        // Controlled Chaos Core Palette strictly mapped to PALETTE.md
        let bg = if !p.background.is_empty() {
            Color::from(p.background.as_str())
        } else {
            Color::from(palette::VOID)
        };
        let surface = if !p.surface.is_empty() {
            Color::from(p.surface.as_str())
        } else {
            Color::from(palette::OBSIDIAN)
        };
        let surface_elevated = if !p.overlay.is_empty() {
            Color::from(p.overlay.as_str())
        } else {
            Color::from(palette::GRAPHITE)
        };
        let border_subtle = if !p.border.is_empty() {
            Color::from(p.border.as_str())
        } else {
            Color::from(palette::IRON)
        };
        let text_primary = if !p.foreground.is_empty() {
            Color::from(p.foreground.as_str())
        } else {
            Color::from(palette::BONE)
        };
        let text_secondary = if !p.muted.is_empty() {
            Color::from(p.muted.as_str())
        } else {
            Color::from(palette::ASH)
        };
        let text_muted = if !p.steel.is_empty() {
            Color::from(p.steel.as_str())
        } else {
            Color::from(palette::STEEL)
        };
        let accent = if !p.highlight.is_empty() {
            Color::from(p.highlight.as_str())
        } else {
            Color::from(palette::ACCENT_BLUE)
        };
        let accent_hover = if !p.hover.is_empty() {
            Color::from(p.hover.as_str())
        } else {
            Color::from("#72D6FF")
        };
        let danger = if !p.accent.is_empty() {
            Color::from(p.accent.as_str())
        } else {
            Color::from(palette::CRIMSON)
        };
        let danger_critical = if !p.danger_critical.is_empty() {
            Color::from(p.danger_critical.as_str())
        } else {
            Color::from(palette::BLOOD_RED)
        };
        let warning = Color::from(palette::INDUSTRIAL_AMBER);
        let warning_subtle = Color::from("#261D0A");
        let success = if !p.success.is_empty() {
            Color::from(p.success.as_str())
        } else {
            Color::from(palette::SUCCESS_GREEN)
        };
        let steel = if !p.steel.is_empty() {
            Color::from(p.steel.as_str())
        } else {
            Color::from(palette::STEEL)
        };

        // WinUI 3 Fluent / Mica Acrylic Tokens
        let acrylic_bg = Color::from(palette::ACRYLIC_BG);
        let acrylic_border = Color::from(palette::ACRYLIC_BORDER);
        let card_bg = surface_elevated;
        let card_border = Color::from("#ffffff0f");
        let indicator_active = accent;

        // Derived surface & border states (Controlled Chaos)
        let surface_hover = Color::from("#1C1C23");
        let surface_active = Color::from("#252530");
        let surface_selected = Color::from("#1A2633");
        let surface_track = Color::from("#0A0A0C");

        let border_hover = Color::from("#44475A");
        let border_focus = accent;
        let border_danger = danger;

        let accent_subtle = Color::from("#142533");
        let danger_subtle = Color::from("#251114");

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
            font_family: manifest.theme.font_family.clone(),
        }
    }
}
