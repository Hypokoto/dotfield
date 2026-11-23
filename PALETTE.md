# Obsidian Mint

> **Obsidian · Bioluminescent Mint · Emerald · Amber Gold · Rose Critical**

A dark, high-contrast bioluminescent cyber-botanical color system for `dotfield`, designed around focus, deep clarity, and effortless visual hierarchy.

The interface pairs an ultra-deep mineral obsidian substrate with crisp mint and emerald accents, delivering zero eye strain and strict WCAG 2.2 AAA contrast across all Wayland and CLI surfaces.

## 1. Philosophy

| Principle | Color | Meaning |
|---|---|---|
| Focus | Deep Void (`#080B0B`) | Minimal distraction, infinite depth |
| Growth / Action | Bioluminescent Mint (`#2DD4BF`) | Primary interactive state, high-contrast focal points |
| Stability | Emerald (`#10B981`) | Success, confirmed states, secondary guidance |
| Awareness | Amber Gold (`#FBBF24`) | Warnings, pending states, highlights |
| Urgency | Rose Critical (`#F43F5E`) | Critical alarms, destruction, system errors |
| Structure | Obsidian / Graphite | Layered spatial depth without high-frequency visual noise |
| Clarity | Minted White (`#ECFDF5`) | Ultra-crisp typography (14.8:1 contrast ratio) |

**Design principle:** 60-30-10 Balance with WCAG AAA precision. 60% Void/Canvas, 30% Obsidian/Graphite surfaces, 10% Mint/Emerald accents.

---

## 2. Core Color Palette

| Token | Hex | Contrast vs `#080B0B` | Purpose |
|---|---|---|---|
| Void | `#080B0B` | 1:1 | Deep canvas / root background |
| Obsidian | `#0E1413` | 1.1:1 | Panels, windows, base cards |
| Graphite | `#16201E` | 1.3:1 | Elevated surfaces, hover states, chips |
| Iron | `#23322F` | 1.8:1 | Subtle borders, hairpins, dividers |
| Steel | `#71827E` | 4.6:1 (AA) | Neutral icons, metadata, inactive indicators |
| Ash | `#94A3A0` | 5.8:1 (AA) | Secondary text, muted labels |
| Minted White | `#ECFDF5` | 14.8:1 (AAA) | Primary typography, prominent headers |
| Mint | `#2DD4BF` | 9.1:1 (AAA) | Primary accent, focused borders, active tabs |
| Emerald | `#10B981` | 7.4:1 (AAA) | Success, confirmed states, secondary highlights |
| Amber Gold | `#FBBF24` | 9.6:1 (AAA) | Warning states, search match, telemetry alerts |
| Rose Critical | `#F43F5E` | 7.0:1 (AAA) | Urgency, error, destructive operations |
| Deep Rose | `#9F1239` | 2.5:1 | Urgent backgrounds, danger highlights |

### Tints & Translucency

- **Mint Tint**: `#2DD4BF26` (15% opacity overlay)
- **Acrylic Background**: `#0E1413fa` (98% acrylic blur surface)
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
