<div align="center">

<img src="logo.svg" alt="wayvid logo" width="100" height="100">

# wayvid

Animated wallpaper manager for Wayland

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.5.0-green.svg)](https://github.com/YangYuS8/wayvid/releases)

[Documentation](https://yangyus8.top/wayvid/) • [Releases](https://github.com/YangYuS8/wayvid/releases)

</div>

## What it does

wayvid plays video files as animated wallpapers on Wayland compositors. **v0.5** introduces a GUI-first design — just open the app, pick a wallpaper, and apply.

**Features:**

- 🖼️ **GUI wallpaper browser** with thumbnails and search
- 🖥️ **Multi-monitor support** with independent wallpapers per display
- ⚡ **Hardware accelerated** decoding (VA-API/NVDEC via mpv)
- 🎮 **Steam Workshop** import (video wallpapers)
- 🌈 **HDR support** with tone-mapping
- 🔧 **CLI tools** for scripting and automation
- 💾 **Wallpaper persistence** - restore wallpapers after restart
- 🔋 **Power management** - auto-pause on battery
- 📥 **System tray** - start minimized and run in background

**Tested on:** Hyprland, Niri

**Should work on:** Sway, River, and other wlr-layer-shell compositors

## Demo

<!-- TODO: Add demo video/GIF here -->
*Demo coming soon*

## Install

### Arch Linux (AUR)

```bash
yay -S wayvid
```

### Nix

```bash
# Direct run
nix run github:YangYuS8/wayvid

# Install to profile
nix profile install github:YangYuS8/wayvid
```

### From source

```bash
git clone https://github.com/YangYuS8/wayvid.git
cd wayvid
flutter build linux --release
cargo build --release -p wayvid-ctl

# Install using script (recommended)
./scripts/install.sh --user

# Or manual install
sudo install -d /usr/local/lib/wayvid
sudo cp -a build/linux/x64/release/bundle/. /usr/local/lib/wayvid/
sudo install -Dm755 packaging/wayvid-gui-wrapper /usr/local/bin/wayvid-gui
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

**Dependencies:** libmpv, libEGL, libwayland-client

## Usage

### GUI (Recommended)

```bash
wayvid-gui
```

The GUI provides:
- Wallpaper library browser with thumbnails
- Monitor selection and preview
- Settings configuration (autostart, power management)
- Minimizes to system tray

### CLI Control

```bash
# Apply wallpaper
wayvid-ctl apply ~/Videos/wallpaper.mp4
wayvid-ctl apply ~/Videos/wallpaper.mp4 --output DP-1

# Control playback
wayvid-ctl pause
wayvid-ctl resume
wayvid-ctl stop

# Check status
wayvid-ctl status
wayvid-ctl status --json

# List monitors
wayvid-ctl outputs
```

### Autostart

The GUI includes autostart options in Settings:

1. **Start with system** - Enable autostart
2. **Minimize to tray** - Keep running in background
3. **Start minimized** - Start directly to tray

With all three enabled, wayvid will:
- Start automatically on login
- Run in the background (tray icon)
- Restore your wallpapers from last session

**Alternative manual configuration:**

```kdl
# niri: ~/.config/niri/config.kdl
spawn-at-startup "wayvid-gui"
```

```conf
# hyprland: ~/.config/hypr/hyprland.conf
exec-once = wayvid-gui
```

**systemd (optional):**
```bash
systemctl --user enable --now wayvid
```

## Configuration

Settings are managed through the GUI and saved automatically to:
```
~/.config/wayvid/settings.yaml
```

For advanced users, legacy config.yaml is still supported:
```yaml
# ~/.config/wayvid/config.yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
layout: fill
volume: 0
```

## Multi-monitor

Use the GUI's Monitor tab, or configure per-output:

```yaml
# ~/.config/wayvid/config.yaml
source:
  type: file
  path: ~/Videos/default.mp4

per_output:
  DP-1:
    source:
      type: file
      path: ~/Videos/left.mp4
  HDMI-A-1:
    source:
      type: file
      path: ~/Videos/right.mp4
```

## Steam Workshop

Import video wallpapers from Wallpaper Engine through the GUI, or:

```bash
wayvid-ctl apply ~/.steam/steam/steamapps/workshop/content/431960/<id>/video.mp4
```

**Note:** Only video wallpapers are supported. Web/scene types require Wallpaper Engine.

## Troubleshooting

**Black screen:**
```bash
mpv ~/Videos/wallpaper.mp4  # Test if video plays
```

**High CPU:**
```yaml
# Enable hardware decode
hwdec: true
```

**View logs:**
```bash
journalctl --user -u wayvid -f
```

## Project Structure

```
lib/                # Root Flutter application
linux/              # Flutter Linux runner
rust/               # Rust service exposed to Flutter through FRB
linux/cargokit/     # Flutter Linux build integration for the Rust service
crates/
├── wayvid-core     # Core types and configuration
├── wayvid-engine   # Integrated playback engine (Wayland layer-shell + MPV)
├── wayvid-library  # Wallpaper library (SQLite + thumbnails)
└── wayvid-ctl      # CLI control tool
```

### Architecture (v0.5)

wayvid v0.5 uses a **single-process architecture**:

- The Flutter GUI (`wayvid-gui`) owns presentation and Riverpod state
- The top-level Rust service owns playback, Wayland, tray, settings, and IPC
- No separate daemon process required
- CLI tools communicate via IPC socket
- Better resource management and simpler deployment

```
┌─────────────────────────────────────┐
│         wayvid-gui                  │
│  ┌─────────────┐  ┌──────────────┐  │
│  │ Flutter UI  │──│ Rust service │  │
│  │  Riverpod   │  │  FRB bridge  │  │
│  └─────────────┘  └──────┬───────┘  │
│                          │          │
│                 Engine / IPC / Tray│
└─────────│─────────────────│─────────┘
          │                 │
    wayvid-ctl         Compositor
```

## Contributing

```bash
flutter build linux --release
cargo build --release -p wayvid-ctl
cargo test --workspace
cargo clippy --workspace
```

## License

MIT OR Apache-2.0

## Acknowledgments

Built with [Flutter](https://flutter.dev/), [flutter_rust_bridge](https://github.com/fzyzc/flutter_rust_bridge), [mpv](https://mpv.io/), and [wayland-rs](https://github.com/Smithay/wayland-rs).
