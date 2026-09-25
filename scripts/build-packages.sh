#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/build-packages.sh' >&2; exit 1; }
version="${VARPAPER_VERSION:-$("$root/scripts/build-version.sh")}"
[[ "$version" =~ ^[0-9]{8}\.g[0-9a-f]{8}$ ]] || { echo "Invalid build version: $version" >&2; exit 1; }
echo "Building VarPaper $version"

if [[ "${VARPAPER_SKIP_BUILD:-0}" != 1 ]]; then
  flutter clean
  flutter pub get
  VARPAPER_VERSION="$version" "$root/scripts/build-linux.sh"
fi

bundle="${VARPAPER_BUNDLE_DIR:-$root/build/linux/x64/release/bundle}"
[[ -x "$bundle/wayvid-gui" ]] || { echo "Flutter release bundle missing: $bundle" >&2; exit 1; }
export VARPAPER_BUNDLE_DIR="$bundle"

"$root/packaging/deb/build.sh" "$version"
"$root/packaging/flatpak/build.sh" "$version"
"$root/packaging/arch/build.sh" "$version"
