#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if git -C "$root" show-ref --verify --quiet refs/heads/main; then
  main_ref=refs/heads/main
elif git -C "$root" show-ref --verify --quiet refs/remotes/origin/main; then
  main_ref=refs/remotes/origin/main
else
  echo 'Cannot find main; fetch the main branch before building' >&2
  exit 1
fi

commit="$(git -C "$root" rev-parse "$main_ref")"
printf '%s.g%s\n' "$(date -u +%Y%m%d)" "${commit:0:8}"
