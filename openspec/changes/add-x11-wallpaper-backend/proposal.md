# Change: Add an X11 wallpaper backend to the Rust engine

## Why

The playback engine connects directly to Wayland and creates wlr-layer-shell surfaces. On a native X11 session it cannot enumerate usable outputs or display wallpapers. Rust monitor discovery also calls `wlr-randr`, so the Flutter monitor list is empty on X11.

## What Changes

- Add a native X11 backend for monitor discovery, background windows, output changes, and per-monitor wallpaper playback.
- Select the backend from the current desktop session: Wayland for native Wayland sessions and X11 for native X11 sessions. Do not treat XWayland inside a Wayland session as the X11 backend.
- Reuse the engine command/event contract and libmpv playback controls across backends.
- Generalize EGL surface handling so Wayland and X11 windows can both render video.
- Return X11 monitors through the existing FRB service and remove Wayland-only wording from runtime UI states.
- Add X11 runtime permission to the Flatpak and X11 library dependencies to the Debian package.

## Impact

- Affected specs: new `x11-backend`; existing `gui-integration` monitor behavior.
- Affected code: `crates/wayvid-engine` engine, EGL, and session modules; `rust/src/engine.rs` and `rust/src/bridge.rs`; Flutter monitor wording; Linux package metadata.
- Related active change: `refactor-flutter-owned-state-and-engine-boundary` owns the FRB service and engine lifecycle. X11 work must preserve that service boundary and avoid reintroducing a daemon, CLI, or Rust-owned persistence.
- This adds a second renderer backend and requires testing on a native X11 session in addition to the existing Wayland path.
