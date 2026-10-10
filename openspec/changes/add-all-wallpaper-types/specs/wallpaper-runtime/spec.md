# Spec Delta

## Purpose

Provide consistent application, controls, persistence, and desktop presentation for all supported wallpaper types on Wayland and X11, maintaining output independence and releasing resources reliably during transitions.

## ADDED Requirements

### Requirement: Typed desktop wallpaper application
The system SHALL select a renderer from the actual wallpaper type and present Video, Scene, Web, Image, and GIF on supported Wayland and native X11 desktops. Image SHALL remain displayed indefinitely and GIF SHALL animate and obey loop configuration. Type-specific options SHALL be identified and unsupported controls SHALL not silently claim an effect.

#### Scenario: Mixed output types
- **WHEN** one monitor displays Video and another displays Scene or Web
- **THEN** both SHALL render independently below application windows
- **AND** applying to one monitor SHALL leave the other assignment unchanged

#### Scenario: Local image and GIF
- **WHEN** user applies an Image and a GIF to separate outputs
- **THEN** the Image SHALL remain visible without ending playback and the GIF SHALL animate using the configured layout and loop behavior

### Requirement: Common playback controls
The system SHALL apply pause/resume, static-frame retention, mute/volume, supported layout, and configured frame limits consistently to supported wallpaper sessions. Static images SHALL not trigger continuous rendering. Existing battery-pause and tray controls SHALL work for Scene/Web as well as media.

#### Scenario: Pause mixed wallpapers
- **WHEN** user pauses all outputs using tray or battery-pause policy
- **THEN** media, Scene animation/scripts/audio, and Web animation/scripts/audio SHALL pause while their last frames remain visible
- **AND** resume SHALL restore their playback without losing assignments

### Requirement: Assignment persistence and truthful status
The system SHALL persist sufficient project/type/property information to restore each supported assignment. Existing path-only media assignments SHALL remain restorable. Success SHALL be reported and persisted only after renderer initialization and first-frame readiness.

#### Scenario: Restart with project assignments
- **WHEN** the application restarts with saved Scene/Web assignments and property overrides
- **THEN** the correct projects SHALL be loaded with the saved properties on their target outputs

#### Scenario: Failed replacement
- **WHEN** a replacement project fails validation or initialization
- **THEN** an output-specific error SHALL be shown and the prior working assignment SHALL remain active and persisted

### Requirement: Lifecycle cleanup
The system SHALL release renderers, textures, framebuffers, scripts, audio consumers, browsers and helper processes when switching/clearing wallpapers, removing outputs, or shutting down. Reconnecting outputs SHALL restore their assignment; repeated lifecycle operations SHALL NOT accumulate native resources.

#### Scenario: Repeated type transitions
- **WHEN** a lifecycle fixture repeatedly switches Video to Scene to Web to Image/GIF and clears the assignment
- **THEN** live renderer, GPU, audio and browser resource counts SHALL return to the baseline after cleanup

### Requirement: Library presentation
The system SHALL offer accurate type labels and filters for the five supported types, type-specific preview fallbacks, property editing, and actionable loading/compatibility errors in supported interface languages.

#### Scenario: Filter Web projects
- **WHEN** user chooses the Web type filter
- **THEN** only Web items SHALL be shown and their details SHALL expose compatibility and property information

### Requirement: Packaged feature availability
Official .deb, Flatpak, and Arch packages SHALL contain or declare all dependencies and auxiliary resources required for the five supported wallpaper types without relying on the developer's reference checkout.

#### Scenario: Installed package smoke test
- **WHEN** each package is installed in its supported test environment without a linux-wallpaperengine checkout
- **THEN** all five type fixtures SHALL load and render on a supported desktop
- **AND** missing optional audio monitoring SHALL degrade visualization gracefully rather than prevent rendering
