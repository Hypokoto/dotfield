# Engineering Plan Review: Phase 4 (Notifications, OSD & System Utilities)

**Review Target:** [`docs/designs/phase-4-notifications-osd-utilities.md`](../designs/phase-4-notifications-osd-utilities.md)
**Reviewer:** Pair Programming Partner (via `gstack-plan-eng-review`)
**Outcome:** APPROVED
**Verdict Date:** 2026-10-31

---

## 1. Executive Summary

Phase 4 defines the integration of lightweight, native Wayland desktop primitives:
- **Notifications**: Mako with 3 urgency tiers (`low`, `normal`, `critical`) tokenized with Controlled Chaos colors and dynamic `makoctl reload` hook.
- **Lockscreen & Idle**: Swaylock minimal ring configuration and Swayidle timeout definitions.
- **Utilities**: Screenshot pipeline (`grim + slurp + wl-copy`) and solid canvas wallpaper loader (`swaybg`).

All dependencies (`mako`, `makoctl`, `swaylock`, `swayidle`, `swaybg`, `grim`, `slurp`, `wl-copy`) are pre-installed in `/usr/sbin/`.

---

## 2. Invariant & Architecture Assessment

| Evaluation Dimension | Status | Notes |
| :--- | :--- | :--- |
| **Zero-Restart Live Reload** | **PASS** | Mako supports `makoctl reload` without terminating active notifications or killing the daemon. |
| **Controlled Chaos Token Adherence** | **PASS** | Mako, Swaylock, and Swaybg strictly bind to `Void #0A0A0C`, `Obsidian #121216`, `Amber #FFB000`, `Crimson #E63946`, `Iron #34343D`, and `Bone #E8E6E3`. |
| **Security & Safety Standards** | **PASS** | Screenshot and wallpaper scripts avoid unquoted expansions, use `set -euo pipefail`, and comply with Semgrep and Gitleaks gates. |
| **Tool Availability** | **PASS** | Verified all required binaries exist locally in `/usr/sbin/`. |

---

## 3. Implementation Checklist for Phase 4 Execution

1. **Mako Notification Daemon (`mako/config`)**:
   - Font, geometry, border size, and default background/text.
   - Urgency override blocks (`[urgency=low]`, `[urgency=normal]`, `[urgency=critical]`).
2. **Swaylock Lockscreen (`osd/swaylock/config`)**:
   - Indicator ring styling with Controlled Chaos color flags (`inside-color`, `ring-color`, `key-hl-color`, `line-color`).
3. **Swayidle Idle Daemon (`osd/swayidle/config`)**:
   - Graceful idle events: screen lock at 290s, display power off at 300s, lock before sleep.
4. **Screenshot Script (`scripts/screenshot.sh`)**:
   - Executable bash script supporting `--area`, `--window`, and `--full`, saving to `~/Pictures/Screenshots/` and copying to Wayland clipboard.
5. **Wallpaper Script (`scripts/wallpaper.sh`)**:
   - Lightweight runner executing `swaybg -c "#0A0A0C"`.

---

## 4. Engineering Verdict

**APPROVED.** Ready for immediate execution.
