<div align="center">

<img src="packaging/varpaper.svg" alt="VarPaper logo" width="100" height="100">

# VarPaper

Video wallpaper manager for Linux Wayland and X11 desktops

[![License](https://img.shields.io/badge/license-GPL--3.0--only-blue?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Flutter](https://img.shields.io/badge/Flutter-3.47.4-02569B?style=for-the-badge&logo=flutter)](https://flutter.dev/)

[![Latest Release](https://img.shields.io/github/v/release/mybna134/VarPaper?style=for-the-badge&label=Latest%20Release&logo=github)](https://github.com/mybna134/VarPaper/releases/latest)
[![Build](https://img.shields.io/github/actions/workflow/status/mybna134/VarPaper/ci.yml?branch=main&style=for-the-badge&label=Build&logo=githubactions)](https://github.com/mybna134/VarPaper/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/mybna134/VarPaper/branch/main/graph/badge.svg)](https://codecov.io/gh/mybna134/VarPaper)

[![English](https://img.shields.io/badge/Language-English-0A66C2?style=for-the-badge)](README.md)
[![简体中文](https://img.shields.io/badge/Language-%E7%AE%80%E4%BD%93%E4%B8%AD%E6%96%87-0A66C2?style=for-the-badge)](README.zh-CN.md)

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

Linux x86_64 builds are packaged as `.deb`, `.flatpak`, and Arch `.pkg.tar.zst` files on the [Releases page](https://github.com/mybna134/VarPaper/releases).

The `.deb` is built on Ubuntu 24.04 and requires glibc 2.39 or newer, `libmpv.so.2`, and the other declared system libraries. On a compatible Debian-based distribution, download it and run:

```bash
sudo apt install ./varpaper_*_amd64.deb
varpaper
```

On Arch Linux, download the `.pkg.tar.zst` package and install it with pacman:

```bash
sudo pacman -U ./varpaper-*-x86_64.pkg.tar.zst
```

For Flatpak, download the `.flatpak` file and run:

```bash
flatpak install ./varpaper_*_x86_64.flatpak
flatpak run io.github.mybna134.varpaper
```

The Flatpak uses the GNOME 50 runtime and has access to your home directory for local videos and Steam Workshop files.

Niri filters the `wlr-layer-shell` protocol from sandboxed Flatpak apps for security, so the Flatpak cannot set wallpapers in a Niri session. Use a native VarPaper build on Niri; changing Flatpak socket permissions cannot expose a protocol filtered by the compositor.

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

## Code coverage

CI uploads Rust and Flutter test coverage to Codecov. The coverage budget sets an 80% target for changed lines, with a 1 percentage point tolerance. Overall coverage is compared with the base commit and may decrease by at most 1 percentage point. The `CODECOV_TOKEN` repository secret must be configured in GitHub Actions.

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
