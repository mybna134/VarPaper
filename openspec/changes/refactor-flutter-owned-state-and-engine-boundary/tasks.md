## 1. Specification and dependency boundary

- [x] 1.1 Validate this OpenSpec change and review affected existing specifications.
- [x] 1.2 Define Flutter Isar collections and the serialized FRB DTOs.
- [x] 1.3 Define the engine configuration and runtime update contract.
- [x] 1.4 Define the scan, preview, apply, and engine lifecycle API.

## 2. Flutter persistence and lifecycle

- [x] 2.1 Add `isar_community` and required generator/runtime dependencies.
- [x] 2.2 Implement Isar initialization and settings repository.
- [x] 2.3 Move GUI, playback, power, library-folder, and wallpaper assignment settings to Isar.
- [x] 2.4 Update Riverpod providers/controller to load Isar before service/engine creation.
- [x] 2.5 Persist settings changes and push engine-relevant changes to Rust.
- [x] 2.6 Replace Rust tray integration with a Flutter tray plugin.
- [x] 2.7 Keep engine lifecycle/single-instance ownership in Flutter without IPC.

## 3. FRB service boundary

- [x] 3.1 Replace YAML/settings-based service initialization with explicit Flutter DTOs.
- [x] 3.2 Add scan-folder, scan-Workshop, monitor, preview, and source metadata APIs.
- [x] 3.3 Add create-engine, update-engine-config, apply, pause, resume, clear, and stop APIs.
- [x] 3.4 Ensure the service is the only orchestrator between library and engine.
- [x] 3.5 Regenerate Dart and Rust FRB bindings.

## 4. wayvid-library refactor

- [x] 4.1 Remove `rusqlite` and SQLite schema/database code.
- [x] 4.2 Remove tags, collections, ratings, usage persistence, and database-specific models.
- [x] 4.3 Keep folder scanning, Workshop scanning, metadata extraction, and in-memory results.
- [x] 4.4 Implement safe preview extraction and image preview path handling.
- [x] 4.5 Ensure `wayvid-library` has no dependency on `wayvid-engine`.

## 5. wayvid-engine refactor

- [x] 5.1 Remove `wayvid-core` dependency and define engine-owned domain/config types.
- [x] 5.2 Accept complete configuration during engine creation.
- [x] 5.3 Add runtime configuration update commands and apply them to sessions.
- [x] 5.4 Ensure engine only consumes engine configuration and source paths/IDs supplied by the service.
- [x] 5.5 Ensure `wayvid-engine` has no dependency on `wayvid-library`.
- [x] 5.6 Preserve EGL cleanup and MPV locale fixes.

## 6. Remove legacy components

- [x] 6.1 Remove `crates/wayvid-ctl` from the workspace and delete its source.
- [x] 6.2 Remove Rust IPC server, IPC DTOs, socket helpers, and single-instance code.
- [x] 6.3 Remove `crates/wayvid-core` and migrate required types to their owning module/service DTOs.
- [x] 6.4 Remove YAML settings/configuration from active runtime code.
- [x] 6.5 Remove obsolete CLI, IPC, daemon, and SQLite packaging/documentation.

## 7. Verification

- [ ] 7.1 Add Flutter tests for Isar settings load/save and engine config updates.
- [ ] 7.2 Add Rust tests for scan DTOs, preview results, engine config, and update commands.
- [x] 7.3 Run Dart format/analyze/test and Isar code generation.
- [x] 7.4 Run Cargo format/check/test for the remaining workspace.
- [x] 7.5 Build the Linux debug and release bundles.
- [ ] 7.6 Run Wayland smoke tests for scan, preview, engine creation, config update, apply, pause, resume, and clear.
- [ ] 7.7 Update this task list after implementation and validate the final OpenSpec state.
