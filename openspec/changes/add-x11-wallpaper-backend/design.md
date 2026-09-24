## Context

`spawn_engine` currently starts a Wayland-only event loop. `EglContext` takes a Wayland display pointer, `WallpaperSession` takes a `WlSurface`, and `detect_monitors` parses `wlr-randr`. The public engine commands and events already use protocol-neutral output names and playback operations.

## Goals / Non-Goals

- Goals: native X11 wallpaper playback, per-monitor assignment, monitor hotplug, clear/pause/resume, and graceful cleanup through the current FRB service.
- Non-goals: rendering through XWayland inside a Wayland session, replacing the desktop shell or file manager, changing persistent wallpaper data, or adding a separate process.

## Decisions

1. **Backend selection.** Introduce a session backend enum at engine startup. Prefer the native Wayland backend when `XDG_SESSION_TYPE=wayland` or a Wayland display is active; use X11 when `XDG_SESSION_TYPE=x11` and `DISPLAY` is available. Report a clear error when neither native backend can initialize. Do not silently switch to XWayland if Wayland layer-shell is unavailable.
2. **X11 protocol.** Keep one Xlib display connection in a dedicated Rust backend module so its native `Display*` can also initialize EGL. Use RandR monitor queries and events for output changes and XFixes for an empty input region. Preserve stable monitor names for the existing `OutputInfo` and FRB `MonitorDto`. Create a managed desktop-type window for each monitor using `_NET_WM_WINDOW_TYPE_DESKTOP` and `_NET_WM_DESKTOP=0xFFFFFFFF` before mapping. Resize and reposition windows on RandR changes, and destroy them on output removal or shutdown. Keep all Xlib handles and unsafe calls inside this module.
3. **Rendering.** Keep libmpv's OpenGL render path. Separate protocol-specific window creation from EGL context/surface operations so the existing playback session can render to either a Wayland or X11 native window. Keep ownership and destruction order explicit: stop player, unbind EGL, destroy EGL surface, destroy native window, then disconnect.
4. **Service boundary.** Keep `EngineCommand`, `EngineEvent`, and FRB wallpaper operations stable. Replace the `wlr-randr` subprocess parser with backend-aware Rust monitor discovery. Expose backend-specific errors without requiring Flutter to manage X11 handles.
5. **Packaging.** Allow the Flatpak X11 socket for native X11 sessions. Declare the native X11 libraries used by the Debian package. Do not add another package format.

## Risks / Trade-offs

- EWMH desktop windows can coexist differently with desktop icon managers. Test at least one common X11 window manager and document any stacking limitation.
- XRandR reports monitor geometry separately from X11 screens; preserve monitor coordinates and handle disconnected outputs without reusing stale windows.
- X11 and Wayland EGL native handles differ. Hide unsafe raw handles in the backend module and add lifecycle tests around surface cleanup.
- Flatpak's X11 socket broadens display-server access only for the X11 backend. Keep Wayland as the selected backend in Wayland sessions.

The desktop-window behavior follows the [Extended Window Manager Hints specification](https://specifications.freedesktop.org/wm/latest-single/); RandR monitor handling follows the [X.Org RandR protocol](https://www.x.org/Projects/XRandR/).

## Migration Plan

No persisted-state migration is required. Existing Wayland installations continue using the Wayland backend; an X11 session selects the new backend at engine startup.
