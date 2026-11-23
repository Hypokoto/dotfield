# Dotfield

> **Modular Linux ricing ecosystem and high-performance Wayland desktop suite.**
> *"Modular configs, zero bloat, pure Wayland performance."*

---

## Overview

In traditional Linux desktop customization (ricing), changing a default terminal, launcher, or hotkey requires navigating bloated monolithic files with hard-to-maintain duplication.

**Dotfield** provides a clean and modular foundation built specifically for modern Wayland workflows:

- **Niri Workflows**: Accelerated by **Amane** (`amane`), a native Rust desktop shell suite built with Vulkan and Vello GPU rendering (rendering engine created by MystiaFin; see [UPSTREAM.md](UPSTREAM.md)), delivering an integrated status bar, horizontal workspace radar, application launcher, terminal manager, control center, and OSD overlays.
- **Sway Workflows**: Lean, distraction-free environment with suppressed status bars (100% vertical screen space), native Fuzzel menus, and zero background daemon overhead.
- **Unified Aesthetic**: Styled end-to-end with the **Controlled Chaos** theme — an industrial, high-contrast dark aesthetic balancing Void Black, Industrial Amber, and Crimson.
- **Automated Deployment**: Managed via idempotent shell scripts (`install.sh`, `setup.sh`) supporting `--sway`, `--niri`, and `--all` targets.

---

## Aesthetic: Cyber Synthwave Palette

Dotfield is styled around the **Cyber Synthwave** theme — a high-contrast cybernetic aesthetic balancing Void Black, Deep Violet, Cyan-Teal, Neon Blue, and Emerald:

| Token | Hex | Role | Visual Preview |
| :--- | :--- | :--- | :--- |
| **Void Black** (`base00`) | `#000000` | Deep canvas & background (60%) | `██████` |
| **Surface** (`base01`) | `#140E29` | Deep violet panel & surface background | `██████` |
| **Overlay** (`base02`) | `#251B4A` | Active cards, elevated surfaces, borders | `██████` |
| **Muted** (`base03`) | `#FFBE74` | Peach amber comments & subtle labels | `██████` |
| **Statusline** (`base04`) | `#11AEB3` | Secondary text & status indicators | `██████` |
| **Cyan-Teal** (`base05`) | `#14B9B5` | Primary foreground text & headings | `██████` |
| **Pure Fallback** (`base06`) | `#FFFFFF` | High-contrast fallback text | `██████` |
| **Red Accent** (`base08`) | `#BE3F50` | Variables, alarms & danger accents | `██████` |
| **Warning Orange** (`base09`) | `#FF7F41` | Constants, warnings & telemetry alerts | `██████` |
| **Yellow Amber** (`base0A`) | `#FFBE74` | Classes & search highlights | `██████` |
| **Emerald** (`base0B`) | `#7CD699` | Success states, strings & active status | `██████` |
| **Neon Blue** (`base0D`) | `#04C5F0` | Primary accent, focused rings & pills | `██████` |
| **Purple** (`base0E`) | `#9147A8` | Keywords, storage & electric hover glows | `██████` |
| **Dark Red** (`base0F`) | `#CE4F48` | Deprecated & critical errors | `██████` |

For design tokens across tools, see [PALETTE.md](PALETTE.md).

---

## Dual Compositor Architecture

Dotfield natively supports two distinct Wayland compositor workflows:

