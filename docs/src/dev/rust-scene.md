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

## Script, properties, audio and time contracts

These contracts describe the current Rust subset. Internal fixture APIs do not
establish live desktop Scene audio support or full Wallpaper Engine compatibility.

| Interface | Current contract | Fixture source |
| --- | --- | --- |
| Scene `init(value)` / `update(value)` | Exported functions on a property's script module. A returned value replaces that property; `undefined` keeps it. Initialization and state are assignment-local. | `script.rs`: `rust_modules_hooks_properties_assignment_isolation_and_error_cleanup` |
| Scene project and layer properties | `engine.userProperties` receives the object supplied to the renderer; script seeds use `scriptproperties`. `thisLayer` and `thisScene.getLayer(nameOrId)` expose supported layers. Applied fields are `origin`, `scale`, `angles`, `size`, `visible`, `color`, `alpha`, `brightness`, `text`, `pointsize`, validated by Rust. | `script.rs`: `rust_script_property_builder_seeds_and_init_project_properties`, `rust_timer_cancel_reentry_vectors_and_registration_budget`; GL script/text fixture above |
| Scene animation time | `engine.time`, `thisScene.time` and `thisScene.currentTime` are elapsed animation seconds; `engine.frametime` and `thisScene.dt` are delta seconds. `thisScene.fps` derives from delta (60 when delta is zero). Pause freezes animation; resume establishes a new baseline. Delta is capped at 0.25 seconds. | `clock.rs`: `injected_steps_pause_without_frames_resume_and_bad_input`; animated texture GL fixture above |
| Scene timers | `engine.setTimeout` / `engine.setInterval` and their global aliases accept callback and optional delay in milliseconds, 0..86400000. They return a cancellation function. Intervals are at least 1 ms. Up to 1024 timers; callbacks registered during a timer callback wait for a later tick. | `script.rs`: `rust_timer_cancel_reentry_vectors_and_registration_budget` |
| Scene audio buffers (internal) | `engine.registerAudioBuffers(16/32/64)` returns stable `Float32Array` fields `left`, `right`, `average`; at most 128 registrations. Explicit snapshots update the arrays. Missing fresh renderer input is silence; these bindings do not start a playback monitor. | `script.rs`: `rust_audio_buffer_registration_is_typed_stable_and_bounded` |
| Scene audio uniforms (internal) | `g_AudioSpectrum16Left/Right`, `g_AudioSpectrum32Left/Right`, `g_AudioSpectrum64Left/Right` and VarPaper RMS extensions `g_AudioLevelLeft/Right`. No fresh snapshot means zero; NaN/Infinity inputs are sanitized. | `runtime_tests.rs`: `rust_scene_audio_shader_channels_silence_and_cleanup` |
| Web properties | `wallpaperPropertyListener.applyUserProperties(properties)` gets initial and updated `{name: {value: ...}}` values with boolean/number/string types retained. Resume reloads current properties. | `native/web/test_host.py`: generated `callbacks.html` and property updates |
| Web audio | `wallpaperRegisterAudioListener(callback)` receives 128 finite values in 0..1: 64 increasing-frequency left bands, then 64 right bands. Wallpaper mute affects generated sound, independently of this input. | `native/web/test_host.py`: `callback_frame`; `audio.rs`: `known_stereo_tones_preserve_frequency_channel_order_and_level` |

### Example parameters from fixtures

The Scene unit fixture supplies `{"opacity":0.2}` to the script host and attaches
this script to a layer's `alpha` property:

```javascript
let ticks = 0;
export function init(value) { return 0.5; }
export function update(value) {
    ticks++;
    return engine.userProperties.opacity + ticks * 0.01;
}
```

Its first update produces `0.21`. A separate assignment supplied with
`{"opacity":0.8}` produces `0.81`. This demonstrates VM input and isolation;
it does not add a GUI Scene property editor or property-override support.
Builder seed fixture parameters are `gain: 0.25`, declared slider default `0.5`
and project opacity `0.8`; the seeded value wins and produces alpha `0.2`.
The timer fixture cancels a zero-delay timer, schedules alpha `0.5` after 100 ms
and schedules alpha `0.25` on a subsequent tick.

The Web fixture supplies:

```json
{
  "enabled": {"value": true},
  "speed": {"value": 0.4},
  "caption": {"value": "initial"}
}
```

It checks callback types, updates `speed` to `0.8` and `caption` to `"updated"`,
and checks delivery again after pause/resume. Its audio input uses 64 values of
`0.25` followed by 64 values of `0.75`, then 128 zeros. Arrays of length 127 or
negative spectrum values are rejected by the host fixture.

Web property overrides must name declared properties and contain only `value`.
Numeric values obey declared bounds; choices must match declared options;
colors are strings of three finite components in 0..1, such as `"0 1 0"`.
The update limit is 64 KiB. `properties.rs` tests accepted values and rejection
of unknown names, type changes, extra fields, invalid colors, range/choice
violations and oversized updates. See [described assignments](building.md#described-assignments-and-project-properties)
for the FRB entry points and persistence boundary. A renderer property update
alone does not write the Flutter assignment database.

### Capture failures and wall-clock boundary

Web uses the shared PulseAudio playback monitor, including PipeWire's PulseAudio
interface. It never falls back to a microphone. The service analyzes 48 kHz
stereo PCM in 2048-sample windows. Disconnection or monitor changes reset the
analysis, supply zeros and retry; input older than 250 ms also supplies zeros.
Capture diagnostics remain in `AudioSnapshot.diagnostic`; they are not a GUI
capture-status widget. Rendering continues without capture. Pausing/closing Web
releases its consumer, and the last consumer synchronously releases the worker.

The fixture `simulated_disconnect_output_switch_and_recovery_reset_inputs`
checks monitor changes and recovery; `stale_capture_supplies_zero_without_losing_diagnostic`
checks stale input and retained diagnostics. `consumers_share_one_service_and_last_drop_joins_worker`
checks shared ownership. The generated tone fixture uses left 375 Hz/right
6000 Hz and checks frequency/channel order. These shared-service results do not
qualify live Scene visualization: desktop Scene assignments do not acquire that
monitor. Scene once/loop sound playback is separate and follows pause/mute/volume.

Animation time is not calendar time. Injectable local time, timezone transitions,
resume-first-frame date refresh and full Scene `Date`/clock compatibility remain
deferred. Ordinary JavaScript VM facilities do not establish that integration.
The existing clock fixture injects monotonic timestamps only. GUI project
property editing, Scene overrides, advanced particles and full script/audio API
qualification remain deferred even where internal helpers exist.

### Reproduce the documented fixtures

- `scripts/check-rust-scene.sh`: Scene CPU/script checks and actual GL pixels,
  including internal audio snapshots, scripts and animation pause/resume.
- `cargo test -p wayvid-engine audio::tests --locked --offline`: shared capture
  analysis and lifecycle fixtures. The actual-monitor test requires explicit
  `VARPAPER_TEST_PULSE=1` and an available playback monitor; without the variable
  it returns early and is not evidence of real capture.
- `scripts/check-native.sh`: Scene checks plus pinned CEF host/browser callbacks
  in private Xvfb. Requires the native build dependencies and CEF SDK/runtime
  described in [Web development](building.md#optional-web-runtime-development).
- `python3 scripts/fixtures/scene_audio_monitor.py --sound`: isolated real Scene
  sound playback; it does not test live Scene system-audio visualization.

The examples and fixture names above were checked against their source. This
documentation update does not constitute a fresh fixture run; executed results
are recorded in the change's `validation.md`.
