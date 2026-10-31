# Phase 3 Design Document: Terminals, Neovim, Multiplexers & Status Bar

**Project:** Dotfield (`dotfield`)
**Date:** 2026-10-31
**Author:** Pair Programming Session (via `gstack-office-hours`)
**Status:** APPROVED
**Scope:** Phase 3 of 7 (Terminals, Neovim, Multiplexers, Bar)

---

## Executive Summary

Phase 3 establishes modular, fragment-first base configurations and dynamic color-token integration for core terminal emulators, developer editors, terminal multiplexers, and status bars. Following Dotfield's governing principle—**"Store logical choices, not duplicated config"**—all tools in Phase 3 decouple static operational settings (fonts, keybindings, cursor rules) from atomic theme/palette fragments managed and compiled by Dotfield.

Every tool strictly adheres to the **Controlled Chaos** palette specification (`PALETTE.md`), providing consistent dark-void aesthetics across Foot, Kitty, Alacritty, Ghostty, Neovim, Zellij, Tmux, and i3status-rust.

---

## 1. Architectural Invariants & Mechanics

### 1.1 Fragment Inclusion Hierarchy
Each target configuration loads its color palette via an isolated drop-in fragment:
- **Foot**: `terminals/foot/foot.ini` includes `include=~/.config/foot/colors.ini`
- **Kitty**: `terminals/kitty/kitty.conf` includes `include colors.conf`
- **Alacritty**: `terminals/alacritty/alacritty.toml` imports `import = ["~/.config/alacritty/colors.toml"]`
- **Ghostty**: `terminals/ghostty/config` specifies `theme = dotfield-theme` (in `themes/dotfield-theme`)
- **Zellij**: `multiplexer/zellij/config.kdl` specifies `theme "dotfield"` (in `themes/dotfield.kdl`)
- **Tmux**: `multiplexer/tmux/tmux.conf` sources `source-file ~/.config/tmux/colors.conf`
- **Neovim**: `nvim/lua/palette.lua` loads `lua/palette_generated.lua` into Neovim highlight groups (`nvim_set_hl`)
- **i3status-rust**: `i3-rust/config.toml` consumes the tokenized Controlled Chaos palette definitions for swaybar.

### 1.2 Zero-Flicker Live-Reload Engines
Dotfield's selective reload engine updates running instances without terminating active shells:
- **Kitty**: IPC socket remote control (`kitten @ set-colors --all ~/.config/kitty/colors.conf`)
- **Tmux**: Server config reload (`tmux source-file ~/.config/tmux/colors.conf`)
- **Foot**: Dynamic reconfiguration on SIGHUP or client reconnect.
- **Zellij**: Hot-reloads KDL theme file upon file modification.
- **Neovim**: Autocommand listening for file modification or `:lua require("palette").reload()`.

---

## 2. Controlled Chaos Palette Mapping

Every fragment is deterministically compiled from `PALETTE.md`:

| Token | Hex | Role in Terminals & Multiplexers |
| :--- | :--- | :--- |
| `Void` | `#0A0A0C` | Terminal background, Neovim Normal background |
| `Obsidian` | `#121216` | Status lines, inactive tabs, gutter background |
| `Graphite` | `#1C1C23` | Active tab background, selection background |
| `Iron` | `#34343D` | Borders, split lines, line numbers |
| `Ash` | `#A1A1AA` | Comments, muted text, inactive window titles |
| `Bone` | `#E8E6E3` | Terminal foreground, primary text, active tab |
| `Crimson` | `#E63946` | Errors, warnings, urgent tabs, critical battery |
| `Amber` | `#FFB000` | Accents, search matches, highlights, warnings |
| `Steel` | `#8B949E` | Information, secondary UI accents, directory icons |

---

## 3. Directory Layout & Deliverables

```
dotfield/
├── terminals/
│   ├── foot/
│   │   ├── foot.ini                     # High-performance Wayland terminal
│   │   └── colors.ini                   # Controlled Chaos palette drop-in
│   ├── kitty/
│   │   ├── kitty.conf                   # GPU-accelerated terminal + socket control
│   │   └── colors.conf                  # Controlled Chaos palette drop-in
│   ├── alacritty/
│   │   ├── alacritty.toml               # Cross-platform GPU terminal
│   │   └── colors.toml                  # Controlled Chaos palette drop-in
│   └── ghostty/
│       ├── config                       # Next-gen native Wayland terminal
│       └── themes/
│           └── dotfield-theme           # Controlled Chaos palette theme
├── multiplexer/
│   ├── zellij/
│   │   ├── config.kdl                   # Modern Rust-native workspace multiplexer
│   │   └── themes/
│   │       └── dotfield.kdl             # Controlled Chaos KDL theme
│   └── tmux/
│       ├── tmux.conf                    # Classic terminal multiplexer
│       └── colors.conf                  # Controlled Chaos status bar & border styling
├── nvim/
│   ├── init.lua                         # Modular entrypoint
│   └── lua/
│       ├── core/
│       │   ├── options.lua              # Sensible defaults (relative numbers, tab widths)
│       │   └── keymaps.lua              # Ergonomic leader mappings
│       ├── palette.lua                  # Zero-dependency highlight generator
│       └── palette_generated.lua        # Controlled Chaos table tokens
└── i3-rust/
    └── config.toml                      # High-performance i3status-rust status bar
```

---

## 4. Verification & Testing Plan

1. **Syntax & Configuration Lints**:
   - `alacritty --dry-run` or TOML validation.
   - `foot --verify-config`.
   - `kitty --config-file ... --dump-commands`.
   - `nvim --headless "+qa"` to verify clean Lua loading without errors or deprecations.
   - `zellij setup --check`.
   - `tmux -f ... new-session -d -s test && tmux kill-server`.
2. **Quality Gates**:
   - `pre-commit run --all-files` (passes all 10 hooks: `trailing-whitespace`, `end-of-file-fixer`, `check-yaml`, `check-toml`, `check-added-large-files`, `gitleaks`, `semgrep`, `cargo-fmt`, `cargo-check`, `cargo-clippy`).
