#!/usr/bin/env python3
"""Check pinned Scene snapshots without network or the reference checkout."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def verify():
    dependencies = json.loads((ROOT / "dependencies.json").read_text())
    for name, source in dependencies.items():
        archive = ROOT / "vendor" / source["archive"]
        if archive.parent != ROOT / "vendor":
            raise ValueError(f"Unexpected archive path: {name}")
        if hashlib.sha256(archive.read_bytes()).hexdigest() != source["sha256"]:
            raise ValueError(f"Dependency snapshot mismatch: {name}")
    upstream = json.loads((ROOT / "upstream-source.json").read_text())
    actual = {p.relative_to(ROOT).as_posix() for p in (ROOT / "upstream").rglob("*") if p.is_file()}
    if actual != set(upstream["files"]):
        raise ValueError("Upstream source inventory differs from pinned manifest")
    for name, digest in upstream["files"].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"Upstream snapshot mismatch: {name}")
    print(f"Verified {len(dependencies)} dependency snapshots and {len(actual)} adapted upstream files")


if __name__ == "__main__":
    verify()
