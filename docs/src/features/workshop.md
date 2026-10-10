# Steam Workshop

Import video wallpapers from Wallpaper Engine.

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

Discovery is separate from playback support. Until the new Scene/Web renderers
are integrated, valid Scene/Web projects report `requires_renderer`, rather
than claiming successful playback. Application executable projects and unknown
types report `unsupported` and are never interpreted as video. A project uses
its declared preview when available and a type fallback otherwise; Scene/Web
entries are not sent to the video thumbnail extractor.

## Using GUI (Recommended)

The easiest way to use Workshop wallpapers is through the GUI:

1. Open `wayvid-gui`
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

**Not supported:**
- Web/HTML wallpapers
- Scene wallpapers with effects
- Interactive wallpapers

Look for "Video" tag in Workshop.

## Troubleshooting

**"No video file found":**
- The wallpaper may not be a video type
- Check the folder for actual video files: `ls ~/.steam/.../431960/<id>/`

**Workshop folder not found:**
- Ensure Steam and Wallpaper Engine are installed
- Workshop content downloads to: `~/.steam/steam/steamapps/workshop/content/431960/`
