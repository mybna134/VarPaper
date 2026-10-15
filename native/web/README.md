# Optional Chromium wallpaper host

`varpaper-web-host` owns CEF on its main thread and never creates desktop
windows or OpenGL contexts. Rust shares one host on the engine thread and gives
each output a separate browser and in-memory request context. The same helper
executable handles CEF subprocesses. Its CEF DLL wrapper is built from the
pinned official SDK; the upstream BSD notice is retained in `CEF-LICENSE`.

`runtime.json` pins the SDK/runtime version, architecture, URL, SHA-256 and IPC
version (currently IPC 2 for properties and audio). The application checks this
manifest and executable permissions before declaring Web ready or launching a
candidate, and checks the actual IPC version on every response. A mismatched
helper requires reinstalling the application; the pinned CEF runtime is unchanged.
Build with `scripts/build-web-host.sh`; `CEF_ROOT` can select an already
extracted matching SDK. SDK downloads are build-time operations. Application
startup never invokes this script. Flutter's Linux CMake build and
`scripts/build-linux.sh` build and bundle the helper, runtime manifest and
licenses using the shared `bundle.cmake` inventory. They exclude libcef and
Chromium resources/locales. Flutter builds the helper in its own build tree;
standalone builds default to `build/native-web`, configurable with
`VARPAPER_WEB_BUILD_DIR`. A prebuilt helper can be selected by setting
`VARPAPER_WEB_HOST_BINARY` to its absolute path before clean configuration;
the adjacent `runtime.json` must match this source tree's SDK/IPC manifest.

The application installs runtime libraries, resources, locale files, snapshots,
LICENSE.txt and CREDITS.html only after the Settings installation action. The
helper starts with this component directory in its own `LD_LIBRARY_PATH`; the
main application has no libcef dependency. Chromium sandboxing remains enabled.
Inside Flatpak, Rust launches only this helper through the packaged Zypak wrapper
with `ZYPAK_CEF_LIBRARY_PATH` pointing at the installed component. Zypak uses the
Flatpak portal for Chromium subprocess sandboxes. A packaged Zypak patch exposes
only the installed component directory read-only and supplies the child loader's
library path; the GUI is not preloaded with
CEF or Zypak. The Flatpak base package builds the pinned Zypak module without
installing any CEF runtime.
CEF initialization failures terminate the helper and become a wallpaper error.
Development overrides using `VARPAPER_WEB_HOST` require `runtime.json` beside
the helper; CMake copies the pinned manifest into its build directory.

Installation replaces an existing runtime with an atomic Linux directory exchange;
an unsuccessful commit preserves the existing runtime. Uninstall blocks new Web
requests, cancels installation, waits for browser/process/audio teardown and then
removes the owned runtime/cache/staging. Deletion errors remain retryable without
removing projects or saved assignments. Startup and restore never download a runtime.

IPC uses an inherited private Unix socket, 32-bit little-endian JSON lengths,
request/browser/generation IDs and an IPC version. JSON is limited to 64 KiB,
paint payloads and local resources to 64 MiB each, with at most 16 browsers.
Paints have sequence numbers and dimensions; resize discards old dimensions.
CEF never touches the engine's GL context. Rust uploads BGRA pixels to a texture,
corrects orientation, restores GL state and swaps only after a fresh paint.
No paint leaves the displayed frame intact. Browser/host failures are reported
without passing HTML to mpv. Host transport failure terminates its process group;
the next application can create a new host.

Local projects use a project-scoped HTTPS scheme handler. It checks decoded
paths, canonical roots, symlink escapes and file budgets. Reads use owned
directory descriptors and `openat` with `O_NOFOLLOW` on each path component,
so filesystem changes cannot redirect an already validated resource.
External requests,
file access, downloads, external protocols and popups are blocked. Missing
optional resources (including the browser-generated favicon request) return
404; a missing entry fails creation. CEF diagnostics report forbidden requests.
Resource responses carry a restrictive Content Security Policy, including
worker scripts; policy reports become wallpaper errors. Window connection APIs
are blocked before page scripts run, and worker WebSocket requests are covered
by the resource policy. JavaScript dialogs, file pickers and device/permission
prompts are denied instead of opening desktop windows.

Pause closes the browser and retains the displayed frame. Resume creates a new
browser at the same viewport, so timers and generated sound stop during pause,
while temporary page state may reset. CEF audio packets feed a bounded per-browser
SDL output (48 kHz stereo, at most 250 ms queued); mute and volume apply to both
HTML media and Web Audio. Closing a browser closes its audio device and queue.

`wallpaperPropertyListener.applyUserProperties` receives initial declared values
after page load, and updated values through the renderer property interface.
Boolean, numeric and string values retain their JSON types; updates must refer
to declared properties, preserve types, obey numeric bounds and declared combo
choices, and provide valid three-component colors. Resume reloads
the latest values. Confirmed assignments can persist initial Web property
overrides through the Flutter store. Calling the runtime property-update API
does not itself save an assignment; the GUI property editor remains pending.
See [property and audio contracts](../../docs/src/dev/rust-scene.md#script-properties-audio-and-time-contracts)
for callback parameters, validation, capture failure and time boundaries.

`wallpaperRegisterAudioListener` is installed before page scripts run. It receives
128 finite values in 0..1 (64 increasing-frequency left bands, then 64 right
bands) from the shared system playback monitor. Silence/unavailable/stale capture
supplies zeros. Wallpaper mute only controls generated sound. Pausing/closing
releases this browser's monitor consumer; the last consumer joins the worker.

Browser cache files live under the component's `browser-cache/<host-pid>`.
The host owns a private process group and acts as a Linux child subreaper. After
CEF shutdown it terminates and reaps remaining zygote/worker descendants before
returning. Normal shutdown removes this cache. Uninstall blocks new Web requests, cancels
downloads, waits for the engine's `StopWeb` acknowledgement and then removes the
component directory, including stale cache/staging files. Other output types
and user project files are preserved. Failed shutdown acknowledgement preserves
the runtime files for retry.

Run the real host regression with a complete flattened pinned runtime:

```sh
xvfb-run -a env VARPAPER_TEST_CEF_SOFTWARE=1 \
  python3 native/web/test_host.py \
  --host build/native-web/varpaper-web-host --runtime /path/to/runtime
```

`VARPAPER_TEST_CEF_SOFTWARE` selects SwiftShader for test environments. The test
covers local CSS/JS/images, Canvas/WebGL, two independent browsers, resize,
pause/resume, resource/network/download/protocol/dialog restrictions,
WebSocket/WebRTC and worker policy diagnostics, frame budgets, typed initial/live
properties, stereo spectrum/zero callbacks, HTML/Web Audio volume/mute, paused
audio counters, renderer crash isolation and all descendant subprocess exits.
Tests use SDL's dummy audio device and produce no sound. Rust's
`web_texture_orientation_first_frame_and_stop_preserve_media`
X11 fixture additionally checks real GL pixels, state restoration, candidate
commit/rollback, property type/range/color/choice rejection, audio-consumer
cleanup and stopping Web while another media output remains active.
Pass `VARPAPER_TEST_X11=1`, `VARPAPER_WEB_HOST` and `VARPAPER_CEF_RUNTIME` when
running that fixture under Xvfb. These tests do not replace real-desktop,
Wayland, packaging or soak acceptance.

The pinned nlohmann/json archive belongs to this Web helper and lives in
`vendor/json.tar.gz`. `dependencies.json` records its source revision and hash;
`JSON-LICENSE` preserves its MIT notice. It has no dependency on the removed
WallpaperEngine Scene source directory.
