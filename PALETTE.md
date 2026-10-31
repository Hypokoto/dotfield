# Controlled Chaos

> **Void Black · Crimson · Industrial Amber · Steel**

A dark, industrial color system for `dotfied`, designed around three principles: chaos, discipline, and productivity.

The interface remains restrained and functional while selective accents communicate intensity, focus, and action.

## 1. Philosophy

| Principle | Color | Meaning |
|---|---|---|
| Chaos | Crimson | Intensity, disruption, raw energy |
| Discipline | Steel | Structure, restraint, consistency |
| Productivity | Industrial Amber | Action, focus, forward momentum |
| Focus | Void Black | Minimal distraction |
| Clarity | Bone White | Readability and information hierarchy |
| Stability | Graphite | Consistent surfaces and structure |

**Design principle:** Controlled contrast. The interface provides order; the accents provide intensity.

These are visual associations, not scientifically established effects on productivity.

---

## 2. Core Color Palette

| Token | Hex | Purpose |
|---|---|---|
| Void | `#0A0A0C` | Primary background |
| Obsidian | `#121216` | Windows, panels, and cards |
| Graphite | `#1C1C23` | Elevated surfaces |
| Iron | `#34343D` | Borders and separators |
| Ash | `#A1A1AA` | Secondary text |
| Bone | `#E8E6E3` | Primary text |
| Crimson | `#E63946` | Chaos and critical emphasis |
| Blood Red | `#9D0208` | Critical warnings and dangerous operations |
| Industrial Amber | `#FFB000` | Primary accent and active states |
| Pale Amber | `#FFD166` | Hover states and highlighted information |
| Steel | `#8B949E` | Neutral states and secondary controls |
| Success Green | `#7BC47F` | Successful operations |

### Color hierarchy

1. **Void** — dominant background.
2. **Obsidian and Graphite** — interface surfaces.
3. **Bone and Ash** — readable information.
4. **Industrial Amber** — primary interactions.
5. **Crimson** — errors, urgency, and destructive actions.
6. **Steel** — neutral states and structural elements.

Do not use every accent everywhere. Each color must communicate a specific meaning.

---

## 3. Component Theme Mapping

### Sway and Niri

| Element | Color |
|---|---|
| Background | `#0A0A0C` |
| Inactive window border | `#34343D` |
| Active window border | `#FFB000` |
| Urgent window border | `#E63946` |
| Primary text | `#E8E6E3` |
| Secondary text | `#A1A1AA` |

Keep the compositor visually quiet. Use amber to identify focus and crimson to identify urgency.

### Terminals

Applies to Foot, Kitty, Ghostty, and Alacritty.

| Element | Color |
|---|---|
| Background | `#0A0A0C` |
| Foreground | `#E8E6E3` |
| Cursor | `#FFB000` |
| Selection background | `#34343D` |
| Selection foreground | `#E8E6E3` |

Use a consistent ANSI palette across all supported terminal emulators.

### Fuzzel and Rofi

| Element | Color |
|---|---|
| Background | `#121216` |
| Selected entry | `#1C1C23` |
| Selected-entry indicator | `#FFB000` |
| Matching text | `#E63946` |
| Primary text | `#E8E6E3` |
| Secondary text | `#A1A1AA` |

Use amber for selection and crimson for matched search terms. Keep the rest of the interface neutral.

### Mako

| Element | Color |
|---|---|
| Background | `#121216` |
| Text | `#E8E6E3` |
| Normal border | `#34343D` |
| Urgent border | `#E63946` |

Notifications should remain compact. Reserve crimson for urgent notifications rather than using it for every message.

### Rust + Slint Profile Switcher

| Element | Color |
|---|---|
| Main background | `#0A0A0C` |
| Cards | `#121216` |
| Elevated cards | `#1C1C23` |
| Borders | `#34343D` |
| Primary text | `#E8E6E3` |
| Secondary text | `#A1A1AA` |
| Active profile border | `#FFB000` |
| Primary action button | `#FFB000` |
| Primary button text | `#0A0A0C` |
| Error message | `#E63946` |
| Disabled controls | `#8B949E` |

