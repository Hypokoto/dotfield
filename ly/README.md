# Ly Display Manager: Obsidian Mint Theme

Dotfield provides unified configuration for **Ly**, a lightweight TUI display manager for modern Wayland/Linux desktop environments.

## Color Palette Mapping

The configuration binds strictly to the **Obsidian Mint** 60-30-10 token hierarchy:

| Token | Hex Code | TrueColor (0xSSRRGGBB) | Ly Role |
| :--- | :--- | :--- | :--- |
| **Void Black** | `#080B0B` | `0x00080B0B` | Background canvas (`bg`), error background (`error_bg`) |
| **Minted White** | `#ECFDF5` | `0x00ECFDF5` | Primary text (`fg`, `text_fg`) |
| **Mint** | `#2DD4BF` | `0x012DD4BF` | Focused border (`border_fg`) |
| **Emerald** | `#10B981` | `0x0010B981` | Matrix highlight (`cmatrix_fg`) |
| **Rose** | `#F43F5E` | `0x01F43F5E` | Error alerts (`error_fg`), failed authentication attempts |
| **Iron** | `#23322F` | `0x0023322F` | Outer frame boundary and subtle dividers |

## Deployment

Depending on the Ly version installed on your system:

### 1. Standard Ly (`config.ini`)
For standard Ly (Arch Linux / pacman `ly` 1.4+):
```bash
sudo cp ly/config.ini /etc/ly/config.ini
```

### 2. Modern Lua Ly (`config.lua`)
For Zig-based Ly or installations supporting Lua configuration:
```bash
sudo cp ly/config.lua /etc/ly/config.lua
```

### 3. Enable Systemd Service
```bash
sudo systemctl enable ly.service
```

## Features

- **Zero Latency**: Disables matrix animations by default for instant cold startup into Niri or Sway.
- **TrueColor Precision**: Uses 24-bit 0xSSRRGGBB TrueColor rendering for exact BLACKSITE color reproduction on Linux KMS TTYs.
- **Session Persistence**: Automatically remembers and persists the last selected compositor session (Niri / Sway) and username across reboots.
- **Tactical Amber Frame**: Highlighted main terminal box with integrated localized weekday and date timestamp.
