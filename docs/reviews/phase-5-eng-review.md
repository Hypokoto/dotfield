# Engineering Plan Review: Phase 5 (Security Hardening & Profile Definitions)

**Review Target:** [`docs/designs/phase-5-security-profiles.md`](../designs/phase-5-security-profiles.md)
**Reviewer:** Pair Programming Partner (via `gstack-plan-eng-review`)
**Outcome:** APPROVED
**Verdict Date:** 2026-10-31

---

## 1. Executive Summary

Phase 5 introduces declarative security mechanisms across four complementary isolation vectors:
- **Application Sandboxing**: Firejail profile (`security/firejail/browser.profile`) with seccomp and capability dropping.
- **Service Isolation**: AppArmor profile abstraction (`security/apparmor/dotfield-daemon`) preventing unauthorized access to SSH/GPG credentials.
- **Desktop Sockets**: Flatpak declarative overrides (`security/flatpak/overrides.toml`) enforcing Wayland-only display and restricting legacy X11/PulseAudio access.
- **Network Perimeter**: NFTables stateful workstation firewall (`security/nft/workstation.nft`).

In addition, Phase 5 expands and validates concrete Dotfield semantic profiles (`profiles/default.toml`, `profiles/minimal.toml`, and `profiles/hardening.toml`) so that they represent complete, valid semantic models that map directly to the components built across Phases 2, 3, and 4.

---

## 2. Invariant & Architecture Assessment

| Evaluation Dimension | Status | Notes |
| :--- | :--- | :--- |
| **Declarative Security Drop-ins** | **PASS** | Clear modular separation between Firejail, AppArmor, Flatpak, and NFTables. |
| **Syntax Testability** | **PASS** | Can be validated locally via `nft -c -f ...` and `apparmor_parser -d ...`. |
| **Profile Semantic Integrity** | **PASS** | Profiles map cleanly to real applications (Foot, Fuzzel, Sway, Niri, Starship, Neovim, Zellij, Mako). |
| **Security & Safety Standards** | **PASS** | Fully adheres to Semgrep static analysis rules and Gitleaks rules. |

---

## 3. Implementation Checklist for Phase 5 Execution

1. **Firejail Profile (`security/firejail/browser.profile`)**:
   - `seccomp`, `caps.drop all`, `noroot`, `private-tmp`, whitelist `~/Downloads`.
2. **AppArmor Profile (`security/apparmor/dotfield-daemon`)**:
   - Deny read access to `~/.ssh/**`, `~/.gnupg/**`, `~/.aws/**`, and audit writes.
3. **Flatpak Overrides (`security/flatpak/overrides.toml`)**:
   - Sockets: `wayland`, `!x11`, `!fallback-x11`, `!pulseaudio`.
   - Filesystems: `xdg-run/pipewire-0`.
4. **NFTables Workstation Filter (`security/nft/workstation.nft`)**:
   - Ingress filter: drop by default, accept established/related, accept loopback (`iif "lo"`), accept ICMP.
5. **Concrete Profiles (`profiles/`)**:
   - Complete `default.toml` (Sway + Foot + Fuzzel + Neovim + Zellij + Mako).
   - Complete `minimal.toml` (Niri + Foot + Fuzzel + Neovim + Tmux).
   - Complete `hardening.toml` (Sway + Fuzzel + Neovim + Zellij + NFTables + AppArmor).

---

## 4. Engineering Verdict

**APPROVED.** Ready for immediate execution.
