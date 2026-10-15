#!/usr/bin/env bash
# Exercise the Rust library extracted from a package in an owned Xvfb session.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
package="${1:?Usage: scripts/check-scene-package.sh PACKAGE}"
[[ $# -eq 1 && -f "$package" ]] || { echo 'Expected one existing package' >&2; exit 1; }
package="$(realpath "$package")"
fixture="$(mktemp -d "${TMPDIR:-/tmp}/varpaper-package-check.XXXXXX")"
trap 'rm -rf "$fixture"' EXIT
case "$package" in
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
[[ -f "$bundle/lib/libwayvid_gui.so" ]] || { echo 'Packaged Rust bridge missing' >&2; exit 1; }
[[ ! -e "$bundle/lib/libvarpaper_scene.so" && ! -L "$bundle/lib/libvarpaper_scene.so" ]] || {
  echo 'Old C++ Scene library found in package' >&2; exit 1
}
cd "$root"
xvfb-run -a env -u WAYLAND_DISPLAY \
  XDG_SESSION_TYPE=x11 XDG_DATA_HOME="$fixture/data" \
  LIBGL_ALWAYS_SOFTWARE=1 SDL_AUDIODRIVER=dummy VARPAPER_TEST_X11=1 \
  VARPAPER_TEST_PACKAGE_LIBRARY="$bundle/lib/libwayvid_gui.so" \
  "${VARPAPER_TEST_FLUTTER:-flutter}" test scripts/fixtures/scene_package_smoke_test.dart
