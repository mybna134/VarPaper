#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/build-linux.sh' >&2; exit 1; }
version="${VARPAPER_VERSION:-$("$root/scripts/build-version.sh")}"
[[ "$version" =~ ^[0-9]{8}\.g[0-9a-f]{8}$ ]] || { echo "Invalid build version: $version" >&2; exit 1; }

# Flutter's Linux tooling requires SemVer even though package versions do not.
flutter build linux --release --build-name="${version/.g/.0.0-g}"
