# Spec Delta

## ADDED Requirements

### Requirement: Deterministic Rendering Resource Lifetime
The system SHALL release owned decoding and rendering resources when playback ownership ends on either supported Linux display backend. Cleanup SHALL respect native graphics lifetime constraints and SHALL be safe to request more than once.

#### Scenario: Clear and reapply a wallpaper
- **WHEN** a user clears a wallpaper and later applies another wallpaper to the same output
- **THEN** the cleared playback's decoder and rendering resources SHALL be released before its native window is destroyed
- **AND** the cleared wallpaper's native window SHALL NOT be retained
- **AND** reapplying a wallpaper SHALL create a new native window
- **AND** repeated cycles SHALL NOT accumulate live resources from earlier playback sessions

#### Scenario: Replace a wallpaper on the same output
- **WHEN** a user replaces the wallpaper while its output and native window remain valid
- **THEN** the native window SHALL retain its identity
- **AND** initialized playback SHALL reuse its existing rendering surface through source hot-swap
- **AND** source replacement SHALL NOT destroy and recreate the native window

#### Scenario: Stop and restart the integrated engine
- **WHEN** the integrated engine stops while one or more outputs have initialized playback
- **THEN** its playback resources SHALL be released before its graphics context and display connection
- **AND** restarting the engine SHALL NOT retain resources from the previous engine instance
- **AND** resources owned by the application UI SHALL remain usable

#### Scenario: Initialization or event-loop failure
- **WHEN** rendering initialization or an engine event loop fails after acquiring native resources
- **THEN** resources acquired by the failed operation SHALL be released on the error exit
- **AND** the original failure SHALL be reported
- **AND** a cleanup failure SHALL produce diagnostics without preventing cleanup attempts for other owned resources

#### Scenario: Repeated cleanup
- **WHEN** cleanup is requested again for an already released playback resource
- **THEN** the system SHALL NOT destroy the same native resource twice

#### Scenario: Playback on unaffected outputs
- **WHEN** one output's playback resources are released
- **THEN** playback on other active outputs SHALL remain renderable
- **AND** applying a replacement source to an existing active output SHALL preserve the current surface reuse behavior