### 1. Niri (Modern Scrollable Tiling + Amane Desktop Shell)
- **Compositor**: [Niri](https://github.com/YaLTeR/niri) infinite horizontal scrollable tiling.
- **Desktop Shell**: **`amane`** (`crates/amane-shell`) — Built with **Pure Rust + Amane** (Vulkan and Vello backends).
- **Features**:
  - Full-width Top Status Bar or Bottom-Centered Taskbar Dock with dynamic multi-monitor tracking.
  - Spatial Workspace Radar tracking Niri's infinite horizontal ribbon via `$NIRI_SOCKET`.
  - Centered Start Menu & Application Grid (`Super`) with fuzzy search and session controls.
  - Spatial Ribbon Overview (`Mod+Tab`) with live window geometry previews.
  - Unified Control Manager & Quick Settings (6 quick toggles, volume/brightness sliders, integrated 20-band CAVA audio visualizer, hero clock, calendar, and notification history).
  - Modern grid-based Application Launcher (`Mod+Space`).
  - Interactive Display Configurator for multi-monitor arrangements.
  - Fast-path 1-byte opcode IPC communication via `$XDG_RUNTIME_DIR/amane.sock`.
  - Supervised by systemd user service (`amane.service`) with auto-recreation across output DPMS/re-negotiation events.
  - Zero WebKit, Electron, or Slint overhead (~60 MB RAM).

### 2. Sway (Minimalist Reference + Fuzzel Native)
- **Compositor**: [Sway](https://github.com/swaywm/sway) i3-compatible tiling.
- **Philosophy**: Pure, lean, distraction-free workflow.
- **Features**:
  - **Zero Bar**: Status bars suppressed via `swaybar_command /bin/true` for 100% vertical screen space.
  - **Fuzzel Native**: Application launcher and terminal selector powered by lightweight `fuzzel`.
  - **Zero Daemon**: Completely independent of `amane`.
  - Fast notifications via `mako` and volume/brightness OSD via `swayosd`.

---

## Integrated Desktop Stack

- **Fastfetch ([fastfetch](fastfetch))**: Aesthetic fetch layout themed with the Controlled Chaos palette and custom braille emblem art.
- **Terminals**: Templates for [Ghostty](terminals/ghostty), [Foot](terminals/foot), and [Kitty](terminals/kitty).
- **Multiplexers**:
  - **[Zellij](multiplexer/zellij)**: Zero-collision modal configuration (`clear-defaults=true`), `Ctrl-a` leader prefix, and instant `Alt` navigation (`Alt-h/j/k/l`, `Alt-n`, `Alt-t`, `Alt-w`, `Alt-z`, `Alt-1..9`).
  - **[Tmux](multiplexer/tmux)**: Harmonized with Zellij keybindings, Vi copy mode with rectangle selection, floating modal popups (`display-popup`), and dynamic prefix indicator.
- **Editor ([Neovim](nvim))**:
  - Universal save on `Ctrl-s` across Normal, Insert, and Visual modes.
  - Unified window navigation (`<C-h>`, `<C-j>`, `<C-k>`, `<C-l>`).
  - Window resizing with `Ctrl-Arrow` keys.
  - Visual block moving with persistent indentation (`<` / `>`).
  - Dropbar VS Code style breadcrumbs and NvChad foundation.
- **Shell Environment**:
  - **[Zsh](shell/zsh)**: Fast interactive startup (~60ms measured, see [docs/BENCHMARKS.md](docs/BENCHMARKS.md)), cached bytecode compilation (`.zwc`), secret interception hook (`zshaddhistory`) preventing API keys/passwords from logging to disk, safe file deletion with `trash-cli`, and directory stack navigation.
  - **[Starship](shell/starship)**: Fast prompt themed with Controlled Chaos tokens.
  - **FZF**: Configured with the Controlled Chaos palette and backed by `fd`.
  - **Modern CLI**: `eza`, `bat`, `ripgrep`, `fd`, `dust`, `duf`, `btop`, `xh`, `doggo`, `zoxide`, `direnv`.
- **Display Manager ([Ly](ly))**: TUI login manager themed with the Controlled Chaos palette.
- **Display Auto-Configuration ([Kanshi](kanshi))**: Automatic monitor profile switching for clamshell, dock, and multi-head setups.
- **Notifications & OSD**: [Mako](mako) notification daemon with automated battery alerts ([`scripts/battery-alert.sh`](scripts/battery-alert.sh)) and [SwayOSD](osd/swayosd).

---

## Installation & Deployment

Dotfield provides dedicated, idempotent shell scripts with subcommand and flag routing:

### 1. Install Dependencies (`install.sh`)
Audits system packages, installs missing tools across package managers (`pacman`, `apt`, `dnf`, `zypper`), checks Rust toolchains, and compiles binaries:

```bash
# Install everything for both Sway and Niri
./install.sh all

# Install only Sway dependencies (skips Rust crate compilation)
./install.sh sway

# Install only Niri + Amane dependencies (includes Rust toolchain & amane build)
./install.sh niri -y
```

### 2. Deploy Configurations (`setup.sh`)
Stages dotfiles into `~/.config/`, links helper scripts, sets up permissions, and signals running compositors:

```bash
# Deploy all configurations (both Sway and Niri)
./setup.sh all

# Deploy specifically for Sway (Fuzzel launcher, zero bar, reloads swaymsg)
./setup.sh sway

# Deploy specifically for Niri (Amane shell, top bar, reloads niri msg)
./setup.sh niri

# Preview installation without modifying files
./setup.sh --dry-run

# Force update and backup existing configs
./setup.sh --force

# Deploy with security hardening (AppArmor, NFTables, Flatpak overrides)
./setup.sh --hardening
```

> [!NOTE]
> **Symlink Deployment**: By default, `setup.sh` creates symbolic links pointing directly into this repository. Keep the repository directory intact at its cloned location to prevent broken links. Pass `--copy` if you prefer static copying instead of symlinks.
>
> **Flatpak Sandboxing**: When `--hardening` is enabled, global Flatpak overrides restrict legacy X11 socket access (`--nosocket=x11`) to enforce pure Wayland. If an older Flatpak app strictly requires X11, grant an individual override via `flatpak override --user <app-id> --socket=x11`.

---

## Default Keybindings

### Sway Keybindings
| Keybinding | Action |
| :--- | :--- |
| `Mod + Return` | Launch default terminal (`foot`) |
| `Mod + Shift + Return` | Interactive Terminal Selector (`fuzzel --dmenu`) |
| `Mod + Space` | Application Launcher (`fuzzel`) |
| `Mod + Shift + D` | Display Configurator (`wlr-randr` / Kanshi) |
| `Mod + Shift + R` | Screen Recorder (`wf-recorder`) |
| `Mod + Shift + E` | Logout Menu (`wlogout`) |
| `Mod + Q` | Kill focused window |
| `Mod + Ctrl + R` | Reload Sway configuration |
| `Mod + Escape` | Lock screen (`swaylock`) |
| `Print` / `Shift + Print` | Screenshot (Full / Area) |

### Niri Keybindings (Windows 11 Fluent Shell & Blacksite Ergonomics)
| Keybinding | Action |
| :--- | :--- |
| `Super_L` | Centered Start Menu & Application Grid (`amane`) |
| `Mod + Space` / `Mod + S` | Application Launcher / Windows Search (`amane`) |
| `Mod + A` / `Mod + I` | Control Center & Quick Settings (`amane`) |
| `Mod + N` | Notification Center & Calendar View (`amane`) |
| `Mod + D` / `Ctrl + Mod + D` | Show Clean Desktop / Scratch (`05 SCRATCH`) |
| `Mod + Tab` / `Alt + Tab` | Task View & Window Switcher (`toggle-overview`) |
| `Mod + E` | File Explorer (`ghostty -e yazi`) |
| `Ctrl + Shift + Escape` | Task Manager (`ghostty -e btop`) |
| `Ctrl + Alt + Delete` / `Mod + Shift + E`| System Security & Powermenu (`amane`) |
| `Alt + F4` / `Mod + Q` | Close active window |
| `Mod + L` / `Mod + Escape` | Lock screen (`scripts/lock.sh`) |
| `Mod + Return` / `Ctrl + Alt + T` | Terminal Selector (`scripts/terminal-select.sh`) |
| `Mod + Ctrl + Return` | Centered 960x600 Floating Scratchpad (`ghostty`) |
| `Ctrl + Mod + Up` / `Down` | Switch virtual workspace up / down |
| `Mod + Shift + P` | Zero-Disruption Presentation Shield (`amane`) |
| `Mod + Shift + D` | Display Configurator overlay (`amane`) |
| `Mod + Shift + C` | Interactive Monthly Calendar (`amane`) |
| `Mod + Shift + R` | Wayland Screen Recorder (`amane`) |
| `Mod + Shift + N/A/B/T` | Network / Audio / Bluetooth / Tray quick views |
| `Print` / `Shift + Print` / `Mod + Shift + S` | Screenshot (Full / Interactive Area) |

### Multiplexer Keybindings (Harmonized across Zellij & Tmux)
| Keybinding | Action |
| :--- | :--- |
| `Ctrl + a` | Multiplexer leader prefix |
| `Alt + h` / `j` / `k` / `l` | Move focus left / down / up / right |
| `Alt + n` | New pane |
| `Alt + t` | New window / tab |
| `Alt + w` | Close focused pane |
| `Alt + z` | Toggle zoom / fullscreen |
| `Alt + [` / `Alt + ]` | Previous / Next window |
| `Alt + 1` .. `Alt + 9` | Jump directly to window/tab 1..9 |
| `Ctrl + g` *(Zellij)* | Toggle Locked Mode (100% key passthrough) |
| `Prefix + f` *(Tmux)* | Floating scratchpad popup terminal |

---

## Workspace Layout

```
dotfield/
├── install.sh              # System package & dependency installer
├── setup.sh                # Dotfile deployment & environment activator
├── compositor/             # Modular configs for Niri and Sway
│   ├── niri/conf.d/
│   └── sway/conf.d/
├── fastfetch/              # Fastfetch configuration & braille emblem art
│   ├── config.jsonc
│   └── logo.txt
├── launcher/fuzzel/        # Fuzzel launcher configuration & theme
├── kanshi/                 # Multi-monitor dynamic display autoconfig
├── shell/                  # Shell configurations (Zsh, Starship)
│   ├── zsh/
│   └── starship/
├── terminals/              # Terminal templates (Foot, Ghostty, Kitty)
├── multiplexer/            # Multiplexer configs (Zellij, Tmux)
├── nvim/                   # Neovim configuration (NvChad + core keymaps)
├── ly/                     # Ly TUI display manager configuration
├── mako/                   # Notification daemon templates & rules
├── osd/                    # On-screen display configs (SwayOSD / Avizo)
├── systemd/user/           # Systemd user services (amane.service)
├── scripts/                # Standalone helper scripts
├── security/               # Hardening rules (AppArmor, Flatpak, NFTables)
├── docs/                   # Architectural blueprints and guides
├── crates/
│   ├── amane-shell/        # Native Rust + Amane Wayland desktop shell for Niri (binary: amane)
│   └── amane/              # Immediate-mode GPU rendering engine (upstream library)
├── .gitleaks.toml          # Secret scanner rules
├── .semgrep.yml            # Static security analysis rules
└── .pre-commit-config.yaml # Pre-commit hook definitions
```

---

## Development & Security Gates

Before committing, all verification gates must pass:

```bash
# 1. Pre-commit hooks
pre-commit run --all-files

# 2. Secret audit (use --no-git only for untracked worktrees)
gitleaks detect -v

# 3. Static security analysis
semgrep --config=.semgrep.yml --error .

# 4. Rust checks & tests
cargo fmt -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## License & Upstream Attribution

This project is licensed under the [MIT License](LICENSE).

- **Amane GPU Rendering Engine** (`crates/amane`): Developed by **MystiaFin** and distributed under the MIT License. See [UPSTREAM.md](UPSTREAM.md) for origin, license details, and upstream repository link.
- **Dotfield Configurations & `amane-shell`** (`crates/amane-shell`): Original work distributed under the [MIT License](LICENSE).
