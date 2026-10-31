# Engineering Plan Review: Phase 6 (Native Core & CLI Engine)

**Review Target:** [`docs/designs/phase-6-native-core-cli-engine.md`](../designs/phase-6-native-core-cli-engine.md)
**Reviewer:** Pair Programming Partner (via `gstack-plan-eng-review`)
**Outcome:** APPROVED
**Verdict Date:** 2026-10-31

---

## 1. Executive Summary

Phase 6 implements the core compilation and operational logic of Dotfield:
- **`crates/dotfield-core`**: A clean, reusable Rust library that parses semantic profile TOML files, validates prerequisites and `$PATH` binary availability, compiles target fragments for all desktop layers, manages transactional state in `~/.dotfield/generations/N/`, atomically updates symlink pointers, and executes selective IPC reloads without desktop restarts.
- **`crates/dotfield-cli`**: The user-facing CLI binary exposing `status`, `check`, `diff`, `apply`, `rollback`, and `list` subcommands via `clap`.

---

## 2. Invariant & Architecture Assessment

| Evaluation Dimension | Status | Notes |
| :--- | :--- | :--- |
| **Separation of Concerns** | **PASS** | Pure library `dotfield-core` decoupled from CLI frontend `dotfield-cli`, enabling easy GUI reuse in Phase 7. |
| **Transactional Generational Engine** | **PASS** | Writes generation fragments directly to `~/.dotfield/generations/N/`, atomically swaps `~/.dotfield/current` with failure compensation, reconciles live symlinks, and tracks rollback history in `state.json`. |
| **Selective Reload Capability** | **PASS** | Target-aware reloader inspects newly compiled fragment targets and signals only active desktop components (`swaymsg`, `niri msg`, `makoctl`, `kitty @ set-colors`, `tmux`). |
| **Error Handling & Code Safety** | **PASS** | Strict `thiserror` / `anyhow` error propagation; zero `unwrap()` or `panic!()` in library crate. |
| **Toolchain & Quality Gates** | **PASS** | Fully tested with `cargo check`, `cargo test`, `cargo fmt`, and `cargo clippy -- -D warnings`. |

---

## 3. Implementation Checklist for Phase 6 Execution

1. **Cargo Workspace Configuration (`Cargo.toml`)**:
   - Define workspace members: `crates/dotfield-core`, `crates/dotfield-cli`.
2. **`crates/dotfield-core` Implementation**:
   - `Cargo.toml`: `serde`, `serde_derive`, `toml`, `thiserror`, `anyhow`, `walkdir`, `similar`, `colored`.
   - `src/error.rs`: Typed error domain.
   - `src/profile.rs`: TOML schema for `apps`, `bindings`, `theme`, `compositor`, `security`.
   - `src/validator.rs`: Prerequisites and binary detection.
   - `src/compiler.rs`: Translates logical profile into fragment strings.
   - `src/generation.rs`: Transactional deployment and rollback mechanics.
   - `src/diff.rs`: Generates unified diffs between current and candidate generations.
   - `src/reloader.rs`: Selective IPC dispatch.
   - Unit tests covering compiler, profile parser, and generation logic.
3. **`crates/dotfield-cli` Implementation**:
   - `Cargo.toml`: `clap`, `anyhow`, `colored`, depending on `dotfield-core`.
   - Subcommands: `status`, `check`, `diff`, `apply`, `rollback`, `list`.
4. **Verification**:
   - Run `cargo fmt -- --check`, `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, and `pre-commit run --all-files`.

---

## 4. Engineering Verdict

**APPROVED.** Ready for immediate execution.
