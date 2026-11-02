# GEMINI.md — Dotfield Engineering & Operational Guidelines

## Project Identity & Core Philosophy

**Dotfield** is a declarative, modular Linux ricing ecosystem and dotfiles suite designed for modern Wayland desktop workflows.

The governing principle of Dotfield is:
> **"Declarative, modular configurations with zero unnecessary bloat."**

Dotfield provides a unified, production-grade Linux desktop experience tailored for two distinct compositor paradigms:
- **Niri**: Infinite scrollable tiling driven by **Amane** (`amane`), a high-performance native Wayland desktop shell built in Rust with Vulkan/Vello GPU rendering.
- **Sway**: Minimalist i3-compatible tiling focused on zero-daemon execution, suppressed status bars, and lightweight native Fuzzel menus.

Dotfield uses drop-in modular configurations (`conf.d/*.kdl`, `conf.d/*.conf`), clean portable helper scripts, security sandboxing (Firejail, AppArmor, Flatpak, NFTables), and idempotent installation scripts (`install.sh`, `setup.sh`).

---

## Architectural Invariants

1. **Logical Separation of Concerns**:
   - `amane` (`crates/amane-shell`): High-performance Wayland desktop shell built with **Pure Rust + Amane** exclusively for **Niri**. Integrates status bar, horizontal workspace radar, application launcher, terminal manager, interactive display configurator, quick settings control center, notification HUD, calendar, and screen recorder natively.
   - **Sway Minimalist Stack**: Strictly independent of `amane` — runs with zero daemons, suppressed bars, and native `fuzzel` overlays.
   - Modular Compositor Configs: Declarative modular configurations (`compositor/niri/conf.d/*.kdl`, `compositor/sway/conf.d/*.conf`).
   - Helper Scripts: Portable helper scripts (`scripts/terminal-select.sh`, `scripts/display-config.sh`, `scripts/osd.sh`, etc.) with fallback support.

2. **Instant IPC & Zero-Lag Overlays (Niri)**:
   - Background daemon communicates via Unix domain socket (`$XDG_RUNTIME_DIR/amane.sock`).
   - Supervised under systemd (`~/.config/systemd/user/amane.service`) with auto-recreation across output DPMS/re-negotiation events and auto-healing CLI IPC.

3. **Pure Rust + Vulkan Desktop UI (Amane)**:
   - GPU-accelerated immediate-mode visual rendering via `amane` with Vulkan and Vello backends.
   - Zero WebKit / Electron / Slint bloat; ultra-low CPU and memory footprints (~60 MB RAM).

4. **Fastfetch Custom Setup**:
   - Aesthetic system fetch configuration located in `fastfetch/` featuring custom braille emblem art and Controlled Chaos palette tokens.

5. **Security & Sandboxing by Default**:
   - Support Firejail profiles, AppArmor abstraction drop-ins, Flatpak permission overrides, and NFTables network isolation.

---

## Workspace & Directory Conventions

```
dotfield/
├── install.sh              # System package & dependency installer
├── setup.sh                # Configuration deployment & service activator
├── compositor/             # Modular configs for Niri and Sway
│   ├── niri/
│   │   └── conf.d/
│   └── sway/
│       └── conf.d/
├── fastfetch/              # Custom fastfetch config & braille emblem art
│   ├── config.jsonc
│   └── logo.txt
├── launcher/               # Base configs & themes for Fuzzel
│   └── fuzzel/
├── kanshi/                 # Multi-monitor dynamic display autoconfig
├── shell/                  # Shell configurations & prompts (Zsh, Starship)
│   ├── zsh/
│   └── starship/
├── terminals/              # Terminal templates (Foot, Kitty, Alacritty, Ghostty)
├── multiplexer/            # Terminal multiplexers (Zellij, Tmux)
├── nvim/                   # Neovim configurations
├── ly/                     # Ly TUI display manager configurations (config.ini, config.lua)
├── mako/                   # Notification daemon templates
├── osd/                    # On-screen display configs (SwayOSD / Avizo)
├── systemd/                # User systemd service templates (amane.service)
│   └── user/
├── security/               # Hardening rules (Firejail, AppArmor, Flatpak, NFTables)
│   ├── apparmour/
│   ├── firejail/
│   ├── flatpak/
│   └── nft/
├── scripts/                # Standalone helper scripts
├── docs/                   # Architectural blueprints and roadmap
├── crates/
│   ├── amane-shell/        # Native Rust + Amane Wayland desktop shell for Niri (binary: amane)
│   └── amane/              # Immediate-mode GPU rendering engine (upstream library)
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
