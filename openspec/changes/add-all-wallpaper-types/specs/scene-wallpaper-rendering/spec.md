# Spec Delta

## Purpose

Render Wallpaper Engine Scene projects as live desktop wallpapers, for the implemented Rust subset, including packaged assets, layered effects, animated textures, core sprite particles, text, sound and basic scripts, with explicit compatibility and failure behavior.

## ADDED Requirements

### Requirement: Scene rendering features
The system SHALL render Scene projects using resource packages, textures, materials, multipass effects, camera/layer transforms, image animation, particles, text, and sound for the implemented Rust subset documented in docs/src/dev/rust-scene.md. Advanced particle modes, full script compatibility, live system-audio visualization and injected local-clock bindings are deferred. Each listed feature SHALL have a reproducible fixture; unsupported required features SHALL produce a compatibility error rather than silent successful blank output.

#### Scenario: Composite animated scene
- **WHEN** a valid fixture contains layered images, an animated texture, an effect, particles, text, and sound
- **THEN** layers and effects SHALL render in the declared order, animation and particles SHALL advance, text SHALL be visible, and sound SHALL obey playback controls

#### Scenario: Project layer size
- **WHEN** a Scene image layer specifies a size different from its texture dimensions
- **THEN** the renderer SHALL use the project size before applying layer scale, and SHALL use texture/model dimensions only as a fallback when size is unspecified

#### Scenario: Parent visibility
- **WHEN** any ancestor layer has visible set to false
- **THEN** its descendant layers SHALL be hidden, regardless of their own visible values

#### Scenario: Unsupported required feature
- **WHEN** a Scene requires an unsupported object, shader feature, or script API
- **THEN** application SHALL report the feature and asset involved
- **AND** SHALL NOT declare full compatibility by silently dropping required content

### Requirement: Scene scripts and properties
The system SHALL execute SceneScript initialization and update hooks with project properties and supported layer bindings. Initial project properties SHALL reach supported bindings and SHALL remain isolated to their assignment. Live GUI property editing and persistence of new overrides are deferred.

#### Scenario: Initial property-driven appearance
- **WHEN** a Scene declares an initial color or numeric property
- **THEN** its supported script binding SHALL receive the validated value
- **AND** its property state SHALL remain isolated from other assignments

### Requirement: Current Scene audio support boundary
The system SHALL expose Scene audio visualization as unavailable in current delivery. The application SHALL NOT start playback monitoring for Scene assignments. Internal snapshot bindings SHALL NOT be advertised as live system-audio support. Wallpaper-generated once/loop sound SHALL obey pause, mute and volume controls.

#### Scenario: Scene without live audio input
- **WHEN** an implemented Scene receives no fresh internal audio snapshot
- **THEN** its audio uniform and script buffers SHALL contain finite zero values
- **AND** the application SHALL NOT claim live system-audio visualization support

### Requirement: Scene animation time
The system SHALL provide monotonic animation time and bounded frame delta to supported Rust Scene bindings. Pausing SHALL freeze animation; resuming SHALL exclude the pause interval from animation delta. Injectable wall-clock/timezone bindings are deferred.

#### Scenario: Pause and resume animated scene
- **WHEN** an animated Scene is paused and resumed
- **THEN** animation SHALL remain frozen during pause and resume without a large time jump
