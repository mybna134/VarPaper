## REMOVED Requirements

### Requirement: GUI-Daemon IPC Communication

**Reason**: The application no longer has a separate daemon or Unix Socket IPC layer. Flutter calls the Rust service through flutter_rust_bridge.

### Requirement: Daemon Process Management

**Reason**: Engine lifecycle is owned by the Flutter application and the engine runs inside the Rust service.

## MODIFIED Requirements

### Requirement: Flutter-Owned Settings Persistence

The Flutter application SHALL persist all user-visible settings using `isar_community` and SHALL load them before creating the Rust service or rendering the main UI.

#### Scenario: Save setting change

- **WHEN** the user changes a GUI, playback, power, library, or wallpaper-assignment setting
- **THEN** Flutter SHALL write the new value to Isar
- **AND** Flutter SHALL push engine-relevant values to the running engine

#### Scenario: Start with existing Isar state

- **WHEN** the application starts
- **THEN** Flutter SHALL load settings from Isar
- **AND** SHALL use those values to create the engine configuration

#### Scenario: No legacy migration

- **WHEN** YAML or SQLite state from an older installation exists
- **THEN** the application SHALL ignore it
- **AND** SHALL initialize missing Isar records with defaults

### Requirement: Rust-Requested Library Scan

Flutter SHALL initiate library scans through the Rust service and SHALL receive scan results through flutter_rust_bridge. The Rust service SHALL perform scanning without SQLite persistence.

#### Scenario: Startup scan

- **WHEN** Flutter initializes the library view
- **THEN** Flutter SHALL request a folder and/or Workshop scan through FRB
- **AND** Rust SHALL return wallpaper metadata and source paths
- **AND** Flutter SHALL own the displayed library state

#### Scenario: Folder scan

- **WHEN** the user adds or refreshes a folder
- **THEN** Flutter SHALL send the path to Rust
- **AND** Rust SHALL scan the path and return the result
- **AND** no SQLite database SHALL be required

### Requirement: Rust-Requested Preview Generation

Flutter SHALL request wallpaper previews through FRB. Existing image previews SHALL be returned as safe paths or platform-supported handles where possible; video previews SHALL be generated from one extracted frame.

#### Scenario: Image preview

- **WHEN** a scanned item has an existing image preview
- **THEN** Rust SHALL return a safe preview path/handle
- **AND** Flutter SHALL display it without requiring the engine

#### Scenario: Video preview

- **WHEN** a scanned item is a video without an existing preview
- **THEN** Flutter SHALL request a preview from Rust
- **AND** Rust SHALL extract one frame and return a safe path or byte result

### Requirement: Flutter-Owned Engine Lifecycle

Flutter SHALL create, update, pause, resume, and stop the engine through the Rust service. Each engine creation SHALL include a complete engine configuration.

#### Scenario: Create engine

- **WHEN** Flutter starts playback
- **THEN** Flutter SHALL construct the engine configuration from Isar-backed state
- **AND** SHALL pass that configuration to Rust
- **AND** Rust SHALL create the engine using only the supplied configuration

#### Scenario: Update engine configuration

- **WHEN** an engine-relevant setting changes while the engine is running
- **THEN** Flutter SHALL persist the setting to Isar
- **AND** SHALL send the updated configuration to Rust
- **AND** Rust SHALL apply it to the running engine

### Requirement: Flutter-Owned Tray and Single Instance

The Flutter application SHALL own system tray integration and single-instance behavior without Unix Socket IPC.

#### Scenario: Tray action

- **WHEN** the user activates a tray action
- **THEN** the Flutter tray plugin SHALL deliver the action to Riverpod/controller state
- **AND** Flutter SHALL call the Rust service directly when playback changes

#### Scenario: Second launch

- **WHEN** a second application instance is started
- **THEN** Flutter SHALL detect the existing instance
- **AND** SHALL activate the existing window through the Flutter desktop integration
- **AND** SHALL not use a Unix Socket IPC request

