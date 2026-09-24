#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
version="${1:?Usage: packaging/flatpak/build.sh VERSION}"
bundle="${VARPAPER_BUNDLE_DIR:-$root/build/linux/x64/release/bundle}"
output="${VARPAPER_OUTPUT_DIR:-$root/dist}"
mkdir -p "$root/build"
work="$(mktemp -d "$root/build/flatpak-package.XXXXXX")"
trap 'rm -rf "$work"' EXIT
app_id=io.github.mybna134.varpaper
[[ "$(uname -m)" == x86_64 ]] || { echo 'Only x86_64 packaging is supported' >&2; exit 1; }

[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-z]+\.[0-9]+)?$ ]] || { echo "Invalid version: $version" >&2; exit 1; }
[[ -x "$bundle/wayvid-gui" ]] || { echo "Flutter release bundle missing: $bundle" >&2; exit 1; }
for command_name in flatpak-builder flatpak ldd; do
  command -v "$command_name" >/dev/null || { echo "$command_name is required" >&2; exit 1; }
done
flatpak info org.gnome.Platform//50 >/dev/null
flatpak info org.gnome.Sdk//50 >/dev/null

stage="$work/app"
install -d "$stage/bundle" "$stage/libraries" "$output"
cp -a "$bundle/." "$stage/bundle/"
install -Dm644 "$root/packaging/varpaper.png" \
  "$stage/bundle/packaging/varpaper.png"
install -Dm644 "$root/packaging/varpaper.png" "$stage/$app_id.png"
install -Dm644 "$root/packaging/varpaper.desktop" "$stage/$app_id.desktop"
install -Dm644 "$root/LICENSE" "$root/NOTICE" "$root/LICENSE-APACHE" "$root/LICENSE-MIT" "$stage/"
cat > "$stage/varpaper" <<'EOF'
#!/bin/sh
cd /app/lib/varpaper
export LD_LIBRARY_PATH="/app/lib:${LD_LIBRARY_PATH:-}"
exec ./wayvid-gui "$@"
EOF
chmod 755 "$stage/varpaper"

# The Flutter bundle links to libmpv and media libraries that are absent from
# the GNOME runtime. Copy the host build's ELF dependency closure into /app/lib.
# Release CI builds this bundle on Ubuntu 24.04 against the GNOME 50 runtime.
declare -A runtime_libs=()
while IFS= read -r soname; do
  runtime_libs["$soname"]=1
done < <(flatpak run --command=sh org.gnome.Platform//50 -c \
  'find /usr/lib -maxdepth 2 -name "lib*.so*" -printf "%f\n"')
while IFS= read -r -d '' elf; do
  while read -r soname arrow path _; do
    if [[ "$soname" == /* && -f "$soname" ]]; then
      path="$soname"
      soname="$(basename "$soname")"
      arrow='=>'
    fi
    [[ "$arrow" == '=>' && "$path" == /* && "$soname" == lib*.so* ]] || continue
    case "$soname" in
      libc.so.*|libm.so.*|libdl.so.*|libpthread.so.*|librt.so.*|libresolv.so.*|libutil.so.*|libnss_*.so.*|libGL.so.*|libEGL.so.*|libGLX.so.*|libGLdispatch.so.*|libvulkan.so.*)
        continue ;;
    esac
    [[ -n "${runtime_libs[$soname]+set}" ]] && continue
    [[ -e "$stage/bundle/lib/$soname" || -e "$stage/libraries/$soname" ]] ||
      cp -L "$path" "$stage/libraries/$soname"
  done < <(ldd "$elf" 2>/dev/null)
done < <(find "$stage/bundle" -type f \( -name 'wayvid-gui' -o -name '*.so' -o -name '*.so.*' \) -print0)

cp "$root/packaging/flatpak/manifest.json" "$work/manifest.json"
flatpak-builder --state-dir="$work/state" --force-clean --repo="$work/repo" --default-branch=stable \
  "$work/build-dir" "$work/manifest.json"
flatpak-builder --state-dir="$work/state" --run "$work/build-dir" "$work/manifest.json" sh -c '
  for elf in /app/lib/varpaper/wayvid-gui /app/lib/varpaper/lib/libwayvid_gui.so; do
    result="$(LD_LIBRARY_PATH=/app/lib:/app/lib/varpaper/lib ldd "$elf" 2>&1)" || {
      printf "%s\n" "$result" | grep -E "not found|version" >&2 || true
      exit 1
    }
    if printf "%s\n" "$result" | grep -q "not found"; then
      printf "%s\n" "$result" | grep "not found" >&2
      exit 1
    fi
  done
'
package="$output/varpaper_${version}_$(uname -m).flatpak"
flatpak build-bundle "$work/repo" "$package" "$app_id" stable
[[ -s "$package" ]] || { echo "Flatpak bundle is empty: $package" >&2; exit 1; }
echo "$package"
