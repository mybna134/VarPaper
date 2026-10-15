# VarPaper

Animated wallpaper manager for Linux Wayland and native X11 desktops, with a
Flutter interface and Rust rendering engine.

Browse Video, Scene, Web, Image and GIF wallpapers from local folders and locally
installed Wallpaper Engine Workshop projects. Apply a wallpaper to one or all
monitors, control playback from the tray and restore saved assignments on startup.
Media uses mpv. Scene uses the implemented Rust subset and requires OpenGL 3.3;
unsupported required features produce diagnostics. Web uses an optional component
installed explicitly from Settings; startup and scanning never download it.

Advanced Scene particles, full SceneScript compatibility, live Scene system-audio
visualization and local clock bindings remain deferred. Web supports project
properties and system-playback audio callbacks. See the
[compatibility table](features/workshop.md) for limits.

## Start

Install a package using the [installation guide](user-guide/installation.md), then
run `varpaper`. Add a directory on **Folders**, browse **Library**, and double-click
to apply to all monitors or open details to select one monitor.

Wayland requires a compositor exposing `wlr-layer-shell`; native X11 requires
RandR. Desktop stacking depends on the compositor/window manager. See
[multi-monitor use](user-guide/multi-monitor.md).

## Guides

- [GUI and settings](user-guide/gui.md)
- [Configuration and persistence](user-guide/configuration.md)
- [Project discovery and compatibility](features/workshop.md)
- [Rust Scene support and fixtures](dev/rust-scene.md)
- [Building and native checks](dev/building.md)

## License

GNU GPL v3 only. See the repository NOTICE for upstream attribution.
