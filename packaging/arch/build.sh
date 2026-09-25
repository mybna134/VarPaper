#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
version="${1:?Usage: packaging/arch/build.sh VERSION}"
bundle="${VARPAPER_BUNDLE_DIR:-$root/build/linux/x64/release/bundle}"
output="${VARPAPER_OUTPUT_DIR:-$root/dist}"
arch_cache="$root/build/arch-pacman-cache"

[[ "$version" =~ ^[0-9]{8}\.g[0-9a-f]{8}$ ]] || {
  echo "Invalid version: $version" >&2
  exit 1
}
[[ -x "$bundle/wayvid-gui" ]] || { echo "Flutter release bundle missing: $bundle" >&2; exit 1; }
command -v tar >/dev/null || { echo 'tar is required' >&2; exit 1; }
command -v zstd >/dev/null || { echo 'zstd is required' >&2; exit 1; }

bundle="$(cd "$bundle" && pwd)"
mkdir -p "$root/build" "$output" "$arch_cache"
output="$(cd "$output" && pwd)"
arch_cache="$(cd "$arch_cache" && pwd)"
stage="$(mktemp -d "$root/build/arch-package.XXXXXX")"
trap 'rm -rf "$stage"' EXIT

cp "$root/packaging/arch/PKGBUILD.in" "$stage/PKGBUILD"
sed -i "s/@VERSION@/$version/g" "$stage/PKGBUILD"
cp "$root/packaging/arch/build-in-container.sh" "$stage/"
cp "$root/packaging/varpaper.desktop" "$root/packaging/varpaper.png" \
  "$root/LICENSE" "$root/NOTICE" "$root/LICENSE-APACHE" "$root/LICENSE-MIT" "$stage/"
tar --zstd -C "$bundle" -cf "$stage/varpaper-bundle.tar.zst" .

if [[ "${VARPAPER_ARCH_USE_DOCKER:-0}" != 1 && -f /etc/arch-release ]] && command -v makepkg >/dev/null; then
  (
    cd "$stage"
    makepkg --syncdeps --noconfirm --force --clean --cleanbuild
  )
  package="$stage/varpaper-$version-1-x86_64.pkg.tar.zst"
  [[ -s "$package" ]] || { echo "Arch package was not created: $package" >&2; exit 1; }
  pacman -Qip "$package" >/dev/null
  bsdtar -tf "$package" | grep -Fxq 'usr/lib/varpaper/wayvid-gui'
  install -m644 "$package" "$output/"
elif command -v docker >/dev/null; then
  docker run --rm \
    -e "HOST_UID=$(id -u)" \
    -e "HOST_GID=$(id -g)" \
    -v "$stage:/stage" \
    -v "$output:/output" \
    -v "$arch_cache:/var/cache/pacman/pkg" \
    archlinux:base-devel \
    bash /stage/build-in-container.sh /stage /output
else
  echo 'Building the Arch package requires makepkg on Arch Linux or Docker' >&2
  exit 1
fi

package="$output/varpaper-$version-1-x86_64.pkg.tar.zst"
[[ -s "$package" ]] || { echo "Arch package was not created: $package" >&2; exit 1; }
echo "$package"
