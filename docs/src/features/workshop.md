# Steam Workshop

Use locally installed Wallpaper Engine projects and standalone media.

## Project discovery

VarPaper scans `project.json` in locally installed Workshop content and in
folders added through the app. Video, Scene and Web declarations retain their
actual type. Standalone images and GIFs also keep separate types; images,
videos and previews inside a project are treated as its assets, rather than
additional wallpapers. The same project found through overlapping folders or
Workshop has one identity.

Scene entries can live inside `scene.pkg`; a loose `scene.json` is not required
when that entry exists in the package. Relative assets are resolved from the
project and explicitly located shared asset directories, such as
`steamapps/common/wallpaper_engine/assets` in an installed Steam library.
VarPaper does not download or redistribute those shared assets. Missing assets,
broken manifests and invalid packages produce loading diagnostics; a bad
project does not abort discovery of other wallpapers. Paths that escape these
roots, including symlink escapes, are rejected.

Discovery is separate from playback support. Scene projects use the built-in Rust renderer. Resource discovery permits
applying a project; unsupported required features are diagnosed during renderer
creation. Web projects require the
optional **Web wallpaper support** setting component and a bundled browser host;
when both are available, the service reports them ready. Missing components
report `requires_renderer`, rather than claiming successful playback. Application executable projects and unknown
types report `unsupported` and are never interpreted as video. A project uses
its declared preview when available and a type fallback otherwise; Scene/Web
entries are not sent to the video thumbnail extractor.

## Using GUI (Recommended)

The easiest way to use Workshop wallpapers is through the GUI:

1. Open `varpaper`
2. Go to **Folders** tab
3. Add your Workshop content folder:
   ```
   ~/.steam/steam/steamapps/workshop/content/431960/
   ```
4. Browse Workshop wallpapers in **Library** tab
5. Double-click to apply

Workshop wallpapers are scanned and applied from the Flutter Library view.

## Find Workshop ID

From URL:
```
https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890
                                                        ^^^^^^^^^^
```

The workshop content is typically at:
```
~/.steam/steam/steamapps/workshop/content/431960/<workshop_id>/
```

## Compatibility

**Supported:**
- Video wallpapers (.mp4, .webm, .mkv)
- Image and GIF media
- Local Web HTML/CSS/JS, Canvas/WebGL after installing **Web wallpaper support**
- Web initial/updated project properties and system-playback audio listener callbacks
- Rust Scene image layers, multipass effects, animated textures, core sprite
  particles, text, once/loop sound and basic SceneScript hooks

Web pages run offline in the shared Chromium host. External requests, downloads,
host-file access and dialogs are blocked with a diagnostic. Pause closes the
page and keeps its last frame; resume rebuilds the page and may reset temporary
JavaScript state. Audio volume/mute covers HTML media and Web Audio. If a Web
assignment fails, its error is shown and it can be reapplied to retry; a failed
candidate preserves the current wallpaper.

**Not supported:**
- GUI project property editing and Scene property overrides (deferred)
- Advanced Scene particles, system-audio visualization and injected clock bindings
- Full Wallpaper Engine SceneScript compatibility and alternate Scene layout modes
- Interactive wallpapers

Saved assignments retain versioned project descriptions and Web overrides supplied
through the service API. The GUI currently uses declared initial properties and
has no property editor. Old path-only assignments are not read or migrated; reapply wallpapers to save the current format.
A missing Web component or failed replacement preserves the saved assignment;
install/repair the component and retry with **Restore last wallpaper** enabled.

For the supported script, audio and time contracts and their fixtures, see
[Rust Scene checks](../dev/rust-scene.md#script-properties-audio-and-time-contracts).

## Troubleshooting

**Project cannot be applied:**
- Check the compatibility reason in wallpaper details and any application error.
- Keep `project.json`, its entry and resources together; do not import only a preview.
- Install or repair **Web wallpaper support** for Web projects.
- For Scene, check the named missing asset or unsupported feature against the Rust support table.

**Workshop folder not found:**
- Ensure Steam and Wallpaper Engine are installed
- Workshop content downloads to: `~/.steam/steam/steamapps/workshop/content/431960/`
