# Optional Chromium wallpaper host

`varpaper-web-host` owns CEF on its main thread and never creates desktop
windows or OpenGL contexts. Rust shares one host on the engine thread and gives
each output a separate browser and in-memory request context. The same helper
executable handles CEF subprocesses. Its CEF DLL wrapper is built from the
pinned official SDK; the upstream BSD notice is retained in `CEF-LICENSE`.

`runtime.json` pins the SDK/runtime version, architecture, URL, SHA-256 and IPC
version. Build with `scripts/build-web-host.sh`; `CEF_ROOT` can select an already
extracted matching SDK. SDK downloads are build-time operations. Application
startup never invokes this script. `scripts/build-linux.sh` builds and bundles
only the helper, runtime manifest and wrapper license. It excludes libcef and
Chromium resources/locales. A direct Flutter build can bundle a prebuilt helper
by setting `VARPAPER_WEB_HOST_BINARY` to its absolute path.

The application installs runtime libraries, resources, locale files, snapshots,
LICENSE.txt and CREDITS.html only after the Settings installation action. The
helper starts with this component directory in its own `LD_LIBRARY_PATH`; the
main application has no libcef dependency. Chromium sandboxing remains enabled.
CEF initialization failures terminate the helper and become a wallpaper error.

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
while temporary page state may reset. Mute uses CEF audio muting; volume updates
currently cover HTML audio/video elements. Web Audio volume, project property
callbacks and system-audio visualization are still pending and are not advertised
as completed renderer capabilities.

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
WebSocket/WebRTC and worker policy diagnostics, frame budgets
and subprocess exit. Rust's `web_texture_orientation_first_frame_and_stop_preserve_media`
X11 fixture additionally checks real GL pixels, state restoration, candidate
commit/rollback and stopping Web while another media output remains active.
Pass `VARPAPER_TEST_X11=1`, `VARPAPER_WEB_HOST` and `VARPAPER_CEF_RUNTIME` when
running that fixture under Xvfb. These tests do not replace real-desktop,
Wayland, packaging or soak acceptance.
