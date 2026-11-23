# Ly Display Manager: Cyber Synthwave Theme

Dotfield provides unified configuration for **Ly**, a lightweight TUI display manager for modern Wayland/Linux desktop environments.

## Color Palette Mapping

The configuration binds strictly to the **Cyber Synthwave** token hierarchy:

| Token | Hex Code | TrueColor (0xSSRRGGBB) | Ly Role |
| :--- | :--- | :--- | :--- |
| **Void Black** | `#000000` | `0x00000000` | Background canvas (`bg`), error background (`error_bg`) |
| **Cyan-Teal** | `#14B9B5` | `0x0014B9B5` | Primary text (`fg`, `text_fg`) |
| **Neon Blue** | `#04C5F0` | `0x0104C5F0` | Focused border (`border_fg`) |
| **Emerald** | `#7CD699` | `0x007CD699` | Matrix highlight (`cmatrix_fg`) |
| **Red Accent** | `#BE3F50` | `0x01BE3F50` | Error alerts (`error_fg`), failed authentication attempts |
| **Purple** | `#251B4A` | `0x00251B4A` | Outer frame boundary and subtle dividers |

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
