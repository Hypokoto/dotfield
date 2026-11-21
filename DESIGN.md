# Design System: Dotfield Smooth Clay Aesthetic

## 1. Visual Theme & Atmosphere
A warm, tactile, pillowy interface inspired by smooth stoneware ceramics, sculpted clay pottery, and soft tactile industrial design. The atmosphere replaces bold, harsh, high-contrast borders and sharp metallic wireframes with **Smooth Claymorphic Softness** — generous continuous corner radii (16px to 28px, full pills), matte tone-on-tone clay surfaces, gentle top-lip specular highlights, and pillowy debossed interaction states. It feels calm, approachable, and deeply tactile — like smooth sculpted porcelain resting in soft natural ambient light.

- **Density:** Daily App Balanced (5/10)
- **Variance:** Offset Asymmetric & Tactile (6/10)
- **Motion:** Spring-Damped Fluidity (6/10)

---

## 2. Color Palette & Roles
The palette is calibrated strictly from organic ceramic clays, bisque, terracotta glazes, and warm stoneware:

- **Matte Dark Clay** (`#161413`) — Primary background canvas, velvety dark earthen clay base.
- **Warm Bisque** (`#1F1C1A`) — Primary container, modal, and drawer surface fill.
- **Terracotta Putty** (`#2B2623`) — Elevated cards, action plaques, and quick setting tiles.
- **Clay Inset Depth** (`#38322D`) — Active, pressed, and selected item fill.
- **Clay Lip Highlight** (`#3D3631`) — Subtle upper rim and pill borders (soft 1px tactile definition).
- **Subtle Clay Seam** (`#26221F`) — Inactive hairline dividers and subtle recessed boundaries.
- **Glazed Terracotta Amber** (`#E08244`) — Singular vibrant accent for active toggles, focus pills, and primary highlights (Saturation: 72%).
- **Light Terracotta** (`#ECA06E`) — Accent hover and active pill glow.
- **Warm Porcelain Sand** (`#F3EDE6`) — Primary high-contrast legible typography.
- **Muted Putty** (`#BDB4AA`) — Secondary metadata, sub-labels, and descriptions.
- **Stone Grey** (`#7D756C`) — Muted captions, placeholder text, and inactive glyphs.
- **Soft Crimson Glaze** (`#C84B4B`) — Destructive power actions and critical alerts.
- **Soft Sage Clay** (`#6B9E78`) — Success states and battery charging indicators.

*(Constraint: Pure black `#000000` is strictly banned; neon purple/blue outer glows are strictly banned).*

---

## 3. Typography Rules
- **Display & Modal Headers:** `Outfit` (Bold / SemiBold) — Smooth geometric sans with friendly rounded curves, relaxed tracking (`-0.01em`), title case / sentence case hierarchy rather than screaming caps.
- **Body & Secondary Labels:** `Outfit` (Regular / Medium) — Warm, open aperture, high legibility across small scale displays.
- **Telemetry & Numerals:** `JetBrainsMono Nerd Font` — Monospaced numerals for tabular clocks, percentages, and metrics.
- **Banned:** `Inter`, `Times New Roman`, generic serifs, gothic lettering, and medieval script are banned in this tactile clay design.

---

## 4. Component Stylings
* **Smooth Clay Modals & Panels:** Encased in continuous smooth pillowy rectangles (`radius::LG` = 22.0px to `radius::XL` = 28.0px). Double metallic brass wires and medieval filigrees (`"⚜"`, `"✥"`) are abolished in favor of a soft single **Clay Lip** (`#3D3631`) with a warm bisque fill (`#1F1C1A`).
* **Tactile Clay Cards:** Elevated with a soft tone-on-tone contrast (`#2B2623`), subtle top rim highlight, and generous 14px–18px rounded corners.
* **Pill Buttons & Toggles:** Sculpted pill-shaped elements (`radius::PILL` = 9999px) with tactile debossed feedback on click (`#38322D`). Active state glows warmly in Glazed Terracotta (`#E08244`).
* **Smooth Sliders:** Recessed clay track (`#161413`) with a pillowy ceramic gauge fill (`#E08244`) and rounded thumb indicator.
* **Dock & Navigation:** Smooth centered pill bar, clay search capsule with soft inner padding, and smooth gold-terracotta active indicators.
* **Power & Destructive Actions:** Smooth Soft Crimson ceramic tile (`#C84B4B`) with warm sand typography.

---

## 5. Layout Principles
- Continuous rounded geometry across all nesting levels (outer modal 26px → inner card 16px → pill button 9999px).
- Generous internal padding (14px–20px) creating soft breathing room around elements.
- Fluid symmetric and balanced card grids without sharp divider lines.

---

## 6. Motion & Interaction
- **Spring-Damped Transitions:** Smooth cubic-ease and spring physics for modal appearance and slider drags.
- **Tactile Feedback:** Buttons subtly press inward on active pointer interaction without jarring displacement.
- **Subtle Glows:** Gentle ceramic warmth instead of harsh blinding neon.

---

## 7. Anti-Patterns (Banned)
- ❌ No sharp rectangular corners (`0px` or `2px`).
- ❌ No double-line wireframe borders or metallic gold filigrees.
- ❌ No harsh neon blues, hot purples, or oversaturated rainbow gradients.
- ❌ No pure black (`#000000`) surfaces.
- ❌ No `Inter` or harsh editorial serif fonts in UI widgets.
- ❌ No emojis or generic AI clip art.
- ❌ No jarring hard borders (`>2px` black or neon outlines).
