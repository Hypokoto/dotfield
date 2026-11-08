# Upstream Attribution & Vendor Notices

## `amane` Rendering Engine

Dotfield vendors and builds upon the **Amane** rendering library:

- **Upstream Repository**: [https://github.com/MystiaFin/amane](https://github.com/MystiaFin/amane)
- **Author**: [MystiaFin](https://github.com/MystiaFin)
- **License**: MIT (retained at [`crates/amane/LICENSE`](crates/amane/LICENSE))
- **Description**: GPU-accelerated immediate-mode Wayland desktop rendering engine using Vulkan and Vello.

### Local Modifications in Dotfield
Dotfield maintains targeted modifications in [`crates/amane`](crates/amane) to support dynamic multi-monitor Wayland workflows:
1. **Output Lifecycle & Hotplugging**: Lazy output lookup in `ensure_plain_windows` and display configuration sync across DPMS sleep/wake events.
2. **GPU Device Loss Recovery**: Clean exit signal propagation (`std::process::exit(1)`) on physical GPU disconnect or context loss, enabling systemd supervisor auto-restart.
3. **Event Loop Resilience**: Calloop dispatch hardening to prevent render pipeline stalls during high-frequency Wayland compositor configure sequences.

---

## Original Work in Dotfield

All remaining components are original works licensed under the MIT License:
- **`crates/amane-shell`**: Native Rust Wayland desktop shell binary (`amane`), including the status bar, workspace radar, application launcher, terminal manager, interactive display configurator, HUD, and IPC server/client.
- **Compositor Modular Configurations**: `compositor/niri/conf.d/*.kdl` and `compositor/sway/conf.d/*.conf`.
- **System Integration & Tooling**: `setup.sh`, `install.sh`, `scripts/*` (e.g., `terminal-select.sh`, `fw-open`, `sandbox-browser.sh`, `display-config.sh`).
- **Security Sandboxing Architecture**: AppArmor daemon profile (`security/apparmor/`), NFTables workstation firewall (`security/nft/`), Flatpak global overrides (`security/flatpak/`), and Bubblewrap unprivileged browser sandbox.
