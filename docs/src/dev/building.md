# Building

The Linux release build creates one Flutter bundle with the Rust playback engine and packages it as `.deb`, `.flatpak`, and `.pkg.tar.zst`.

## Requirements

- Flutter 3.47.4 and Rust 1.88 or newer
- GTK 3, libmpv, Wayland, EGL, Fontconfig, SDL2, and Ayatana AppIndicator development packages
- CMake 3.24 or newer, a C++20 compiler, Python 3 and curl for the pinned Web helper build
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

## Debian package

To build only the `.deb` from an existing release bundle:

```bash
packaging/deb/build.sh 20261015.g12345678
```

`VARPAPER_BUNDLE_DIR` selects the input bundle and `VARPAPER_OUTPUT_DIR` selects
its output directory. The builder checks both the staged and extracted payloads
with `scripts/check-linux-bundle.py`. The package installs the Web host (also
its subprocess executable) under `/usr/lib/varpaper/libexec`, its pinned
`data/runtime.json` and licenses, and the Rust engine containing Scene support.
It excludes the unused JNI library, which would otherwise require a JVM.

System dependencies include the media/graphics libraries, DejaVu fallback font,
PulseAudio client, OpenSSL and the shared system libraries needed by the optional
Web component. No CEF package is required and no Chromium runtime, resources or
locales are bundled. Flutter's own `data/icudtl.dat` remains required. The builder
derives the minimum glibc version from the packaged ELF files; build on the
oldest supported Debian/Ubuntu baseline for releases. Repackaging an Arch build
does not establish compatibility with older Debian/Ubuntu versions.

## Flatpak package

`packaging/flatpak/build.sh VERSION` requires GNOME Platform and SDK 50. All
input ELF files must fit the runtime's glibc ABI; the builder rejects newer
binaries before compiling its modules. Build Rust against a compatible distro
or the GNOME SDK rather than repackaging a newer host binary.

The manifest builds libass/libplacebo/mpv against the GNOME runtime.
VarPaper owns Wayland/X11 surfaces and supplies mpv with an EGL render context;
mpv does not create its own X11 windows. GNOME supplies SDL2, PulseAudio, OpenSSL, graphics
libraries and the DejaVu font used by Rust Scene. The builder validates the
complete application bundle and supplemental libraries inside the SDK sandbox.
`--library-dir /app/lib` tells the shared bundle checker where Flatpak's
additional libraries live.

The base package includes Zypak v2025.09 to start CEF children through the
Flatpak portal. Rust wraps only the Web helper, sets its component library path,
and leaves Chromium sandboxing enabled. A local Zypak patch exposes the installed
component directory read-only to sandboxed children and supplies their library
search path. CEF binaries/resources/locales are
excluded from the base package; the normal component installer uses the
Flatpak application's private `XDG_DATA_HOME`. Network permission permits the
explicit component download; Web page requests remain blocked by the host's
request policy. Builder downloads and module caches are retained under `build/flatpak-state`.
Browser execution checks must use an installed Flatpak, since
Zypak's portal integration does not work from `flatpak-builder --run`.

With the package installed, an existing complete pinned runtime in a temporary
subdirectory of the application's private data, and Xvfb available, run the
native Web fixture in the installed GNOME runtime:

```bash
# Set this to the temporary copy, not the live component used by another session.
runtime="$HOME/.var/app/io.github.mybna134.varpaper/data/web-fixture/runtime"
xvfb-run -a flatpak run --user --unshare=network --socket=x11 \
  --env=LIBGL_ALWAYS_SOFTWARE=1 --env=SDL_AUDIODRIVER=dummy \
  --env=VARPAPER_TEST_CEF_SOFTWARE=1 \
  --env=VK_ICD_FILENAMES="$runtime/vk_swiftshader_icd.json" \
  --command=python3 io.github.mybna134.varpaper \
  "$PWD/native/web/test_host.py" \
  --host /app/lib/varpaper/libexec/varpaper-web-host --runtime "$runtime"
```

