# WezTerm Configuration — Dotfield

High-performance GPU-accelerated terminal emulator configured for modern Wayland workflows, featuring **BLACKSITE Industrial Noir** aesthetics, zero-wake idle frame parking, and unified navigation.

## Architecture

- **Backend**: Direct GPU rendering via `WebGpu` with HighPerformance power profile and 120 FPS max throughput.
- **Wayland Native**: Runs under pure Wayland (`enable_wayland = true`, `window_decorations = "NONE"`) with zero XWayland dependencies.
- **Power Efficiency**: `cursor_blink_rate = 0` and `animation_fps = 1` ensure zero GPU/CPU wakeups when the terminal is idle.
- **BLACKSITE Palette**: Full 65-20-8-5-2 palette integration (Void Black `#08080A`, Obsidian `#111216`, Graphite `#181920`, Iron `#282A36`, Bone `#E6E6EB`, Tactical Amber `#FFB000`, Crimson `#E63946`).
- **Telemetry HUD**: Dynamic tab bar with angled active tab indicators, zoomed pane warnings (`[ZOOM]`), active mode/leader badges, and localized clock/date status.

## File Hierarchy

```
terminals/wezterm/
├── wezterm.lua     # Core entrypoint, typography, geometry, rendering options
├── colors.lua      # BLACKSITE 16-color ANSI palette and UI element color maps
├── keys.lua        # Alt-based splits, Neovim-aware navigation, leader multiplexing
├── status.lua      # Dynamic tab styling and right-hand telemetry HUD
└── README.md       # Technical specification and keybindings reference
```

## Keybindings

### Native Splits & Navigation (Alt-based)

| Shortcut | Action |
| :--- | :--- |
| `Alt + Enter` | Split Right (Vertical line) |
| `Alt + Shift + Enter` | Split Down (Horizontal line) |
| `Alt + h` | Navigate to left pane / Neovim split |
| `Alt + j` | Navigate to bottom pane / Neovim split |
| `Alt + k` | Navigate to top pane / Neovim split |
| `Alt + z` or `Ctrl + Shift + Enter` | Toggle independent pane zoom in/out (matches Ghostty `toggle_split_zoom`) |
| `Alt + o` | Cycle to next pane (preserves zoom state across panes) |
| `Alt + w` | Close active pane |

### Native Tabs

| Shortcut | Action |
| :--- | :--- |
| `Alt + t` | Spawn new tab |
| `Alt + ]` | Next tab |
| `Alt + [` | Previous tab |
| `Alt + 1..5` | Jump directly to tab 1 through 5 |

### Search, Copy Mode & QuickSelect

| Shortcut | Action |
| :--- | :--- |
| `Alt + s` / `Alt + f` | Open scrollback search |
| `Alt + u` | Trigger QuickSelect (1-key copy for URLs, paths, Git SHAs, IPs) |
| `Alt + x` | Enter Vi-style copy mode |
| `Ctrl + Shift + c` | Copy selection to clipboard |
| `Ctrl + Shift + v` | Paste from clipboard |

### Multiplexer Leader (`Ctrl + A`)

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + A` then `h/j/k/l` | Resize active pane by 5 cells |
| `Ctrl + A` then `r` | Hot-reload WezTerm configuration |
| `Ctrl + A` then `Space` | Trigger QuickSelect |
| `Ctrl + A` then `Ctrl + A` | Send literal `Ctrl + A` to shell/program |
