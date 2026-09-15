# Change: Replace iced GUI with Flutter and flutter_rust_bridge

## Why

The current `wayvid-gui` presentation layer is implemented with iced. The application already has a substantial Rust playback, library, settings, IPC, and desktop-integration backend that should remain stable while the UI is migrated to Flutter.

## What Changes

- Replace the iced desktop UI with a Linux Flutter application using `flutter_rust_bridge`.
- Convert the Rust GUI crate into a bridge/service library while retaining the playback engine, Wayland integration, library, settings, IPC, single-instance, and ksni tray behavior.
- Keep the `wayvid-gui` executable name, settings paths, desktop entry, systemd entry, CLI IPC protocol, and user-facing workflows compatible.
- Add Flutter/Cargokit build, test, release packaging, and documentation integration.

## Impact

- Affected specs: `gui-integration`, `video-playback`, and desktop/building documentation.
- Affected code: the root Flutter application, top-level `rust/` FRB service, Rust core crates, CI, packaging, install scripts, and GUI documentation.
- The Rust playback engine and shared JSON IPC wire format remain unchanged.
