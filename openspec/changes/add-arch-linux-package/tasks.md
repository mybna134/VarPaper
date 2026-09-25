## 1. Arch package

- [x] 1.1 Add an Arch `PKGBUILD` with x86_64 metadata, runtime dependencies, and the app bundle, launcher, desktop entry, icon, and license files.
- [x] 1.2 Add a local Arch package build script that accepts the generated release version and writes the `.pkg.tar.zst` to `dist/`.
- [x] 1.3 Check the package's shared-library requirements against the Arch build environment and fail with the unresolved dependency names.

## 2. Release workflow and docs

- [x] 2.1 Build the Arch package in an Arch Linux environment as part of the release workflow.
- [x] 2.2 Verify the package metadata and expected files, upload it as a workflow artifact, and attach it to the same GitHub release as `.deb` and `.flatpak`.
- [x] 2.3 Extend the local aggregate package build and update installation/build documentation for pacman users.

## 3. Validation

- [x] 3.1 Validate the OpenSpec change and package scripts.
- [x] 3.2 Build the `.pkg.tar.zst` and inspect its metadata and contents in the Arch environment.
