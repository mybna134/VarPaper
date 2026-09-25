## Context

The release build creates a Flutter bundle on Ubuntu 24.04 and copies it into both packages. The Flatpak script runs `ldd` on every `.so` in that bundle and copies dependencies missing by filename from GNOME 50. Ubuntu's `libmpv.so.2` needs FFmpeg 6 (`libavcodec.so.60`, `libavfilter.so.9`, and related libraries); GNOME 50 supplies FFmpeg 7 (`libavcodec.so.61`, `libavfilter.so.10`). These are different ABIs and cannot be exchanged by creating symlinks. The GNOME runtime already provides GL dispatch libraries and defines an automatic `org.freedesktop.Platform.GL` extension point. It also defines `org.freedesktop.Platform.codecs-extra` on the `25.08-extra` branch.

## Goals

- Reuse runtime and extension libraries when their ABI matches the Flatpak build.
- Reduce the application bundle without relying on host distribution libraries that have a different ABI.
- Preserve video decoding and GPU rendering on supported systems.

## Decisions

1. Keep the current Ubuntu-built Flutter bundle as the source of Flutter files and the Debian package. Build a separate `libmpv.so.2` in the GNOME 50 SDK, install it in the Flatpak, and ensure the Rust library can load and call that version. Pin source archives and checksums in the Flatpak manifest or adjacent build inputs.
2. Select a minimal mpv configuration that retains the playback, OpenGL, VA-API, and NVIDIA capabilities used by VarPaper. Build any mpv prerequisites absent from the SDK as pinned modules. Prefer the SDK's FFmpeg and other compatible libraries to bundled copies.
3. Treat the GNOME runtime and its GL and codecs-extra extensions as the source of their own libraries. Do not declare the GL extension in the app manifest: the runtime owns and automatically mounts it. Do not copy Mesa drivers or GL dispatch libraries into `/app/lib`.
4. Collect only the dependency closure of binaries and native assets actually loaded by the Linux app. Exclude `libdartjni.so` when it remains unused by the Linux app. Resolve the remaining libraries inside a Flatpak sandbox and reject missing dependencies or incompatible symbol versions.
5. Compare packaged size with the current 71,785,536-byte release artifact and publish the remaining bundled-library inventory during CI. Do not set a size threshold until a working runtime-compatible build establishes a baseline.

## Risks and mitigations

- A newer `libmpv.so.2` can change behavior despite retaining the SONAME. Check the Rust playback API usage and perform a playback smoke check in the Flatpak sandbox.
- Optional codec extensions can be absent or masked. Basic playback must work with the base runtime FFmpeg; extension codecs should become available when installed.
- The GNOME runtime can change within branch 50. Pin native source versions and check library resolution against the exact runtime installed in CI.

## Migration

Flatpak users receive the smaller application on the next release. The runtime and extension remain managed by Flatpak. Debian packaging is unaffected.
