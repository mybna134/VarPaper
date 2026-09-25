## ADDED Requirements

### Requirement: Runtime-compatible Flatpak media dependencies

The Flatpak package SHALL use media libraries built against its declared runtime and SHALL obtain ABI-compatible graphics and codec libraries from that runtime or its extensions.

#### Scenario: Flatpak release build

- **WHEN** a Flatpak release package is built against GNOME 50
- **THEN** its mpv library links to the FFmpeg ABI supplied by GNOME 50
- **AND** its application payload contains only native libraries unavailable from the runtime or mounted extensions
- **AND** no Mesa driver or unused JNI/JVM library is copied into the application payload

#### Scenario: Missing runtime dependency

- **WHEN** a required native library or symbol version cannot be resolved inside the Flatpak sandbox
- **THEN** the release build fails and identifies the missing dependency

### Requirement: Independent Debian dependencies

The Debian package SHALL continue to use the Linux Flutter bundle and declare distribution media dependencies through package metadata.

#### Scenario: Debian release build

- **WHEN** a Debian release package is built
- **THEN** it contains the Flutter executable, runtime libraries, assets, and Rust playback library
- **AND** it continues to resolve mpv and system graphics libraries through apt dependencies
