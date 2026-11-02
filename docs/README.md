# Dotfield Engineering Documentation

Welcome to the **Dotfield** architecture, design specifications, and engineering documentation repository.

---

## Documentation Index

### Core Architecture & Guidelines
- **[ARCHITECTURE.md](ARCHITECTURE.md)**: System blueprints, compiler engine, atomic staging pipeline, and daemon IPC.
- **[PROFILES.md](PROFILES.md)**: Profile models, semantic schema specification, and inheritance.
- **[ROADMAP.md](ROADMAP.md)**: Multi-phase engineering roadmap and milestones.
- **[../GEMINI.md](../GEMINI.md)**: Engineering invariants, directory conventions, and pre-commit security gates.
- **[../PALETTE.md](../PALETTE.md)**: Controlled Chaos theme specification and design tokens.

---

### Architectural Design Specifications (`docs/designs/`)
Detailed architectural blueprints for specific system capabilities:
- **[Phase 10: Niri + Amane Desktop Shell](designs/phase-10-niri-amane-shell.md)**: Native Rust + Vulkan GPU-accelerated shell.
- **[Phase 2: Compositor & Shell Architecture](designs/phase-2-compositor-launchers-shell.md)**: Modular Sway and Niri configuration drop-ins.
- **[Phase 3: Terminals, Neovim & Multiplexers](designs/phase-3-terminals-nvim-multiplexers-bar.md)**: Terminal emulators and harmonized multiplexers.
- **[Phase 4: Notifications, OSD & Desktop Utilities](designs/phase-4-notifications-osd-utilities.md)**: Mako, SwayOSD, and power/audio daemons.
- **[Phase 5: Security Sandboxing Profiles](designs/phase-5-security-profiles.md)**: Firejail, AppArmor, Flatpak, and NFTables rulesets.
- **[Phase 6: Native Core CLI Engine](designs/phase-6-native-core-cli-engine.md)**: Rust AST compiler and diff engine.
- **[Phase 7: GUI Utilities & Daemon](designs/phase-7-gui-utilities-daemon.md)**: Daemon IPC and overlay management.
- **[Phase 8: Non-Rust Latency & Productivity](designs/phase-8-non-rust-latency-productivity.md)**: Sub-15ms Zsh, Starship caching, and FZF.
- **[Phase 9: UI/UX Aesthetic Enhancements](designs/phase-9-ui-ux-enhancement.md)**: Controlled Chaos colorways and Ly display manager.
- **[Live Desktop Compiler Architecture](designs/live-desktop-compiler-architecture.md)**: Dynamic reloads and zero-lag reconfiguration.

---

### Engineering Reviews (`docs/reviews/`)
Comprehensive milestone post-mortems and verification sign-offs:
- **[Phase 1 Review](reviews/phase-1-eng-review.md)**: Foundation, git hygiene, and security gates.
- **[Phase 3 Review](reviews/phase-3-eng-review.md)**: Terminal templates and multiplexer synchronization.
- **[Phase 4 Review](reviews/phase-4-eng-review.md)**: OSD, notification HUD, and battery alerting.
- **[Phase 5 Review](reviews/phase-5-eng-review.md)**: Security profiles and sandboxing.
- **[Phase 6 Review](reviews/phase-6-eng-review.md)**: Core compilation engine.
- **[Phase 7 Review](reviews/phase-7-eng-review.md)**: Desktop shell overlays and IPC sockets.
- **[Phase 8 Review](reviews/phase-8-eng-review.md)**: Shell performance benchmarking and secret interception.
- **[Phase 10 Review](reviews/phase-10-eng-review.md)**: Niri integration and Amane visual quality audit.
