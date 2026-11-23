# Cyber Synthwave

> **Void Black · Deep Violet · Rich Purple · Cyan-Teal · Neon Blue · Emerald**

A high-contrast cyber-synthwave color system for `dotfield` and `amane`, designed around maximum readability, deep atmospheric depth, and neon precision.

The interface pairs a pitch-black void substrate (`#000000`) and deep violet/purple surfaces (`#140e29`, `#251b4a`) with high-contrast cyan-teal (`#14B9B5`), electric blue (`#04C5F0`), amber gold (`#FF7F41`, `#ffbe74`), and purple highlights (`#9147a8`).

## 1. Philosophy

| Principle | Color | Meaning |
|---|---|---|
| Focus | Pitch Void (`#000000`) | Clean infinite contrast, true OLED blacks |
| Structure | Deep Violet (`#140e29`) & Rich Purple (`#251b4a`) | Atmospheric layering without gray wash |
| Foreground | Cyan-Teal (`#14B9B5`) | Razor-sharp text and active reading clarity |
| Action | Neon Blue (`#04C5F0`) | High-priority interactive indicators and focus borders |
| Guidance | Emerald (`#7cd699`) | Success states, strings, validated operations |
| Warmth | Amber Peach (`#ffbe74`) | Muted comments, keycap hints, secondary icons |
| Urgency | Crimson & Rose (`#BE3F50`, `#CE4F48`) | Alarms, destructive actions, battery critical |
| Magic | Electric Purple (`#9147a8`) | Active borders, keywords, visual highlights |

---

## 2. Core Color Palette

| Token | Hex | Role |
|---|---|---|
| Background (`base00`) | `#000000` | Deep canvas / root background |
| Surface (`base01`) | `#140e29` | Top bar, modal backdrops, window chrome |
| Overlay (`base02`) | `#251b4a` | Elevated cards, calendar, popovers, borders |
| Muted (`base03` / `base07`) | `#ffbe74` | Comments, subtle captions, keycaps |
| Statusline (`base04`) | `#11AEB3` | Secondary text, status indicators |
| Foreground (`base05`) | `#14B9B5` | Primary text typography, titles, clock |
| Pure Fallback (`base06`) | `#ffffff` | High-contrast fallback text |
| Red Accent (`base08`) | `#BE3F50` | Variables, danger accents |
| Warning Orange (`base09`) | `#FF7F41` | Warnings, telemetry alerts |
| Yellow Amber (`base0A`) | `#ffbe74` | Classes, search highlights |
| Strings Green (`base0B`) | `#7cd699` | Success, battery healthy, active connections |
| Cyan Blue (`base0D`) | `#04C5F0` | Primary accent, focused rings, workspace pills |
| Purple (`base0E`) | `#9147a8` | Active border hover glows, keywords |
| Dark Red (`base0F`) | `#CE4F48` | Deprecated, critical system errors |

### Tints & Translucency

- **Accent Tint**: `#04C5F026` (15% opacity neon blue glow)
- **Acrylic Background**: `#140e29fa` (98% acrylic blur surface)
- **Drop Shadow**: `rgba(0, 0, 0, 0.40)`

---

## 3. Component Theme Mapping

### Sway and Niri

| Element | Color |
|---|---|
| Background | `#000000` |
| Inactive window border | `#251b4a` |
| Active window border | `#04C5F0` |
| Urgent window border | `#BE3F50` |
| Primary text | `#14B9B5` |
| Secondary text | `#11AEB3` |

### Terminals (Foot, Ghostty, Kitty)

| Element | Color |
|---|---|
| Background | `#000000` |
| Foreground | `#14B9B5` |
| Cursor | `#04C5F0` |
| Cursor Text | `#000000` |
| Selection background | `#251B4A` |
| Selection foreground | `#FFFFFF` |

### Fuzzel & App Launcher

| Element | Color |
|---|---|
| Background | `#000000f2` |
| Prompt / Border | `#04C5F0ff` |
| Primary text | `#14B9B5ff` |
| Selection background | `#251B4Aff` |
| Selection text | `#FFFFFFff` |
| Matching text | `#04C5F0ff` |

### Mako & Notification HUD

| Element | Color |
|---|---|
| Background | `#000000` |
| Text | `#14B9B5` |
| Normal border | `#04C5F0` |
| Urgent border | `#BE3F50` |
| Progress bar | `#04C5F0` |

### Amane Desktop Shell (Niri Status Bar, Flyouts, Control Center, Overview)

| Element | Color |
|---|---|
| Background / Panels | `#000000` / `#140e29` |
| Cards / Elevated surfaces | `#251b4a` |
| Borders & Dividers | `#251b4a` |
| Primary text / Headings | `#14B9B5` |
| Secondary / Muted labels | `#11AEB3` |
| Primary Accent & Active Pips | `#04C5F0` (Electric Neon Blue) |
| Accent Hover & Highlights | `#9147a8` (Electric Purple) |
| Warnings & Notifications | `#FF7F41` (Warning Orange) |
| Critical / Alarms / Mute | `#BE3F50` (Red Accent) |
| Success & Status | `#7cd699` (Emerald Green) |

---

## 4. Semantic Status Colors

| State | Color | Hex |
|---|---|---|
| Active / Focused | Electric Neon Blue | `#04C5F0` |
| Success / Confirmed | Emerald Green | `#7cd699` |
| Warning / Pending | Warning Orange | `#FF7F41` |
| Error / Critical | Red Accent | `#BE3F50` |
| Deprecated / Severe | Dark Red | `#CE4F48` |
| Secondary / Status | Cyan-Teal | `#11AEB3` |

---

## 5. Terminal ANSI Palette

| ANSI Color | Normal | Bright |
|---|---|---|
| Black | `#140E29` | `#251B4A` |
| Red | `#BE3F50` | `#CE4F48` |
| Green | `#7CD699` | `#7CD699` |
| Yellow | `#FF7F41` | `#FFBE74` |
| Blue | `#04C5F0` | `#61E6FF` |
| Magenta | `#9147A8` | `#9147A8` |
| Cyan | `#11AEB3` | `#14B9B5` |
| White | `#14B9B5` | `#FFFFFF` |

---

## 6. Implementation Rules

1. **One canonical palette:** Every tool and compositor surface resolves to Cyber Synthwave tokens.
2. **WCAG AAA Compliance:** Primary text and key interaction targets strictly maintain ≥ 7:1 contrast against canvas.
3. **Restrained Glow:** Neon Blue and Purple accents highlight state changes without aggressive over-saturation.
4. **Semantic Rigor:** Errors are Red (`#BE3F50`), Warnings are Orange (`#FF7F41`), Active elements are Neon Blue (`#04C5F0`), Success is Green (`#7CD699`).
