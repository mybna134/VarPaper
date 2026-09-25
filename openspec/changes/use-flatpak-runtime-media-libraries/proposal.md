# Change: Use Flatpak runtime media libraries

## Why

The current Flatpak copies the Ubuntu 24.04 `libmpv.so.2` dependency closure into the application. GNOME 50 provides FFmpeg and mounts graphics and codec extensions, but the Ubuntu libraries require older, incompatible SONAMEs. The published Flatpak therefore contains 101 extra shared libraries (about 190 MiB unpacked), while the Debian package delegates these dependencies to apt.

## What Changes

- Build the Flatpak-specific `libmpv.so.2` against the GNOME 50 SDK and its FFmpeg libraries, using pinned upstream sources and reproducible build options.
- Use GNOME 50 runtime libraries and its automatically mounted GL and codec extensions where the required ABI is available. Package only libraries absent from the runtime.
- Keep the Debian package on its existing Ubuntu build path.
- Exclude unused Linux native assets, including the JNI library that currently pulls a JVM into the Flatpak.
- Fail the Flatpak build if required libraries cannot be resolved in its sandbox; report which libraries remain bundled and their sizes.

## Impact

- Affected spec: `specs/flatpak-packaging/spec.md` (new capability specification).
- Affected code: `packaging/flatpak/manifest.json`, `packaging/flatpak/build.sh`, `.github/workflows/ci.yml`, and Flatpak-specific native build inputs.
- The Flatpak and Debian builds will use different native media builds. This requires an ABI check for the Rust playback library against the Flatpak `libmpv.so.2`.
