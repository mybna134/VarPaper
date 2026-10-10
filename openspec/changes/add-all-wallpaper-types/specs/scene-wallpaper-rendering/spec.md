# Spec Delta

## Purpose

Render Wallpaper Engine Scene projects as live desktop wallpapers, including packaged assets, layered effects, animated objects, scripts, clocks, and audio visualization, with explicit compatibility and failure behavior.

## ADDED Requirements

### Requirement: Scene rendering features
The system SHALL render Scene projects using resource packages, textures, materials, multipass effects, camera/layer transforms, image animation, particles, text, and sound for the feature subset implemented by the pinned reference revision. Each listed feature SHALL have a reproducible fixture; unsupported required features SHALL produce a compatibility error rather than silent successful blank output.

#### Scenario: Composite animated scene
- **WHEN** a valid fixture contains layered images, an animated texture, an effect, particles, text, and sound
- **THEN** layers and effects SHALL render in the declared order, animation and particles SHALL advance, text SHALL be visible, and sound SHALL obey playback controls

#### Scenario: Unsupported required feature
- **WHEN** a Scene requires an unsupported object, shader feature, or script API
- **THEN** application SHALL report the feature and asset involved
- **AND** SHALL NOT declare full compatibility by silently dropping required content

### Requirement: Scene scripts and properties
The system SHALL execute SceneScript initialization and update hooks with project properties and supported layer bindings. Property edits SHALL update the associated rendering and SHALL remain isolated to their assignment.

#### Scenario: Property-driven appearance
- **WHEN** user changes a declared Scene color or numeric property
- **THEN** the associated script/material SHALL receive the validated value and update the wallpaper
- **AND** the value SHALL be restored on restart

### Requirement: Scene audio visualization
The system SHALL provide live system-playback audio spectrum and level data to supported Scene effects and script interfaces. Audio capture SHALL use playback monitoring rather than microphone capture by default, SHALL be shared across consumers, and SHALL yield finite zero data when silence or capture unavailability occurs.

#### Scenario: Frequency-responsive scene
- **WHEN** known low/high frequency tones play through the selected system output while an audio-responsive Scene fixture runs
- **THEN** the corresponding spectrum bands SHALL respond and drive its visible effect
- **AND** muting wallpaper-generated sound SHALL NOT disable visualization of other system audio

#### Scenario: Capture unavailable
- **WHEN** the playback monitor is unavailable or disconnects
- **THEN** visualization inputs SHALL become zero and the scene SHALL continue rendering
- **AND** the application SHALL expose a nonfatal capture diagnostic and recover when the monitor returns

### Requirement: Scene time and clock behavior
The system SHALL provide monotonic animation time, frame delta, and system date/local time to supported SceneScript and text bindings. Pausing SHALL freeze animation updates; resuming SHALL use current wall-clock time without incorporating the entire pause interval into animation delta.

#### Scenario: Local clock text
- **WHEN** a Scene clock fixture reads system date and local time
- **THEN** its text SHALL match the configured system timezone and update across minute/day changes

#### Scenario: Pause and resume clock scene
- **WHEN** a clock and animated Scene is paused and later resumed
- **THEN** animation SHALL remain frozen during pause and resume without a large time jump
- **AND** the first resumed clock update SHALL show the current local time