This uses an existing runtime copy and does not exercise the Settings installer.
The fixture checks page/resource rendering, controls, renderer failure and
process cleanup, including children launched through the Flatpak portal.

## Arch package

Build from a release bundle in an isolated rolling Arch container:

```bash
VARPAPER_ARCH_USE_DOCKER=1 packaging/arch/build.sh 20261015.g12345678
```

`VARPAPER_BUNDLE_DIR` and `VARPAPER_OUTPUT_DIR` select the input and output.
The Docker build updates Arch and installs the recipe's runtime/build dependency
lists. On Arch, omitting the Docker override runs `makepkg --syncdeps` locally,
which may install dependencies on the host. Package downloads are cached under
`build/arch-pacman-cache`.

The recipe includes OpenSSL, CA certificates, PulseAudio, graphics/media libraries,
DejaVu fonts and the system libraries used by the optional CEF runtime (NSS/NSPR,
ALSA, CUPS, Mesa and X11). CEF itself is neither a pacman dependency nor a package
file; users install the pinned component through Settings. The helper, pinned
manifest and Scene/CEF/JSON notices are installed under `/usr/lib/varpaper`.
The application icon is installed both there and in the system icon theme.

Python is a build dependency for the shared bundle checker. It checks required
artifacts, manifest identity, forbidden CEF/old Scene files and all ELF links in
makepkg's staging tree and again after archive extraction inside Arch. The outer
builder also checks the extracted archive on the build host. Only the helper's
missing `libcef.so` is permitted; inherited `LD_LIBRARY_PATH` cannot hide missing
base dependencies. `--manifest` lets this checker run in the container against
the pinned source manifest without mounting the source checkout.

Arch packages target the current rolling system. A successful package build does
not establish compatibility with older Arch snapshots or Debian/Flatpak ABIs.
Use the package Scene fixture below and `native/web/test_host.py` with an existing
pinned runtime for execution checks in private Xvfb sessions.

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

`flutter build linux`, `flutter run -d linux` and `scripts/build-linux.sh` use the
same CMake rules to compile the helper on Linux x86_64 using the pinned SDK
(set `CEF_ROOT` to reuse an extracted SDK). `native/web/bundle.cmake` lists the
helper, runtime manifest and licenses installed in the base bundle; Chromium
libraries, resources and locales are excluded. The helper is built under the
Flutter build directory, while a standalone `scripts/build-web-host.sh` build
uses `build/native-web` (override with `VARPAPER_WEB_BUILD_DIR`).

To use a prebuilt helper, set `VARPAPER_WEB_HOST_BINARY` to its absolute path
before a clean Flutter build. Its adjacent `runtime.json` must match the pinned
SDK/IPC manifest; missing or mismatched manifests fail configuration. Other
architectures retain media/Scene support without a Web helper. Build-time SDK
downloads do not install the user's runtime component.

`scripts/build-linux.sh` checks the resulting bundle's artifact inventory and
native links. To check a direct Flutter build and also verify the optional
helper against an already installed, flattened CEF runtime:

```bash
python3 scripts/check-linux-bundle.py build/linux/x64/release/bundle
python3 scripts/check-linux-bundle.py build/linux/x64/release/bundle \
  --web-runtime /path/to/runtime
```

The main application must resolve its libraries without CEF. Without
`--web-runtime`, only the helper's intentional missing `libcef.so` is allowed;
with it, all helper dependencies must also resolve. The check rejects old Scene
libraries, embedded reference checkout paths, bundled CEF runtime files and
helper RPATHs. It does not launch the GUI or install a component.

