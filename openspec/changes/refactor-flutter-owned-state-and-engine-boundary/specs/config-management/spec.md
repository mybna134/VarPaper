## REMOVED Requirements

### Requirement: YAML Configuration Format

**Reason**: User-visible settings are no longer persisted as YAML. Flutter Isar is the only settings store.

### Requirement: Configuration File Locations

**Reason**: The application no longer reads active settings from `~/.config/wayvid/config.yaml` or system-wide YAML files.

### Requirement: Example Configuration Generation

**Reason**: CLI and YAML configuration workflows are removed.

### Requirement: Hot Configuration Reload

**Reason**: Configuration changes originate in Flutter and are pushed directly to the running engine.

## ADDED Requirements

### Requirement: Isar Settings Store

The application SHALL use Flutter `isar_community` collections as the persistent source of truth for user settings.

#### Scenario: Missing Isar records

- **WHEN** a settings record does not exist
- **THEN** Flutter SHALL create it with validated defaults

#### Scenario: Settings update

- **WHEN** Flutter updates a setting
- **THEN** the update SHALL be committed to Isar
- **AND** the affected engine configuration SHALL be sent to Rust if an engine is running

### Requirement: Explicit Engine Configuration

The engine SHALL receive a complete validated configuration from Flutter at creation time and SHALL accept subsequent configuration updates from Flutter.

#### Scenario: Invalid value

- **WHEN** Flutter attempts to send an out-of-range value
- **THEN** Flutter SHALL clamp or reject it before persistence
- **AND** Rust SHALL not receive an invalid engine configuration

