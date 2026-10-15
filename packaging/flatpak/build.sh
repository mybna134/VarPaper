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
for command_name in flatpak-builder flatpak ldd readelf python3; do
  command -v "$command_name" >/dev/null || { echo "$command_name is required" >&2; exit 1; }
done
flatpak info org.gnome.Platform//50 >/dev/null
flatpak info org.gnome.Sdk//50 >/dev/null

# Reject a host bundle newer than the runtime before downloading/building media.
required_glibc="$(readelf --version-info "$bundle/wayvid-gui" "$bundle"/lib/*.so "$bundle/libexec/varpaper-web-host" \
  | grep -oE 'GLIBC_[0-9]+\.[0-9]+' | cut -d_ -f2 | sort -Vu | tail -n 1)"
runtime_glibc="$(flatpak run --command=getconf org.gnome.Platform//50 GNU_LIBC_VERSION)"
runtime_glibc="${runtime_glibc##* }"
[[ "$required_glibc" =~ ^[0-9]+\.[0-9]+$ && "$runtime_glibc" =~ ^[0-9]+\.[0-9]+$ ]] || {
  echo 'Could not determine bundle/runtime glibc compatibility' >&2
  exit 1
}
if [[ "$(printf '%s\n' "$required_glibc" "$runtime_glibc" | sort -V | tail -n 1)" != "$runtime_glibc" ]]; then
  echo "Bundle requires glibc $required_glibc; GNOME 50 provides $runtime_glibc. Rebuild on the CI Ubuntu 24.04 baseline or against the GNOME SDK." >&2
  exit 1
fi

stage="$work/app"
install -d "$stage/bundle" "$stage/libraries" "$output"
cp -a "$bundle/." "$stage/bundle/"
# The transitive jni package registers a Linux FFI plugin, but VarPaper does
# not call JNI on Linux. Scanning that unused library would pull in a JVM.
rm -f "$stage/bundle/lib/libdartjni.so"
python3 "$root/scripts/check-linux-bundle.py" "$stage/bundle"
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
  # These link against libraries built below or supplied by the Platform.
  # Never traverse a host CEF dependency closure into the base package.
  case "$elf" in
    "$stage/bundle/lib/libwayvid_gui.so"|"$stage/bundle/libexec/varpaper-web-host") continue ;;
  esac
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
done < <(find "$stage/bundle" -type f \( -name 'wayvid-gui' -o -name 'varpaper-web-host' -o -name '*.so' -o -name '*.so.*' \) -print0)

cp "$root/packaging/flatpak/manifest.json" "$work/manifest.json"
cp "$root/packaging/flatpak/zypak-cef-runtime.patch" "$work/"
(
  cd "$work"
  # Reuse downloads/build cache across package rebuilds. The local symlink also
  # lets older --run versions use the default state path without --state-dir.
  install -d "$root/build/flatpak-state"
  ln -s "$root/build/flatpak-state" .flatpak-builder
  flatpak-builder --force-clean --disable-updates --repo=repo --default-branch=stable \
    build-dir manifest.json
  flatpak-builder --run build-dir manifest.json python3 \
    "$root/scripts/check-linux-bundle.py" /app/lib/varpaper --library-dir /app/lib
  flatpak-builder --run build-dir manifest.json sh -eu -c '
    test -x /app/bin/zypak-wrapper
    test -r /app/share/licenses/zypak/LICENSE
    test -r /usr/share/fonts/dejavu/DejaVuSans.ttf
    test -r /usr/lib/x86_64-linux-gnu/libpulse.so.0
    for elf in /app/bin/zypak-helper /app/bin/zypak-sandbox /app/lib/*.so*; do
      [ -f "$elf" ] || continue
      result="$(LD_LIBRARY_PATH=/app/lib:/app/lib/varpaper/lib ldd "$elf" 2>&1)" || {
        printf "%s\n" "$result" >&2
        exit 1
      }
      if printf "%s\n" "$result" | grep -q "not found"; then
        printf "%s\n" "$result" >&2
        exit 1
      fi
    done
    readelf -d /app/lib/libmpv.so.2 | grep -q "libavcodec.so.61"
    if find /app/lib -maxdepth 1 \( -name "libav*.so*" -o -name "libjvm.so*" -o -name "libGL.so*" -o -name "libEGL.so*" -o -name "libcef.so*" \) | grep -q .; then
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
