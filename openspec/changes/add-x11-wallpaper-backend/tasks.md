## 1. Rust backend
- [x] 1.1 Introduce backend selection and fail clearly for unsupported sessions.
- [x] 1.2 Add XRandR monitor enumeration and event-driven output updates.
- [x] 1.3 Create and manage per-monitor X11 desktop windows with non-interactive behavior.
- [x] 1.4 Generalize EGL native windows and playback sessions for Wayland and X11.
- [x] 1.5 Route existing apply, clear, pause, resume, status, and shutdown commands through either backend.

## 2. Integration
- [x] 2.1 Replace `wlr-randr`-only monitor discovery in the Rust FRB service.
- [x] 2.2 Update backend-neutral UI text and error reporting.
- [x] 2.3 Add X11 dependencies and Flatpak X11 socket permission.
- [x] 2.4 Update Linux support documentation.

## 3. Verification
- [x] 3.1 Run Rust tests for backend selection and X11 monitor enumeration.
- [x] 3.2 Run X11 monitor/window lifecycle tests under Xvfb or a native Xorg session.
- [x] 3.3 Smoke-test image apply, clear, geometry changes, and shutdown under Xvfb.
- [x] 3.4 Re-run Wayland startup/output enumeration, Flutter analysis/tests, and Rust workspace tests.
- [ ] 3.5 Build both `.deb` and `.flatpak` with the X11 dependencies.

Native Xorg video playback and a live RandR hotplug event remain to be checked on a real X11 desktop. Local Flatpak packaging fails on this Arch host because its media libraries require a newer GLIBC than GNOME Platform 50; release CI builds on Ubuntu 24.04.
