# Dotfield

> **Declarative Linux ricing engine, dotfile compiler, and high-performance Wayland desktop suite built with Rust.**
> *"Store logical choices, not duplicated config."*

---

## Overview

In traditional Linux desktop customization (ricing), changing a default terminal from `foot` to `kitty` or switching application shortcuts requires manually editing multiple configuration files across compositors (Niri, Sway), hotkey daemons, launcher scripts, and window rules.

**Dotfield** eliminates this fragmentation with an abstract semantic profile model:

```toml
[apps]
terminal = "foot"
launcher = "fuzzel"
editor = "nvim"
multiplexer = "zellij"

[bindings]
terminal = "Super+Return"
launcher = "Super+space"
editor = "Super+e"
```

Running `dotfield apply` parses this logical model and generates target-specific configuration fragments (e.g., Niri `spawn "foot"`, Sway `bindsym Mod4+Return exec foot`). Altering an app or keybinding only touches the high-level profile, while target configuration files and process reloads are handled automatically.

---

## Aesthetic: Controlled Chaos Palette

Dotfield is styled around the **Controlled Chaos** theme — an industrial, high-contrast dark aesthetic balancing Void Black, Industrial Amber, and Crimson:

| Token | Hex | Role | Visual Preview |
| :--- | :--- | :--- | :--- |
| **Void Black** | `#0A0A0C` | Deep background | `██████` |
| **Obsidian** | `#121216` | Panel / Surface background | `██████` |
| **Graphite** | `#1C1C23` | Active cards / popups | `██████` |
| **Iron** | `#34343D` | Borders & subtle dividers | `██████` |
| **Ash** | `#A1A1AA` | Inactive text & secondary labels | `██████` |
| **Bone White**| `#E8E6E3` | Primary foreground & headings | `██████` |
| **Crimson** | `#E63946` | Errors, alarms, critical indicators | `██████` |
| **Amber** | `#FFB000` | Primary accent & active highlights | `██████` |
| **Pale Amber**| `#FFD166` | Secondary accent & command builtins | `██████` |
| **Steel** | `#8B949E` | Muted accents, paths & icons | `██████` |
| **Green** | `#7BC47F` | Success states & strings | `██████` |
| **Blue** | `#82AAFF` | Links & function identifiers | `██████` |

For design tokens across tools, see [PALETTE.md](PALETTE.md).

---

## Dual Compositor Architecture

Dotfield natively supports two distinct Wayland compositor workflows:

