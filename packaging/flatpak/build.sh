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

[[ "$version" =~ ^[0-9]{8}\.g[0-9a-f]{8}$ ]] || { echo "Invalid version: $version" >&2; exit 1; }
[[ -x "$bundle/wayvid-gui" ]] || { echo "Flutter release bundle missing: $bundle" >&2; exit 1; }
for command_name in flatpak-builder flatpak ldd readelf; do
  command -v "$command_name" >/dev/null || { echo "$command_name is required" >&2; exit 1; }
done
flatpak info org.gnome.Platform//50 >/dev/null
flatpak info org.gnome.Sdk//50 >/dev/null

stage="$work/app"
install -d "$stage/bundle" "$stage/libraries" "$output"
cp -a "$bundle/." "$stage/bundle/"
# The transitive jni package registers a Linux FFI plugin, but VarPaper does
# not call JNI on Linux. Scanning that unused library would pull in a JVM.
rm -f "$stage/bundle/lib/libdartjni.so"
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

# mpv is built below in the GNOME SDK against the Platform FFmpeg ABI. Do not
# traverse the host libwayvid_gui.so dependency closure: it reaches Ubuntu's
# libmpv and FFmpeg, which are incompatible with the GNOME 50 runtime.
declare -A runtime_libs=()
while IFS= read -r soname; do
  runtime_libs["$soname"]=1
done < <(flatpak run --command=sh org.gnome.Platform//50 -c \
  'find /usr/lib -name "lib*.so*" -printf "%f\n"')
while IFS= read -r -d '' elf; do
  [[ "$elf" != "$stage/bundle/lib/libwayvid_gui.so" ]] || continue
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
(
  cd "$work"
  # Keep Flatpak Builder's state in the disposable work directory. This also
  # lets --run use its default state path on older CI versions that reject
  # --state-dir when running a command.
  flatpak-builder --force-clean --repo=repo --default-branch=stable \
    build-dir manifest.json
  flatpak-builder --run build-dir manifest.json sh -c '
    for elf in /app/lib/varpaper/wayvid-gui /app/lib/varpaper/lib/libwayvid_gui.so /app/lib/libmpv.so.2; do
      result="$(LD_LIBRARY_PATH=/app/lib:/app/lib/varpaper/lib ldd "$elf" 2>&1)" || {
        printf "%s\n" "$result" | grep -E "not found|version" >&2 || true
        exit 1
      }
      if printf "%s\n" "$result" | grep -q "not found"; then
        printf "%s\n" "$result" | grep "not found" >&2
        exit 1
      fi
    done
    readelf -d /app/lib/libmpv.so.2 | grep -q "libavcodec.so.61"
    if find /app/lib -maxdepth 1 \( -name "libav*.so*" -o -name "libjvm.so*" -o -name "libGL.so*" -o -name "libEGL.so*" \) | grep -q .; then
      echo "Unexpected runtime library bundled in /app/lib" >&2
      exit 1
    fi
  '
)
package="$output/varpaper_${version}_$(uname -m).flatpak"
flatpak build-bundle "$work/repo" "$package" "$app_id" stable
[[ -s "$package" ]] || { echo "Flatpak bundle is empty: $package" >&2; exit 1; }
find "$stage/libraries" -type f -printf '%f %s bytes\n' | sort
stat -c 'Flatpak bundle: %s bytes' "$package"
echo "$package"
