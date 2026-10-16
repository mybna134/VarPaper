#!/usr/bin/env bash
# Exercise the Rust library extracted from a package in an owned Xvfb session.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
package="${1:?Usage: scripts/check-scene-package.sh PACKAGE | --installed BUNDLE}"
if [[ "$package" == --installed ]]; then
  [[ $# -eq 2 && -d "$2" ]] || { echo 'Expected installed bundle directory' >&2; exit 1; }
  bundle="$(realpath "$2")"
else
  [[ $# -eq 1 && -f "$package" ]] || { echo 'Expected one existing package' >&2; exit 1; }
  package="$(realpath "$package")"
fi
fixture_prefix=varpaper-package-check
fixture_test=scripts/fixtures/scene_package_smoke_test.dart
if [[ "${VARPAPER_TEST_COMPONENT_INSTALL:-0}" == 1 ]]; then
  fixture_prefix=varpaper-package-components
  fixture_test=scripts/fixtures/component_package_smoke_test.dart
fi
fixture="$(mktemp -d "${TMPDIR:-/tmp}/$fixture_prefix.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
case "$package" in
  --installed) ;;
  *.deb)
    dpkg-deb --extract "$package" "$fixture/root"
    bundle="$fixture/root/usr/lib/varpaper"
    ;;
  *.pkg.tar.zst)
    mkdir "$fixture/root"
    tar --zstd -xf "$package" -C "$fixture/root"
    bundle="$fixture/root/usr/lib/varpaper"
    ;;
  *.flatpak)
    ostree --repo="$fixture/repo" init --mode=archive-z2
    flatpak build-import-bundle "$fixture/repo" "$package"
    ostree --repo="$fixture/repo" checkout --user-mode \
      app/io.github.mybna134.varpaper/x86_64/stable "$fixture/root"
    bundle="$fixture/root/files/lib/varpaper"
    ;;
  *) echo 'Expected a VarPaper .deb, .pkg.tar.zst or .flatpak' >&2; exit 1 ;;
esac
python3 "$root/scripts/check-linux-bundle.py" "$bundle"
[[ -f "$bundle/lib/libwayvid_gui.so" ]] || { echo 'Packaged Rust bridge missing' >&2; exit 1; }
[[ ! -e "$bundle/lib/libvarpaper_scene.so" && ! -L "$bundle/lib/libvarpaper_scene.so" ]] || {
  echo 'Old C++ Scene library found in package' >&2; exit 1
}
# The explicit helper override expects its pinned manifest beside the binary.
# Keep that test-only layout outside the extracted application inventory.
mkdir "$fixture/host"
ln -s "$bundle/libexec/varpaper-web-host" "$fixture/host/varpaper-web-host"
ln -s "$bundle/data/runtime.json" "$fixture/host/runtime.json"
cd "$root"
display_runner=(xvfb-run -a -s '-screen 0 1280x1024x24')
if [[ "${VARPAPER_TEST_PACKAGE_DISPLAY:-0}" == 1 ]]; then
  [[ -n "${DISPLAY:-}" ]] || { echo 'Owned display missing' >&2; exit 1; }
  display_runner=()
fi
pulse_server="unix:$fixture/no-pulse"
if [[ "${VARPAPER_TEST_AUDIO_AVAILABLE:-0}" == 1 ]]; then
  [[ "${PULSE_SERVER:-}" == unix:*varpaper-audio-check-*/* ]] || {
    echo 'Available audio requires the private audio fixture' >&2; exit 1;
  }
  pulse_server="$PULSE_SERVER"
fi
"${display_runner[@]}" env -u WAYLAND_DISPLAY \
  XDG_SESSION_TYPE=x11 XDG_DATA_HOME="$fixture/data" XDG_CACHE_HOME="$fixture/cache" \
  PULSE_SERVER="$pulse_server" VARPAPER_TEST_CEF_SOFTWARE=1 \
  VARPAPER_WEB_HOST="$fixture/host/varpaper-web-host" \
  VK_ICD_FILENAMES="$fixture/data/varpaper/components/web/runtime/vk_swiftshader_icd.json" \
  LIBGL_ALWAYS_SOFTWARE=1 SDL_AUDIODRIVER=dummy VARPAPER_TEST_X11=1 \
  VARPAPER_TEST_PACKAGE_LIBRARY="$bundle/lib/libwayvid_gui.so" \
  "${VARPAPER_TEST_FLUTTER:-flutter}" test --no-pub "$fixture_test"
