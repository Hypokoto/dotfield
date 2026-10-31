# Dotfield

> **Declarative Linux ricing engine and dotfile compiler in Rust.**
> *Store logical choices, not duplicated config.*

---

## The Core Concept

Instead of manually editing and synchronizing multiple configuration files whenever you change an application or keybinding:

```toml
# profiles/default.toml
[apps]
terminal = "foot"
launcher = "fuzzel"

[bindings]
terminal = "Super+Return"
launcher = "Super+space"
```

Running `dotfield apply` parses this semantic model and compiles target-specific configuration fragments:

```sway
# ~/.config/sway/config.d/00-generated.conf
bindsym Mod4+Return exec foot
bindsym Mod4+space exec fuzzel
```

Switching from `foot` to `kitty`, or `fuzzel` to `rofi`, only changes your profile model — target configuration fragments and selective process reloads are handled automatically.

---

## System Architecture

- **`dotfield-core`**: TOML profiles, schema validation, diff engine, transactional atomic deployment, rollback ledger.
- **`dotfield-cli`**: CLI commands (`status`, `check`, `diff`, `apply`, `rollback`, `list`).
- **`dotfield-gui`**: Fast Wayland overlays built with **Rust + Slint** (`rice-menu`, `profile-switcher`, keybind HUD).
- **`dotfieldd`**: Optional lightweight event daemon for dynamic profile switching.

For complete architectural details, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

---

## Multi-Phase Roadmap

- **Phase 1**: Foundation, Documentation, `.gitignore`, Pre-commits, Gitleaks, and Semgrep *(Current)*
- **Phase 2**: Compositor (Sway, Niri), Launchers (Rofi, Fuzzel), and Shell (Zsh, Starship)
- **Phase 3**: Terminals (Foot, Kitty, Alacritty, Ghostty), Neovim, Multiplexers (Tmux, Zellij), and i3status-rust
- **Phase 4**: Desktop OSD (SwayOSD/Avizo), Mako notifications, and utilities
- **Phase 5**: Security sandboxing (Firejail, AppArmor, Flatpak, NFTables) & concrete profiles
- **Phase 6**: Native Rust CLI (`dotfield-core`, `dotfield-cli`)
- **Phase 7**: Custom Slint GUI utilities (`rice-menu`, `profile-switcher`)

For detailed milestone breakdowns, see [docs/ROADMAP.md](docs/ROADMAP.md).

---

## Development & Security Gates

Before committing, run all verification gates:

```bash
# Pre-commit checks
pre-commit run --all-files

# Secret scanning
gitleaks detect --no-git --redact -v

# Static analysis
semgrep --config=.semgrep.yml --error .

# Rust checks
cargo check
cargo fmt -- --check
```
