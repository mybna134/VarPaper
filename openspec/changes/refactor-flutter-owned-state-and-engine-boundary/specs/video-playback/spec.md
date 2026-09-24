## MODIFIED Requirements

### Requirement: Engine and Library Boundary

`wayvid-engine` and `wayvid-library` SHALL be independent sibling modules. Neither module SHALL depend on or call the other. The Rust service SHALL orchestrate scan results, source selection, and engine commands.

#### Scenario: Apply scanned wallpaper

- **WHEN** Flutter selects a wallpaper returned by `wayvid-library`
- **THEN** Flutter or the Rust service SHALL pass the source path and engine configuration to `wayvid-engine`
- **AND** `wayvid-engine` SHALL not query the library or its persistence layer

### Requirement: Flutter-Provided Engine Configuration

The playback engine SHALL not read application settings or configuration files. It SHALL receive its complete configuration from Flutter through the Rust service when created and SHALL accept updates while running.

#### Scenario: Engine creation

- **WHEN** Flutter creates the playback engine
- **THEN** the full playback, layout, HDR, hardware-decoding, FPS, volume, loop, and power configuration SHALL be supplied explicitly

#### Scenario: Runtime update

- **WHEN** Flutter changes an engine-relevant setting
- **THEN** the new value SHALL be sent to the engine
- **AND** the engine SHALL apply the update without reading settings or library state

