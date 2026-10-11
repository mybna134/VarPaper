#!/usr/bin/env bash
# Build-time SDK acquisition; this is never called by the running application.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/build-web-host.sh (CEF_ROOT may select a local SDK)' >&2; exit 1; }
[[ "$(uname -m)" == x86_64 ]] || { echo 'No pinned Web runtime for this architecture' >&2; exit 1; }
mapfile -t manifest < <(python3 - "$root/native/web/runtime.json" <<'PY'
import json, sys
with open(sys.argv[1]) as source:
    manifest=json.load(source)
for name in ['distribution','url','sha256']:
    print(manifest[name])
PY
)
sdk="${CEF_ROOT:-$root/build/cef/${manifest[0]}}"
if [[ ! -f "$sdk/include/cef_version.h" ]]; then
    [[ -z "${CEF_ROOT:-}" ]] || { echo "CEF_ROOT does not contain SDK headers: $sdk" >&2; exit 1; }
    archive="$root/build/cef-sdk.tar.bz2"
    mkdir -p "$root/build/cef"
    trap 'rm -f "$archive.partial"' EXIT
    curl --fail --proto '=https' --tlsv1.2 --output "$archive.partial" "${manifest[1]}"
    printf '%s  %s\n' "${manifest[2]}" "$archive.partial" | sha256sum --check --status
    mv "$archive.partial" "$archive"
    tar --extract --bzip2 --file "$archive" --directory "$root/build/cef" --no-same-owner
    rm -f "$archive"
fi
cmake -S "$root/native/web" -B "$root/build/native-web" -DCEF_ROOT="$sdk" -DCMAKE_BUILD_TYPE=Release
cmake --build "$root/build/native-web" --parallel "${VARPAPER_BUILD_JOBS:-4}"
