# Design

## Context

See `proposal.md` for motivation. This change crosses the Linux runner and Flutter startup/tray code.

Observed in the current source:

- `lib/main.dart` loads the Isar settings through `WayvidController.create`, configures `skipTaskbar` from `startMinimized`, and omits `show()`/`focus()` when that preference is true, before calling `runApp`.
- `linux/runner/my_application.cc` realizes the Flutter view and connects `first-frame` to a callback that unconditionally calls `gtk_widget_show` on the top-level window. Thus the first rendered frame reopens the window regardless of the Dart decision. This is the source-level cause; this planning phase has not reproduced it in a running desktop session.
- The locally resolved `window_manager` 0.5.2 Linux implementation treats `waitUntilReadyToShow` as an immediate success, and the Dart wrapper invokes its `VoidCallback` without awaiting an async callback. It does not suppress the runner's first-frame display.
- Existing tray open actions call `show()`/`focus()` directly without clearing the startup `skipTaskbar` setting. Existing widget tests cover tray methods but bypass `main()` and cannot exercise GTK first-frame behavior.
- The store already persists `startMinimized`; its current save/load test does not assert that field. Main specs still describe older YAML/daemon and `--minimized` autostart behavior. This delta adds the startup contract without changing those historical requirements.

## Goals / Non-Goals

**Goals:**
- Give Dart a single authoritative decision about initial top-level window visibility, using the existing persisted preference.
- Make window operations awaitable and testable independently of Rust initialization.
- Preserve Flutter view realization, plugin registration and background startup while the top-level window stays hidden.

**Non-Goals:**
- Native access to Isar, new IPC or method channels, dependency upgrades, CLI flags, and autostart policy changes.
- Changing the close-to-tray preference or inventing fallback behavior for desktops without a tray host.

## Decisions

### 1. Remove the runner's automatic first-frame presentation

Remove the unconditional top-level show callback and its signal connection. Keep child-view visibility and realization, container setup, plugin registration and focus setup needed to initialize the Flutter view. Dart explicitly shows the normal-start window; hidden startup leaves the top-level window unshown.

An explicit Dart `hide()` after the first frame would permit a visible flash and retain competing visibility owners. Reading the persisted setting in C++ would duplicate storage ownership and unnecessarily couple the runner to Isar.

### 2. Sequence startup configuration and presentation explicitly

Extract a small startup-window function in `lib/main.dart` accepting the loaded GUI settings. Await window readiness/configuration without an async `VoidCallback`, then explicitly await normal-start presentation. The minimized branch makes no show/focus call. Retain the current size, minimum size, centering, title and startup taskbar configuration, and keep `runApp` unconditional.

There is no need for a new state manager: the choice is a one-time decision from persisted settings. Method-channel mocks can exercise the production function without starting Rust or opening the production database.

### 3. Share the explicit user restore path

Use a common function for tray-driven presentation that awaits `setSkipTaskbar(false)`, `show()` and `focus()` in that order. Route show, library, settings and hidden-window icon clicks through it; retain existing navigation and visible-window hide behavior. This resets the startup-only taskbar hint and avoids inconsistent tray actions.

Keeping each direct show call would require duplicating the same restore sequence in four places. A common function is sufficient; broader window lifecycle refactoring is unnecessary.

## Risks / Trade-offs

- [Removing the first-frame handler affects rendering/realization assumptions] → Build the Linux bundle and verify both normal and hidden startup on supported Wayland and X11 desktops; check hidden startup stays alive and tray restore renders usable content.
- [Widget mocks cannot detect native presentation or focus stealing] → Use actual desktop smoke checks to observe the main window before and after startup, including delayed startup; automated Dart tests alone do not prove the native fix.
- [Tray availability and taskbar behavior vary by desktop] → Validate restore on a desktop with a tray host and interpret taskbar eligibility according to desktop support. Existing no-host behavior stays outside this change.
- [Show/focus may previously outlive the readiness future] → Await production presentation operations directly and assert method order in tests.

## Migration Plan

No settings migration is needed. Build and deliver the runner and Dart changes together through the existing Linux packaging. Validate saved true/false preferences across complete process restarts and verify tray restore plus configured wallpaper restoration. Rollback reverts both code changes together; stored settings remain compatible. Before any push, pass all checks in the `quality` job of `.github/workflows/ci.yml` as required by `AGENTS.md`.
