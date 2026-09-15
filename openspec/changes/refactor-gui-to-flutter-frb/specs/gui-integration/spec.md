## ADDED Requirements

### Requirement: Root Flutter Application Layout

The repository SHALL use the root directory as the Flutter application and SHALL keep the Rust FRB service in a top-level `rust/` crate, while shared Rust libraries remain under `crates/`.

#### Scenario: Build from repository root

- **WHEN** a developer runs Flutter commands from the repository root
- **THEN** Flutter resolves `pubspec.yaml`, Linux runner files, generated bindings, and Cargokit without a nested Flutter path

#### Scenario: Rust service ownership

- **WHEN** Flutter invokes a backend operation
- **THEN** the generated FRB bindings call the top-level `rust/` service crate
- **AND** the service continues to use the shared crates for engine and library behavior

### Requirement: Stable Collapsible Navigation Layout

The GUI SHALL keep the navigation rail within a fixed collapsed or expanded width and SHALL avoid rendering expanded labels or controls inside the collapsed constraint.

#### Scenario: Collapse the navigation rail

- **WHEN** the user collapses the sidebar
- **THEN** the rail uses its compact width and icon-only controls
- **AND** no title, button label, or navigation destination overflows into the content area

#### Scenario: Expand the navigation rail

- **WHEN** the user expands the sidebar
- **THEN** the rail uses its extended width and displays navigation labels and engine controls without changing page state

### Requirement: Flutter Rust Service GUI Boundary

The GUI SHALL render with Flutter and SHALL access Rust playback, library, settings, monitor, tray, and IPC operations through a typed `flutter_rust_bridge` service facade.

#### Scenario: Flutter starts the service

- **WHEN** `wayvid-gui` starts
- **THEN** Flutter initializes one Rust service instance
- **AND** the service loads compatible settings and reports the initial engine, monitor, library, and tray state

#### Scenario: Rust operation failure

- **WHEN** a Rust operation fails
- **THEN** the bridge returns a structured error code and message
- **AND** Flutter displays the error without terminating the application

### Requirement: Flutter GUI Functional Parity

The Flutter GUI SHALL provide the existing Library, Folders, Monitors, Settings, About, thumbnail, target-monitor, engine-control, theme, language, tray, and wallpaper-restore workflows.

#### Scenario: Apply a selected wallpaper

- **WHEN** the user double-clicks a wallpaper or presses Apply
- **THEN** Flutter calls the Rust service with the wallpaper and selected output
- **AND** the existing playback engine applies the wallpaper
- **AND** the per-monitor restore state is persisted

#### Scenario: Minimize to tray

- **WHEN** the user closes the window with minimize-to-tray enabled
- **THEN** Flutter hides the window
- **AND** Rust keeps the engine and ksni tray alive

#### Scenario: Restore from tray or second instance

- **WHEN** the user selects Show Window in the tray or starts a second GUI instance
- **THEN** the existing GUI window is shown and focused
- **AND** no second playback engine is created

### Requirement: Flutter Linux Distribution

The project SHALL build and package the Flutter Linux bundle as `wayvid-gui` while preserving the existing desktop entry, systemd entry, installation commands, and `wayvid-ctl` IPC compatibility.

#### Scenario: Release bundle contains runtime files

- **WHEN** a Linux release package is built
- **THEN** it contains the Flutter executable, Flutter `lib` runtime, Flutter `data` assets, Rust bridge library, and existing native playback dependencies

#### Scenario: Existing settings and IPC remain usable

- **WHEN** a user upgrades from the iced GUI
- **THEN** the existing settings YAML is loaded without migration loss
- **AND** existing `wayvid-ctl` commands continue to communicate through the same JSON IPC protocol
