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
for command_name in dpkg-deb readelf python3 ldd; do
  command -v "$command_name" >/dev/null || { echo "$command_name is required" >&2; exit 1; }
done

install -d "$stage/root/DEBIAN" "$stage/root/usr/bin" "$stage/root/usr/lib/varpaper" \
  "$stage/root/usr/share/applications" "$stage/root/usr/share/icons/hicolor/256x256/apps" \
  "$stage/root/usr/share/doc/varpaper" "$output"
cp -a "$bundle/." "$stage/root/usr/lib/varpaper/"
# Match the Flatpak/Arch inventories: JNI is unused on Linux, and its
# generated library embeds the build machine's JVM path.
rm -f "$stage/root/usr/lib/varpaper/lib/libdartjni.so"
bundle="$stage/root/usr/lib/varpaper"
python3 "$root/scripts/check-linux-bundle.py" "$bundle"

# This binary is linked to libmpv.so.2. libmpv1 provides libmpv.so.1 and
# cannot satisfy that dependency, even when apt accepts the package name.
readelf -d "$bundle/lib/libwayvid_gui.so" | grep -Fq '[libmpv.so.2]' || {
  echo 'The Debian dependency list expects libmpv.so.2' >&2
  exit 1
}
elf_files=("$bundle/wayvid-gui" "$bundle"/lib/*.so)
elf_files+=("$bundle/libexec/varpaper-web-host")
glibc_min="$(readelf --version-info "${elf_files[@]}" \
  | grep -oE 'GLIBC_[0-9]+\.[0-9]+' | cut -d_ -f2 | sort -Vu | tail -n 1)"
[[ -n "$glibc_min" ]] || { echo 'Could not determine the minimum glibc version' >&2; exit 1; }

install -Dm644 "$root/packaging/varpaper.png" \
  "$stage/root/usr/lib/varpaper/packaging/varpaper.png"
install -Dm644 "$root/packaging/varpaper.png" \
  "$stage/root/usr/share/icons/hicolor/256x256/apps/io.github.mybna134.varpaper.png"
install -Dm644 "$root/packaging/varpaper.desktop" \
  "$stage/root/usr/share/applications/io.github.mybna134.varpaper.desktop"
install -Dm644 "$root/LICENSE" "$root/NOTICE" "$root/LICENSE-APACHE" "$root/LICENSE-MIT" \
  "$stage/root/usr/share/doc/varpaper/"
cat > "$stage/root/usr/bin/varpaper" <<'EOF'
#!/bin/sh
cd /usr/lib/varpaper
exec ./wayvid-gui "$@"
EOF
chmod 755 "$stage/root/usr/bin/varpaper"
cat > "$stage/root/DEBIAN/control" <<EOF
Package: varpaper
Version: $deb_version
Section: video
Priority: optional
Architecture: $arch
Maintainer: VarPaper contributors
Depends: libc6 (>= $glibc_min), libgtk-3-0 | libgtk-3-0t64, libayatana-appindicator3-1, libmpv2, libwayland-client0, libwayland-egl1, libx11-6, libxrandr2, libxfixes3, libegl1, libgl1, libsdl2-2.0-0,
 libstdc++6, libgcc-s1, libssl3 | libssl3t64, ca-certificates,
 libfontconfig1, fonts-dejavu-core, libpulse0,
 libnss3, libnspr4, libcups2 | libcups2t64, libgbm1,
 libxcomposite1, libxdamage1, libxkbcommon0, libasound2 | libasound2t64
Description: Linux animated wallpaper manager
 VarPaper is a Flutter application with a Rust playback engine.
 Scene rendering is integrated into the Rust engine. Web wallpapers require
 the optional pinned Chromium component installed explicitly in Settings.
EOF

package="$output/varpaper_${deb_version}_${arch}.deb"
dpkg-deb --root-owner-group --build "$stage/root" "$package"
dpkg-deb --info "$package" >/dev/null
# Validate the archived payload as well as the staging tree. CEF itself must
# remain absent; only the helper may have an unresolved libcef.so dependency.
dpkg-deb --extract "$package" "$stage/extracted"
python3 "$root/scripts/check-linux-bundle.py" "$stage/extracted/usr/lib/varpaper"
echo "$package"
