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
   - `dotfield-core`: Pure library for TOML parsing, schema validation, AST compilation, diff calculation, atomic deployment, and rollback state management.
   - `dotfield-cli`: Native terminal interface (`status`, `check`, `diff`, `apply`, `rollback`, `list`).
   - `dotfield-gui`: Fast, lightweight Wayland-native graphical tools built with **Rust + Slint** (`rice-menu`, `profile-switcher`, keybind cheat-sheet HUD).
   - `dotfieldd`: Optional event listener daemon for dynamic profile switching on hardware events (battery, AC, display connect).

2. **Atomic & Idempotent Deployments**:
   - Never write directly to live config paths while compiling.
   - Compile target fragments to a `.dotfield/staging` directory first.
   - Verify integrity and syntax before atomically moving/symlinking into place (`~/.config/sway/config.d/00-generated.conf`, etc.).
   - Persist generational state in `.dotfield/state.json` to enable instant one-command rollback (`dotfield rollback`).

3. **Selective Reload Engine**:
   - Do not restart the entire Wayland compositor session or reload every process.
   - Compare the diff between the previous generation and the newly compiled generation.
   - Reload *only* the specific tools whose configurations changed (e.g. `swaymsg reload` for Sway, `niri msg action reload-config` for Niri, `makoctl reload` for Mako).

4. **Security & Sandboxing by Default**:
   - Integrate Linux security mechanisms into profile generation (`profiles/hardening.toml`).
   - Support Firejail profiles, AppArmor abstraction drop-ins, Flatpak permission overrides, and NFTables network isolation.

---

## Workspace & Directory Conventions

```
dotfield/
├── profiles/               # Logical profiles (minimal.toml, hardening.toml, default.toml)
├── compositor/             # Modular templates & base configs for Sway, Niri
│   ├── sway/
│   │   └── conf.d/
│   └── niri/
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
├── mako/                   # Notification daemon templates
├── osd/                    # On-screen display configs (SwayOSD / Avizo)
├── security/               # Hardening rules (Firejail, AppArmor, Flatpak, NFTables)
│   ├── apparmour/
│   ├── firejail/
│   ├── flatpak/
│   └── nft/
├── docs/                   # Architectural blueprints, roadmap, profile specs
├── tests/                  # Integration tests and fixture profiles
├── crates/ (or src/)       # Rust implementations (core, cli, gui)
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
