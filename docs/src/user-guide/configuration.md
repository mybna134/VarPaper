# Configuration

Wayvid settings are managed in the Flutter Settings page and persisted by
Isar Community. The application does not read legacy YAML configuration and
does not migrate settings from older versions.

The persisted settings include:

- window appearance and sidebar state;
- playback volume, loop mode, layout, hardware decoding, speed and HDR;
- power-saving behavior;
- library folders;
- per-output wallpaper assignments;
- autostart and tray preferences.

When the engine is running, playback-related changes are sent immediately to
the in-process Rust engine through flutter_rust_bridge.
