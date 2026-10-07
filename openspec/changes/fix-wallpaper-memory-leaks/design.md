# Design

## Context

See `proposal.md` for motivation and the three delta specs for the lifetime and cache contracts.

The integrated engine owns a dedicated thread and one shared EGL context across output sessions. `EglWindow::drop` and `EglContext::drop` only log; manual surface destruction exists but is omitted by several Wayland exit paths. `WallpaperSession::cleanup_egl` drops MPV without first ensuring its GL context is current. This is unsafe after another cleanup unbinds the shared context: libmpv's installed `render.h` requires the original GL context to be current for its GL render API calls, including destruction. X11 already has an explicit session-before-window cleanup path, but shares the incomplete EGL wrappers.

Wayland clear and shutdown destroy layer objects before rendering sessions; `GlobalRemove` does not remove sessions or layer surfaces; `Closed` removes only the layer map entry. Raw Wayland proxies do not send protocol destructors merely because a Rust proxy is dropped. Events identify surfaces only by output name, allowing a late old event to affect a replacement on that output.

Flutter stores `Future<PreviewDto?>` values in `_thumbnailFutures`; completed Futures retain encoded bytes. The map is cleared only on refresh. Existing GUI specs already require an LRU limit of 100 entries. Existing main specs also describe an older daemon/IPC architecture, shared decoding, and disk formats/paths that differ from current implementation. This change adds focused lifetime/retention requirements; it does not silently reconcile those unrelated specifications. Active X11 and Flutter boundary changes overlap these files and must remain compatible.

## Goals / Non-Goals

**Goals:** Make teardown deterministic on success and recoverable failure paths, keep native work on its owning engine thread, bound application preview retention, and prove native releases rather than relying on RSS alone.

**Non-Goals:** Implement shared decoding, change codec/hardware-decoding selection or frame pacing, replace the display backends, redesign the FRB service, or promise a fixed total process RSS. GPU drivers, image decoding, and allocators can retain memory independently of live application resources.

## Decisions

### 1. Own native EGL resources with thread-local RAII

Introduce a stable shared EGL owner used by contexts and surfaces (a thread-local `Rc` owner, not cross-thread shared rendering). Surface handles hold enough ownership to destroy themselves while the EGL instance/display is valid. Explicit surface destruction consumes or marks the native handle released; subsequent drop is a no-op. Keep the EGL library loaded until the last owned object is released. Enforce parent lifetime and reject rendering with a released or foreign surface.

Use an initialization guard immediately after successful display initialization, so API binding, config selection, and context creation failures unwind acquired resources. Destroy owned contexts and balance engine-owned display initialization when their final dependents end. Before adding `eglTerminate`, verify display ownership: it must not terminate a display still used by Flutter or another EGL client. Keep native connections alive through teardown, and only unbind this owner's current context. Log cleanup failures and continue independent cleanup rather than abandoning all resources after the first error.

Alternative: add manual cleanup calls at every current exit site. Rejected as the sole protection because future early returns and constructor failures would repeat the existing omissions. Explicit ordered cleanup remains useful, backed by RAII.

### 2. Centralize ordered playback teardown

Provide one idempotent session release operation: bind its surviving EGL surface and original context; free the MPV render context/player; unbind and destroy the EGL surface; release its `wl_egl_window` wrapper. The backend subsequently destroys the native output window (`wl_surface` plus layer object, or X11 window) when the wallpaper is cleared or output ownership ends. Session drop invokes the same protection through retained ownership so `?` exits cannot bypass release. Test cleanup of two outputs consecutively, since the first cleanup unbinds the context needed by the second.

On clear, remove the playing session and destroy its native window after playback cleanup. Keep monitor enumeration/output bindings while the monitor remains connected. Reapply creates new Wayland protocol surfaces or a new X11 window and new EGL playback resources. On replacement, keep the current hot-swap behavior, preserving both the native window and an initialized EGL surface; replacing a source before configuration completes must also reuse the pending native window.

Wallpaper clear, output removal, compositor closure, backend error exit, and engine shutdown destroy owned native windows after playback teardown. Final engine cleanup releases contexts/display resources before closing the native connection.

Rebinding a valid original context is a normal cleanup step, including after another output cleanup unbinds it. On a recoverable binding failure, retain playback ownership and report the error so teardown can retry before native windows are destroyed. Initialization failures unwind acquired resources and preserve the original diagnostic. Permanently lost GPU contexts or disconnected graphics drivers have not been observed and are outside this focused leak repair; they do not gate normal lifecycle cleanup or introduce process isolation.

Use an `EngineState` teardown helper for clear, removal, `Closed`, and shutdown that releases the session before removing its layer entry, with drop protection for event-loop errors. Replacement reuses the existing entry. Give `LayerSurfaceInfo` explicit protocol cleanup protection for the end of window ownership. Release `wl_output` on output removal or engine shutdown only where the negotiated version supports `release`; older bindings end with connection teardown. Retain existing application event meanings and Flutter assignment restoration on reconnection. Keep the X11 session-before-window destruction order, including clear, and retain its source hot-swap behavior.

