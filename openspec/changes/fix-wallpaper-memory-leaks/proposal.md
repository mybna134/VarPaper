# Proposal

## Why

VarPaper retains native rendering resources after some wallpaper and output lifecycle transitions, and its Flutter thumbnail Future map retains every requested preview until a library refresh. The engine audit reproduced an EGL surface and context remaining queryable after unbinding and Rust destruction; these defects can accumulate memory, although they do not yet establish the cause of growth during uninterrupted playback.

## What Changes

- Give EGL displays, contexts, and surfaces deterministic, idempotent ownership and cleanup, including partial initialization and error exits.
- Preserve each output's native window and initialized EGL surface across wallpaper replacement through the current hot-swap path.
- On wallpaper clear, output removal, compositor closure, and engine shutdown, release MPV with its original OpenGL context current, then EGL resources, then native window resources. Reapply after clear creates a new window.
- Explicitly destroy owned Wayland protocol objects instead of relying on proxy drops; ignore stale callbacks from replaced surfaces.
- Bound Flutter preview retention by LRU entry and byte budgets, deduplicate concurrent requests, and prevent late completions from restoring cleared cache entries.
- Add native resource lifecycle regressions, Flutter cache regressions, and repeatable playback/lifecycle memory measurements.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `video-playback`: Add a precise cross-backend rendering-resource ownership and teardown contract.
- `wayland-backend`: Add complete, idempotent output/surface teardown and stale-event handling.
- `gui-integration`: Extend existing thumbnail cache limits with retained-byte accounting and asynchronous lifecycle behavior.

## Impact

- Rust: `crates/wayvid-engine/src/egl.rs`, `mpv.rs`, `engine/session.rs`, `engine/mod.rs`, and `engine/x11.rs`; inspect service/controller lifecycle integrations for compatibility.
- Flutter: preview retention in `lib/main.dart`, a small cache helper if needed, and controller/cache tests using the existing fake service.
- Keep the Flutter/FRB service API, persisted settings, wallpaper controls, and disk preview cache behavior compatible. Internal Rust resource wrappers may change, with workspace callers updated together. No new runtime dependency is intended.
- Existing `add-x11-wallpaper-backend` and `refactor-flutter-owned-state-and-engine-boundary` changes overlap these files; this change complements their cleanup promises without restarting their broader architecture work.
