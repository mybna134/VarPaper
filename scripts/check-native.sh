#!/usr/bin/env bash
# Real native fixtures; use a private Xvfb session and temporary CEF resources.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/check-native.sh' >&2; exit 1; }
"$root/scripts/check-rust-scene.sh"
"$root/scripts/build-web-host.sh"
sdk="${CEF_ROOT:-$root/build/cef/$(python3 -c 'import json; print(json.load(open("native/web/runtime.json"))["distribution"])')}"
sdk="$(cd "$sdk" && pwd)"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/varpaper-native-check.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
mkdir "$fixture/runtime"
cp -as "$sdk/Release/." "$fixture/runtime/"
cp -as "$sdk/Resources/." "$fixture/runtime/"
export XDG_SESSION_TYPE=x11 SDL_AUDIODRIVER=dummy VARPAPER_TEST_CEF_SOFTWARE=1
unset WAYLAND_DISPLAY
export VARPAPER_TEST_X11=1
export VARPAPER_WEB_HOST="$root/build/native-web/varpaper-web-host"
export VARPAPER_CEF_RUNTIME="$fixture/runtime"
export VARPAPER_TEST_CEF_SDK="$sdk"
xvfb-run -a python3 native/web/test_host.py --host "$VARPAPER_WEB_HOST" --runtime "$VARPAPER_CEF_RUNTIME"
xvfb-run -a cargo test --workspace -- --test-threads=1
