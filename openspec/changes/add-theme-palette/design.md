## Context

The Flutter shell creates light and dark `ThemeData` with `Colors.teal` as `colorSchemeSeed`. Settings are stored through `SettingsDto` and `AppSettingsRecord` in Isar. FlClash's theme page shows selectable preset swatches and a custom palette dialog.

## Decisions

- Store the selected color as an opaque ARGB integer, custom swatches as a list of integers, and the tonal style as a stable string in the GUI settings record. Keep the seed color non-null with teal as the default; existing Isar records must receive defaults when new fields are absent. Unknown tonal-style values fall back to the default.
- Use Flutter's Material color scheme generation for both brightness modes. Changing the seed recolors the app without changing the user's theme mode.
- Offer a small fixed set of preset colors plus custom swatches, with one selected marker, custom-swatch removal, a tonal-style selector, and a reset to teal. Removing the selected custom swatch reverts the seed to teal. The color picker shall let users choose an exact color and preview it before saving. Avoid adding a color-picker package if Flutter controls are sufficient.
- Keep all palette state in the existing settings/controller flow. Do not send palette-only changes to the Rust engine; it has no visual theme setting.

## Risks / Trade-offs

- An Isar schema change requires regenerating `settings_store.g.dart`; verify an existing database opens and reads the new fields with defaults.
- Very light or dark custom seeds can reduce contrast in a raw swatch, so the selected indicator should use a contrasting color from the generated scheme.
