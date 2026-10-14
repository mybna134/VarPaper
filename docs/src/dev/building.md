# Building

The Linux release build creates one Flutter bundle with the Rust playback engine and packages it as `.deb`, `.flatpak`, and `.pkg.tar.zst`.

## Requirements

- Flutter 3.47.4 and Rust 1.88 or newer
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

## Rust Scene migration development

Scene business logic runs directly in `crates/wayvid-engine/src/scene/` under
`migrate-scene-renderer-to-rust`. `model` parses bounded JSON, `resources`
reuses the library's project-scoped loose/package/shared resource access,
`texture` decodes bounded TEX payloads, and `runtime`/`gpu` own the render
context, surface, texture and framebuffer. The renderer holds its EGL surface
until GPU cleanup finishes; creation, resize, frames and release run with that
surface current on its owning thread. Failed replacement allocations leave the
previous target usable. These Rust fixtures do not load a Scene dynamic library.
The application factory still uses the previous adapter while the remaining
Rust object, effect and script functionality is being implemented.

With the system dependencies listed above and Xvfb/xauth installed:

```bash
cargo +1.88.0 check --workspace --all-targets --locked --offline
cargo +1.88.0 test -p wayvid-engine scene::texture::tests --locked --offline
env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 VARPAPER_TEST_X11=1 \
  xvfb-run -a cargo test -p wayvid-engine \
  scene::runtime::tests::rust_scene_first_frame_state_and_failed_allocation_cleanup \
  -- --exact --nocapture
env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 VARPAPER_TEST_X11=1 \
  VARPAPER_EXPECT_SCENE_GL_UNAVAILABLE=1 MESA_GL_VERSION_OVERRIDE=3.0 \
  xvfb-run -a cargo test -p wayvid-engine \
  scene::runtime::tests::rust_scene_first_frame_state_and_failed_allocation_cleanup \
  -- --exact --nocapture
```

`--offline` requires that locked dependencies and the 1.88 toolchain were
previously downloaded. The private Xvfb test checks clear-color pixels, GL state
restoration, caller unpack buffers, allocation failure and twenty create/release
cycles against GPU/EGL baselines. The second GL command verifies rejection
when OpenGL 3.3 is unavailable. The fixtures generate assets without Workshop
files. Run `scripts/check-rust-scene.sh` for the generated resource, texture, shader
and actual GL checks; see [Rust Scene checks](rust-scene.md) and the migration's
`validation.md` for executed checks; these results
do not yet establish full Rust Scene compatibility.

## Rust Scene application integration

The renderer factory directly creates `RustSceneRenderer` in the engine.
Scene has no separate library, C ABI, CMake target or library-path setting.
Use `cargo check --workspace --all-targets --locked` to compile it and
`scripts/check-rust-scene.sh` to run the generated fixture suite.
See [Rust Scene checks](rust-scene.md) for the supported subset and deferred
features. Unsupported required content fails with an asset diagnostic.

## Optional Web runtime development

Settings contains **Web wallpaper support** with explicit install, progress,
cancel, retry and uninstall actions. Startup, scanning and applying a project
never download Chromium. Missing components leave Web projects visible and
link their disabled apply action to Settings. Installation is currently pinned
to the official CEF Linux x86_64 minimal distribution; other architectures show
an unavailable state. The base application does not bundle this runtime.

The component manager stores libraries, resources, locales and the upstream
license/credits under the application's local data directory at
`varpaper/components/web/runtime`. It verifies the fixed archive SHA-256,
rejects archive links and unsafe paths, limits sizes and commits installation
after validation. Restart validates the installation record and file hashes.
Uninstall removes the owned component directory and temporary files, preserving
wallpaper projects and saved assignments. Web browser playback is now connected through the shared host, provided the
bundle contains `libexec/varpaper-web-host`. Local HTML/CSS/JS, Canvas/WebGL,
resize, pause/resume, typed project callbacks and audio callbacks have passed the
native fixtures. The GUI property editor/persistence and full Scene audio/script
compatibility remain pending; see
[native Web component notes](../../../native/web/README.md).

`scripts/build-linux.sh` compiles the helper using the pinned SDK (set `CEF_ROOT`
to reuse an extracted SDK). The base bundle includes its manifest and wrapper
license but excludes the optional CEF runtime. A direct Flutter build can select
a prebuilt helper with `VARPAPER_WEB_HOST_BINARY`. Build-time SDK downloads do
not install the user's runtime component.

Scene is built as part of Cargo and linked into the application. Web still
requires CMake, a C++ compiler and SDL2; its pinned JSON dependency and MIT
notice are in `native/web`. `scripts/check-native.sh` runs the Rust Scene
fixtures and the optional CEF fixtures in private Xvfb sessions. Build packages
from a fresh Flutter output directory so historical Scene libraries cannot be
carried into the bundle.
