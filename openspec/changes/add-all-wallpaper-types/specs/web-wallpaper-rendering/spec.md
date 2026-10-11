# Spec Delta

## Purpose

Display local Wallpaper Engine Web projects as animated desktop backgrounds with browser rendering, project-scoped resource loading, property and audio callbacks, and controlled browser lifecycle across output changes.

## ADDED Requirements

### Requirement: Browser wallpaper rendering
When the optional Web support component is installed and compatible, the system SHALL render Web project HTML, CSS, JavaScript, Canvas, and WebGL content to the assigned desktop output. Relative local resources SHALL resolve within the project context, and output resizing/scaling SHALL update the browser viewport.

#### Scenario: Local animated web wallpaper
- **WHEN** Web support is installed and user applies a Web fixture containing relative scripts/images, Canvas animation, and WebGL content
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

### Requirement: Optional Web support installation
The system SHALL provide a localized "Web wallpaper support" setting with installation status, version, disk usage, progress, cancellation, retry, and uninstall actions. Web support SHALL be absent by default. Neither startup, scanning, applying, nor restoring wallpapers SHALL automatically download its runtime. Only an explicit install action SHALL download the architecture-compatible pinned Chromium/CEF runtime from the official CEF distribution source, using HTTPS and a packaged SHA-256 manifest. Required libraries, snapshots, resources, locales, and license files SHALL be installed together. The base application SHALL run without loading or linking the optional runtime.

#### Scenario: Default installation
- **WHEN** the application is first installed without the Web component
- **THEN** Web projects SHALL remain discoverable with previews and actual types
- **AND** applying a Web project SHALL report that Web wallpaper support must be installed in settings without downloading anything or replacing the current wallpaper

#### Scenario: Explicit component install
- **WHEN** user installs Web wallpaper support in settings
- **THEN** the pinned runtime SHALL download only from declared official HTTPS sources with progress feedback
- **AND** Web support SHALL become available only after all required files and checksums are validated and atomically committed

#### Scenario: Interrupted or invalid installation
- **WHEN** download is cancelled, disconnected, fails checksum validation, exceeds archive limits, lacks disk space, or targets an unsupported architecture
- **THEN** the setting SHALL show an actionable reason and allow a valid retry
- **AND** partial files SHALL NOT enable Web playback or damage an existing validated component

### Requirement: Web support uninstallation
The system SHALL let users uninstall the optional Web component to reclaim disk space. It SHALL prevent new Web playback, stop active Web sessions and their host/subprocess/audio consumers before removing runtime files, and remove component download/staging/cache data. Other wallpaper types SHALL continue working. User wallpaper projects, shared assets, and saved project/property descriptions SHALL remain intact.

#### Scenario: Uninstall during playback
- **WHEN** user uninstalls Web support while a Web wallpaper is active
- **THEN** its playback SHALL stop and component processes SHALL exit before deletion succeeds
- **AND** media/Scene outputs and user wallpaper files SHALL remain intact
- **AND** component disk usage SHALL be reclaimed and Web playback SHALL become unavailable

#### Scenario: Restore after uninstall
- **WHEN** the application restarts with saved Web assignments but no installed Web component
- **THEN** assignments SHALL report the missing component without downloading it or erasing their saved properties
- **AND** installing the component again SHALL permit restoration
