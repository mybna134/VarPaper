## ADDED Requirements

### Requirement: Arch Linux release package

The release workflow SHALL build an x86_64 Arch Linux package in `.pkg.tar.zst` format from the VarPaper Linux release bundle. The package SHALL declare its required Arch runtime packages and install the application, launcher, desktop entry, icon, and license notices.

#### Scenario: Arch package build
- **WHEN** the release workflow builds Linux packages for a version
- **THEN** it creates a `.pkg.tar.zst` package for x86_64 using Arch package tooling
- **AND** the package metadata declares the runtime dependencies required by the application
- **AND** the package contains the application bundle and desktop integration files

### Requirement: Arch package release publication

The release workflow SHALL publish the Arch package as an artifact for the same versioned GitHub release as the Debian and Flatpak packages.

#### Scenario: Release assets
- **WHEN** the release workflow creates or updates a versioned GitHub release
- **THEN** the matching `.pkg.tar.zst` file SHALL be attached alongside the `.deb` and `.flatpak` files
- **AND** the workflow SHALL fail if the Arch package is missing or empty
