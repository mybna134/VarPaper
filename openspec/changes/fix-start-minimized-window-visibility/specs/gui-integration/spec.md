# Spec Delta

## ADDED Requirements

### Requirement: Startup Window Visibility
The GUI application SHALL apply the persisted "Start minimized" preference before presenting its main window. When enabled, the main window SHALL remain hidden throughout startup without flashing or taking focus, while application initialization, wallpaper restoration according to existing settings, and tray initialization continue. When disabled or no preference is saved, the main window SHALL open normally.

#### Scenario: Start with the saved preference enabled
- **WHEN** the application starts with "Start minimized" saved as enabled
- **THEN** the main window remains hidden before and after the first UI frame
- **AND** startup does not take focus from the active application
- **AND** background initialization and configured wallpaper restoration continue
- **AND** tray controls are available on desktops that support the tray

#### Scenario: Normal startup
- **WHEN** the application starts with "Start minimized" disabled or no saved preference
- **THEN** the main window is shown and focused with the configured window dimensions

#### Scenario: Preference survives an application restart
- **WHEN** the user changes "Start minimized", quits, and launches the application again
- **THEN** the new launch uses the saved value to decide the initial main-window visibility

### Requirement: Restore Hidden Startup Window
The GUI application SHALL allow the user to show and focus a window hidden at startup using the tray icon or the tray menu actions for showing the application, library, or settings. On desktops with a taskbar, the restored window SHALL participate in the taskbar normally. The startup preference SHALL affect subsequent launches, without preventing explicit window restoration in the current session.

#### Scenario: Restore using a tray action
- **WHEN** the window is hidden after minimized startup and the user clicks the tray icon or selects the show, library, or settings action
- **THEN** the main window becomes visible and focused
- **AND** it is eligible for normal taskbar display
- **AND** library and settings actions open their requested page

#### Scenario: Toggle a restored window
- **WHEN** the user clicks the tray icon while the restored main window is visible
- **THEN** the window is hidden and the application continues running
- **AND** another tray icon click shows and focuses the window again
