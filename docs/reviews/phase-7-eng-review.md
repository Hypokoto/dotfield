# Engineering Plan Review: Phase 7 (Custom GUI Utilities & Event Daemon)

**Review Target:** [`docs/designs/phase-7-gui-utilities-daemon.md`](../designs/phase-7-gui-utilities-daemon.md)
**Reviewer:** Pair Programming Partner (via `gstack-plan-eng-review`)
**Outcome:** APPROVED
**Verdict Date:** 2026-10-31

---

## 1. Executive Summary

Phase 7 delivers the graphical and automated interface layers of Dotfield:
- **`crates/dotfield-gui`**: A native Wayland graphical tool built with **Rust + Slint**. It implements an instantaneous on-screen keybind cheat-sheet HUD (`--hud`) and an interactive profile switcher card viewer (`--switcher`), styled in strict accordance with the Controlled Chaos design system tokens (`Void`, `Obsidian`, `Graphite`, `Iron`, `Bone`, `Amber`, `Crimson`).
- **`crates/dotfieldd`**: A background daemon and hardware event monitor that evaluates system power status (`/sys/class/power_supply`) to trigger automatic, transactional profile switching (e.g. `default.toml` when on AC power vs. `minimal.toml` when running on battery) via `dotfield-core`.

---

## 2. Invariant & Architecture Assessment

| Evaluation Dimension | Status | Notes |
| :--- | :--- | :--- |
| **Workspace Separation** | **PASS** | Adds `dotfield-gui` and `dotfieldd` cleanly into the Cargo workspace without circular dependencies; both consume `dotfield-core`. |
| **Slint Integration & UI Design** | **PASS** | Uses Slint declarative markup compiled via `build.rs`. Strictly implements Controlled Chaos palette (`#0A0A0C`, `#121216`, `#1C1C23`, `#34343D`, `#E8E6E3`, `#FFB000`, `#E63946`). |
| **Searchable Keybind HUD** | **PASS** | Dynamically reflects logical keybind definitions extracted from the active profile or default profile. |
| **Visual Profile Switcher** | **PASS** | Dispatches transactional generation deployments directly through `dotfield_core::GenerationManager::deploy`. |
| **Hardware Daemon Mechanics** | **PASS** | Robust Linux power supply sysfs parser (`/sys/class/power_supply/AC*` / `BAT*`), supporting both `--daemon` foreground event loop and `--once` deterministic evaluation. |
| **Toolchain & Quality Gates** | **PASS** | Verified to comply with `cargo fmt -- --check`, `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, and all 10 pre-commit hooks. |

---

## 3. Implementation Checklist for Phase 7 Execution

1. **Workspace Member Registration (`Cargo.toml`)**:
   - Add `crates/dotfield-gui` and `crates/dotfieldd` to `members`.
2. **`crates/dotfield-gui` Implementation**:
   - `Cargo.toml`: `slint = "1.8"`, `clap = { version = "4.5", features = ["derive"] }`, `dotfield-core = { path = "../dotfield-core" }`, `anyhow`.
   - `build.rs`: Compiles Slint templates (`ui/hud.slint`, `ui/switcher.slint`).
   - `ui/palette.slint`: Controlled Chaos color constants and common component styling.
   - `ui/hud.slint`: Searchable Keybind HUD window.
   - `ui/switcher.slint`: Card-based Profile Switcher window.
   - `src/main.rs`: CLI parser (`--hud`, `--switcher`) invoking the respective Slint windows.
3. **`crates/dotfieldd` Implementation**:
   - `Cargo.toml`: `dotfield-core = { path = "../dotfield-core" }`, `clap = { version = "4.5", features = ["derive"] }`, `anyhow`, `colored`.
   - `src/power.rs`: Sysfs power supply monitor determining AC vs. Battery state.
   - `src/main.rs`: Daemon event loop and `--once` execution mode applying appropriate profiles.
4. **Documentation & Roadmap**:
   - Update [`docs/ROADMAP.md`](../ROADMAP.md) marking Phase 7 items complete.
5. **Quality Verification**:
   - Run `cargo fmt -- --check`, `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, `gitleaks detect --no-git --redact -v`, `semgrep --config=.semgrep.yml --error .`, and `pre-commit run --all-files`.

---

## 4. Engineering Verdict

**APPROVED.** Ready for immediate execution.
