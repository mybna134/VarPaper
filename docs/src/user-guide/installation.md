# Installation

VarPaper publishes Linux packages for x86_64 in three formats: `.deb`, `.flatpak` and Arch `.pkg.tar.zst`.

## Debian and Ubuntu

The `.deb` is built on Ubuntu 24.04 and requires glibc 2.39 or newer, `libmpv.so.2`, and the other declared system libraries. On a compatible Debian-based distribution, download the `.deb` from the [latest release](https://github.com/mybna134/VarPaper/releases/latest), then install it:

```bash
sudo apt install ./varpaper_*_amd64.deb
varpaper
```

The package installs the Flutter application bundle, desktop entry, icon, and license notices. System libraries such as GTK and libmpv are installed through the package manager.

## Arch Linux

Download the `.pkg.tar.zst` file from the [latest release](https://github.com/mybna134/VarPaper/releases/latest), then install it with pacman:

```bash
sudo pacman -U ./varpaper-*-x86_64.pkg.tar.zst
```

The package declares the Arch runtime libraries it needs. Pacman installs them together with VarPaper.

## Flatpak

Install the GNOME 50 runtime from Flathub, download the `.flatpak` from the [latest release](https://github.com/mybna134/VarPaper/releases/latest), then run:

```bash
flatpak install ./varpaper_*_x86_64.flatpak
flatpak run io.github.mybna134.varpaper
```

The Flatpak has access to the home directory so it can find local wallpapers and Steam Workshop content.

Niri filters the `wlr-layer-shell` protocol from sandboxed Flatpak apps for security. The Flatpak therefore cannot set wallpapers in a Niri session. Use a native VarPaper build on Niri; Flatpak socket overrides cannot restore a protocol filtered by the compositor.

## Wallpaper renderers

Video, Image and GIF use mpv. The built-in Rust Scene renderer requires OpenGL
3.3 and supports the [documented subset](../dev/rust-scene.md). The base install
excludes Chromium/CEF runtime files. For Web projects, explicitly install
**Web wallpaper support** in Settings; startup, scanning, applying and restoring
never download it automatically. The current component manifest targets Linux
x86_64. See the [GUI guide](gui.md#web-wallpaper-support) for install/uninstall,
local-resource restrictions and pause behavior.

## Build packages from source

Install Flutter, Rust, Linux development libraries, `dpkg-deb`, `flatpak-builder`, and the GNOME 50 Flatpak runtime and SDK. Then run:

```bash
git clone https://github.com/mybna134/VarPaper.git
cd VarPaper
./scripts/build-packages.sh
```

All three packages are written to `dist/`. Build on Ubuntu 24.04 for a Flatpak whose media libraries are compatible with the GNOME 50 runtime.

## Uninstall

```bash
sudo apt remove varpaper
# or
flatpak uninstall io.github.mybna134.varpaper
```
