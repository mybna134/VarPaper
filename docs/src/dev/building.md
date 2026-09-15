# Building

## Dependencies

**Arch:**
```bash
sudo pacman -S rust mpv wayland wayland-protocols libxkbcommon fontconfig mesa
```

**Ubuntu/Debian:**
```bash
sudo apt install cargo libmpv-dev libwayland-dev libxkbcommon-dev libfontconfig-dev libegl-dev
```

**Fedora:**
```bash
sudo dnf install cargo mpv-libs-devel wayland-devel libxkbcommon-devel fontconfig-devel mesa-libEGL-devel
```

## Build

```bash
git clone https://github.com/YangYuS8/wayvid
cd wayvid

# Release build: Flutter GUI bundle plus Rust CLI
flutter build linux --release
cargo build --release -p wayvid-ctl

# Install using script (recommended)
./scripts/install.sh --user

# Or manual install
sudo install -d /usr/local/lib/wayvid
sudo cp -a build/linux/x64/release/bundle/. /usr/local/lib/wayvid/
sudo install -Dm755 packaging/wayvid-gui-wrapper /usr/local/bin/wayvid-gui
sudo install -Dm755 target/release/wayvid-ctl /usr/local/bin/
```

## Binaries

v0.5 produces a Flutter application bundle and one CLI binary:
- `wayvid-gui` - Wrapper for the root Flutter bundle with Rust service
- `wayvid-ctl` - CLI control tool for scripting

## Test

```bash
cargo test --workspace
cargo clippy --workspace
```

## Verify

```bash
wayvid-gui --version
wayvid-ctl --version
```
