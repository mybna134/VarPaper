#!/usr/bin/env bash
# Generated Rust Scene assets and real pixels in a private software GL session.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/check-rust-scene.sh' >&2; exit 1; }
fixture="$(mktemp -d "${TMPDIR:-/tmp}/varpaper-rust-scene.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
unset WAYLAND_DISPLAY VARPAPER_TEST_AUDIO_TONES
unset VARPAPER_TEST_SCENE_REFERENCE_OUTPUT
unset VARPAPER_EXPECT_SCENE_GL_UNAVAILABLE MESA_GL_VERSION_OVERRIDE
export XDG_SESSION_TYPE=x11 LIBGL_ALWAYS_SOFTWARE=1
export VARPAPER_TEST_SCENE_FIXTURE_EXPORT="$fixture/project"
cargo test -p wayvid-engine scene:: --locked -- \
  --skip scene::runtime::tests:: --skip scene::sound::tests::
unset VARPAPER_TEST_SCENE_FIXTURE_EXPORT
export VARPAPER_TEST_X11=1
xvfb-run -a cargo test -p wayvid-engine scene::runtime::tests:: --locked -- \
  --test-threads=1 --nocapture 2>&1 | tee "$fixture/gl.log"
python3 - "$fixture/gl.log" <<'PY'
from pathlib import Path
import sys
output = Path(sys.argv[1]).read_text()
required = [
    "rust_scene_audio_shader_channels_silence_and_cleanup",
    "rust_scene_script_property_and_text_pixels_isolation_pause_and_cleanup",
    "rust_scene_text_font_fallback_layout_budget_visibility_and_cleanup",
    "rust_scene_particle_sprites_seeded_frames_parent_visibility_pause_and_cleanup",
    "rust_scene_animated_images_atlas_rotation_pause_resume_and_cleanup",
    "rust_scene_effect_files_named_targets_copy_bindings_overrides_and_cleanup",
    "rust_scene_multipass_pingpong_parameters_and_failure_cleanup",
    "rust_scene_first_frame_state_and_failed_allocation_cleanup",
    "rust_scene_layer_camera_parent_radian_rotation_blending_and_uv_crop",
    "rust_scene_twelve_texture_formats_shared_upload_and_shader_cleanup",
    "rust_scene_loose_package_shared_frames_and_include_compilation",
    "rust_scene_shader_material_uniforms_defaults_and_driver_failure_cleanup",
]
assert all("test scene::runtime::tests::" + name in output for name in required)
assert f"test result: ok. {len(required)} passed;" in output, "Rust GL fixtures were not executed"
PY
VARPAPER_EXPECT_SCENE_GL_UNAVAILABLE=1 MESA_GL_VERSION_OVERRIDE=3.0 \
  xvfb-run -a cargo test -p wayvid-engine \
  scene::runtime::tests::rust_scene_first_frame_state_and_failed_allocation_cleanup \
  --locked -- --exact --nocapture 2>&1 | tee "$fixture/gl30.log"
python3 - "$fixture/gl30.log" <<'PY'
from pathlib import Path
import sys
assert "test result: ok. 1 passed;" in Path(sys.argv[1]).read_text()
PY
