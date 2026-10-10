# Spec Delta

## Purpose

Provide reliable discovery and loading of local and installed Workshop wallpaper projects while preserving their actual type, resource context, identity, and actionable compatibility diagnostics across the application.

## ADDED Requirements

### Requirement: Wallpaper type preservation
The system SHALL distinguish Video, Scene, Web, Image, and GIF throughout discovery, display, application, and restoration. Application and unknown project types SHALL be reported as unsupported and SHALL NOT be executed or interpreted as Video.

#### Scenario: Discover supported project types
- **WHEN** user scans installed Workshop content or an added folder containing Video, Scene, and Web projects and standalone Image/GIF files
- **THEN** all valid supported items SHALL appear with their actual types and project metadata
- **AND** project-contained assets SHALL NOT appear as duplicate standalone wallpapers

#### Scenario: Unsupported project
- **WHEN** a project declares Application or an unknown type
- **THEN** its declared type and unsupported reason SHALL be available in the library
- **AND** application SHALL be disabled without executing project programs

### Requirement: Project resource context
The system SHALL resolve each project entry and its relative resources from its project directory, packaged resources, and explicitly located Wallpaper Engine shared assets. The system SHALL retain existing standalone media identities and SHALL deduplicate projects discovered through overlapping sources.

#### Scenario: Packed scene entry
- **WHEN** a Scene entry is inside scene.pkg rather than a loose file
- **THEN** the project SHALL be discoverable and its entry and referenced resources SHALL resolve from the package

#### Scenario: Missing shared asset
- **WHEN** a project requires a Wallpaper Engine shared asset that cannot be located
- **THEN** loading SHALL report the missing asset and an actionable reason
- **AND** other library items SHALL remain usable

### Requirement: Resource validation
The system SHALL reject malformed manifests/packages and resource paths that escape the project or configured shared asset roots, including archive entries and symlink escapes. A broken item SHALL NOT abort an entire scan.

#### Scenario: Invalid resource path
- **WHEN** an entry or dependent asset resolves outside allowed roots
- **THEN** loading SHALL fail with a resource-specific error
- **AND** unrelated files SHALL NOT be read as project assets

#### Scenario: Corrupt package
- **WHEN** a package declares out-of-bounds entries or invalid compressed data
- **THEN** the item SHALL receive a clear error without crashing the application

### Requirement: Project preview selection
The system SHALL prefer a project's declared preview and SHALL use a type-specific fallback when unavailable. Scene/Web previews SHALL NOT be sent to a video thumbnail extractor as if they were videos.

#### Scenario: Missing web preview
- **WHEN** a Web project has no valid preview image
- **THEN** its card SHALL show a Web fallback without blocking the interface or repeatedly retrying video extraction
