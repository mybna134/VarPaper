## ADDED Requirements

### Requirement: Engine and Library Boundary

`wayvid-engine` MAY depend on and call `wayvid-library` for project discovery/parsing, shared-asset lookup, and project/package resource access. `wayvid-library` SHALL NOT depend on `wayvid-engine`. Flutter and the Rust service SHALL orchestrate displayed scan results, source selection, and engine lifecycle commands. Persistent application settings and wallpaper assignments SHALL remain Flutter-owned.

#### Scenario: Apply scanned wallpaper

- **WHEN** Flutter selects a wallpaper returned by `wayvid-library`
- **THEN** Flutter or the Rust service SHALL pass the source path and engine configuration to `wayvid-engine`
- **AND** `wayvid-engine` MAY use library APIs to resolve the selected project and its rendering resources
- **AND** neither crate SHALL read or write persistent application settings or wallpaper assignments

### Requirement: Flutter-Provided Engine Configuration

The playback engine SHALL not read application settings or configuration files. It SHALL receive its complete configuration from Flutter through the Rust service when created and SHALL accept updates while running.

#### Scenario: Engine creation

- **WHEN** Flutter creates the playback engine
- **THEN** the full playback, layout, HDR, hardware-decoding, FPS, volume, loop, and power configuration SHALL be supplied explicitly

#### Scenario: Runtime update

- **WHEN** Flutter changes an engine-relevant setting
- **THEN** the new value SHALL be sent to the engine
- **AND** the engine SHALL apply the update without reading persistent application settings