Avoid decorative animations, glowing borders, and unnecessary transitions. Prioritize responsive controls, readable information, and explicit feedback.

---

## 4. Semantic Status Colors

Colors should represent application state consistently across the CLI, GUI, and desktop.

| State | Color | Hex |
|---|---|---|
| Active profile | Industrial Amber | `#FFB000` |
| Changes pending | Industrial Amber | `#FFB000` |
| Configuration valid | Success Green | `#7BC47F` |
| Deployment successful | Success Green | `#7BC47F` |
| Configuration drift | Crimson | `#E63946` |
| Deployment failed | Crimson | `#E63946` |
| Security policy failure | Crimson | `#E63946` |
| Critical operation | Blood Red | `#9D0208` |
| Informational state | Steel | `#8B949E` |
| Disabled feature | Steel | `#8B949E` |

**Important:** An active profile is not necessarily secure. A successful deployment does not prove that the resulting security policy is correct. Display these states separately.

---

## 5. Terminal ANSI Palette

Use this palette for consistent terminal output and syntax highlighting.

| ANSI Color | Normal | Bright |
|---|---|---|
| Black | `#1C1C23` | `#53535D` |
| Red | `#E63946` | `#FF6B76` |
| Green | `#7BC47F` | `#9AE6A0` |
| Yellow | `#FFB000` | `#FFD166` |
| Blue | `#82AAFF` | `#A4C0FF` |
| Magenta | `#C792EA` | `#D9ABF4` |
| Cyan | `#8B949E` | `#B0B8C1` |
| White | `#E8E6E3` | `#FFFFFF` |

Use semantic colors deliberately in CLI output:

- Amber: actions requiring attention.
- Crimson: errors and dangerous operations.
- Green: confirmed success.
- Steel: neutral information.
- Bone: ordinary output.

Avoid communicating critical information through color alone. Include explicit text such as `ERROR`, `WARNING`, or `SUCCESS`.

---

## 6. Canonical Theme Definition

Suggested location:

`theme/controlled-chaos.toml`

```toml
[meta]
name = "Controlled Chaos"
variant = "dark"
description = "Industrial dark theme for dotfied"

[colors]
background = "#0A0A0C"
surface = "#121216"
surface_alt = "#1C1C23"
border = "#34343D"

foreground = "#E8E6E3"
foreground_muted = "#A1A1AA"
disabled = "#8B949E"

[accent]
chaos = "#E63946"
discipline = "#8B949E"
productivity = "#FFB000"

[semantic]
success = "#7BC47F"
warning = "#FFB000"
error = "#E63946"
critical = "#9D0208"
info = "#8B949E"

[interaction]
accent_hover = "#FFD166"
selection = "#34343D"
focus = "#FFB000"
```

This is the canonical source of truth for the palette.

Application-specific configurations should consume or derive from these tokens wherever practical. Keep generated configurations separate from hand-maintained files so regeneration does not overwrite custom changes.

---

## 7. Implementation Rules

1. **One canonical palette:** Do not independently invent colors for individual applications.
2. **Consistent semantics:** A red error in the CLI should represent the same class of failure in the GUI.
3. **Minimal accents:** Use amber for interaction and crimson for danger; keep most surfaces neutral.
4. **Readable contrast:** Verify text and control contrast, especially for secondary text and disabled controls.
5. **No color-only communication:** Pair status colors with labels, icons, or other recognizable indicators.
6. **Minimal animation:** Prefer immediate feedback over decorative motion.
7. **Profile independence:** Changing from Sway to Niri must not change the theme's identity.
8. **Security clarity:** Distinguish configuration validity, deployment success, and actual security enforcement.

---

## Final Identity

**Controlled Chaos**

- Void Black provides focus.
- Crimson represents intensity.
- Steel represents restraint.
- Industrial Amber represents deliberate action.
- Bone White provides clarity.

The objective is not to make every component visually aggressive. It is to build a disciplined environment in which the few strong accents have clear meaning.
