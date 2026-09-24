# Multi-Monitor

## Using GUI (Recommended)

1. Open `wayvid-gui`
2. Select monitor from the bottom bar
3. Browse wallpapers in Library tab
4. Double-click to apply to selected monitor

## List Outputs

```bash
Open the Monitors view in the Flutter application
# eDP-1: 1920x1080 @ (0, 0)
# DP-1: 2560x1440 @ (1920, 0)
```

## Different Video Per Monitor

### Via CLI

```bash
Select each output in the Flutter application and apply the desired wallpaper
```

### Via Config File

```yaml
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

## Same Video All Monitors

```bash
Use Apply in Library to apply to all outputs
```

Or via config:
```yaml
source:
  type: file
  path: ~/Videos/wallpaper.mp4
```

## Control Per Output

```bash
Use Pause, Resume, Clear, and Apply in the Flutter application
```

## Hotplug

Monitors are detected automatically. No restart needed.
