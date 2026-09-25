#!/usr/bin/env bash
set -euo pipefail

stage="${1:?Usage: build-in-container.sh STAGE OUTPUT}"
output="${2:?Usage: build-in-container.sh STAGE OUTPUT}"
host_uid="${HOST_UID:?HOST_UID is required}"
host_gid="${HOST_GID:?HOST_GID is required}"
restore_host_ownership() {
  chown -R "$host_uid:$host_gid" "$stage"
}
trap restore_host_ownership EXIT

pacman -Syu --noconfirm
pacman -S --noconfirm --needed \
  base-devel \
  fontconfig gtk3 libayatana-appindicator libglvnd \
  libx11 libxfixes libxkbcommon libxrandr mpv wayland

useradd --create-home varpaper-builder
chown -R varpaper-builder:varpaper-builder "$stage"
runuser -u varpaper-builder -- makepkg --noconfirm --force --clean --cleanbuild \
  --config /etc/makepkg.conf \
  --dir "$stage"

shopt -s nullglob
packages=("$stage"/*.pkg.tar.zst)
[[ "${#packages[@]}" -eq 1 ]] || {
  printf 'Expected one Arch package in %s, found %s\n' "$stage" "${#packages[@]}" >&2
  exit 1
}
package="${packages[0]}"
[[ -s "$package" ]]
pacman -Qip "$package" | tee "$stage/package-info.txt"
bsdtar -xOf "$package" .PKGINFO > "$stage/pkginfo"
grep -Fxq 'pkgname = varpaper' "$stage/pkginfo"
grep -Fxq 'arch = x86_64' "$stage/pkginfo"
for dependency in \
  fontconfig gtk3 libayatana-appindicator libglvnd \
  libx11 libxfixes libxkbcommon libxrandr mpv wayland; do
  grep -Fxq "depend = $dependency" "$stage/pkginfo" || {
    echo "Arch package metadata is missing dependency $dependency" >&2
    exit 1
  }
done
bsdtar -tf "$package" | tee "$stage/package-files.txt" >/dev/null
for expected in \
  usr/bin/varpaper \
  usr/lib/varpaper/wayvid-gui \
  usr/share/applications/io.github.mybna134.varpaper.desktop \
  usr/share/icons/hicolor/256x256/apps/io.github.mybna134.varpaper.png \
  usr/share/licenses/varpaper/LICENSE; do
  grep -Fxq "$expected" "$stage/package-files.txt" || {
    echo "Arch package is missing $expected" >&2
    exit 1
  }
done

install -m644 "$package" "$output/"
