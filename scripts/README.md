# Development Scripts

Development and packaging scripts for VarPaper.

## Available Scripts

### Development Checks
- `dev-check.sh` - Check development environment setup
- `quick-check.sh` - Fast workspace checks (format, clippy)
- `pre-push-check.sh` - Full pre-commit validation (tests, build)
- `build-linux.sh` - Build the Linux Flutter bundle with the generated version
- `build-packages.sh` - Build the `.deb` and `.flatpak` release packages

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

# Linux release packages (.deb and .flatpak)
./scripts/build-packages.sh

# Run GUI
flutter run -d linux
```