Scene is built as part of Cargo and linked into the application. Web still
requires CMake, a C++ compiler and SDL2; its pinned JSON dependency and MIT
notice are in `native/web`. `scripts/check-native.sh` runs the Rust Scene
parser/GL/script fixtures, private PulseAudio sound fixture, CEF host and native
workspace lifecycle fixtures in private Xvfb sessions. CI installs PulseAudio
and its `pactl` utility; the fixture starts a minimal private null-sink server
without loading hardware devices. Other fixtures use an absent private Pulse
socket and temporary data/config/state directories. For the existing isolated
PipeWire fixture, set `VARPAPER_TEST_AUDIO_BACKEND=pipewire` (WirePlumber 0.5+
required). Native gate details are in [scripts](../../../scripts/README.md). Build packages
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

Run `flutter pub get` before these checks; the runner uses `test --no-pub`.
Set `VARPAPER_TEST_FLUTTER` to a Flutter executable when it is not on PATH.
Debian extraction needs dpkg-deb, Arch extraction needs tar/zstd, and Flatpak
extraction needs flatpak/ostree. This does not install or activate a package;
the smoke test loads its library against the host libraries. Flatpak's separate
build-time link check runs inside the GNOME runtime. These checks supplement
full installation and desktop qualification.

### Explicit component Settings acceptance on extracted packages

After the offline Scene package check, opt in to the real installer and Settings
buttons using the same package runner:

```bash
VARPAPER_TEST_COMPONENT_INSTALL=1 scripts/check-scene-package.sh /path/to/package
```

This fixture loads the package's actual Rust library and Web helper, uses the
real Settings widget/controller (only settings storage is in memory), and
checks missing-component discovery/application, media and Scene without CEF,
explicit official download, five-type application, typed initial/updated Web
properties and 128-band audio callbacks checked through X11 pixels, active Web uninstall,
zero component disk usage and preserved project files. It runs in an owned
Xvfb display with private data/cache, no desktop playback monitor and dummy
SDL output. This opt-in downloads the pinned CEF archive; normal package/CI
checks never enable it. The runner deletes its temporary files after shutdown.

An extracted Flatpak payload runs against host libraries with this command;
it does not establish Flatpak sandbox/portal acceptance. Likewise an extracted
Debian payload does not establish Debian package installation or ABI support.
Record installed-distribution and sandbox checks separately before marking
Task 8.3 complete.


### Installed package acceptance

Install the package with the target distribution's package manager, then run
against that installed bundle (not an extracted payload):

```bash
VARPAPER_TEST_COMPONENT_INSTALL=1 scripts/check-scene-package.sh --installed /usr/lib/varpaper
```

`VARPAPER_TEST_PACKAGE_DEFAULT_ONLY=1` stops after the missing-component and
four-type checks. Disconnect the test container's network, or use Flatpak's
`--unshare=network`, for this phase; the flag itself does not isolate networking.
No CEF files may be present in the base package, and the default phase must not
create a component installation. The delivered Scene subset is exercised; live
Scene system audio and injected local clock bindings remain unavailable.

To test nonzero Web spectrum callbacks, run the same check through the private
null-sink fixture. It never discovers hardware or connects to desktop audio:

```bash
VARPAPER_TEST_COMPONENT_INSTALL=1 python3 scripts/fixtures/scene_audio_monitor.py \
  --backend pulseaudio --command scripts/check-scene-package.sh --installed /usr/lib/varpaper
```

Without this wrapper, package checks use a nonexistent private Pulse socket and
assert zero spectrum callbacks. Both cases also check property updates, pause
and resume, active Web uninstall, zero disk usage and preserved user projects.

For Flatpak, run in the installed app's Platform sandbox with the bundle at
`/app/lib/varpaper`. Supply the Flutter test SDK, Git and readelf as test tools;
they are not application dependencies. An outer owned Xvfb can pass
`VARPAPER_TEST_PACKAGE_DISPLAY=1` to reuse that private display. Set `TMPDIR` to
a test directory under the app's persistent cache: portal children cannot reuse
files from the parent sandbox's private `/tmp`. For available audio, expose only
the private fixture socket directory, preserve its `PULSE_SERVER` and disable
the normal desktop Pulse socket. Keep installation, sandbox and audio results
separate from host extraction results in the change's validation record.
