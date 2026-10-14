# Rust Scene migration checks

Scene rendering uses `crates/wayvid-engine/src/scene/` directly from the
application factory. The C++ Scene implementation and adapter are removed.
Current delivery covers the implemented Rust subset described below;
they do not establish full Wallpaper Engine compatibility.

Run the generated CPU assets and actual GL fixtures with:

```bash
./scripts/check-rust-scene.sh
```

This uses private Xvfb and software Mesa and verifies that twelve GL fixtures and one GL version
rejection fixture really execute. It requires Cargo, the engine's system
dependencies, Python 3, Xvfb and xauth. No reference checkout, Workshop assets,
CEF runtime or desktop audio server is needed. Generated files go to an owned
temporary directory and are removed after the check.

| Implemented path | Rust fixture and verified behavior |
| --- | --- |
| Scene/model/material resources | `assets::tests` validates typed objects, dependency cycles, required mode errors, bad fields, missing assets, loose/PKGV/shared parity and escaping symlinks. |
| TEXV0005/TEXI0001, TEXB1/2/3 and non-video 4 | `texture::tests` decodes RGBA/R/RG/DXT1/3/5, each raw/LZ4, plus encoded PNG. DXT alpha/partial blocks, animation records, truncation, length and budget failures are checked. |
| Layer geometry | `rust_scene_layer_camera_parent_radian_rotation_blending_and_uv_crop` verifies top-down coordinates, radians, parent transforms/visibility, alpha blending and UV crop; common-semantics reference screenshots match exactly after origin normalization. Explicit project size and inherited parent visibility follow the user-approved behavior; reference differences are recorded separately. |
| Multi-pass materials | `rust_scene_multipass_pingpong_parameters_and_failure_cleanup` verifies two sequential gain passes, ping-pong targets, transactional parameter updates and ten lifecycle cycles. |
| Effect files | `rust_scene_effect_files_named_targets_copy_bindings_overrides_and_cleanup` verifies named RGBA targets, copy commands, explicit texture binding, per-instance constant overrides, disabled effects, GL state and failure cleanup; the common-effects reference screenshot matches with zero RGB error. |
| Particle core (partial) | `particle::tests` verifies seeded simulation, bounded updates and mode diagnostics. `rust_scene_particle_sprites_seeded_frames_parent_visibility_pause_and_cleanup` verifies actual sprite pixels, movement, determinism, ancestor visibility, pause/resume and owner counts. Trail/rope, control points, remaining initializers/operators and audio modulation remain pending. Explicit rejection prevents blank output but does not complete task 3.3. |
| Text | `rust_scene_text_font_fallback_layout_budget_visibility_and_cleanup` verifies Rust glyph coverage, multiline layout, project fonts, missing-font fallback, transactional content updates, empty-text clearing, parent visibility, bounds and ten lifecycle cycles. |
| Sound | The separate private-audio fixture `sound::tests::private_sound_controls_pcm_pause_mute_volume_and_cleanup` checks decoded PCM rate, actual monitor RMS changes, mute/pause/resume, clear, stream cleanup and partial sound loading failure. |
| SceneScript core | `script::tests` checks init/update, project properties, builder seeds, vectors, layer lookup, assignment isolation and exceptions; `rust_scene_script_property_and_text_pixels_isolation_pause_and_cleanup` verifies independent opacity pixels, dynamic text, pause/resume and VM/GPU counts. Timer registration and scheduling are Rust-owned. Stable audio arrays accept explicit snapshots internally; desktop monitor integration and injected wall-clock APIs are deferred. |
| GL texture cache | `rust_scene_twelve_texture_formats_shared_upload_and_shader_cleanup` samples both red layers for all 12 format combinations; one resource read and one upload are shared by the layers. |
| Shader expansion | `shader::tests` covers nested `.h` includes, cycles, depth, source budgets, comments, combo defaults/overrides and unsafe/missing resources. Format vocabulary uses GLSL macros; the driver validates syntax and types. |
| Shader compilation and program ownership | The texture fixture tests bad vertex/fragment code and failed linkage, with shader/program/mesh/texture/FBO counts returning to baseline. |
| Shader metadata and material uniforms | `rust_scene_shader_material_uniforms_defaults_and_driver_failure_cleanup` tests default textures, vec3/float defaults and overrides, vec4/mat3/int/bool/float arrays, conditional default textures, invalid values and injected program allocation failure. Uniform types/counts come from driver reflection. |
| Resource frame parity | `rust_scene_loose_package_shared_frames_and_include_compilation` compares whole RGBA frames byte for byte, actually sampling a texture through nested includes and combo macros. Missing shared assets fail before GPU creation. |
| Context/state ownership | `rust_scene_first_frame_state_and_failed_allocation_cleanup` tests clear pixels, failed resize preserving the old target, partial allocations and 20 repeated create/release cycles. GL 3.0 is rejected. |

The current compatibility boundary is explicit: particle `rope`, `ropetrail`,
control points/children, unsupported initializers/operators, non-sprite
renderers and non-zero perspective flags fail Scene creation with a named
required-feature error. Sound modes outside `once` and `loop` fail in the
same way. The CPU fixtures exercise these rejection paths so unsupported
content cannot become a blank successful frame. `scripts/check-rust-scene.sh`
is the reproducible command for the generated assets and the Rust fixture
suite.

