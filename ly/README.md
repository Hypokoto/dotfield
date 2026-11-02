# Ly Display Manager — Controlled Chaos Theme

Dotfield provides unified configuration for **Ly**, a lightweight TUI display manager for Linux.

## Color Palette Mapping

The configuration binds strictly to the **Controlled Chaos** specification (`PALETTE.md`):

| Token | Hex | ANSI / RGB | Ly Role |
| :--- | :--- | :--- | :--- |
| **Void Black** | `#0A0A0C` | `0` / `0x0A0A0C` | Background canvas (`bg`), border background (`border_bg`) |
| **Bone** | `#E8E6E3` | `7` / `0xE8E6E3` | Primary text (`fg`, `text_fg`) |
| **Industrial Amber** | `#FFB000` | `3` / `0xFFB000` | Focused border (`border_fg`), visual focus highlight |
| **Crimson** | `#E63946` | `1` / `0xE63946` | Error alerts (`error_fg`), failed authentication attempts |
| **Iron** | `#34343D` | — | Subtle box dividers |

## Deployment

Depending on the Ly version installed on your system:

### 1. Classic Ly (`config.ini`)
For C-based Ly (default in most distribution packages like Arch Linux / AUR `ly`):
```bash
sudo cp ly/config.ini /etc/ly/config.ini
```

### 2. Modern Ly (`config.lua`)
For modern Zig-based Ly:
```bash
sudo cp ly/config.lua /etc/ly/config.lua
```

### 3. Enable Systemd Service
```bash
sudo systemctl enable ly.service
```

## Features

- **Zero Latency**: Disables superfluous falling terminal matrix animations for instant cold startup.
- **Session Remembering**: Automatically remembers the last selected desktop session (Niri / Sway) and username.
- **Controlled Chaos Border**: Rounded borders with Industrial Amber highlight (`#FFB000`) and Crimson error flash (`#E63946`).
- **Clock Display**: Integrated localized weekday and date timestamp.
