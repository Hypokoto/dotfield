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
| Background | `#080B0B` |
| Inactive window border | `#23322F` |
| Active window border | `#2DD4BF` |
| Urgent window border | `#F43F5E` |
| Primary text | `#ECFDF5` |
| Secondary text | `#94A3A0` |

### Terminals (Foot, Ghostty, Kitty)

| Element | Color |
|---|---|
| Background | `#080B0B` |
| Foreground | `#ECFDF5` |
| Cursor | `#2DD4BF` |
| Cursor Text | `#080B0B` |
| Selection background | `#16201E` |
| Selection foreground | `#ECFDF5` |

### Fuzzel & App Launcher

| Element | Color |
|---|---|
| Background | `#080B0Bf2` |
| Prompt / Border | `#2DD4BFff` |
| Primary text | `#ECFDF5ff` |
| Selection background | `#16201Eff` |
| Selection text | `#2DD4Bfff` |
| Matching text | `#10B981ff` |

### Mako & Notification HUD

| Element | Color |
|---|---|
| Background | `#080B0B` |
| Text | `#ECFDF5` |
| Normal border | `#2DD4BF` |
| Urgent border | `#F43F5E` |
| Progress bar | `#2DD4BF` |

### Amane Desktop Shell (Niri Status Bar, Flyouts, Control Center, Overview)

| Element | Color |
|---|---|
| Background / Panels | `#080B0B` / `#0E1413` |
| Cards / Elevated surfaces | `#16201E` |
| Borders & Dividers | `#23322F` |
| Primary text / Headings | `#ECFDF5` |
| Secondary / Muted labels | `#94A3A0` |
| Primary Accent & Active Pips | `#2DD4BF` (Bioluminescent Mint) |
| Accent Hover & Highlights | `#10B981` (Emerald) |
| Warnings & Notifications | `#FBBF24` (Amber Gold) |
| Critical / Alarms / Mute | `#F43F5E` (Rose Critical) |
| Neutral Controls & Muted Dots | `#71827E` (Steel) |

---

## 4. Semantic Status Colors

| State | Color | Hex |
|---|---|---|
| Active / Focused | Bioluminescent Mint | `#2DD4BF` |
| Success / Confirmed | Emerald | `#10B981` |
| Warning / Pending | Amber Gold | `#FBBF24` |
| Error / Critical | Rose Critical | `#F43F5E` |
| Destructive Operation | Deep Rose | `#9F1239` |
| Neutral / Inactive | Steel | `#71827E` |

---

## 5. Terminal ANSI Palette

| ANSI Color | Normal | Bright |
|---|---|---|
| Black | `#16201E` | `#23322F` |
| Red | `#F43F5E` | `#FB7185` |
| Green | `#10B981` | `#34D399` |
| Yellow | `#FBBF24` | `#FDE047` |
| Blue | `#38BDF8` | `#7DD3FC` |
| Magenta | `#C084FC` | `#E879F9` |
| Cyan | `#2DD4BF` | `#5EEAD4` |
| White | `#ECFDF5` | `#FFFFFF` |

---

## 6. Implementation Rules

1. **One canonical palette:** Every tool and compositor surface resolves to Obsidian Mint tokens.
2. **WCAG AAA Compliance:** Primary text and key interaction targets strictly maintain ≥ 7:1 contrast against canvas.
3. **Restrained Glow:** Bioluminescent accents highlight state changes without aggressive over-saturation.
4. **Semantic Rigor:** Errors are Rose (`#F43F5E`), Warnings are Amber (`#FBBF24`), Active elements are Mint (`#2DD4BF`), Success is Emerald (`#10B981`).