The GL guard restores framebuffer, viewport, color/scissor state, program,
VAO/array buffer, 16 pass texture units and the caller's active unit/binding,
blend/depth/cull state, and unpack buffer/layout. A tiny caller PBO and a caller
texture on unit 19 are tested. GPU owners are released with their retained EGL
surface current on the render thread.

JSON is limited to 1 MiB per file, 4096 objects/resources and dependency depth
128. Texture dimensions are 1..16384, with at most 4096 images and 16 mips per
image. Each stored/decoded mip and RGBA output is limited to 64 MiB; texture
allocation and the retained texture cache are limited to 256 MiB. Shader source
is limited to 1 MiB per file, 4 MiB per expansion, 256 include visits, include
depth 32 and 256 combos. Programs allow at most 1024 active uniforms, with
bounded diagnostic logs. Errors include resource/shader/pass names.

Animated TEX playback is implemented: selected uploaded images and normalized
atlas rotation/translation reach the material shader.
`rust_scene_animated_images_atlas_rotation_pause_resume_and_cleanup` verifies
image changes, atlas regions, rotated frames, pause without intermediate frames,
resume without a large delta, and GPU cleanup. `clock::tests` injects monotonic
time and rejects backwards/nonfinite input; frame gaps are capped at 250 ms. The direct
Rust runtime currently rejects puppet models and particle features outside the qualified core rather than reporting a blank
successful frame. Advanced SceneScript APIs, desktop monitor audio and injected local clocks
are deferred. Scene uses stretch rendering; alternate layout modes and live GUI
property editing remain outside current delivery. Unknown required shader modules/formats are diagnosed.
Rust image entry tests cover basic and multi-pass fixtures; further
shader vocabulary and encoded formats require
their own compatibility fixtures before being advertised.

Effects allow up to 64 instances, 64 passes and 64 named targets per instance,
with at most 4096 steps per layer. Named targets use RGBA8 and divide layer
dimensions by a positive scale; their aggregate allocation is capped at 256 MiB.
Targets and textures are assignment-local. Undeclared/unwritten reads, feedback,
unknown commands and unsupported target formats fail with a compatibility error.
Copy commands follow the pinned reference; its unsupported swap command is
explicitly rejected. Effect materials retain independent instance overrides.

The private audio runner has moved to
`scripts/fixtures/scene_audio_monitor.py`. It now runs only the implemented sound playback fixture; live Scene
monitor/FFT visualization is deferred.
The migration's `validation.md` distinguishes current Rust results from
historical C++ results and desktop/package checks that have not run.

For optional reference comparison, set `VARPAPER_TEST_SCENE_REFERENCE_OUTPUT`
to a new directory when running the exact layer geometry GL fixture, then use:

```bash
env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 xvfb-run -a \
  python3 scripts/fixtures/compare_scene_reference.py \
  --reference /path/to/pinned/linux-wallpaperengine \
  --cases /path/to/generated/cases --output /path/to/new/comparison
```

The reference executable is a development input. It is never used by the
application or its build. The runner uses dummy audio and a private unavailable
Pulse socket, and records separate approved size/visibility differences.

Text uses ab_glyph with project fonts from the declared path, materials/fonts
or fonts. Missing fonts fall back to installed DejaVu Sans/Liberation Sans;
corrupt declared font files and escaping paths produce an asset error.
Limits: 4096 glyphs / 64 KiB text, 1..512 pixel size, 4096×4096 canvas,
64 MiB font cache and bounded glyph raster work. Text textures share the
256 MiB texture cache budget. Content updates rasterize and upload candidates
before replacing the previous texture.

Sound streams use audio-only libmpv decoding of owned Linux memfd bytes.
No Scene C ABI or C++ playback owner is used. Streams start paused/muted;
assignment controls enable playback. Only once/loop modes are qualified,
with at most 64 streams and 64 MiB per sound under the CPU resource graph
budget. libmpv terminate/destroy completes before storage is released.
Run the actual audio qualification on an isolated virtual device:

```bash
python3 scripts/fixtures/scene_audio_monitor.py --sound
```

This requires PipeWire, pipewire-pulse, WirePlumber policy, D-Bus, pactl and Xvfb.
The default runner and `--sound` both run the supported sound fixture.
The sound variant plays only Scene-owned PCM into a private null sink;
volume attenuation follows mpv/Pulse playback volume semantics.

SceneScript uses an assignment-local rquickjs runtime/context. Binding functions
are separate ES modules with retained exports. Known layer fields are validated
in Rust before use; text changes rasterize/upload a candidate. No host module
loader or filesystem/network/process API is installed. Initial memory, stack
and execution limits are 32 MiB / 512 KiB / 20 ms; pending jobs are capped at
64 per frame. Failed update bindings are disabled and report layer/property
context. Timer handles are cancellation functions, with 1024 registrations,
millisecond delays, animation-clock scheduling and no same-frame reentrant
execution of newly registered callbacks. `engine.registerAudioBuffers(16|32|64)`
creates assignment-local stable typed arrays with a 128-registration budget;
arrays are updated in place from explicitly supplied snapshots. Missing fresh
render inputs become silence; NaN/Infinity inputs become zero. Shader passes
receive stereo 16/32/64 bands and the RMS extensions. The private monitor,
effect/particle response and full script audio matrix remain pending. Vec2/Vec3/Vec4 and
property builder value adapters are small JavaScript helpers; host lifecycle
and scheduling remain Rust code. These checks do not yet qualify the full
script API matrix.
