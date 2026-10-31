# Phase 4 Design Document: Notifications, Lockscreen, Idle & System Utilities

**Project:** Dotfield (`dotfield`)
**Date:** 2026-10-31
**Author:** Pair Programming Session (via `gstack-office-hours`)
**Status:** APPROVED
**Scope:** Phase 4 of 7 (Desktop OSD, Notifications & Utilities)

---

## Executive Summary

Phase 4 delivers Wayland-native notification styling, screen locking, idle timeout management, solid wallpaper backgrounds, and screenshot capture pipelines. In keeping with Dotfield's core philosophy—**"Store logical choices, not duplicated config"**—all utility configurations are modularized and tokenized with the **Controlled Chaos** palette (`PALETTE.md`).

The architecture prioritizes native Wayland primitives with low memory footprints and instant reload capabilities (`makoctl reload`), ensuring zero flickering and no desktop session restarts.

---

## 1. Architectural Invariants & Mechanics

### 1.1 Notification Daemon (`mako`)
- **Location**: `mako/config`
- **Dynamic Reload**: `makoctl reload` re-reads configuration without restarting the notification service.
- **Urgency Tokenization**:
  - `[urgency=low]`: `Obsidian #121216` background, `Ash #A1A1AA` text, `Iron #34343D` border.
  - `[urgency=normal]`: `Void #0A0A0C` background, `Bone #E8E6E3` text, `Amber #FFB000` border.
  - `[urgency=critical]`: `Obsidian #121216` background, `Bone #E8E6E3` text, `Crimson #E63946` border, timeout=0 (persistent).

### 1.2 Lockscreen & Idle Management (`swaylock` & `swayidle`)
- **Swaylock** (`osd/swaylock/config`):
  - Minimal indicator ring with Controlled Chaos colors:
    - Inside: `Void #0A0A0C`
    - Ring normal: `Iron #34343D`
    - Ring verifying: `Amber #FFB000`
    - Ring wrong: `Crimson #E63946`
    - Key highlight: `Bone #E8E6E3`
- **Swayidle** (`osd/swayidle/config`):
  - 300s (5m): Dim/turn off displays (`swaymsg "output * power off"` or `niri msg action power-off-monitors`).
  - 600s (10m): Lock screen via `swaylock`.
  - Resume: Turn displays back on.
  - Before sleep: Lock screen immediately (`swaylock -f`).

### 1.3 Screenshot Pipeline (`scripts/screenshot.sh`)
- Zero-overhead script utilizing standard tools: `grim`, `slurp`, and `wl-copy`.
- Supports modes:
  - `--area` (interactive region selection via `slurp`).
  - `--window` (focused window selection).
  - `--full` (all connected monitors).
- Automatically writes to `~/Pictures/Screenshots/<timestamp>.png` and copies image buffer directly to Wayland clipboard (`wl-copy`).

### 1.4 Solid Canvas Wallpaper (`swaybg`)
- Script/launcher for solid color background using `Void #0A0A0C` (`swaybg -c "#0A0A0C"`).

---

## 2. Directory Layout & Deliverables

```
dotfield/
├── mako/
│   └── config                   # Wayland notification styling with Controlled Chaos
├── osd/
│   ├── swaylock/
│   │   └── config               # Minimal ring lockscreen config
│   └── swayidle/
│       └── config               # Graceful idle timer rules
└── scripts/
    ├── screenshot.sh            # Area, window, and fullscreen screenshot utility
    └── wallpaper.sh             # Solid canvas / background setter
```

---

## 3. Verification & Testing Plan

1. **Syntax & Config Verification**:
   - `mako --dry-run` or validation against man page specs.
   - `swaylock -C` / argument validation.
   - Shell script linting with `shellcheck` or clean bash execution tests.
2. **Quality Gates**:
   - `pre-commit run --all-files` (all 10 hooks: whitespace, EOF, YAML, TOML, gitleaks, semgrep, cargo-fmt, cargo-check, cargo-clippy).
