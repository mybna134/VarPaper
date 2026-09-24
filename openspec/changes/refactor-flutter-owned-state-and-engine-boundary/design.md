## Context

The current runtime is Flutter plus a Rust service, but settings, library persistence, tray integration, single-instance behavior, IPC, and several domain models still come from the previous daemon-oriented architecture. The target is a single Flutter desktop application with a narrow FRB service boundary.

## Goals / Non-Goals

### Goals

- Make Flutter/Isar the source of truth for user settings.
- Keep Rust responsible for Wayland, libmpv, scanning, and preview extraction.
- Make Flutter the owner of engine lifecycle, configuration, tray, and single-instance behavior.
- Prevent `wayvid-engine` and `wayvid-library` from importing or calling each other.
- Remove all legacy CLI, IPC, SQLite, YAML settings, and `wayvid-core` dependencies.
- Preserve existing visible GUI workflows: scan, preview, apply, pause, resume, clear, monitor display, and settings changes.

### Non-Goals

- No compatibility migration for YAML settings, SQLite data, CLI commands, or IPC clients.
- No standalone daemon process.
- No direct Flutter implementation of Wayland or libmpv rendering.
- No engine/library shared persistence layer.

## Decisions

### 1. Flutter owns persistent state

Define Isar collections in the Flutter project for:

- application settings;
- playback settings;
- GUI/window settings;
- power settings;
- library folder configuration;
- per-output wallpaper assignments if persistence is still required by the UI.

The Flutter controller/provider loads Isar before creating the Rust service. Rust receives settings as request DTOs and does not read or write the settings database.

### 2. Flutter owns desktop lifecycle

Use a Flutter Linux plugin for:

- system tray;
- tray menu events;
- single-instance locking and existing-window activation;
- window show/hide/focus behavior.

The Rust service must not contain ksni, Unix Socket, or single-instance code.

### 3. Rust service is a stateless FRB facade

The Rust service exposes operations such as:

- `scan_library` / `scan_folder` / `scan_workshop`;
- `load_preview` or `load_preview_path`;
- `create_engine(config)`;
- `update_engine_config(config)`;
- `apply_wallpaper(source, output)`;
- `pause`, `resume`, and `clear`;
- `stop_engine`.

The service may keep short-lived runtime handles and scan results, but it must not own persistent application settings.

### 4. Library and engine are sibling modules

The dependency direction is:

```text
Flutter → Rust service → wayvid-library
                      → wayvid-engine
```

`wayvid-library` must not depend on `wayvid-engine`. `wayvid-engine` must not depend on `wayvid-library`. DTO conversion and application orchestration happen in the Rust service or Flutter layer.

Each crate owns its own types:

- library types describe scanned wallpaper sources and metadata;
- engine types describe render configuration, outputs, commands, and playback state;
- FRB DTOs are the only cross-language API types.

### 5. Library persistence is removed

`wayvid-library` becomes an in-memory scanner and preview provider. Folder and Workshop scans return values to Flutter. SQLite schema, migrations, tags, collections, ratings, and database CRUD are removed unless an equivalent Flutter/Isar feature is explicitly needed by the UI.

Thumbnail/preview files may still be cached on the filesystem. This cache is not the wallpaper library database.

### 6. Engine configuration is explicit

Flutter builds an engine configuration from Isar-backed settings and sends it during engine creation. Configuration changes are persisted by Flutter first, then sent through FRB to the running engine. The engine applies updates to existing sessions without reading settings or library state.

The configuration boundary should include at least:

- volume;
- FPS limit;
- loop mode;
- hardware decoding mode;
- layout mode;
- HDR/tone mapping options;
- power pause behavior;
- target/output preferences where applicable.

### 7. Preview transport

Use the least-copy transport available for each source:

- existing image or Workshop preview: return a path/URI for Flutter to display directly;
- video preview: Rust extracts one frame using the preview implementation and returns a cache path or encoded bytes;
- do not pass an engine-owned GPU texture or `EglWindow` handle into Dart.

The exact FRB representation must be validated against Linux Flutter and `isar_community`/Dart memory ownership during implementation. Correctness and lifetime safety take priority over an unsafe raw-pointer zero-copy API.

## Data Flow

```text
Flutter Isar
    │
    ├─ settings/config ──> FRB ──> Rust service ──> wayvid-engine
    │                                      │
    │                                      └─> engine sessions / Wayland / MPV
    │
    └─ scan request ─────> FRB ──> Rust service ──> wayvid-library
                                                │
                                                └─> scan result / preview result
```

## Risks / Trade-offs

- Removing SQLite removes existing library metadata, tags, collections, ratings, and usage history. This is intentional and has no migration path.
- Removing IPC makes `wayvid-ctl` and external automation unavailable.
- Flutter tray and single-instance plugins may differ in desktop support; the plugin boundary must be tested on the supported Wayland environment.
- FRB generated APIs will change substantially; all generated Dart and Rust bindings must be regenerated together.
- Returning file paths is safer and simpler than exposing raw image handles. Raw handles must not outlive the Rust allocation or be used after an asynchronous call.

## Migration Plan

1. Add Isar models, initialization, repositories, and Riverpod state providers.
2. Define the new FRB DTO and service API.
3. Remove Rust-owned settings, tray, single-instance, IPC, CLI, and core dependencies.
4. Refactor `wayvid-library` to remove SQLite and return scan/preview results.
5. Refactor `wayvid-engine` to own its configuration and accept runtime updates.
6. Regenerate FRB bindings and update Flutter pages/controller.
7. Replace Rust tray/single-instance behavior with Flutter plugins.
8. Remove obsolete packaging/docs/configuration and update the build workspace.
9. Run Rust, Dart, Isar code generation, Linux build, and Wayland smoke tests.

## Open Questions

- Whether persistent tags, favorites, ratings, and collections are still required. This proposal assumes they are removed with SQLite unless reimplemented in Isar.
- Whether the selected wallpaper assignments should be stored as Isar records or remain only in the in-memory Flutter state.

