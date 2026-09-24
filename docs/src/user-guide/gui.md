# GUI

wayvid v0.5 introduces a GUI-first experience. Just run:

```bash
wayvid-gui
```

## Interface

The GUI has a sidebar navigation with four main views:

```
┌─────────────────────────────────────────────────┐
│  📁 Library    │                                │
│  📂 Folders    │   ┌─────┐ ┌─────┐ ┌─────┐    │
│  ⚙️ Settings   │   │ 🎬  │ │ 🎬  │ │ 🎬  │    │
│  ℹ️ About      │   │ vid │ │ vid │ │ vid │    │
│                │   └─────┘ └─────┘ └─────┘    │
│                │                                │
│                │   🔍 Search...                 │
├────────────────┴────────────────────────────────┤
│  Monitors: [DP-1 ✓] [HDMI-A-1] [eDP-1]         │
└─────────────────────────────────────────────────┘
```

## Views

### Library
Browse your wallpaper collection with thumbnails. Click to preview, double-click to apply.

### Folders
Manage source folders for wallpapers. Add/remove folders to scan.

### Settings
- **Autostart**: Launch wayvid-gui at login
- **Minimize to tray**: Keep running in background when window is closed
- **Start minimized**: Start directly to tray without showing window
- **Power management**: Pause on battery or fullscreen apps
- **Performance**: FPS limits, hardware decode options

### About
Version info and links.

## Usage

1. **Browse** - View wallpapers in Library tab
2. **Select monitor** - Click monitor in bottom bar
3. **Apply** - Double-click a wallpaper

## Wallpaper Persistence

wayvid automatically saves your wallpaper settings and restores them on restart:

- **Per-monitor wallpapers**: Each monitor remembers its wallpaper
- **Engine state**: Engine auto-starts if it was running before
- **Settings location**: Flutter application support directory (Isar Community)

To enable full persistence:
1. Enable "Start with system" in Settings
2. Enable "Minimize to tray"
3. Enable "Start minimized"

Now wayvid will start automatically on login, restore your wallpapers, and run quietly in the tray.

## System Tray

The GUI minimizes to system tray when closed. Right-click tray icon for:
- Show/hide window
- Pause/resume playback
- Quit

## Autostart

Enable in Settings → Autostart, or manually:

```kdl
# niri: ~/.config/niri/config.kdl
spawn-at-startup "wayvid-gui"
```

```conf
# hyprland: ~/.config/hypr/hyprland.conf
exec-once = wayvid-gui
```

> **Note:** Use the "Start minimized" option in Settings to control whether wayvid opens in tray or shows the window on startup.

## Troubleshooting

**No monitors shown:**
- Check that a Wayland or X11 desktop session is running
- On Wayland, ensure the compositor supports wlr-layer-shell
- On X11, ensure RandR is available

**Fonts broken (Chinese):**
```bash
sudo pacman -S noto-fonts-cjk  # Arch
```

**Thumbnails not loading:**
- Check the application cache directory permissions
- Ensure ffmpeg is installed for video thumbnails
