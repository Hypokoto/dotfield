# Ly Display Manager: BLACKSITE Industrial Noir Theme

Dotfield provides unified configuration for **Ly**, a lightweight TUI display manager for modern Wayland/Linux desktop environments.

## Color Palette Mapping

The configuration binds strictly to the **BLACKSITE Industrial Noir** 65-20-8-5-2 token hierarchy:

| Token | Hex Code | TrueColor (0xSSRRGGBB) | Ly Role |
| :--- | :--- | :--- | :--- |
| **Void Black** | `#08080A` | `0x0008080A` | Background canvas (`bg`), error background (`error_bg`) |
| **Bone** | `#E6E6EB` | `0x00E6E6EB` | Primary text (`fg`, `text_fg`) |
| **Tactical Amber** | `#FFB000` | `0x01FFB000` | Focused border (`border_fg`), matrix highlight (`cmatrix_fg`) |
| **Crimson** | `#E63946` | `0x01E63946` | Error alerts (`error_fg`), failed authentication attempts |
| **Iron** | `#282A36` | `0x00282A36` | Outer frame boundary and subtle dividers |

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
