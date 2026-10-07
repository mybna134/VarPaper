# Spec Delta

## ADDED Requirements

### Requirement: Complete Wayland Playback Teardown
The system SHALL stop playback and release all owned rendering and surface protocol resources for a wallpaper that is cleared, an output that is removed, or a layer surface that is closed by the compositor. Replacing a wallpaper on an existing output SHALL preserve its native layer surface and underlying surface. Teardown SHALL preserve playback and surface ownership on other outputs.

#### Scenario: Clear a wallpaper while its output remains connected
- **WHEN** a user clears the wallpaper on an existing output
- **THEN** playback and its rendering resources SHALL be released
- **AND** the owned layer surface and underlying surface SHALL be destroyed after playback cleanup
- **AND** the output binding SHALL remain available while the output remains connected
- **AND** reapplying a wallpaper SHALL create new layer and underlying surfaces

#### Scenario: Output removal during playback
- **WHEN** the compositor removes an output that has active wallpaper playback
- **THEN** decoding for that output SHALL stop
- **AND** its playback session and rendering surface SHALL be released
- **AND** its owned layer surface, underlying surface, and releasable output binding SHALL be released
- **AND** the application SHALL still receive the output removal notification

#### Scenario: Compositor closes a wallpaper surface
- **WHEN** the compositor closes a wallpaper layer surface
- **THEN** the corresponding playback session and owned surface resources SHALL be released
- **AND** the system SHALL NOT keep decoding for that closed surface

#### Scenario: Clear followed by removal or closure
- **WHEN** a previously cleared output subsequently produces a removal or surface closure event
- **THEN** teardown SHALL complete without double destruction or recreating playback

#### Scenario: Late event from a replaced surface
- **WHEN** a configure, closure, or frame callback arrives for an old surface after replacement on the same output
- **THEN** the event SHALL NOT modify, resume, or destroy the replacement playback surface

#### Scenario: Engine exits with active surfaces
- **WHEN** the engine shuts down or exits because of an event-loop failure
- **THEN** playback resources SHALL be released before their owned Wayland surfaces
- **AND** those surfaces SHALL be explicitly destroyed before disconnecting from the compositor
