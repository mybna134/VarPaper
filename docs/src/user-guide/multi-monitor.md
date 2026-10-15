# Multi-Monitor

Open **Monitors** to see detected Wayland or X11 outputs and their assignments.
On Wayland, the compositor must expose `wlr-layer-shell`; native X11 uses RandR.

## Apply to one or all monitors

In **Library**, open wallpaper details and choose **Apply to** beside the desired
monitor name. Repeat for another output to assign a different wallpaper.
Double-clicking a library item or choosing **Apply to all monitors** targets all
outputs. Video, Scene, Web, Image and GIF retain their actual types; each project
must meet its renderer requirements before application succeeds.

Details also provide **Unapply from** a named monitor and **Unapply from all
monitors**. A failed candidate preserves the previous assignment. Web needs the
optional component; Scene needs OpenGL 3.3 and only supports the implemented Rust
subset and stretch layout.

## Restore assignments

Enable **Restore last wallpaper** in Settings. Assignments are stored by output
name in Isar, including versioned project context and supplied Web property
overrides. Old path-only media records are not read or migrated; reapply wallpapers to save the current format. Reconnecting a display
with a different name may require assigning its wallpaper again.

Output discovery and rendering react to desktop output events. Physical hotplug,
background stacking and mixed-renderer behavior still require qualification on
the target desktop; the nested Wayland and Xvfb fixtures do not establish support
for every compositor or desktop icon manager. See [development checks](../dev/building.md).
