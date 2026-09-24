<div align="center">

<img src="packaging/varpaper.svg" alt="VarPaper logo" width="100" height="100">

# VarPaper

Video wallpaper manager for Linux Wayland and X11 desktops

[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

[Project](https://github.com/mybna134/VarPaper) • [Releases](https://github.com/mybna134/VarPaper/releases)

</div>

VarPaper is a Flutter desktop app with a Rust and mpv playback engine. It finds video files in folders you add and in locally installed Wallpaper Engine Workshop content. Choose a wallpaper in the library, then apply it to one monitor or all monitors.

## Features

- Browse local videos with previews, search, and source filters.
- Apply different wallpapers to different monitors, or apply one to every monitor.
- Find locally installed Wallpaper Engine Workshop video wallpapers. Workshop downloads are managed through Steam; VarPaper does not download them.
- Restore per-monitor wallpaper assignments on startup.
- Pause, resume, or change wallpapers from the system tray.
- Configure playback, appearance, autostart, and battery pause in the app.

Playback uses mpv with automatic hardware decoding when available. On Wayland, the compositor must support `wlr-layer-shell`; Wayland sessions use layer-shell even when XWayland is present. Native X11 sessions use RandR for monitor discovery and desktop-type wallpaper windows; their stacking behavior depends on the window manager and desktop icon manager.

## Install

Linux x86_64 builds are packaged as `.deb` and `.flatpak` files on the [Releases page](https://github.com/mybna134/VarPaper/releases).

For Debian or Ubuntu, download the `.deb` file and run:

```bash
sudo apt install ./varpaper_*_amd64.deb
varpaper
```

For Flatpak, download the `.flatpak` file and run:

```bash
flatpak install ./varpaper_*_x86_64.flatpak
flatpak run io.github.mybna134.varpaper
```

The Flatpak uses the GNOME 50 runtime and has access to your home directory for local videos and Steam Workshop files.

## Use

Add a video folder on the **Folders** page. Browse or search on **Library**, then double-click a wallpaper to apply it to all monitors. The wallpaper details let you apply it to a specific monitor. Use **Monitors** to inspect current assignments and **Settings** to control startup, playback, and power options.

The app saves settings and monitor assignments in an Isar database in the platform application support directory. Existing YAML settings are not migrated. Enable **Launch at login** and **Restore last wallpaper** in Settings if you want wallpapers restored automatically after logging in. **Minimize to tray** and **Start minimized** control the window separately.

Wallpaper Engine Workshop content must already be installed locally through Steam. VarPaper discovers those folders and plays video projects; web and scene projects are not supported for playback.

## Build from source

You need Flutter, Rust, Linux development libraries for GTK, mpv, Wayland, EGL, X11, RandR, XFixes, and the app indicator. Package builds also need `dpkg-deb`, `flatpak-builder`, and the GNOME 50 Flatpak runtime and SDK.

```bash
git clone https://github.com/mybna134/VarPaper.git
cd VarPaper
./scripts/build-packages.sh
```

The script writes both packages to `dist/`. For a local Linux bundle, run `./scripts/build-linux.sh`. For development, run `flutter run -d linux` and `cargo test --workspace`.

## Project structure

```text
lib/                    Flutter app and settings
linux/                  Flutter Linux runner and Rust build integration
rust/                   Flutter/Rust bridge service
crates/wayvid-engine/    Wayland/X11 wallpaper playback
crates/wayvid-library/   Folder and Workshop scanning, previews
packaging/              Debian and Flatpak recipes
scripts/                Build and development commands
```

Flutter owns the UI, settings, tray, and application lifecycle. The Rust service scans wallpapers and runs the playback engine in the same application process; there is no separate daemon to start.

## License

VarPaper is licensed under [GNU GPL v3 only](LICENSE). See [NOTICE](NOTICE) for upstream attribution. The retained [Apache-2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT) texts apply to upstream material and do not replace VarPaper's GPL license.