Alternative: recreate the entire engine whenever an output changes. Rejected because it disrupts unaffected outputs and conceals resource ownership errors.

### 3. Identify Wayland events by surface lifetime

Carry a monotonically increasing surface generation in surface/layer/frame callback user data alongside output name. Check it against the current layer entry before handling configure, closure, or frame readiness; ignore events from replaced surfaces. Frame callbacks are compositor-destroyed after `done`, so do not introduce invalid destructor requests for them. Output removal remains tied to registry global identity, including outputs whose human-readable name was unavailable.

Alternative: match only output names. Rejected because clear/reapply and reconnect can reuse the name before old events are dispatched.

### 4. Bound completed previews and pending request ownership

Use a small Dart cache helper with injectable limits for tests, defaulting to 100 completed entries and 32 MiB encoded bytes. Maintain LRU access order; account `PreviewDto.bytes.length` exactly. Path-only and failure entries count toward the entry budget. Oversized previews reach their caller but are not retained. Flutter's separate decoded image cache retains its existing policy; these budgets do not describe all visible image memory.

Track pending loads separately, deduplicated by wallpaper ID and generation, with a maximum of 100 tracked requests. When all slots are occupied, return the existing placeholder/fallback result without starting additional work or caching a permanent failure; a later request can retry. A completed result moves into the bounded LRU and removes its pending entry. Completed loads from a previous refresh/shutdown generation must not remove or overwrite newer entries. Refresh clears completed references and advances the generation; pending loads from older generations still count against the total outstanding-work limit until they finish, so repeated refresh cannot bypass the bound. Shutdown also closes the cache. Already running FRB work can complete for its original caller but cannot repopulate the cache. Disk cache storage and preview DTOs remain unchanged.

Alternative: rely on Flutter's `ImageCache`, or periodically clear the whole Future map. Neither provides a precise bound for encoded preview bytes retained by Futures, and whole-map clearing discards useful recent entries.

### 5. Verify releases directly and measure memory separately

Add a narrow test seam around native destruction to assert paired acquisitions/releases, ordering, and partial initialization failures. Promote the independent audit into a reproducible regression: under Xvfb, capture EGL handles, explicitly unbind, drop the Rust owners, and confirm handles are no longer usable; do not let display termination or a thread exit mask a missing surface release. Run real X11 multi-output session cleanup sequentially. A small Wayland protocol fixture or instrumented compositor session must verify surface destruction, removal/closure ordering, and stale-event rejection; unit tests alone cannot establish native Wayland cleanup.

Run at least 100 apply/render/clear cycles and 20 engine start/render/stop cycles, draining engine events. After clear, require playback resources and wallpaper native windows to return to baseline; reapply acquires a new window rather than reusing the destroyed handle. Require all engine-owned resources to return to the pre-engine baseline after engine stop. Verify replacement retains the same native window and initialized EGL surface identities. Exercise failed initialization and two-output cleanup. For the GUI, cover entry/byte eviction, oversized/path-only/failure previews, deduplication, pending saturation, and delayed completions across refresh/shutdown.

Separately sample RSS, PSS, anonymous memory, and available GPU process metrics every 5 seconds: compare three-minute warm-up with ten-minute measurement windows for uninterrupted looping video, lifecycle cycling, and scrolling more than 1000 previews. Keep wallpaper, resolution, output count, backend, decoder, and build fixed between before/after runs. Investigate a sustained post-warm-up increase above 1 MiB/minute or 32 MiB over a window; these are investigation triggers, not universal allocator guarantees. Record direct resource counts and cache budgets as deterministic acceptance evidence. If uninterrupted playback still grows, isolate its allocation source and report it explicitly rather than declaring the original symptom solved solely from lifecycle fixes.

## Risks / Trade-offs

- Shared EGL display termination could break Flutter rendering → verify native display identity and ownership; release only the engine's resources.
- Cleanup must bind the original valid GL context → test consecutive multi-output cleanup and a recoverable binding failure before retry; do not treat an unbound context as a fault.
- RAII plus existing manual cleanup can double-free → use one release state and test explicit cleanup followed by drop.
- Stale compositor events can target a reconnected output → validate surface generation before mutation.
- Cache eviction may reload previews from disk → preserve disk cache and pending deduplication; test that repeat widget rebuilds do not reload retained failures.
- Pending work cannot currently be cancelled via FRB → bound new work, invalidate late cache writes, and account separately for outstanding work held by callers.
- Existing test success does not demonstrate absence of leaks → require native handle/count checks and report hardware-dependent tests that could not run.

## Migration Plan

Implement ownership and teardown first, then preview retention, then the regression and memory runs. No settings or disk-cache migration is required. Validate the OpenSpec change and run appropriate Rust/Flutter checks; all checks in the CI `quality` job must pass before any push. Rollback reverts these code changes without deleting user data, but restores the known lifecycle defects.
