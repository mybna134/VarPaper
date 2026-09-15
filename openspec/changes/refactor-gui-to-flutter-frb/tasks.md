## 1. OpenSpec and Rust service boundary

- [x] 1.1 Add and validate the proposal, design, and capability deltas.
- [x] 1.2 Convert `wayvid-gui` from an iced binary to a Rust library with FRB dependencies and a public bridge module.
- [x] 1.3 Extract stable DTOs, service errors, initialization snapshots, commands, and event types.
- [x] 1.4 Move engine/library/settings/tray/IPC orchestration behind `WayvidService` without changing engine or JSON IPC semantics.
- [x] 1.5 Add Rust unit tests for DTO conversion, settings compatibility, event mapping, and service command behavior.

## 2. Flutter application

- [x] 2.1 Make the repository root the Linux Flutter project and configure FRB v2 Cargokit generation.
- [x] 2.2 Add generated Dart bindings and a testable Riverpod-provided `WayvidController`.
- [x] 2.2a Keep the Flutter app at repository root and the FRB service in top-level `rust/`.
- [x] 2.2b Fix collapsed/expanded `NavigationRail` width and control constraints.
- [ ] 2.3 Implement Library, Folders, Monitors, Settings, and About pages with English/Chinese localization.
- [x] 2.4 Implement thumbnails, filtering, target monitor selection, double-click apply, folder selection, and error/status states.
- [x] 2.5 Implement window sizing, start-minimized, minimize-to-tray, single-instance show, and tray event handling.
- [ ] 2.6 Add Dart unit/widget tests for controller state transitions and all page workflows.

## 3. Build, packaging, and documentation

- [x] 3.1 Remove iced-only dependencies from the active GUI target and update workspace metadata.
- [ ] 3.2 Add Flutter build scripts and CI checks for codegen, analyze, tests, and Linux release builds.
- [ ] 3.3 Update AppImage, Debian, AUR, Nix, desktop, systemd, install, and uninstall packaging for the Flutter bundle.
- [x] 3.4 Update README and user/developer GUI documentation to describe Flutter + FRB.

## 4. Verification

- [x] 4.1 Run Rust format, workspace tests, and Flutter analyze/tests.
- [x] 4.2 Build the release Flutter Linux bundle and verify the `wayvid-gui` executable, `lib`, and `data` files.
- [ ] 4.3 Run real Wayland acceptance checks for playback, monitors, tray, restore, single instance, settings, and `wayvid-ctl`.
- [ ] 4.4 Mark all completed tasks and validate the final OpenSpec state.
