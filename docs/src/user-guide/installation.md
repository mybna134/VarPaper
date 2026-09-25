# Installation

VarPaper publishes Linux packages for x86_64 in two formats: `.deb` and `.flatpak`.

## Debian and Ubuntu

The `.deb` is built on Ubuntu 24.04 and requires glibc 2.39 or newer, `libmpv.so.2`, and the other declared system libraries. On a compatible Debian-based distribution, download the `.deb` from the [latest release](https://github.com/mybna134/VarPaper/releases/latest), then install it:

```bash
sudo apt install ./varpaper_*_amd64.deb
varpaper
```

The package installs the Flutter application bundle, desktop entry, icon, and license notices. System libraries such as GTK and libmpv are installed through the package manager.

## Flatpak

Install the GNOME 50 runtime from Flathub, download the `.flatpak` from the [latest release](https://github.com/mybna134/lwe-flutter/releases/latest), then run:

```bash
flatpak install ./varpaper_*_x86_64.flatpak
flatpak run io.github.mybna134.varpaper
```

The Flatpak has access to the home directory so it can find local wallpapers and Steam Workshop content.

## Build packages from source

Install Flutter, Rust, Linux development libraries, `dpkg-deb`, `flatpak-builder`, and the GNOME 50 Flatpak runtime and SDK. Then run:

```bash
git clone https://github.com/mybna134/lwe-flutter.git
cd lwe-flutter
./scripts/build-packages.sh
```

Both packages are written to `dist/`. Build on Ubuntu 24.04 for a Flatpak whose media libraries are compatible with the GNOME 50 runtime.

## Uninstall

```bash
sudo apt remove varpaper
# or
flatpak uninstall io.github.mybna134.varpaper
```
