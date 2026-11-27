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

    // Apple HIG Continuous Squircle Radii
    pub const IOS_WIDGET_SM: f32 = 20.0; // System Small 1x1 bento widget
    pub const IOS_WIDGET_MD: f32 = 22.0; // System Medium 2x1 bento widget
    pub const IOS_WIDGET_LG: f32 = 26.0; // System Large 2x2 bento widget
    pub const IOS_TOGGLE: f32 = 16.0; // iOS Control Center quick squircle toggle
    pub const IOS_CAPSULE: f32 = 999.0; // Dynamic Island / capsule pill
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

/// Bento Grid and Widget Board geometry metrics
pub mod bento {
    pub const BOARD_WIDTH: f32 = 380.0;
    pub const BOARD_PAD: f32 = 14.0;
    pub const SECTION_W: f32 = 352.0; // BOARD_WIDTH - 2 * BOARD_PAD
    pub const SECTION_PAD: f32 = 14.0;
    pub const CELL_1X1: f32 = 156.0;
    pub const CELL_1X1_H: f32 = 112.0; // Uniform 1x1 card height
    pub const CELL_2X1_W: f32 = 324.0; // SECTION_W - 2 * SECTION_PAD == 2 * CELL_1X1 + GRID_GAP
    pub const CELL_2X2_H: f32 = 324.0; // Large (2x2) fixed card height
    pub const GRID_GAP: f32 = 12.0;
    pub const GAUGE_SIZE: f32 = 64.0;
    pub const GAUGE_STROKE: f32 = 6.0;
    pub const CAVA_BAR_W: f32 = 4.0;
    pub const CAVA_BAR_GAP: f32 = 3.0;
    pub const CAVA_MAX_H: f32 = 32.0;
}

pub mod palette {
    // Canonical Obsidian Mint Palette (PALETTE.md)
    pub const OBSIDIAN_MINT_VOID: &str = "#080B0B";
    pub const OBSIDIAN_MINT_SURFACE: &str = "#0E1413";
    pub const OBSIDIAN_MINT_GRAPHITE: &str = "#16201E";
    pub const OBSIDIAN_MINT_IRON: &str = "#23322F";
    pub const OBSIDIAN_MINT_ASH: &str = "#94A3A0";
    pub const OBSIDIAN_MINT_PRIMARY_TEXT: &str = "#ECFDF5";
    pub const OBSIDIAN_MINT_ACCENT: &str = "#2DD4BF";
    pub const OBSIDIAN_MINT_EMERALD: &str = "#10B981";
    pub const OBSIDIAN_MINT_HIGHLIGHT: &str = "#FBBF24";
    pub const OBSIDIAN_MINT_DANGER: &str = "#F43F5E";
    pub const OBSIDIAN_MINT_DEEP_DANGER: &str = "#9F1239";
    pub const OBSIDIAN_MINT_STEEL: &str = "#71827E";
    pub const OBSIDIAN_MINT_TINT: &str = "#2DD4BF26";
    pub const OBSIDIAN_MINT_ACRYLIC_BG: &str = "#0E1413fa";

    // Canonical Controlled Chaos Palette (Legacy)
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
    pub border_specular: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_on_accent: Color,
    pub text_on_elevated: Color,
    pub text_on_danger: Color,
    pub text_on_danger_critical: Color,

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
        let border_specular = Color::rgba(255, 255, 255, 18);

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
            border_specular,
            text_primary,
            text_secondary,
            text_muted,
            text_on_accent: Color::from("#000000"),
            text_on_elevated: text_primary,
            text_on_danger: Color::WHITE,
            text_on_danger_critical: Color::WHITE,
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
        let border_specular = Color::rgba(255, 255, 255, 30);

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
            border_specular,
            text_primary,
            text_secondary,
            text_muted,
            text_on_accent: Color::WHITE,
            text_on_elevated: text_primary,
            text_on_danger: Color::WHITE,
            text_on_danger_critical: Color::WHITE,
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

