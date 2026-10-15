#!/usr/bin/env bash
# Real layer-shell/EGL checks in an owned nested compositor.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
[[ $# -eq 0 ]] || { echo 'Usage: scripts/check-wayland.sh' >&2; exit 1; }
for command in niri Xvfb xvfb-run cargo python3; do
  command -v "$command" > /dev/null || { echo "Missing dependency: $command" >&2; exit 1; }
done
exec xvfb-run -a python3 scripts/fixtures/wayland_session.py
