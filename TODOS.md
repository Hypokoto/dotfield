# TODOS.md — Dotfield & Amane Roadmap Backlog

This document captures architectural enhancements, deferred features, and expansion tracks documented during system design and specification reviews.

---

## Ambient Desktop & Widgets Ecosystem

### `CEO-EXP-3`: Ambient Desktop Grid (`desktop.rs`)
- **Status**: Backlog / Post-Phase 17
- **Target Subsystem**: `crates/amane-shell/src/ui/desktop.rs`
- **Description**: Render an ambient, non-interactive (or semi-interactive) desktop grid on `amane::Layer::Bottom` across configured monitors.
- **Key Requirements**:
  - Optional `[desktop_widgets]` section in `shell.toml` (`enabled = false` by default).
  - Multi-monitor support: display on non-focused monitors or behind active tiling windows.
  - Smart DPMS integration: suspend all rendering when outputs sleep or when covered by fullscreen surfaces.
  - Zero performance overhead when covered; adhere to the 0 Hz frame parking philosophy.

### `CEO-EXP-4`: Interactive Widget Board Customizer (`[Customize]` Drawer)
- **Status**: Obsolete / Closed (`ui/widgets/board.rs` removed in `1bded9e`)
- **Target Subsystem**: `crates/amane-shell/src/ui/widgets/board.rs`

---

## Amane Settings UI (`Phase 18` Deferred Expansions)

### `CEO-EXP-5`: Live KDL Diff Preview (`services/niri_config.rs`)
- **Status**: Backlog / Post-Phase 18
- **Target Subsystem**: `crates/amane-shell/src/ui/settings.rs`, `crates/amane-shell/src/services/niri_config.rs`
- **Description**: Render an inline unified diff of `99-amane-settings.kdl` inside a confirmation popover before running `niri validate` and `niri msg action load-config-file`.
