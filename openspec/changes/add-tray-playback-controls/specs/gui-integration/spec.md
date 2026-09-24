## ADDED Requirements

### Requirement: Tray Playback Controls

The Flutter application SHALL provide show, previous wallpaper, next wallpaper, pause or resume, mute or unmute, and quit actions in its system tray menu. Playback actions SHALL use controller state and update their labels when state or language changes.

#### Scenario: Switch wallpaper from the tray

- **WHEN** the user chooses next or previous wallpaper
- **THEN** each active display SHALL move through the wallpaper library in name order with wraparound
- **AND** displays with independent wallpapers SHALL retain independent assignments

#### Scenario: Pause and resume from the tray

- **WHEN** the user pauses playback from the tray
- **THEN** the engine SHALL pause active wallpapers and the menu SHALL offer Resume
- **AND** newly applied wallpapers SHALL remain paused until resumed

#### Scenario: Mute and unmute from the tray

- **WHEN** the user changes mute state from the tray
- **THEN** the setting SHALL be persisted and applied to active players immediately
- **AND** the menu SHALL offer the opposite action

#### Scenario: Playback is unavailable

- **WHEN** the engine, display, or wallpaper library required by an action is unavailable
- **THEN** the corresponding tray action SHALL be disabled
