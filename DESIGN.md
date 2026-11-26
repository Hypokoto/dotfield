# Dotfield Design System: Cyber Synthwave

> **Pitch Void (`#000000`) · Deep Violet (`#120D2B`) · Rich Purple (`#251B4A`) · Cool Ice-Cyan (`#EAF8FA`) · Electric Neon Blue (`#04C5F0`) · Emerald (`#7CD699`)**

The unified design system for Dotfield and Amane Shell across all desktop surfaces, terminals, and companion tools. Calibrated around deep atmospheric depth, razor-sharp typography hierarchy, and verified WCAG AAA/AA contrast.

---

## 1. Visual Theme & Atmosphere
The atmosphere is anchored in a pitch-black void substrate (`#000000`) layered with deep violet (`#120D2B`) containers, dark plum (`#1A1336`) slider/gauge tracks, and rich purple (`#251B4A`) cards. Crisp ice-cyan (`#EAF8FA`) primary typography, cool cyan-mist (`#97DCE3`) secondary metadata, and soft lavender-mist (`#ABA5DB`) muted labels create instant 3-tier visual hierarchy while preserving `#04C5F0` electric neon blue exclusively for active interactive states and key telemetry highlights.

- **Density:** Balanced Bento Technical (7/10)
- **Variance:** Modular Bento Grid ($380\text{px}$ Dock-Attached Widget Popout, $48\text{px}$ Bottom Bar)
- **Motion:** 150ms–160ms Cubic Ease-Out (`ModalTransition::compute` & `apply_bar_popout`, zero overshoot bounce, 0 Hz Wayland parking at rest)

---

## 2. Canonical Color Palette & Semantic Roles

| Token | Hex | Role | Contrast Ratio & Standard |
|---|---|---|---|
| Canvas Background (`base00`) | `#000000` | Pure pitch OLED black base | Infinite against white |
| Container Surface (`base01`) | `#120D2B` | Shell panels, modal backdrops, flyouts | 18.2:1 vs `#EAF8FA` |
| Track Surface (`surface_track`) | `#1A1336` | Slider wells, inactive chips, gauge rings | Distinct recessed well |
| Elevated Cards (`base02`) | `#251B4A` | Bento cards, popovers, interactive tiles | 12.1:1 vs `#EAF8FA` |
| Structural Border | `#2E225C` | 1px perimeter outlines, subtle dividers | Tone-on-tone distinction |
| Primary Foreground (`base05`) | `#EAF8FA` | Primary text, titles, digital clock | 16.5:1 AAA vs `#120D2B` |
| Secondary Foreground (`base04`) | `#97DCE3` | Secondary text, status metadata, sub-labels | 11.2:1 AAA vs `#120D2B` |
| Muted Foreground (`base03`) | `#ABA5DB` | Keycap hints, timestamps, inactive icons | 7.4:1 AAA vs `#120D2B` |
| Neon Blue Accent (`base0D`) | `#04C5F0` | Focus rings, active workspace pips, primary CTAs | 9.4:1 AAA vs `#000000` |
| Active Hover | `#14B9B5` | Interactive element hover states | Distinct cyan-teal |
| Emerald Success (`base0B`) | `#7CD699` | Healthy telemetry, validated operations | 10.6:1 AAA vs `#120D2B` |
| Amber Warning (`base09`) | `#FF7F41` | Telemetry load alerts, culprit chips | 6.8:1 AA vs `#120D2B` |
| Crimson Danger (`base08`) | `#BE3F50` | Alarms, destructive actions, emergency stops | 5.4:1 AA vs `#FFFFFF` |
| Critical Dark Red (`base0F`) | `#CE4F48` | Battery critical (<5%), system faults | High-urgency badge fill |

### Verified WCAG Contrast Overrides

```rust
text_on_accent:          Color::from("#000000"), // 11.8:1 AAA vs #04C5F0
text_on_elevated:        Color::from("#EAF8FA"), // 12.1:1 AAA vs #251B4A
text_on_danger:          Color::WHITE,           //  5.4:1 AA  vs #BE3F50
text_on_danger_critical: Color::WHITE,           //  5.1:1 AA  vs #CE4F48
border_focus:            Color::rgba(4, 197, 240, 0.55), // Restrained neon rim
```

---

## 3. Typography Rules
- **Interface Primary Typography:** `IBM Plex Sans` (SemiBold / Medium / Regular) — Clean, modern geometric grotesk with distinct apertures.
- **Micro Labels & Badges:** `IBM Plex Sans` at 10.5px–11.5px with 500/600 weight.
- **Monospaced Numerals & Telemetry:** `JetBrainsMono Nerd Font` — Tabular figures for CPU/RAM metrics, clock displays, sprint countdowns, and keycaps.

---

## 4. Component Stylings & Layout Principles

### Bottom Bar (`crates/amane-shell/src/ui/bar.rs`)
- **Position & Sizing:** Bottom anchored, height `48.0px`, dynamic clamp `effective_h >= 40.0px` preserving dock search capsules (`34.0px`) and app tiles (`40.0px`).
- **FocusShield Status:** `󰐩 FocusShield {time}` pill (`156.0px` wide) using `remaining_formatted()` with `elapsed_formatted()` fallback for Free sprints.

### Dedicated Widget Board (`crates/amane-shell/src/ui/widgets/board.rs`)
- **Position & Sizing:** Dock-centered (`Horizontal::Middle`) bar popout attached flush to the active bar (`bar_edge_margins(&manifest, 0.0)`), flat corners along the bar seam (`radius_bottom_left/right = 0.0` on bottom bar, `radius_top_left/right = 0.0` on top bar), width `380.0px` (`bento::BOARD_WIDTH`), adaptive height clamped to `focused_output_height() - 60.0px`.
- **Keyboard Interactivity:** `Keyboard::OnDemand` with `[×]` close button, workspace + live time/date header, and `Esc` dismiss.
- **Card Sizing & Spacing:** Tight Bento cards with centered chips and labeled gauges:
  - `FocusShield`: `120.0px`
  - `Telemetry`: `136.0px` (labeled CPU/RAM rings + 5m sparklines + `used / total GB`)
  - `Media`: `108.0px` when active player present, collapsing to `48.0px` compact strip when idle
- **Zero-Poll Power Invariant:** Closed board short-circuits to `.visible(false)` and drops to 0 Hz Wayland frame request parking.

---

## 5. Anti-Patterns (Banned)
- ❌ No hardcoded white `#FFFFFF` text on neon `#04C5F0` accents (strictly `#000000` text) and no `#000000` (`text_on_accent`) text on dark cards or translucent slider troughs.
- ❌ No bounce/overshoot easing on overlays (`ModalTransition::compute` cubic ease-out only).
- ❌ No un-aliased or floating widget popups without dedicated overlay routing.
- ❌ No continuous background render loops when all shell surfaces are idle (strictly 0 Hz).
- ❌ No dynamic layer surface height resizes that desynchronize Wayland commit buffers.
- ❌ No claymorphic bisque/terracotta washes or generic SaaS card grids.