    pub fn obsidian_mint() -> Self {
        let bg = Color::from(palette::OBSIDIAN_MINT_VOID);
        let surface = Color::from(palette::OBSIDIAN_MINT_SURFACE);
        let surface_elevated = Color::from(palette::OBSIDIAN_MINT_GRAPHITE);
        let border_subtle = Color::from(palette::OBSIDIAN_MINT_IRON);
        let text_primary = Color::from(palette::OBSIDIAN_MINT_PRIMARY_TEXT);
        let text_secondary = Color::from(palette::OBSIDIAN_MINT_ASH);
        let text_muted = Color::from(palette::OBSIDIAN_MINT_STEEL);
        let accent = Color::from(palette::OBSIDIAN_MINT_ACCENT);
        let accent_hover = Color::from(palette::OBSIDIAN_MINT_EMERALD);
        let danger = Color::from(palette::OBSIDIAN_MINT_DANGER);
        let danger_critical = Color::from(palette::OBSIDIAN_MINT_DEEP_DANGER);
        let warning = Color::from(palette::OBSIDIAN_MINT_HIGHLIGHT);
        let warning_subtle = Color::from("#2B210B");
        let success = Color::from(palette::OBSIDIAN_MINT_EMERALD);
        let steel = Color::from(palette::OBSIDIAN_MINT_STEEL);

        let acrylic_bg = Color::from(palette::OBSIDIAN_MINT_ACRYLIC_BG);
        let acrylic_border = Color::from(palette::OBSIDIAN_MINT_IRON);
        let card_bg = surface_elevated;
        let card_border = Color::from(palette::OBSIDIAN_MINT_IRON);
        let indicator_active = accent;

        let surface_hover = Color::from("#1B2926");
        let surface_active = Color::from("#20332F");
        let surface_selected = Color::from("#1D302C");
        let surface_track = Color::from(palette::OBSIDIAN_MINT_VOID);

        let border_hover = Color::from("#2D423E");
        let border_focus = Color::rgba(0x2D, 0xD4, 0xBF, 0x88);
        let border_danger = Color::rgba(0xF4, 0x3F, 0x5E, 0x88);
        let border_specular = Color::rgba(255, 255, 255, 18);

        let accent_subtle = Color::from("#0D332D");
        let danger_subtle = Color::from("#361118");

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
            border_specular,
            text_primary,
            text_secondary,
            text_muted,
            text_on_accent: Color::from("#080B0B"),
            text_on_elevated: text_primary,
            text_on_danger: Color::WHITE,
            text_on_danger_critical: Color::WHITE,
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
            accent_tint: Color::from(palette::OBSIDIAN_MINT_TINT),
        }
    }

    pub fn cyber_synthwave() -> Self {
        let bg = Color::from("#000000"); // base00
        let surface = Color::from("#140e29"); // base01
        let surface_elevated = Color::from("#251b4a"); // base02
        let border_subtle = Color::from("#342766");
        let text_primary = Color::from("#EAF8FA"); // Crisp cool ice-white (>=10.5:1 on #251b4a)
        let text_secondary = Color::from("#97DCE3"); // Soft cyan-slate (~7.8:1 on #251b4a)
        let text_muted = Color::from("#ABA5DB"); // Lifted lavender-slate (~5.2:1 on #251b4a)
        let accent = Color::from("#04C5F0"); // base0D
        let accent_hover = Color::from("#97DCE3");
        let danger = Color::from("#BE3F50"); // base08
        let danger_critical = Color::from("#CE4F48"); // base0F
        let warning = Color::from("#FF7F41"); // base09
        let warning_subtle = Color::from("#331a0e");
        let success = Color::from("#7cd699"); // base0B
        let steel = Color::from("#ffbe74"); // base07

        let acrylic_bg = Color::from("#140e29fa");
        let acrylic_border = Color::from("#342766");
        let card_bg = surface_elevated;
        let card_border = Color::from("#3D2E75");
        let indicator_active = accent;

        let surface_hover = Color::from("#2e205c");
        let surface_active = Color::from("#3b2975");
        let surface_selected = Color::from("#2e225c");
        let surface_track = Color::from("#1A1336"); // Recessed deep violet track (avoids harsh #000000 holes)

        let border_hover = Color::from("#9147a8"); // base0E
        let border_focus = Color::rgba(0x04, 0xC5, 0xF0, 140); // Softened neon blue rim
        let border_danger = Color::rgba(0xBE, 0x3F, 0x50, 160); // base08
        let border_specular = Color::rgba(234, 248, 250, 26);

        let accent_subtle = Color::from("#102e3b");
        let danger_subtle = Color::from("#301217");

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
            border_specular,
            text_primary,
            text_secondary,
            text_muted,
            text_on_accent: Color::from("#000000"), // 11.8:1 AAA vs #04C5F0
            text_on_elevated: Color::from("#61E6FF"), // 9.8:1 AAA vs #251b4a
            text_on_danger: Color::WHITE,
            text_on_danger_critical: Color::WHITE,
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
            font_family: "IBM Plex Sans".to_string(),
            font_sans: "IBM Plex Sans".to_string(),
            font_mono: "JetBrainsMono Nerd Font".to_string(),
            accent_tint: Color::from("#04C5F026"),
        }
    }

    pub fn ios_glass() -> Self {
        let bg = Color::from("#000000"); // Apple OLED Black
        let surface = Color::from("#1C1C1E"); // Apple iOS System Grouped Background (solid opaque)
        let surface_elevated = Color::from("#2C2C2E"); // Apple iOS Secondary Grouped Card (solid opaque)
        let surface_hover = Color::from("#3A3A3C"); // Apple iOS Tertiary
        let surface_active = Color::from("#007AFF"); // Apple System Blue
        let surface_selected = Color::from("#3A3A3C");
        let surface_track = Color::from("#3A3A3C"); // Apple slider / switch track

        let border_subtle = Color::from("#38383A"); // Apple 1px structural separator
        let border_hover = Color::from("#48484A");
        let border_focus = Color::from("#007AFF");
        let border_danger = Color::from("#FF3B30");
        let border_specular = Color::rgba(255, 255, 255, 20);

        let text_primary = Color::WHITE;
        let text_secondary = Color::from("#8E8E93"); // Apple System Gray
        let text_muted = Color::from("#636366");

        let accent = Color::from("#007AFF"); // Apple System Blue
        let accent_hover = Color::from("#0A84FF");
        let accent_subtle = Color::rgba(0, 122, 255, 45);
        let accent_tint = Color::rgba(0, 122, 255, 30);

        let danger = Color::from("#FF3B30"); // Apple System Red
        let danger_critical = Color::from("#D70015");
        let danger_subtle = Color::rgba(255, 59, 48, 40);

        let warning = Color::from("#FF9500"); // Apple System Amber / Orange
        let warning_subtle = Color::rgba(255, 149, 0, 40);

        let success = Color::from("#34C759"); // Apple System Green
        let steel = Color::from("#8E8E93"); // Apple System Gray

        let acrylic_bg = Color::from("#1C1C1E"); // Solid opaque structured surface
        let acrylic_border = Color::from("#38383A");
        let card_bg = Color::from("#2C2C2E"); // Solid opaque Bento card surface
        let card_border = Color::from("#38383A");
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
            border_specular,
            text_primary,
            text_secondary,
            text_muted,
            text_on_accent: Color::WHITE,
            text_on_elevated: Color::WHITE,
            text_on_danger: Color::WHITE,
            text_on_danger_critical: Color::WHITE,
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

    pub fn from_palette_spec(spec: &crate::services::palettes::PaletteSpec) -> Self {
        let bg = Color::from(spec.bg);
        let surface = Color::from(spec.surface);
        let surface_elevated = Color::from(spec.overlay);
        let surface_track = Color::from(spec.track);
        let text_primary = Color::from(spec.fg);
        let text_secondary = Color::from(spec.fg_secondary);
        let text_muted = Color::from(spec.fg_muted);
        let accent = Color::from(spec.accent);
        let border_subtle = Color::from(spec.border);
        let success = Color::from(spec.success);
        let warning = Color::from(spec.warning);
        let danger = Color::from(spec.danger);

        let mix = |a: Color, b: Color, t: f32| -> Color {
            let lerp = |x: u8, y: u8| -> u8 {
                ((x as f32) + ((y as f32) - (x as f32)) * t).clamp(0.0, 255.0) as u8
            };
            Color::rgb(
                lerp(a.red(), b.red()),
                lerp(a.green(), b.green()),
                lerp(a.blue(), b.blue()),
            )
        };

        let surface_hover = mix(surface_elevated, text_primary, 0.08);
        let surface_active = mix(surface_elevated, accent, 0.18);
        let surface_selected = mix(surface_elevated, accent, 0.14);

        let border_hover = mix(border_subtle, text_primary, 0.18);
        let border_focus = Color::rgba(accent.red(), accent.green(), accent.blue(), 145);
        let border_danger = Color::rgba(danger.red(), danger.green(), danger.blue(), 160);
        let border_specular = if spec.is_dark {
            Color::rgba(255, 255, 255, 22)
        } else {
            Color::rgba(0, 0, 0, 24)
        };

        let accent_luma = 0.299 * (accent.red() as f32)
            + 0.587 * (accent.green() as f32)
            + 0.114 * (accent.blue() as f32);
        let text_on_accent = if accent_luma > 150.0 {
            Color::from("#000000")
        } else {
            Color::WHITE
        };

        let acrylic_bg = Color::rgba(surface.red(), surface.green(), surface.blue(), 248);

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
            border_specular,
            text_primary,
            text_secondary,
            text_muted,
            text_on_accent,
            text_on_elevated: text_primary,
            text_on_danger: Color::WHITE,
            text_on_danger_critical: Color::WHITE,
            accent,
            accent_hover: mix(accent, text_primary, 0.2),
            accent_subtle: mix(surface, accent, 0.16),
            danger,
            danger_critical: danger,
            danger_subtle: mix(surface, danger, 0.16),
            warning,
            warning_subtle: mix(surface, warning, 0.16),
            success,
            steel: text_muted,
            acrylic_bg,
            acrylic_border: border_subtle,
            card_bg: surface_elevated,
            card_border: border_subtle,
            indicator_active: accent,
            font_family: "IBM Plex Sans".to_string(),
            font_sans: "IBM Plex Sans".to_string(),
            font_mono: "JetBrainsMono Nerd Font".to_string(),
            accent_tint: Color::rgba(accent.red(), accent.green(), accent.blue(), 38),
        }
    }

    pub fn from_manifest(manifest: &ShellManifest) -> Self {
        let style = manifest.theme.style.to_lowercase();
        let mut custom_spec = None;
        let mut tokens = if style.contains("ios")
            || style.contains("apple")
            || style.contains("bento")
            || style.contains("glass")
        {
            Self::ios_glass()
        } else if style == "playful-clay" || style.contains("playful") {
            Self::playful_clay()
        } else if style == "controlled-chaos" || style.contains("chaos") {
            Self::dark_controlled_chaos()
        } else if style == "obsidian-mint" {
            Self::obsidian_mint()
        } else if style == "nordic-slate" || style == "win11-dark" || style == "minimal" {
            let spec =
                crate::services::palettes::find_palette_or_alias(&style, manifest.theme.dark_mode);
            custom_spec = Some(spec);
            Self::from_palette_spec(spec)
        } else if let Some(spec) = crate::services::palettes::find_palette(&style) {
            if spec.id == "cyber-synthwave" {
                Self::cyber_synthwave()
            } else {
                custom_spec = Some(spec);
                Self::from_palette_spec(spec)
            }
        } else {
            Self::cyber_synthwave()
        };

        let default_palette = crate::config::PaletteConfig::default();
        let is_synthwave = style == "cyber-synthwave"
            || (custom_spec.is_none()
                && !style.contains("ios")
                && !style.contains("apple")
                && !style.contains("bento")
                && !style.contains("glass")
                && !style.contains("clay")
                && !style.contains("playful")
                && !style.contains("chaos")
                && !style.contains("controlled")
                && !style.contains("mint"));

        let p = &manifest.palette;
        let palette_active = *p != default_palette;
        if is_synthwave {
            if p.background != default_palette.background {
                tokens.bg = Color::from(p.background.as_str());
            }
            if p.surface != default_palette.surface {
                let sc = Color::from(p.surface.as_str());
                tokens.surface = sc;
                tokens.acrylic_bg = Color::rgba(sc.red(), sc.green(), sc.blue(), 250);
            }
            if p.overlay != default_palette.overlay {
                tokens.surface_elevated = Color::from(p.overlay.as_str());
                tokens.card_bg = Color::from(p.overlay.as_str());
            }
            if p.border != default_palette.border {
                tokens.border_subtle = Color::from(p.border.as_str());
                tokens.acrylic_border = Color::from(p.border.as_str());
                tokens.card_border = Color::from(p.border.as_str());
            }
            if p.foreground != default_palette.foreground {
                tokens.text_primary = Color::from(p.foreground.as_str());
            }
            if p.hover != default_palette.hover {
                tokens.text_secondary = Color::from(p.hover.as_str());
                tokens.accent_hover = Color::from(p.hover.as_str());
            }
            if p.muted != default_palette.muted {
                tokens.text_muted = Color::from(p.muted.as_str());
            }
            if p.accent != default_palette.accent {
                let ac = Color::from(p.accent.as_str());
                tokens.accent = ac;
                tokens.border_focus = Color::rgba(ac.red(), ac.green(), ac.blue(), 140);
                tokens.indicator_active = ac;
                tokens.accent_tint = Color::rgba(ac.red(), ac.green(), ac.blue(), 38);
            }
            if p.highlight != default_palette.highlight {
                tokens.warning = Color::from(p.highlight.as_str());
            }
            if p.steel != default_palette.steel {
                tokens.steel = Color::from(p.steel.as_str());
            }
            if p.success != default_palette.success {
                tokens.success = Color::from(p.success.as_str());
            }
            if p.danger_critical != default_palette.danger_critical {
                let dc = Color::from(p.danger_critical.as_str());
                tokens.danger_critical = dc;
                tokens.border_danger = Color::rgba(dc.red(), dc.green(), dc.blue(), 160);
            }
        } else if let Some(spec) = custom_spec {
            if palette_active && p.accent != spec.accent {
                let ac = Color::from(p.accent.as_str());
                tokens.accent = ac;
                tokens.border_focus = Color::rgba(ac.red(), ac.green(), ac.blue(), 145);
                tokens.indicator_active = ac;
                tokens.accent_tint = Color::rgba(ac.red(), ac.green(), ac.blue(), 38);
            }
        } else if palette_active {
            let ac = Color::from(p.accent.as_str());
            tokens.accent = ac;
            tokens.border_focus = Color::rgba(ac.red(), ac.green(), ac.blue(), 140);
            tokens.indicator_active = ac;
            tokens.accent_tint = Color::rgba(ac.red(), ac.green(), ac.blue(), 38);
        }

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
    fn test_tokens_from_manifest_defaults_to_cyber_synthwave() {
        let manifest = ShellManifest::default();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.bg, Color::from("#000000"));
        assert_eq!(tokens.text_primary, Color::from("#EAF8FA"));
        assert_eq!(tokens.text_secondary, Color::from("#97DCE3"));
        assert_eq!(tokens.text_muted, Color::from("#ABA5DB"));
        assert_eq!(tokens.surface_track, Color::from("#1A1336"));
        assert_eq!(tokens.accent, Color::from("#04C5F0"));
        assert_eq!(tokens.font_sans, "IBM Plex Sans");
        assert_eq!(tokens.text_on_accent, Color::from("#000000"));
        assert_eq!(tokens.text_on_elevated, Color::from("#61E6FF"));
    }

    #[test]
    fn test_tokens_from_manifest_obsidian_mint_style() {
        let mut manifest = ShellManifest::default();
        manifest.theme.style = "obsidian-mint".to_string();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(
            tokens.text_primary,
            Color::from(palette::OBSIDIAN_MINT_PRIMARY_TEXT)
        );
        assert_eq!(tokens.accent, Color::from(palette::OBSIDIAN_MINT_ACCENT));
    }

    #[test]
    fn test_tokens_from_manifest_controlled_chaos_style() {
        let mut manifest = ShellManifest::default();
        manifest.theme.style = "controlled-chaos".to_string();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.text_primary, Color::from(palette::BONE));
        assert_eq!(tokens.accent, Color::from(palette::INDUSTRIAL_AMBER));
    }

    #[test]
    fn test_tokens_from_manifest_playful_clay_style() {
        let mut manifest = ShellManifest::default();
        manifest.theme.style = "playful-clay".to_string();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.text_primary, Color::from(palette::CLAY_TEXT_PRIMARY));
        assert_eq!(tokens.accent, Color::from(palette::CLAY_PRIMARY));
    }

    #[test]
    fn test_tokens_from_manifest_ios_glass_style() {
        let mut manifest = ShellManifest::default();
        manifest.theme.style = "ios-glass".to_string();
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.text_primary, Color::WHITE);
        assert_eq!(tokens.accent, Color::from("#007AFF"));
    }

    #[test]
    fn test_tokens_from_manifest_curated_palettes_and_counterparts() {
        for (id, expected_bg, expected_accent) in [
            ("gruvbox-dark-hard", "#1d2021", "#fe8019"),
            ("gruvbox-light-soft", "#f2e5bc", "#af3a03"),
            ("everforest-dark-medium", "#2d353b", "#a7c080"),
            ("ayu-mirage", "#1f2430", "#ffcc66"),
            ("catppuccin-latte", "#eff1f5", "#8839ef"),
            ("tokyo-night-storm", "#24283b", "#7aa2f7"),
            ("kanagawa-wave", "#1f1f28", "#7e9cd8"),
            ("monokai-pro", "#2d2a2e", "#ffd866"),
        ] {
            let mut manifest = ShellManifest::default();
            manifest.theme.style = id.to_string();
            let tokens = ThemeTokens::from_manifest(&manifest);
            assert_eq!(tokens.bg, Color::from(expected_bg), "bg mismatch for {id}");
            assert_eq!(
                tokens.accent,
                Color::from(expected_accent),
                "accent mismatch for {id}"
            );
        }

        let cat_mocha = crate::services::palettes::find_palette("catppuccin-mocha").unwrap();
        assert_eq!(cat_mocha.counterpart, "catppuccin-latte");
        assert!(cat_mocha.is_dark);
    }

    #[test]
    fn test_tokens_cyan_accent_override_on_curated_palette() {
        let mut manifest = ShellManifest::default();
        let gruv = crate::services::palettes::find_palette("gruvbox-dark-medium").unwrap();
        crate::services::appearance::transition_manifest_palette(&mut manifest, gruv);
        crate::services::appearance::transition_manifest_accent(&mut manifest, "#04C5F0");
        let tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(tokens.bg, Color::from("#282828"));
        assert_eq!(tokens.accent, Color::from("#04C5F0"));

        manifest.theme.style = "nordic-slate".to_string();
        let nordic_tokens = ThemeTokens::from_manifest(&manifest);
        assert_eq!(nordic_tokens.bg, Color::from("#2e3440"));
        assert_eq!(nordic_tokens.accent, Color::from("#04C5F0"));
    }
}
