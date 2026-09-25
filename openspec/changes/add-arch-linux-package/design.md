## Context

The release workflow builds the Flutter Linux bundle and uses it to create a Debian package and a Flatpak. Arch Linux needs a native pacman archive whose runtime dependencies resolve against Arch packages. The output should use the standard `makepkg`/`PKGBUILD` flow and `.pkg.tar.zst` format.

## Goals / Non-Goals

- Goals: produce an x86_64 Arch package from the release bundle, declare runtime dependencies, verify its contents, and publish it with the matching release.
- Non-Goals: add an AUR repository, publish a pacman repository, support architectures other than x86_64, or change Debian and Flatpak package behavior.

## Decisions

1. Add `packaging/arch/PKGBUILD` to install the Flutter bundle, launcher, desktop entry, icon, and license notices using normal Arch filesystem locations. Declare the native runtime packages required by the bundled executable and Rust library.
2. Build the package in an Arch Linux environment with `makepkg`; do not create a pacman archive by renaming another package format or by generating Arch metadata on Ubuntu.
3. Reuse the release Flutter bundle where ABI-compatible, and make the Arch build fail if the bundled binary's required SONAMEs cannot be resolved with the selected Arch dependencies.
4. Verify the `.PKGINFO` metadata and archive contents in CI, upload the package as an Actions artifact, then include it in the same versioned GitHub release upload as `.deb` and `.flatpak`.
5. Update the local aggregate package build and installation instructions so users can produce and install the new artifact.

## Risks / Trade-offs

- Arch's rolling runtime can change library SONAMEs. The Arch build must validate the required shared libraries against the current Arch package set so incompatible releases fail before publishing.
- The release workflow gains an Arch build environment and another artifact to retain and clean up.

## Migration Plan

No migration is required. Future release workflow runs will publish the Arch package alongside the existing Linux packages.

## Open Questions

- None.
