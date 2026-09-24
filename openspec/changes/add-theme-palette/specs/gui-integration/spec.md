## ADDED Requirements

### Requirement: GUI Theme Palette

The Flutter GUI SHALL let the user choose and persist a seed color for the application's Material color scheme from Settings > Appearance.

#### Scenario: Select a preset color

- **WHEN** the user selects a preset palette swatch
- **THEN** the selected swatch SHALL be visibly marked
- **AND** the active interface SHALL use a color scheme generated from that seed immediately
- **AND** the existing light, dark, or system theme mode SHALL remain unchanged

#### Scenario: Add and select a custom color

- **WHEN** the user confirms a color in the custom picker
- **THEN** that color SHALL be available as a swatch and become the selected seed
- **AND** the picker SHALL show a preview before confirmation

#### Scenario: Remove a custom color

- **WHEN** the user removes a custom swatch
- **THEN** that swatch SHALL disappear from the palette and saved settings
- **AND** if it was selected, the GUI SHALL select the teal default

#### Scenario: Select a tonal style

- **WHEN** the user selects a tonal style
- **THEN** the active interface SHALL regenerate its color scheme using the selected style
- **AND** the style SHALL be saved for the next launch

#### Scenario: Restore palette on startup

- **WHEN** the GUI restarts after a palette selection
- **THEN** the saved seed color, custom swatches, and tonal style SHALL be restored from Isar
- **AND** both light and dark themes SHALL derive their color schemes from the saved seed

#### Scenario: Default and reset

- **WHEN** there is no saved palette or the user activates reset
- **THEN** the GUI SHALL use its current teal seed color
- **AND** the reset action SHALL remove custom swatches
- **AND** the reset action SHALL restore the default tonal style

#### Scenario: Existing installation

- **WHEN** an existing Isar settings record has no palette fields
- **THEN** the GUI SHALL open the record successfully and use the teal default
