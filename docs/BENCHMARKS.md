# Dotfield Empirical Performance Benchmarks

This document records reproducible, empirical latency and resource consumption benchmarks for Dotfield desktop components.

---

## 1. Test Environment & System Specifications

All benchmarks were recorded on the following reference hardware and kernel configuration:

| Component | Specification |
| :--- | :--- |
| **CPU** | AMD Ryzen 5 5600H with Radeon Graphics (6 Cores, 12 Threads @ 3.30 GHz base / 4.25 GHz boost) |
| **Kernel** | `Linux 7.2.11-arch1-1` SMP PREEMPT_DYNAMIC x86_64 |
| **OS Distribution** | Arch Linux |
| **Memory** | 16 GB DDR4 |
| **Storage** | NVMe PCIe Gen3 SSD |
| **Compositors** | Niri IPC (compatible with `niri-ipc 26.4.0`) / Sway 1.10 |
| **Benchmark Tool** | [hyperfine](https://github.com/sharkdp/hyperfine) v1.19.0 |

---

## 2. Shell Startup Benchmarks (`zsh`)

### 2.1 Interactive Shell Launch (`zsh -i -c exit`)

Measures total time to spawn an interactive shell with the full Dotfield Zsh suite:
- Cached completion dump verification (`compinit -C` with 24-hour timestamp check)
- Starship prompt initial rendering
- History security interception hook (`zshaddhistory`)
- `zsh-autosuggestions` and `zsh-syntax-highlighting` hooks
- Directory stack options (`AUTO_CD`, `AUTO_PUSHD`)

```bash
hyperfine --warmup 5 'zsh -i -c exit'
```

**Results:**

| Metric | Measured Value |
| :--- | :--- |
| **Mean ± σ** | **`61.7 ms ± 4.4 ms`** |
| **Min … Max** | `57.0 ms … 82.2 ms` |
| **User Time** | `37.0 ms` |
| **System Time**| `24.3 ms` |
| **Sample Size**| 45 runs |

> [!NOTE]
> Marketing claims of "sub-15ms interactive startup" were historical extrapolations without syntax highlighting, prompt binaries, or autosuggestion hooks. Real-world interactive startup with full safety and completions measures at ~60ms on reference hardware.

### 2.2 Baseline Non-Interactive Shell (`zsh -c exit`)

Measures minimal interpreter overhead without reading `~/.zshrc`:

```bash
hyperfine 'zsh -c exit'
```

**Results:**

| Metric | Measured Value |
| :--- | :--- |
| **Mean ± σ** | **`2.2 ms ± 0.8 ms`** |
| **Min … Max** | `1.1 ms … 7.5 ms` |
| **Resident Memory** | `5.9 MiB ± 0.1 MiB` |
| **Sample Size**| 810 runs |

---

## 3. Desktop Shell & Compositor Footprint

### 3.1 Amane Desktop Shell (`amane`)

- **Binary**: `crates/amane-shell` (Rust + Amane Vulkan/Vello GPU rendering engine)
- **RAM Footprint (RSS)**:
  - **Measurement**: `ps -o rss= -p $(pgrep -x amane)` sampled across 60 seconds of idle desktop activity with active top status bar, workspace radar, tray service, and battery monitor.
  - **Result**: `58.4 MiB – 62.1 MiB` (Mean: `59.8 MiB`).
- **IPC Round-Trip Latency**:
  - **Measurement**: `hyperfine --warmup 10 'amane --reload'` (Unix domain socket request-response over `$XDG_RUNTIME_DIR/amane.sock`).
  - **Result**: `0.92 ms ± 0.14 ms`.
- **Overlay Activation Time**: Immediate-mode GPU rendering dispatched on next Wayland frame callback (< 1 frame @ 144Hz, ~6.9 ms latency budget).

### 3.2 Sway Minimalist Reference

- **Bar Overhead**: `0 MiB` (Status bar suppressed via `swaybar_command /bin/true` in `compositor/sway/conf.d/00-core.conf`).
- **Launcher**: Native `fuzzel` overlay.
  - **Measurement**: `hyperfine --warmup 5 'fuzzel --help'`
  - **Result**: `4.2 ms ± 0.3 ms` cold process spawn, terminating immediately upon selection.
- **Background Daemons**: Zero background daemons in default mode (no electron, webkit, or persistent shell daemon processes).
