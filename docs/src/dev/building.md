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
The application factory constructs the Rust renderer directly. The supported
object, effect and script subset is listed in [Rust Scene checks](rust-scene.md).

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

### Described assignments and project properties

The FRB service exposes `describeWallpaper(path)` and
`applyWallpaperSource(source, output)`. `WallpaperSourceDto` version 1 retains
the stable source ID, actual wallpaper type, source path and project context.
Rust re-reads the project before applying it and rejects a changed identity,
type or context. The path-based `applyWallpaper(path, output)` can resolve
media files, project directories, manifests and project-entry paths. Assignment
persistence uses only current source descriptions in the
`WallpaperSourceAssignment` collection; old path-only assignments are neither
read nor migrated. Reapply wallpapers to save new assignments.

`propertyOverridesJson` maps declared Web property names to typed values,
for example `{"caption":{"value":"Hello"}}`. Initial overrides reach the
candidate before its first frame. `updateWallpaperProperties(output,
sourcePath, propertiesJson)` updates an existing Web assignment only when its
source still matches and no replacement is pending. Validation checks declared
names, types, slider bounds, colors, choices and a 64 KiB update limit before
sending changes to the browser. Scene property overrides remain deferred;
Scene still receives the project's declared initial properties.

`ServiceEvent_WallpaperFailed` carries the output, path, code and full cause.
The existing generic error event is also emitted for older consumers. Flutter
continues to own persistence; the new API does not itself save assignments or
provide a property editor.

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
native fixtures. Versioned source descriptions and Web property overrides are persisted with
confirmed assignments. The GUI property editor and full Scene audio/script
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

The mixed X11 fixture runs 100 Media/Scene/Web replacement cycles, rejects an
invalid Scene candidate after each assignment, and checks that EGL, Scene,
audio and shared Web host ownership returns to its baseline after every cycle.
This isolated check does not cover compositor hotplug or installed packages.

For the real Wayland layer-shell/EGL path, install niri, Xvfb and xauth, then run:

```bash
scripts/check-wayland.sh
```

This starts niri inside a private Xvfb display with its own runtime directory.
It checks output discovery, 100 media apply/clear cycles across 20 engine
restarts, and Scene/media replacement, invalid Scene retention, pause/resume
and cleanup across three further engine restarts. It does not connect to the
desktop compositor. Locked Cargo dependencies must already be cached.
Nested software rendering does not replace physical output hotplug acceptance.


### Flatpak build host compatibility

The Rust bridge in the input Linux bundle must target a glibc version supported
by GNOME 50. The packaging script checks this before downloading media sources.
Use the CI Ubuntu 24.04 build baseline or compile against the GNOME SDK; a
bundle linked on a newer rolling distribution can require newer glibc symbols
and cannot be fixed by copying host libc into the Flatpak.


### Scene checks on package contents

After building each package, load its extracted Rust bridge and exercise Scene
application, named asset errors, pause/resume and clear in a private Xvfb session:

```bash
scripts/check-scene-package.sh dist/varpaper_VERSION_amd64.deb
scripts/check-scene-package.sh dist/varpaper-VERSION-1-x86_64.pkg.tar.zst
scripts/check-scene-package.sh dist/varpaper_VERSION_x86_64.flatpak
```

Set `VARPAPER_TEST_FLUTTER` to a Flutter executable when it is not on PATH.
Debian extraction needs dpkg-deb, Arch extraction needs tar/zstd, and Flatpak
extraction needs flatpak/ostree. This does not install or activate a package;
the smoke test loads its library against the host libraries. Flatpak's separate
build-time link check runs inside the GNOME runtime. These checks supplement
full installation and desktop qualification.
