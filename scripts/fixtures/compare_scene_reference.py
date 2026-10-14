#!/usr/bin/env python3
"""Compare authored Rust fixture projects with a separate pinned reference.

Requires Pillow and a private Xvfb DISPLAY. The application never uses this
reference executable. Explicit-size and parent-visibility differences were
approved by the user; other differences keep the RGB tolerance of two.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

from PIL import Image, ImageChops


APPROVED = {
    "rotation": "project size instead of texture dimensions",
    "crop": "project size instead of texture dimensions",
    "parent-visible": "project size instead of texture dimensions",
    "parent-hidden": "ancestor visibility and project size",
}
GOLDEN = {
    "common-effects": ((30, 30, 34, 34), (32, 0, 0)),
    "rotation": ((12, 8, 20, 24), (255, 0, 0)),
    "parent-visible": ((40, 20, 56, 28), (128, 0, 127)),
    "parent-hidden": ((0, 0, 0, 0), (255, 0, 0)),
    "crop": ((0, 0, 64, 64), (255, 0, 0)),
    "common-rotation": ((14, 8, 18, 24), (255, 0, 0)),
    "common-parent-alpha": ((44, 23, 52, 25), (128, 0, 127)),
}


def validate_project_pixels(name, image):
    if name not in GOLDEN:
        return
    (left, top, right, bottom), foreground = GOLDEN[name]
    for y in range(64):
        for x in range(64):
            expected = foreground if left <= x < right and top <= y < bottom else (0, 0, 255)
            actual = image.getpixel((x, y))
            assert all(abs(a - b) <= 2 for a, b in zip(actual, expected)), (name, x, y, actual, expected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference", required=True, type=Path)
    parser.add_argument("--cases", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    binary = args.reference.resolve(strict=True)
    assert os.environ.get("DISPLAY") and os.environ.get("XDG_SESSION_TYPE") == "x11"
    args.output.mkdir()  # Never overwrite an earlier comparison or project.
    reports = []
    with tempfile.TemporaryDirectory(prefix="varpaper-reference-runtime-") as runtime:
        env = os.environ.copy()
        env.pop("WAYLAND_DISPLAY", None)
        env.update(XDG_RUNTIME_DIR=runtime, SDL_AUDIODRIVER="dummy",
                   PULSE_SERVER="unix:" + runtime + "/no-pulse", LIBGL_ALWAYS_SOFTWARE="1")
        for case in sorted(args.cases.resolve(strict=True).iterdir()):
            if not case.is_dir():
                continue
            output = args.output / case.name
            output.mkdir()
            screenshot = output / "reference-window.png"
            log_path = output / "reference.log"
            with log_path.open("w") as log:
                command = [str(binary), "--window", "0x0x64x64", "--bg", str(case),
                           "--assets-dir", str(case), "--silent", "--no-audio-processing",
                           "--noautomute", "--screenshot", str(screenshot), "--screenshot-delay", "2"]
                process = subprocess.Popen(command, cwd=binary.parent, env=env,
                                           stdout=log, stderr=log, start_new_session=True)
                try:
                    deadline = time.monotonic() + 30
                    while not screenshot.exists():
                        if process.poll() is not None:
                            raise RuntimeError(case.name + " reference exited: " + log_path.read_text()[-4000:])
                        if time.monotonic() > deadline:
                            raise RuntimeError(case.name + " screenshot timeout")
                        time.sleep(0.05)
                    # The reference writes PNG on a background thread.
                    while True:
                        try:
                            with Image.open(screenshot) as source:
                                source.load()
                                reference = source.convert("RGB").transpose(Image.Transpose.FLIP_TOP_BOTTOM)
                            break
                        except (OSError, ValueError):
                            if time.monotonic() > deadline:
                                raise
                            time.sleep(0.05)
                    with Image.open(case / "rust.png") as source:
                        rust = source.convert("RGB")
                    assert reference.size == rust.size == (64, 64)
                    validate_project_pixels(case.name, rust)
                    diff = ImageChops.difference(reference, rust)
                    data = diff.tobytes()
                    changed = sum(max(data[i:i + 3]) > 2 for i in range(0, len(data), 3))
                    reference.save(output / "reference-top-origin.png")
                    diff.save(output / "difference.png")
                    reports.append({"case": case.name, "maximum_rgb_error": max(data),
                                    "pixels_over_tolerance": changed, "tolerance": 2,
                                    "approved_semantic_difference": APPROVED.get(case.name)})
                finally:
                    try:
                        os.killpg(process.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(process.pid, signal.SIGKILL)
                        process.wait()
    report = {"reference_revision": "b016d7d1fdcf4e5fd2f9c9fa420a8aaa07fee02d",
              "reference_binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "reference_window_origin_normalized": True, "cases": reports}
    (args.output / "comparison.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    common = [case for case in reports if case["approved_semantic_difference"] is None]
    assert len(common) >= 2, "No common-semantics reference fixtures were executed"
    assert all(case["pixels_over_tolerance"] == 0 for case in common), common


if __name__ == "__main__":
    main()