### 1. Niri (Modern Scrollable Tiling + Amane Desktop Shell)
- **Compositor**: [Niri](https://github.com/YaLTeR/niri) infinite horizontal scrollable tiling.
- **Desktop Shell**: **`dotfield-shell`** — Built with **Pure Rust + Amane** (Vulkan and Vello backends).
- **Features**:
  - Top status bar with hardware telemetry (CPU, RAM, network, battery, clock).
  - Modern grid-based Application Launcher with keyboard long-press scrolling.
  - Modern grid-based Terminal Manager with active terminal indicators.
  - Interactive Display Configurator for multi-monitor arrangements.
  - Quick Settings Control Center and Notification HUD.
  - Interactive Monthly Calendar and Wayland Screen Recorder.
  - Sub-millisecond IPC communication via `$XDG_RUNTIME_DIR/amane.sock`.
  - Zero WebKit, Electron, or Slint overhead.

### 2. Sway (Minimalist Reference + Fuzzel Native)
- **Compositor**: [Sway](https://github.com/swaywm/sway) i3-compatible tiling.
- **Philosophy**: Pure, lean, distraction-free workflow.
- **Features**:
  - **Zero Bar**: Status bars suppressed via `swaybar_command /bin/true` for 100% vertical screen space.
  - **Fuzzel Native**: Application launcher and terminal selector powered by lightweight `fuzzel`.
  - **Zero Daemon**: Completely independent of `dotfield-shell`.
  - Fast notifications via `mako` and volume/brightness OSD via `swayosd`.

---

## Integrated Desktop Stack

- **Terminals**: Templates for [Foot](terminals/foot), [Kitty](terminals/kitty), [Alacritty](terminals/alacritty), and [Ghostty](terminals/ghostty).
- **Multiplexers**:
  - **[Zellij](multiplexer/zellij)**: Zero-collision modal configuration (`clear-defaults=true`), `Ctrl-a` leader prefix, and instant `Alt` navigation (`Alt-h/j/k/l`, `Alt-n`, `Alt-t`, `Alt-w`, `Alt-z`, `Alt-1..9`).
  - **[Tmux](multiplexer/tmux)**: Harmonized with Zellij keybindings, Vi copy mode with rectangle selection, floating modal popups (`display-popup`), and dynamic prefix indicator.
- **Editor ([Neovim](nvim))**:
  - Universal save on `Ctrl-s` across Normal, Insert, and Visual modes.
  - Seamless window navigation (`<C-h>`, `<C-j>`, `<C-k>`, `<C-l>`).
  - Window resizing with `Ctrl-Arrow` keys.
  - Visual block moving with persistent indentation (`<` / `>`).
  - Dropbar VS Code style breadcrumbs and NvChad foundation.
- **Shell Environment**:
  - **[Zsh](shell/zsh)**: Sub-15ms startup, cached bytecode compilation (`.zwc`), secret interception hook (`zshaddhistory`) preventing API keys/passwords from logging to disk, safe file deletion with `trash-cli`, and directory stack navigation.
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

# Install only Niri + Amane dependencies (includes Rust toolchain & dotfield-shell build)
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
```

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

### Niri Keybindings
| Keybinding | Action |
| :--- | :--- |
| `Mod + Return` | Launch default terminal (`foot`) |
| `Mod + Shift + Return` | Terminal Manager Grid overlay (`dotfield-shell`) |
| `Mod + Space` | Application Launcher Grid overlay (`dotfield-shell`) |
| `Mod + Shift + Space` | Control Center & Quick Settings overlay |
| `Mod + Shift + C` | Monthly Calendar overlay |
| `Mod + Shift + R` | Wayland Screen Recorder overlay |
| `Mod + Shift + D` | Display Configurator overlay |
| `Mod + Shift + E` | Powermenu overlay |
| `Mod + Shift + N/A/B/T`| Network / Audio / Bluetooth / Tray overlays |
| `Mod + Q` | Close focused window |
| `Mod + Escape` | Lock screen |
| `Print` / `Shift + Print` | Screenshot (Full / Area) |

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
├── launcher/fuzzel/        # Fuzzel launcher configuration & theme
├── kanshi/                 # Multi-monitor dynamic display autoconfig
├── shell/                  # Shell configurations (Zsh, Starship)
│   ├── zsh/
│   └── starship/
├── terminals/              # Terminal templates (Foot, Kitty, Alacritty, Ghostty)
├── multiplexer/            # Multiplexer configs (Zellij, Tmux)
├── nvim/                   # Neovim configuration (NvChad + core keymaps)
├── ly/                     # Ly TUI display manager configuration
├── mako/                   # Notification daemon templates & rules
├── osd/                    # On-screen display configs (SwayOSD / Avizo)
├── scripts/                # Standalone helper scripts
├── security/               # Hardening rules (Firejail, AppArmor, Flatpak, NFT)
├── docs/                   # Architectural blueprints, roadmaps, and designs
│   ├── ARCHITECTURE.md
│   ├── PROFILES.md
│   ├── ROADMAP.md
│   ├── designs/
│   └── reviews/
├── crates/
│   ├── dotfield-shell/     # Native Rust + Amane Wayland desktop shell
│   └── amane/              # Immediate-mode GPU rendering engine
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

# 2. Secret audit
gitleaks detect --no-git --redact -v

# 3. Static security analysis
semgrep --config=.semgrep.yml --error .

# 4. Rust checks
cargo fmt -- --check
cargo check
cargo clippy -- -D warnings
```

---

## License

This project is licensed under the [MIT License](LICENSE).
Individual sub-crates and components are available under the MIT License.
