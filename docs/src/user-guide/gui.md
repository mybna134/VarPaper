# GUI

Start VarPaper with `varpaper` (or its desktop launcher).

## Add and apply wallpapers

1. Open **Folders** and add a local media or project directory. Steam manages
   downloads of Wallpaper Engine Workshop content; VarPaper reads installed files.
2. Open **Library**, search or filter by source and actual wallpaper type.
3. Double-click a wallpaper to apply it to all monitors. Click its details to
   apply to one named monitor, or unapply from one or all monitors.
4. Open **Monitors** to inspect assignments. Details show unavailable project
   reasons; application errors also surface failures during loading.

| Type | Playback and controls |
| --- | --- |
| Video | mpv media playback, with media decoding and playback settings. |
| Image | Static media displayed indefinitely. |
| GIF | Animated media; remains a separate library type. |
| Scene | Built-in Rust subset: layers, effects, animated textures, core sprite particles, text, sound and basic scripts. Requires OpenGL 3.3; uses stretch layout. |
| Web | Local HTML/CSS/JS and Canvas/WebGL, after installing the optional component. |

Project previews use the declared preview or a type fallback. Scene/Web assets
are not treated as videos for thumbnail extraction. Application executable
projects and unknown types are shown as unsupported and are never executed.
Unsupported required Scene features fail with an asset diagnostic. A failed
replacement preserves the previous wallpaper and saved assignment.

## Web wallpaper support

Open **Settings → Web wallpaper support** to inspect status, version and disk
usage, install with progress, cancel, retry or uninstall. Only explicit
installation downloads the pinned component. Scanning, application and startup
restoration never download it. Missing or damaged components leave projects
visible and provide a settings entry from details.

Web pages use project-local resources. External requests, downloads and arbitrary
host files are blocked. Pause closes the browser and retains its last frame;
resume rebuilds it, so page state may reset. Volume and mute cover HTML media and
Web Audio. Uninstall stops Web sessions before removing component/cache files;
projects and saved descriptions remain available for later installation.

## Properties, audio and time

The GUI uses declared initial project properties; its property editor is pending.
Assignments use only the current versioned source format; old path-only records
are not read or migrated and require reapplying wallpapers. Assignments can
preserve Web property overrides supplied through the
service API. Scene overrides are currently rejected. Advanced particles, full
SceneScript compatibility, live Scene system-audio visualization and local
clock/timezone bindings remain deferred.

Web audio callbacks receive system playback, through PulseAudio or the PipeWire
PulseAudio interface. No microphone fallback is used. Missing/disconnected
capture produces zeros while the service retries; rendering continues. Wallpaper
mute affects generated sound independently of captured system audio. Scene
animation freezes on pause; resume avoids accumulating a large animation step.
See [compatibility and fixtures](../dev/rust-scene.md).

## Settings and restoration

Settings controls appearance, playback, power saving, startup and tray behavior.
Hardware decoding, speed and HDR settings concern media. They do not add those
capabilities to Scene/Web; Scene currently supports stretch layout only.

Settings and per-monitor assignments are stored in Isar in the platform
application support directory. Legacy YAML is not loaded or migrated. Enable
**Launch at login** and **Restore last wallpaper** for automatic startup
restoration. **Minimize to tray** and **Start minimized** control window visibility
separately. Missing Web support or a failed restore keeps saved source/property
data so it can be retried after the underlying problem is repaired.

## System tray

Use the tray menu to show/hide the window, pause/resume playback, change the
wallpaper or quit. Closing the window keeps the application running when
**Minimize to tray** is enabled.

## Troubleshooting

- **No monitors:** use a Wayland session exposing `wlr-layer-shell`, or native
  X11 with RandR. X11 stacking depends on the desktop/window manager.
- **Scene fails:** inspect its asset diagnostic and check OpenGL 3.3 availability
  and the supported Rust feature table.
- **Web unavailable:** inspect component status in Settings and install or repair
  explicitly. The application bundle must also contain the Web host.
- **Missing project resources:** retain the full project directory and any
  required Wallpaper Engine shared assets; see [Workshop](../features/workshop.md).
