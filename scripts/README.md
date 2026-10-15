# Development Scripts

Development and packaging scripts for VarPaper.

## Available Scripts

### Development Checks
- `check-native.sh` - CI native gate: Rust Scene parsing/GL/scripts, private sound playback, CEF host and X11/component lifecycle fixtures
- `check-rust-scene.sh` - Generated Rust Scene assets and software GL pixels, including OpenGL 3.0 rejection
- `fixtures/scene_audio_monitor.py --backend pulseaudio` - Actual Scene sound controls and playback monitor on a private null sink; default backend is PipeWire/WirePlumber 0.5+
- `dev-check.sh` - Check development environment setup
- `quick-check.sh` - Fast workspace checks (format, clippy)
- `pre-push-check.sh` - Full pre-commit validation (tests, build)
- `build-linux.sh` - Build the Linux Flutter bundle with the generated version
- `check-linux-bundle.py BUNDLE [--web-runtime PATH]` - Check bundle artifacts and native links; Chromium remains optional
- `build-packages.sh` - Build the `.deb`, `.flatpak`, and Arch `.pkg.tar.zst` release packages

Package versions use the UTC build date and the first eight characters of the `main` commit ID.

## Usage

### Development Environment Check
```bash
# Check that all dependencies and tools are available
./scripts/dev-check.sh
```

### Quick Development Check
```bash
# Fast clippy and format check
./scripts/quick-check.sh
```

### Pre-push Validation
```bash
# Full validation before pushing (all checks + tests)
./scripts/pre-push-check.sh

# Skip tests for faster feedback
SKIP_TESTS=1 ./scripts/pre-push-check.sh
```

## Testing

The project uses Rust's built-in test framework across the workspace:

```bash
# Run all workspace tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p wayvid-library
cargo test -p wayvid_gui
cargo test -p wayvid-engine
```

## Building

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release --workspace

# Linux release packages (.deb, .flatpak, and .pkg.tar.zst)
./scripts/build-packages.sh

# Run GUI
flutter run -d linux
```

## Native CI checks

`quality` runs `scripts/check-native.sh` with PulseAudio, `pactl`, Xvfb/xauth,
software GL and the dependencies listed in `.github/workflows/ci.yml`. It builds
the pinned CEF helper, executes the Rust Scene parser/script/GL checks, then
runs actual sound playback against a private PulseAudio daemon started without
default configuration or hardware discovery. Only a virtual null sink exists.
The sound fixture checks PCM levels, mute/volume, pause/resume and cleanup.
Set `VARPAPER_TEST_AUDIO_BACKEND=pipewire` to use the existing private PipeWire
fixture instead. No command connects to the desktop audio server.

The remaining native checks use a nonexistent private Pulse socket to validate
zero-input fallback, temporary data/config/state directories, a private Xvfb
display and dummy SDL audio. The Web fixture checks the pinned helper against a
temporary runtime assembled from SDK files; startup does not install a user
component. Workspace tests enable actual X11/CEF lifecycle checks, with required
fixture selection asserted in their output. GL fixtures also assert execution
counts, so an absent display cannot silently count as a completed check.
`CEF_ROOT` may reuse an extracted pinned SDK; `VARPAPER_WEB_BUILD_DIR` may select
an isolated helper build directory. A fresh checkout needs no Scene dynamic
library or linux-wallpaperengine checkout. Deferred Scene features remain listed
in the compatibility documents; running internal snapshot/script tests does not
extend the supported feature set.
