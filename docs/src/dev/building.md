# Building

The Linux release build creates one Flutter bundle with the Rust playback engine and packages it as `.deb`, `.flatpak`, and `.pkg.tar.zst`.

## Requirements

- Flutter 3.47.4 and a stable Rust toolchain
- GTK 3, libmpv, Wayland, EGL, and Ayatana AppIndicator development packages
- `dpkg-deb`, `flatpak-builder`, and the GNOME 50 Flatpak runtime and SDK
- Docker for the Arch package build on non-Arch hosts, or `makepkg` on Arch Linux
- `zstd` for building the compressed Arch package
- Ubuntu 24.04 for release Flatpak builds

## Build all packages

```bash
./scripts/build-packages.sh
```

The script runs a clean Flutter Linux release build once, then writes the three packages to `dist/`. Every build uses the UTC date and the short commit ID of `main` as its version, for example `20260924.g1c192b04`. Fetch `main` before building from a checkout that does not have it locally. Use `./scripts/build-linux.sh` to build only the Flutter bundle with the same version.

To repackage an existing Flutter bundle during development:

```bash
VARPAPER_SKIP_BUILD=1 ./scripts/build-packages.sh
```

This mode still checks the Flatpak bundle's runtime library dependencies and the Arch package's shared-library dependencies. It fails when host libraries require a newer glibc than the GNOME 50 runtime provides or when the bundle has unresolved libraries in Arch.

## Checks

```bash
flutter analyze lib/main.dart lib/src/l10n.dart test/widget_test.dart
flutter test
dpkg-deb --info dist/*.deb
```

## Native Scene development

The in-development Scene component uses pinned repository snapshots from
linux-wallpaperengine. Its standalone build and first-frame fixtures need no
reference checkout. Install CMake 3.24+, a C++20 compiler, Python 3, pkg-config,
GLEW, GLFW3, SDL2, LZ4, FreeType, FFmpeg development libraries and libmpv.

```bash
python3 native/wallpaperengine/verify_sources.py
cmake -S native/wallpaperengine -B build/native-scene -DCMAKE_BUILD_TYPE=Release
cmake --build build/native-scene --target varpaper_scene_test -j4
xvfb-run -a ctest --test-dir build/native-scene --output-on-failure
xvfb-run -a env VARPAPER_TEST_X11=1 XDG_SESSION_TYPE=x11 SDL_AUDIODRIVER=dummy \
  VARPAPER_SCENE_LIBRARY="$PWD/build/native-scene/libvarpaper_scene.so" \
  cargo test -p wayvid-engine -- --test-threads=1
```

The Rust Scene owner retains its native library, project-scoped resource
callbacks and EGL surface. Creation, frames and destruction run on the render
thread with that surface current. Native exceptions return caller-owned error
messages. Read buffers transfer temporarily and are released exactly once.
Only versioned C ABI symbols are exported, keeping QuickJS symbols separate
from libmpv's JavaScript engine. See `native/wallpaperengine/scene.h` and the
component README for callback and cleanup contracts.

These fixtures currently cover the minimal Scene clear-color frame, Image to
Scene replacement, invalid candidate rollback, native failure cleanup and
thread ownership. Full SceneScript/audio visualization, Web, packaging and
desktop qualification remain tracked in `add-all-wallpaper-types`; scanning
still reports Scene/Web as requiring their completed renderer integration.
