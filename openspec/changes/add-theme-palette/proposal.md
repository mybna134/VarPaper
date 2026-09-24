# Change: Add theme palette settings

## Why

VarPaper currently uses a fixed teal seed color for both light and dark themes. Users cannot adjust the interface palette from Settings. FlClash provides a useful reference for choosing a seed color from preset and custom swatches.

## What Changes

- Add a localized palette section under Appearance with preset color swatches, a custom color picker, custom-swatch removal, a tonal-style selector, selected-state feedback, and a reset action.
- Persist the selected seed color, custom swatches, and tonal style in the existing Flutter-owned Isar settings record. Keep teal and the current Material style as defaults for existing users.
- Generate the light and dark Material color schemes from the selected seed color and update the interface immediately.
- Keep this setting separate from wallpaper colors, playback, and the existing light/dark/system mode.

## Impact

- Affected specs: `gui-integration`
- Affected code: `lib/main.dart`, `lib/src/storage/settings_store.dart` and its generated Isar schema, `lib/src/l10n.dart`, focused Flutter tests
- No Rust service or wallpaper-engine change
