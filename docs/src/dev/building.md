# Building

The Linux release build creates one Flutter bundle with the Rust playback engine and packages it as `.deb` and `.flatpak`.

## Requirements

- Flutter 3.47.4 and a stable Rust toolchain
- GTK 3, libmpv, Wayland, EGL, and Ayatana AppIndicator development packages
- `dpkg-deb`, `flatpak-builder`, and the GNOME 50 Flatpak runtime and SDK
- Ubuntu 24.04 for release Flatpak builds

## Build both packages

```bash
./scripts/build-packages.sh
```

The script runs a clean Flutter Linux release build once, then writes the two packages to `dist/`. Every build uses the UTC date and the short commit ID of `main` as its version, for example `20260924.g1c192b04`. Fetch `main` before building from a checkout that does not have it locally. Use `./scripts/build-linux.sh` to build only the Flutter bundle with the same version.

To repackage an existing Flutter bundle during development:

```bash
VARPAPER_SKIP_BUILD=1 ./scripts/build-packages.sh
```

This mode still checks the Flatpak bundle's runtime library dependencies. It fails when host libraries require a newer glibc than the GNOME 50 runtime provides.

## Checks

```bash
flutter analyze lib/main.dart lib/src/l10n.dart test/widget_test.dart
flutter test
dpkg-deb --info dist/*.deb
```
