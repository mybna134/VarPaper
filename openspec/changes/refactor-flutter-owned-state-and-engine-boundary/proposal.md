# Change: Move persistence to Flutter Isar and isolate the Rust engine boundary

## Why

The current application still contains the legacy `wayvid-core`, SQLite library persistence, CLI, Unix IPC, Rust tray integration, and a backend structure inherited from the daemon architecture. The desired runtime is a Flutter-owned desktop application where Flutter owns persistent state and desktop lifecycle, Rust exposes scanning/preview/engine services through FRB, and the rendering engine receives explicit configuration from Flutter.

## What Changes

- **BREAKING** Move all application settings persistence to Flutter using `isar_community`.
- **BREAKING** Remove the command-line tool, Unix IPC protocol/server, and `wayvid-core` crate.
- **BREAKING** Remove SQLite from `wayvid-library`; library scans return in-memory results to Flutter.
- Move system tray integration to a Flutter desktop plugin.
- Move single-instance handling to Flutter; it must not depend on IPC.
- Make Flutter/Rust service the only orchestrator between library scanning and the rendering engine.
- Make `wayvid-engine` independent of `wayvid-library` and independent of `wayvid-core`.
- Send an explicit engine configuration from Flutter whenever an engine is created.
- Send configuration updates to the engine when Flutter settings change.
- Keep wallpaper scanning and preview generation in Rust, requested by Flutter through FRB.
- Return image preview paths/handles directly where possible; generate one-frame previews for video sources through the Rust preview service.
- Drop compatibility with existing YAML and SQLite state. No migration is required.

## Impact

- Affected specs: `gui-integration`, `config-management`, and `video-playback`.
- Affected code: Flutter models/providers, FRB bridge, Rust service, engine crate, library crate, Linux plugins, packaging, documentation, and workspace manifests.
- Removed code: `crates/wayvid-core`, `crates/wayvid-ctl`, Rust IPC, CLI IPC client, Rust tray implementation, and Rust single-instance implementation.
- New Flutter dependency: `isar_community` and its required generator/runtime support.

