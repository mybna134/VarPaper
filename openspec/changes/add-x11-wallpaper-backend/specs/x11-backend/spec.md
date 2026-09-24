## ADDED Requirements

### Requirement: Native X11 Session Selection
The system SHALL select the X11 wallpaper backend on a native X11 desktop session and SHALL retain the Wayland backend on a native Wayland session.

#### Scenario: Native X11 login
- **WHEN** the user runs VarPaper with an X11 session and a usable `DISPLAY`
- **THEN** the Rust engine SHALL connect to that X11 server
- **AND** it SHALL offer X11 monitors to the existing Flutter service

#### Scenario: Wayland login with XWayland available
- **WHEN** the user runs VarPaper in a Wayland session that also exposes `DISPLAY`
- **THEN** the engine SHALL keep the Wayland backend
- **AND** it SHALL NOT create X11 desktop windows through XWayland

### Requirement: X11 Monitor and Wallpaper Management
The system SHALL discover X11 monitors with RandR and SHALL create one background desktop window for each monitor receiving a wallpaper.

#### Scenario: Apply to one monitor
- **WHEN** the user applies a wallpaper to a named X11 monitor
- **THEN** the Rust engine SHALL render it within that monitor's geometry
- **AND** other monitors SHALL retain their current wallpapers
- **AND** the wallpaper window SHALL NOT intercept normal keyboard or pointer input

#### Scenario: Monitor layout change
- **WHEN** RandR reports a monitor addition, removal, resolution change, or position change
- **THEN** the engine SHALL update output events and window geometry
- **AND** it SHALL release rendering resources for removed monitors

### Requirement: X11 Playback Lifecycle
The X11 backend SHALL support the existing engine playback commands and cleanly release X11 and EGL resources.

#### Scenario: Clear and shutdown
- **WHEN** a wallpaper is cleared or the engine shuts down
- **THEN** playback SHALL stop for the affected monitor
- **AND** EGL surfaces and X11 windows SHALL be destroyed
- **AND** no desktop window SHALL remain after shutdown
