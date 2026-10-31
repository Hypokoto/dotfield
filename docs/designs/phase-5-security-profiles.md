# Phase 5 Design Document: Security Hardening & Profile Definitions

**Project:** Dotfield (`dotfield`)
**Date:** 2026-10-31
**Author:** Pair Programming Session (via `gstack-office-hours`)
**Status:** APPROVED
**Scope:** Phase 5 of 7 (Security Hardening & Profile Definitions)

---

## Executive Summary

Phase 5 formalizes Dotfield's declarative security layer and reference profile schemas. Guided by the principle **"Store logical choices, not duplicated config"**, Dotfield decouples high-level user posture (e.g. daily driver vs. hardened sandbox workstation) from individual tool configurations.

Security drop-in fragments provide modular sandboxing for desktop applications (Firejail, Flatpak), system service restrictions (AppArmor), and network perimeter control (NFTables). Simultaneously, concrete profiles (`default.toml`, `minimal.toml`, and `hardening.toml`) reflect full semantic models that cleanly compile into the configurations built across Phases 2, 3, and 4.

---

## 1. Architectural Invariants & Mechanics

### 1.1 Declarative Security Drop-ins (`security/`)
- **Firejail (`security/firejail/browser.profile`)**:
  - Sandboxing template for untrusted web browsers and electron runtimes.
  - Enables `seccomp`, `caps.drop all`, `nonewprivs`, `noroot`, private `/tmp`, and restricts access to sensitive directories (`~/.ssh`, `~/.gnupg`, `~/.dotfield`).
- **AppArmor (`security/apparmor/dotfield-daemon`)**:
  - Abstraction profile protecting system integrity against daemon compromise.
  - Restricts raw socket access and denies read permissions to secret stores.
- **Flatpak (`security/flatpak/overrides.toml`)**:
  - Declarative global and app-level overrides enforcing Wayland-only display sockets (`--socket=wayland`, `--nosocket=x11`, `--nosocket=fallback-x11`).
- **NFTables (`security/nft/workstation.nft`)**:
  - Declarative workstation packet filter.
  - Stateful ingress rules (drop all incoming by default, accept `ct state established,related`, accept loopback `iif lo`).
  - Optional egress tagging for auditability.

### 1.2 Concrete TOML Profiles (`profiles/`)
- **`profiles/default.toml`**: Balanced daily-driver environment.
  - Compositor: Sway
  - Apps: Foot, Fuzzel, Neovim, Zellij, Mako
  - Security: Standard Flatpak overrides, basic NFTables workstation filter.
- **`profiles/minimal.toml`**: Ultra-lightweight resource footprint.
  - Compositor: Niri
  - Apps: Foot, Fuzzel, Neovim, Tmux
  - Security: Baseline sandboxing, minimal background overhead.
- **`profiles/hardening.toml`**: Maximum security and strict sandboxing.
  - Compositor: Sway
  - Apps: Foot, Fuzzel, Neovim, Zellij
  - Security: Firejail browser isolation, strict NFTables, AppArmor abstraction drop-in.

---

## 2. Directory Layout & Deliverables

```
dotfield/
├── security/
│   ├── firejail/
│   │   └── browser.profile          # Browser/electron sandboxing profile
│   ├── apparmor/
│   │   └── dotfield-daemon          # AppArmor security profile abstraction
│   ├── flatpak/
│   │   └── overrides.toml           # Declarative Flatpak socket & device overrides
│   └── nft/
│       └── workstation.nft          # Stateful packet filter ruleset
└── profiles/
    ├── default.toml                 # Daily-driver balanced profile
    ├── minimal.toml                 # Lightweight minimal profile
    └── hardening.toml               # Sandboxed hardened profile
```

---

## 3. Verification & Testing Plan

1. **Syntax & Linter Checks**:
   - `nft -c -f security/nft/workstation.nft`: Validate packet filter syntax without loading into kernel.
   - `apparmor_parser -d security/apparmor/dotfield-daemon`: Validate AppArmor ruleset grammar.
   - TOML parsing of all profiles (`profiles/*.toml`) and Flatpak configuration (`security/flatpak/*.toml`).
2. **Quality Gates**:
   - `pre-commit run --all-files` (all 10 hooks: trailing-whitespace, end-of-file-fixer, check-yaml, check-toml, check-added-large-files, gitleaks, semgrep, cargo-fmt, cargo-check, cargo-clippy).
