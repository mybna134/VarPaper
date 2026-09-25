## 1. Flatpak native dependencies

- [x] 1.1 Pin and build `libmpv.so.2` and only its missing prerequisites in the GNOME 50 SDK.
- [x] 1.2 Keep GL, FFmpeg, and compatible codec libraries supplied by the runtime and extensions out of the application payload.
- [x] 1.3 Remove unused Linux native assets from the Flatpak staging bundle, including `libdartjni.so` if no Linux load path uses it.

## 2. Release integration

- [x] 2.1 Update the Flatpak packaging script and manifest to use the SDK-built mpv library and collect only unresolved runtime dependencies.
- [x] 2.2 Update CI to build the Flatpak-specific native libraries independently of the Debian package.
- [x] 2.3 Report bundled-library inventory and compressed artifact size in CI.

## 3. Acceptance

- [x] 3.1 Confirm all required libraries and symbol versions resolve inside the GNOME 50 sandbox.
- [ ] 3.2 Confirm video playback and GPU rendering with runtime GL drivers on Wayland and X11.
- [x] 3.3 Confirm the Flatpak contains no JNI/JVM, Mesa driver, or Ubuntu FFmpeg 6 libraries and is smaller than the 71,785,536-byte baseline.
