# Phase 7 Design Document: Custom GUI Utilities (Slint) & Event Daemon (`dotfield-gui` & `dotfieldd`)

**Project:** Dotfield (`dotfield`)
**Date:** 2026-10-31
**Author:** Pair Programming Session (via `gstack-office-hours`)
**Status:** APPROVED
**Scope:** Phase 7 of 7 (Slint GUI Utilities & Hardware Event Daemon)

---

## Executive Summary

Phase 7 completes the Dotfield desktop suite by providing native Wayland graphical interfaces (`dotfield-gui`) and an automated event daemon (`dotfieldd`). Built on **Rust + Slint**, `dotfield-gui` gives users an instantaneous on-screen keybinding cheat-sheet (`--hud`) and visual profile manager (`--switcher`), styled in accordance with the Controlled Chaos palette. Meanwhile, `dotfieldd` listens to hardware state changes (such as AC versus Battery transitions via `/sys/class/power_supply`) to automatically trigger transactional generational profile deployments without human intervention.

---

## 1. Architectural Invariants & Mechanics

### 1.1 Crate Workspace Separation
```
dotfield/
├── Cargo.toml                       # Workspace definition
└── crates/
    ├── dotfield-core/               # Pure library crate (Phase 6)
    ├── dotfield-cli/                # Command-line binary crate (Phase 6)
    ├── dotfield-gui/                # Native Wayland Slint GUI crate (Phase 7)
    │   ├── Cargo.toml
    │   ├── build.rs                 # Compiles Slint UI definitions
    │   ├── ui/
    │   │   ├── palette.slint        # Controlled Chaos color tokens & shared styles
    │   │   ├── hud.slint            # Searchable Keybind cheat-sheet HUD
    │   │   └── switcher.slint       # Card-based Profile Switcher
    │   └── src/
    │       └── main.rs              # App entrypoint dispatching --hud or --switcher
    └── dotfieldd/                   # Event listener daemon crate (Phase 7)
        ├── Cargo.toml
        └── src/
            ├── main.rs              # Daemon entrypoint, signals, and dispatch
            └── power.rs             # /sys/class/power_supply AC/Battery monitor
```

### 1.2 Slint UI Components & Visual Tokens
In strict accordance with [`PALETTE.md`](../../PALETTE.md):
- **Backgrounds**: `Void` (`#0A0A0C`), `Obsidian` (`#121216`), `Graphite` (`#1C1C23`)
- **Borders & Dividers**: `Iron` (`#34343D`)
- **Foreground / Text**: `Bone` (`#E8E6E3`), `Ash` (`#A1A1AA` for muted secondary)
- **Accents**: `Amber` (`#FFB000`) for active selections/highlights, `Crimson` (`#E63946`) for security/hardening badges.

#### Keybind HUD (`--hud`)
- Queries active profile (`~/.dotfield/state.json` or fallback to `profiles/default.toml`).
- Displays a clean table of logical actions (`terminal`, `launcher`, `editor`, `multiplexer`, `lock`, `screenshot`) mapped to their modifier bindings.
- Includes a live search filter box for instantaneous keybind lookup.

#### Profile Switcher (`--switcher`)
- Enumerates available profiles in `profiles/*.toml` (`default`, `minimal`, `hardening`).
- Displays interactive cards showing profile metadata (apps, compositor, security tier, active indicator).
- Clicking "Apply" invokes `dotfield_core::GenerationManager::deploy` to atomically compile, stage, swap symlinks, and selectively reload desktop components without restarting the session.

### 1.3 Background Event Daemon (`dotfieldd`)
- Monitors `/sys/class/power_supply/` for power source state:
  - **AC Power** (`online = 1`): Automatically activates compositor-matched profile (`default.toml` on Sway or `minimal.toml` on Niri, auto-detected via active compositor socket `SWAYSOCK` vs `NIRI_SOCKET`).
  - **Battery Power** (`online = 0` / discharging): Automatically switches to engine-matched low-power profile (`battery.toml` on Sway or `minimal.toml` on Niri, reducing background load, low resource footprint).
- Supports `--daemon` (continuous polling loop) and `--once` (single evaluation for automated scripts and testing, with optional `--dry-run`).
- Communicates transitions via logging and executes atomic transitions through `dotfield-core`.

---

## 2. CLI Interface & UX

```bash
# GUI Utilities
dotfield-gui --hud           # Opens the Keybind Cheat-Sheet HUD overlay
dotfield-gui --switcher      # Opens the Profile Switcher card interface
dotfield-gui --help          # Shows GUI utility options

# Daemon
dotfieldd --daemon           # Runs daemon in foreground (systemd user unit target)
dotfieldd --once             # One-shot hardware state evaluation and apply
```

---

## 3. Verification & Quality Gates

1. **Rust Toolchain Quality**:
   - `cargo fmt -- --check`: Zero formatting drift across all 4 workspace crates.
   - `cargo check`: Clean type checking.
   - `cargo test`: Comprehensive unit and integration test coverage.
   - `cargo clippy -- -D warnings`: Zero-warning policy on all crates including `dotfield-gui` and `dotfieldd`.
2. **Pre-commit Gate**:
   - Passes all 10 pre-commit hooks (`trailing-whitespace`, `end-of-file-fixer`, `check-yaml`, `check-toml`, `check-added-large-files`, `gitleaks`, `semgrep`, `cargo-fmt`, `cargo-check`, `cargo-clippy`).
