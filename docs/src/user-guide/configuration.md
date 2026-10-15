# Configuration

VarPaper settings are managed in the Flutter Settings page and persisted by
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


## Wallpaper assignments

New assignments use the current versioned source description: source identity,
path, actual type, project context and supplied Web property overrides. Only a
successfully initialized candidate is saved. A failed replacement or restore
preserves its saved assignment. Old path-only assignment records are not read or
migrated; reapply wallpapers to create new assignments.

The GUI uses declared initial project properties and has no property editor yet.
Web overrides can be supplied through the service API; Scene overrides remain
unsupported. Web support installation is explicit in Settings and never triggered
by saved assignments. Uninstall preserves projects and saved descriptions.

Media playback options such as hardware decoding and HDR do not configure the
Scene/Web renderer. Scene currently uses stretch layout. For audio monitoring,
animation clocks and the deferred Scene features, see the
[GUI compatibility notes](gui.md#properties-audio-and-time).
