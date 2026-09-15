## Context

The existing GUI combines iced rendering with backend orchestration in one Rust crate. Flutter must become the presentation layer without moving Wayland/MPV rendering into Flutter or breaking the standalone GUI architecture and `wayvid-ctl` IPC.

## Goals / Non-Goals

- Goals: functional parity, stable Rust backend ownership, typed FRB boundary, Linux Wayland support, compatible packaging, and testable Dart state management.
- Non-goals: Windows/macOS/mobile/web support, changing the engine render loop, or changing the JSON IPC protocol.

## Decisions

- Decision: Make the repository root the Flutter application and place its `cdylib`/`rlib` FRB service in top-level `rust/`; shared engine/library crates remain under `crates/`. The Flutter bundle is the user-facing `wayvid-gui` executable.
- Decision: Expose DTOs and a `WayvidService` facade instead of exposing `wayvid-core` and engine internals directly.
- Decision: Keep ksni and Unix IPC/single-instance logic in Rust; Flutter controls the application window through a small window-manager adapter.
- Decision: Use FRB v2 Cargokit integration, pin the bridge/codegen versions together, and commit generated bridge sources.
- Decision: Manage UI state through Riverpod, using a `ChangeNotifierProvider` for the long-lived service controller during the migration. The provider owns injection and rebuild propagation; Rust async methods handle library/scanning/thumbnail work and a polled event queue handles engine and tray events.
- Decision: Keep settings YAML and IPC JSON shapes compatible. Bridge paths are strings, image data is byte data, and bridge failures use a stable code/message error DTO.
- Decision: Move visible UI translations to Flutter ARB resources for English and Simplified Chinese; keep Rust tray labels synchronized through the existing locale mechanism.

## Data Flow

```text
Flutter pages -> Riverpod -> generated FRB bindings -> WayvidService
                                                      -> engine/library/settings/tray/IPC
Rust events <- AppController.pollEvents() <- FRB event DTOs
```

## Risks / Trade-offs

- Flutter release output is a bundle rather than one self-contained Rust binary; package scripts must install its executable, `lib`, and `data` contents together.
- FRB generated code is toolchain-sensitive; CI must regenerate and verify it with the pinned codegen version.
- Flutter window plugins can vary across desktop environments; the adapter keeps this concern isolated and manual Wayland acceptance tests cover close/show/focus behavior.

## Migration Plan

1. Add the OpenSpec change and bridge/service DTOs without changing engine behavior.
2. Add the Flutter Linux application and Cargokit integration.
3. Port pages and lifecycle behavior, then remove iced-only code and dependencies.
4. Update CI and distribution packaging to build/install the Flutter bundle.
5. Run Rust, Dart, build, packaging, and real Wayland acceptance checks.
