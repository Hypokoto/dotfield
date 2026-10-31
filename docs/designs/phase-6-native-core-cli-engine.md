# Phase 6 Design Document: Native Core & CLI Engine (`dotfield-core` & `dotfield-cli`)

**Project:** Dotfield (`dotfield`)
**Date:** 2026-10-31
**Author:** Pair Programming Session (via `gstack-office-hours`)
**Status:** APPROVED
**Scope:** Phase 6 of 7 (Native Rust Core & CLI Engine)

---

## Executive Summary

Phase 6 implements Dotfield's declarative compilation pipeline, validation suite, transactional generational deployment engine, and command-line interface in idiomatic Rust. It embodies the governing principle:
> **"Store logical choices, not duplicated config."**

The architecture establishes a multi-crate Cargo workspace separating the pure library (`dotfield-core`) from the CLI interface (`dotfield-cli`). The engine compiles abstract profile models (`profiles/*.toml`) into fragment drop-ins (Sway or Niri bindings, Fuzzel colors, Foot colors, and Mako notifications; Kitty, Zellij, and Tmux fragments are planned), stages deployments transactionally in `~/.dotfield/generations/N/`, atomically swaps the active symlink pointer (`~/.dotfield/current`), and reloads only the specific tools whose configurations changed via their native IPC channels.

---

## 1. Architectural Invariants & Mechanics

### 1.1 Crate Workspace Separation
```
dotfield/
├── Cargo.toml                       # Workspace definition
└── crates/
    ├── dotfield-core/               # Pure library crate
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs             # Typed error hierarchy (thiserror)
    │       ├── profile.rs           # Profile schema & TOML deserializers
    │       ├── validator.rs         # Path validation & binary availability
    │       ├── compiler.rs          # Semantic AST to target fragments
    │       ├── generation.rs        # Generational state & atomic swapper
    │       ├── diff.rs              # Structural & text diff engine
    │       └── reloader.rs          # Selective IPC reload engine
    └── dotfield-cli/                # Command-line binary crate
        ├── Cargo.toml
        └── src/
            ├── main.rs              # CLI entrypoint & clap dispatch
            └── commands/
                ├── mod.rs
                ├── status.rs        # Show active profile, generation, and drift
                ├── check.rs         # Verify profile schema & prerequisites
                ├── diff.rs          # Preview fragment diffs before applying
                ├── apply.rs         # Staged compile, atomic swap, and selective reload
                ├── rollback.rs      # Instant reversion to generation N-1
                └── list.rs          # Enumerate available profiles
```

### 1.2 Generational Symlink Swapper (Shape B)
1. **Compilation**: Writes target fragments into `~/.dotfield/generations/<N>/` with strictly monotonic directory allocation and `meta.json`.
2. **Target Symlink Binding**: Live targets (e.g. `~/.config/sway/config.d/00-generated-bindings.conf`) point to `~/.dotfield/current/...`, reconciling active targets and removing stale symlinks for dropped targets.
3. **Atomic Pointer Swap, State Persistence & Failure Compensation**:
   - Updates `~/.dotfield/current` -> `~/.dotfield/generations/<N>/` via atomic temporary symlink rename, reverting to the prior generation pointer and prior live targets if binding or state persistence fails.
   - Atomically persists `state.json` via PID-scoped temporary file rename only after target binding succeeds; if persistence fails, full compensation rollback restores prior symlinks and live bindings.
   - State self-healing: `get_state()` validates active state against `current/meta.json`, reconstructing or synchronizing state automatically if `state.json` is missing or out-of-sync.
   - Cross-process advisory lock (`.lock` via `flock`) serializes concurrent deployments and rollbacks.

### 1.3 Target-Aware Reload Engine (Native IPC & File-Watch)
Instead of reloading the entire desktop session, Dotfield inspects the compiled fragment targets in the newly deployed generation and triggers reloads using native IPC channels or file-watch mechanisms:
- **Sway**: `swaymsg reload`
- **Niri**: `niri msg action load-config-file`
- **Mako**: `makoctl reload`
- **Kitty**: `kitty @ --to unix:<runtime_dir>/kitty-dotfield.sock set-colors --all --configured ~/.config/kitty/colors.conf`
- **Tmux**: `tmux source-file ~/.config/tmux/colors.conf`
- **Zellij**: Automatic file-watch hot-reload

---

## 2. CLI Interface & UX

```bash
dotfield status       # Current generation, active profile, drift status
dotfield check        # Validate profile syntax, missing binaries, keybind conflicts
dotfield diff         # Diff between active generation and candidate profile
dotfield apply        # Compile, atomically stage, link, and reload changed services
dotfield rollback     # Revert current pointer to generation N-1 and reload
dotfield list         # List available profiles in profiles/
```

---

## 3. Verification & Quality Gates

1. **Rust Toolchain Quality**:
   - `cargo fmt -- --check`: Zero formatting drift.
   - `cargo check`: Clean type checking.
   - `cargo test`: Comprehensive unit tests for AST compilation, profile deserialization, diff computation, and atomic state transitions.
   - `cargo clippy -- -D warnings`: Zero-warning policy on production code.
2. **Pre-commit Gate**:
   - Passes all 10 pre-commit hooks (`trailing-whitespace`, `end-of-file-fixer`, `check-yaml`, `check-toml`, `check-added-large-files`, `gitleaks`, `semgrep`, `cargo-fmt`, `cargo-check`, `cargo-clippy`).
