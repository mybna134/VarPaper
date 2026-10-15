#!/usr/bin/env bash
# Real native fixtures; use a private Xvfb session and temporary CEF resources.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/check-native.sh' >&2; exit 1; }
"$root/scripts/build-web-host.sh"
sdk="${CEF_ROOT:-$root/build/cef/$(python3 -c 'import json; print(json.load(open("native/web/runtime.json"))["distribution"])')}"
sdk="$(cd "$sdk" && pwd)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/varpaper-native-check.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir "$fixture/runtime" "$fixture/data" "$fixture/config" "$fixture/state"
# Tests may acquire the playback monitor, but only the dedicated sound fixture
# below may connect to a server. Every other test exercises zero-input fallback.
export PULSE_SERVER="unix:$fixture/no-pulse" PULSE_RUNTIME_PATH="$fixture"
export XDG_DATA_HOME="$fixture/data" XDG_CONFIG_HOME="$fixture/config" XDG_STATE_HOME="$fixture/state"
export LIBGL_ALWAYS_SOFTWARE=1
unset VARPAPER_TEST_SCENE_SOUND VARPAPER_TEST_PULSE
"$root/scripts/check-rust-scene.sh"
python3 "$root/scripts/fixtures/scene_audio_monitor.py" --backend "${VARPAPER_TEST_AUDIO_BACKEND:-pulseaudio}"
cp -as "$sdk/Release/." "$fixture/runtime/"
cp -as "$sdk/Resources/." "$fixture/runtime/"
export XDG_SESSION_TYPE=x11 SDL_AUDIODRIVER=dummy VARPAPER_TEST_CEF_SOFTWARE=1
unset WAYLAND_DISPLAY
export VARPAPER_TEST_X11=1
export VARPAPER_WEB_HOST="${VARPAPER_WEB_BUILD_DIR:-$root/build/native-web}/varpaper-web-host"
export VARPAPER_CEF_RUNTIME="$fixture/runtime"
export VARPAPER_TEST_CEF_SDK="$sdk"
# Resolve the pinned software Vulkan driver from the temporary runtime.
export VK_ICD_FILENAMES="$fixture/runtime/vk_swiftshader_icd.json"
xvfb-run -a python3 native/web/test_host.py --host "$VARPAPER_WEB_HOST" --runtime "$VARPAPER_CEF_RUNTIME"
xvfb-run -a cargo test --workspace --locked -- --test-threads=1 2>&1 | tee "$fixture/workspace.log"
python3 - "$fixture/workspace.log" <<'PYCODE'
from pathlib import Path
import sys
output = Path(sys.argv[1]).read_text()
required = [
    "engine::x11::scene_fixture_tests::textured_scene_loose_packed_shared_and_missing_resources",
    "web_lifecycle_tests::live_uninstall_failure_retry_reinstall_and_restart_preserve_user_sources",
]
assert all(name in output for name in required), "Native workspace fixtures were not selected"
assert "test result: FAILED" not in output
print("Rust Scene, private audio, Web host and native workspace checks passed")
PYCODE
