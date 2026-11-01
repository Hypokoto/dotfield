# GEMINI.md — Dotfield Engineering & Operational Guidelines

## Project Identity & Core Philosophy

**Dotfield** is a declarative Linux ricing engine and dotfile compiler built in Rust.
The governing principle of Dotfield is:
> **"Store logical choices, not duplicated config."**

In traditional ricing, changing a default terminal from `foot` to `kitty` or switching hotkeys requires manually editing multiple configuration files across compositors (Sway, Niri), hotkey daemons, and window rules. Dotfield replaces this friction with an abstract profile model:

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

Running `dotfield apply` parses this semantic model and compiles target-specific configuration fragments (e.g., Sway `bindsym Mod4+Return exec foot`, Niri `spawn "foot"`). Switching an app or keybind alters only the high-level profile, while the target config fragments are regenerated automatically.

---

## Architectural Invariants

1. **Logical Separation of Concerns**:
   - `dotfield-shell`: High-performance Wayland desktop shell built with **Rust + Amane**. Integrates status bar, application launcher, terminal manager, interactive display configurator, quick settings control center, and notification HUD natively.
   - Modular Compositor Configs: Declarative modular configurations (`compositor/niri/conf.d/*.kdl`, `compositor/sway/conf.d/*.conf`).
   - Helper Scripts: Portable helper scripts (`scripts/terminal-select.sh`, `scripts/display-config.sh`, etc.) with fallback support.

2. **Instant IPC & Zero-Lag Overlays**:
   - Background daemon communicates via Unix domain socket (`$XDG_RUNTIME_DIR/amane.sock`).
   - Runs silently in the background; compositor keybindings trigger overlays with sub-millisecond response.

3. **Pure Rust + Vulkan Desktop UI (Amane)**:
   - GPU-accelerated immediate-mode visual rendering via `amane` with Vulkan and Vello backends.
   - Zero WebKit / Electron / Slint bloat; ultra-low CPU and memory footprints.

4. **Security & Sandboxing by Default**:
   - Support Firejail profiles, AppArmor abstraction drop-ins, Flatpak permission overrides, and NFTables network isolation.

---

## Workspace & Directory Conventions

```
dotfield/
├── compositor/             # Modular configs for Niri and Sway
│   ├── niri/
│   │   └── conf.d/
│   └── sway/
│       └── conf.d/
├── launcher/               # Base configs & themes for Fuzzel
│   └── fuzzel/
├── kanshi/                 # Multi-monitor dynamic display autoconfig
├── shell/                  # Shell configurations & prompts (Zsh, Starship)
│   ├── zsh/
│   └── starship/
├── terminals/              # Terminal templates (Foot, Kitty, Alacritty, Ghostty)
├── multiplexer/            # Terminal multiplexers (Zellij, Tmux)
├── nvim/                   # Neovim configurations
├── i3-rust/                # i3status-rust and bar configurations
├── i3bar-river/            # i3bar-river integration
├── mako/                   # Notification daemon templates
├── osd/                    # On-screen display configs (SwayOSD / Avizo)
├── security/               # Hardening rules (Firejail, AppArmor, Flatpak, NFTables)
│   ├── apparmour/
│   ├── firejail/
│   ├── flatpak/
│   └── nft/
├── docs/                   # Architectural blueprints and roadmap
├── crates/
│   └── dotfield-shell/     # Native Rust + Amane Wayland desktop shell
├── .gitleaks.toml          # Secret scanner rules
├── .semgrep.yml            # Static security analysis rules
└── .pre-commit-config.yaml # Pre-commit hook definitions
```

---

## Quality, Safety & Git Pre-Commit Rules

Before every commit, the following gates MUST pass without exceptions:

1. **Pre-commit Hooks**:
   Run `pre-commit run --all-files` to format and validate:
   - Trailing whitespace & EOF formatting.
   - Valid TOML and YAML syntax.
   - File size limits.

2. **Secret Auditing (Gitleaks)**:
   - Run `gitleaks detect --no-git --redact -v`.
   - Never commit API keys, personal access tokens, SSH private keys, or credentials.

3. **Static Security Analysis (Semgrep)**:
   - Run `semgrep --config=.semgrep.yml --error .`.
   - Prevent command injection patterns, overly permissive permissions (`0777`), and unhardened shell scripts.

4. **Rust Code Quality**:
   - `cargo fmt -- --check`: Code formatting enforcement.
   - `cargo check`: Clean type checking.
   - `cargo clippy -- -D warnings`: Zero-warning policy on production crates.
