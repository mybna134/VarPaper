#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
version="${1:?Usage: packaging/deb/build.sh VERSION}"
bundle="${VARPAPER_BUNDLE_DIR:-$root/build/linux/x64/release/bundle}"
output="${VARPAPER_OUTPUT_DIR:-$root/dist}"
arch="$(dpkg --print-architecture)"
[[ "$arch" == amd64 ]] || { echo "Only amd64 packaging is supported: $arch" >&2; exit 1; }
deb_version="$version"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

[[ "$version" =~ ^[0-9]{8}\.g[0-9a-f]{8}$ ]] || { echo "Invalid version: $version" >&2; exit 1; }
[[ -x "$bundle/wayvid-gui" ]] || { echo "Flutter release bundle missing: $bundle" >&2; exit 1; }
command -v dpkg-deb >/dev/null || { echo 'dpkg-deb is required' >&2; exit 1; }

install -d "$stage/DEBIAN" "$stage/usr/bin" "$stage/usr/lib/varpaper" \
  "$stage/usr/share/applications" "$stage/usr/share/icons/hicolor/256x256/apps" \
  "$stage/usr/share/doc/varpaper" "$output"
cp -a "$bundle/." "$stage/usr/lib/varpaper/"
install -Dm644 "$root/packaging/varpaper.png" \
  "$stage/usr/lib/varpaper/packaging/varpaper.png"
install -Dm644 "$root/packaging/varpaper.png" \
  "$stage/usr/share/icons/hicolor/256x256/apps/io.github.mybna134.varpaper.png"
install -Dm644 "$root/packaging/varpaper.desktop" \
  "$stage/usr/share/applications/io.github.mybna134.varpaper.desktop"
install -Dm644 "$root/LICENSE" "$root/NOTICE" "$root/LICENSE-APACHE" "$root/LICENSE-MIT" \
  "$stage/usr/share/doc/varpaper/"
cat > "$stage/usr/bin/varpaper" <<'EOF'
#!/bin/sh
cd /usr/lib/varpaper
exec ./wayvid-gui "$@"
EOF
chmod 755 "$stage/usr/bin/varpaper"
cat > "$stage/DEBIAN/control" <<EOF
Package: varpaper
Version: $deb_version
Section: video
Priority: optional
Architecture: $arch
Maintainer: VarPaper contributors
Depends: libgtk-3-0 | libgtk-3-0t64, libayatana-appindicator3-1 | libappindicator3-1, libmpv2 | libmpv1, libwayland-client0, libx11-6, libxrandr2, libxfixes3, libegl1, libgl1
Description: Linux animated wallpaper manager
 VarPaper is a Flutter application with a Rust playback engine.
EOF

package="$output/varpaper_${deb_version}_${arch}.deb"
dpkg-deb --root-owner-group --build "$stage" "$package"
dpkg-deb --info "$package" >/dev/null
echo "$package"
