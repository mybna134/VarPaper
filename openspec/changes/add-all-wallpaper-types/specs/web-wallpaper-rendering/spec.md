# Spec Delta

## Purpose

Display local Wallpaper Engine Web projects as animated desktop backgrounds with browser rendering, project-scoped resource loading, property and audio callbacks, and controlled browser lifecycle across output changes.

## ADDED Requirements

### Requirement: Browser wallpaper rendering
The system SHALL render Web project HTML, CSS, JavaScript, Canvas, and WebGL content to the assigned desktop output. Relative local resources SHALL resolve within the project context, and output resizing/scaling SHALL update the browser viewport.

#### Scenario: Local animated web wallpaper
- **WHEN** user applies a Web fixture containing relative scripts/images, Canvas animation, and WebGL content
- **THEN** its content SHALL render without a foreground browser window and animate on the selected output

#### Scenario: Resize output
- **WHEN** the assigned monitor resolution or scale changes
- **THEN** the webpage viewport and desktop image SHALL adapt to the new pixel size without stretching a stale frame

### Requirement: Web properties and audio interface
The system SHALL deliver initial and changed project property values through the Wallpaper Engine user-property callback contract and system-playback spectrum through its audio-listener contract, including wallpaperPropertyListener.applyUserProperties and wallpaperRegisterAudioListener.

#### Scenario: User property edit
- **WHEN** user edits a supported declared Web property
- **THEN** the page SHALL receive its typed value and update appearance without reloading unrelated assignments

#### Scenario: Audio-responsive webpage
- **WHEN** a Web fixture registers an audio listener and system audio is playing
- **THEN** the callback SHALL receive ordered finite spectrum data
- **AND** silence or unavailable capture SHALL supply zero data without stopping the page

### Requirement: Web runtime isolation
The system SHALL restrict local resource access to project/configured shared roots, disable executable launching and unrelated host-file access, and block remote network requests by default with an explicit diagnostic. Browser failure SHALL be reported for the affected assignment without terminating the GUI or unrelated outputs.

#### Scenario: Browser renderer failure
- **WHEN** a wallpaper browser renderer exits unexpectedly
- **THEN** the assignment SHALL enter an error state with a retry action
- **AND** other wallpaper assignments and the GUI SHALL remain responsive

#### Scenario: External resource request
- **WHEN** a page requests a remote URL or an unrelated local file
- **THEN** the request SHALL be blocked and its reason SHALL be available for diagnosis
