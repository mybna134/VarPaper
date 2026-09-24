# Change: Add playback controls to the system tray

## Why

Users should be able to control wallpaper playback without opening the main window.

## What Changes

- Add previous, next, pause, resume, mute, and unmute tray actions alongside show and quit.
- Keep tray labels and availability synchronized with controller state and language.
- Make mute changes effective on currently playing wallpapers.

## Impact

- Affected spec: `gui-integration`
- Affected code: Flutter controller and tray menu, settings patch, mpv runtime config
