#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
version="${1:-$(sed -n 's/^version: \([0-9][^+]*\).*/\1/p' pubspec.yaml)}"
[[ -n "$version" ]] || { echo 'Cannot determine version' >&2; exit 1; }

if [[ "${VARPAPER_SKIP_BUILD:-0}" != 1 ]]; then
  flutter clean
  flutter pub get
  flutter build linux --release
fi

bundle="${VARPAPER_BUNDLE_DIR:-$root/build/linux/x64/release/bundle}"
[[ -x "$bundle/wayvid-gui" ]] || { echo "Flutter release bundle missing: $bundle" >&2; exit 1; }
export VARPAPER_BUNDLE_DIR="$bundle"

"$root/packaging/deb/build.sh" "$version"
"$root/packaging/flatpak/build.sh" "$version"
