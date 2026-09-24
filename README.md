<div align="center">

<img src="logo.svg" alt="VarPaper logo" width="100" height="100">

# VarPaper

Animated wallpaper manager for Wayland and X11

[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

[Project](https://github.com/mybna134/lwe-flutter) • [Releases](https://github.com/mybna134/lwe-flutter/releases)

</div>

## What it does

VarPaper plays video files as animated wallpapers on Wayland and X11 desktops. Open the app, pick a wallpaper, and apply it to one display or all displays.

**Features:**

- 🖼️ **GUI wallpaper browser** with thumbnails and search
- 🖥️ **Multi-monitor support** with independent wallpapers per display
- ⚡ **Hardware accelerated** decoding (VA-API/NVDEC via mpv)
- 🎮 **Steam Workshop** import (video wallpapers)
- 🌈 **HDR support** with tone-mapping
- 💾 **Wallpaper persistence** - restore wallpapers after restart
- 🔋 **Power management** - auto-pause on battery
- 📥 **System tray** - start minimized and run in background

**Tested on:** Hyprland, Niri

**Should work on:** Sway, River, and other wlr-layer-shell compositors

Native X11 sessions use RandR to discover monitors and create desktop-type wallpaper windows. Wayland sessions continue to use layer-shell, even when XWayland is available. X11 window stacking can depend on the window manager and desktop icon manager.

## Demo

<!-- TODO: Add demo video/GIF here -->
*Demo coming soon*

## Install

Release downloads contain `.deb` and `.flatpak` packages.

```bash
sudo apt install ./varpaper_*.deb
# or
flatpak install ./varpaper_*.flatpak
```

### Build packages from source

```bash
git clone https://github.com/mybna134/lwe-flutter.git
cd lwe-flutter
./scripts/build-packages.sh
```

The build requires Flutter, Rust, Linux development libraries (including X11, RandR, and XFixes headers), `dpkg-deb`, `flatpak-builder`, and the GNOME 50 Flatpak runtime and SDK. Packages are written to `dist/`.

## Usage

### GUI

```bash
varpaper
```

The GUI provides:
- Wallpaper library browser with thumbnails
- Monitor selection and preview
- Settings configuration (autostart, power management)
- Minimizes to system tray

### Autostart

The GUI includes autostart options in Settings:

1. **Start with system** - Enable autostart
2. **Minimize to tray** - Keep running in background
3. **Start minimized** - Start directly to tray

With all three enabled, VarPaper will:
- Start automatically on login
- Run in the background (tray icon)
- Restore your wallpapers from last session

**Alternative manual configuration:**

```kdl
# niri: ~/.config/niri/config.kdl
spawn-at-startup "varpaper"
```

```conf
# hyprland: ~/.config/hypr/hyprland.conf
exec-once = varpaper
```

## Configuration

Settings are managed by the Flutter UI and saved in the Isar Community
database under the platform application support directory. Existing YAML
settings are intentionally not migrated.

## Multi-monitor

Use the GUI's Monitor tab to apply a wallpaper to all outputs or to a
specific output.

## Steam Workshop

Import video wallpapers from Wallpaper Engine through the GUI.

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
├── wayvid-engine   # Integrated playback engine (Wayland/X11 + MPV)
└── wayvid-library  # In-memory wallpaper scanning and previews
```

### Architecture (v0.5)

wayvid v0.5 uses a **single-process architecture**:

- The Flutter GUI (`wayvid-gui`) owns presentation and Riverpod state
- Flutter owns settings, tray, and application lifecycle
- The top-level Rust service owns scanning and playback behind FRB
- No separate daemon process required
- Better resource management and simpler deployment

```
┌─────────────────────────────────────┐
│         wayvid-gui                  │
│  ┌─────────────┐  ┌──────────────┐  │
│  │ Flutter UI  │──│ Rust service │  │
│  │  Riverpod   │  │  FRB bridge  │  │
│  └─────────────┘  └──────┬───────┘  │
│                          │          │
│                    Engine service │
└─────────│─────────────────│─────────┘
                            │
                       Compositor
```

## Contributing

```bash
./scripts/build-linux.sh
cargo test --workspace
```

## License

VarPaper is licensed under [GNU GPL v3 only](LICENSE). See [NOTICE](NOTICE) for upstream attribution. The retained [Apache-2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) texts apply to upstream material and do not replace VarPaper's GPL license.

## Acknowledgments

Built with [Flutter](https://flutter.dev/), [flutter_rust_bridge](https://github.com/fzyzc/flutter_rust_bridge), [mpv](https://mpv.io/), and [wayland-rs](https://github.com/Smithay/wayland-rs).
