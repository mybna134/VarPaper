# Change: Build and publish an Arch Linux package

## Why

Linux releases currently provide Debian and Flatpak packages. Arch Linux users need a native pacman package, especially when Flatpak's Wayland sandbox filters protocols required by VarPaper on some compositors.

## What Changes

- Add an Arch `PKGBUILD` and build script that package the Linux release bundle as an x86_64 `.pkg.tar.zst` with Arch runtime dependencies declared in package metadata.
- Extend GitHub Actions to build and verify the package, upload it as a workflow artifact, and attach it to the matching GitHub release with the `.deb` and `.flatpak` packages.
- Document installing the package with pacman and include it in local package build instructions.

## Impact

- Affected specs: new `linux-packaging` capability.
- Affected code: `packaging/arch/`, `.github/workflows/ci.yml`, `scripts/build-packages.sh`, `scripts/README.md`, and Linux installation/build documentation.
