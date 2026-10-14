#!/usr/bin/env python3
"""Rust Scene sound playback test on a private, silent PipeWire graph.
Requires PipeWire/Pulse tools, WirePlumber 0.5+ (policy profile), D-Bus and Xvfb.
Never loads hardware monitors or connects to the user's desktop audio server.
"""
import argparse
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
CONFIG = """context.properties = { core.daemon = true core.name = pipewire-0 default.clock.rate = 48000 }
context.spa-libs = { audio.convert.* = audioconvert/libspa-audioconvert support.* = support/libspa-support }
context.modules = [
 { name = libpipewire-module-protocol-native }
 { name = libpipewire-module-metadata }
 { name = libpipewire-module-spa-node-factory }
 { name = libpipewire-module-client-node }
 { name = libpipewire-module-client-device }
 { name = libpipewire-module-adapter }
 { name = libpipewire-module-link-factory }
 { name = libpipewire-module-access }
]
"""


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sound",action="store_true",help="Run the supported Rust Scene sound fixture (also the default)")
    parser.parse_args()
    processes = []
    logs = []
    with tempfile.TemporaryDirectory(prefix="varpaper-audio-check-") as directory:
        root = Path(directory)
        env = os.environ.copy()
        env.update(
            XDG_RUNTIME_DIR=directory,
            PIPEWIRE_RUNTIME_DIR=directory,
            PIPEWIRE_REMOTE="pipewire-0",
            PULSE_RUNTIME_PATH=directory,
            PULSE_SERVER="unix:" + directory + "/native",
            XDG_CONFIG_HOME=directory + "/config",
            XDG_STATE_HOME=directory + "/state",
            XDG_SESSION_TYPE="x11",
            SDL_AUDIODRIVER="dummy",
            GIO_USE_VFS="local",
            GSETTINGS_BACKEND="memory",
            VARPAPER_TEST_SCENE_SOUND="1",
        )
        for name in ("DBUS_SESSION_BUS_ADDRESS", "WAYLAND_DISPLAY", "PIPEWIRE_CONFIG_DIR", "PIPEWIRE_CONFIG_PREFIX", "WIREPLUMBER_CONFIG_DIR"):
            env.pop(name, None)
        config = root / "fixture.conf"
        config.write_text(CONFIG)

        def spawn(name, command):
            log = (root / (name + ".log")).open("w")
            logs.append(log)
            process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=log, stderr=log, start_new_session=True)
            processes.append(process)
            return process

        def pulse(*args):
            return subprocess.run(["pactl", *args], env=env, capture_output=True, text=True, timeout=5)

        try:
            spawn("pipewire", ["pipewire", "-c", str(config)])
            deadline = time.monotonic() + 5
            while not (root / "pipewire-0").exists():
                if time.monotonic() > deadline:
                    raise RuntimeError("Private PipeWire socket did not start")
                time.sleep(0.05)
            # policy provides stream linking, without ALSA/BlueZ/camera monitors.
            spawn("policy", ["dbus-run-session", "--", "wireplumber", "-p", "policy"])
            spawn("pulse", ["pipewire-pulse"])
            deadline = time.monotonic() + 8
            while pulse("info").returncode:
                if time.monotonic() > deadline:
                    raise RuntimeError("Private Pulse protocol did not start")
                time.sleep(0.05)
            result = pulse("load-module", "module-null-sink", "sink_name=varpaper-fixture", "rate=48000", "channels=2")
            if result.returncode:
                raise RuntimeError(result.stderr)
            deadline = time.monotonic() + 8
            while True:
                result = pulse("set-default-sink", "varpaper-fixture")
                if not result.returncode:
                    break
                if time.monotonic() > deadline:
                    raise RuntimeError(result.stderr)
                time.sleep(0.05)
            # The graph must contain only the virtual sink; never test hardware.
            result = pulse("list", "short", "sinks")
            rows = result.stdout.strip().splitlines()
            if result.returncode or len(rows) != 1 or rows[0].split()[1] != "varpaper-fixture":
                raise RuntimeError("Unexpected sink in private audio graph: " + result.stdout)
            fixture="scene::sound::tests::private_sound_controls_pcm_pause_mute_volume_and_cleanup"
            test = spawn("test", ["xvfb-run", "-a", "cargo", "test", "-p", "wayvid-engine", fixture, "--locked", "--", "--exact", "--nocapture"])
            deadline = time.monotonic() + 180
            while test.poll() is None:
                if any(p.poll() is not None for p in processes[:3]):
                    raise RuntimeError("Private audio service exited early")
                if time.monotonic() > deadline:
                    raise RuntimeError("Scene audio fixture timed out")
                time.sleep(0.05)
            output = (root / "test.log").read_text()
            print(output)
            if test.returncode or "test result: ok. 1 passed;" not in output:
                raise RuntimeError("Scene audio fixture failed")
        except Exception:
            for log in root.glob("*.log"):
                print(log.name + ":\n" + log.read_text()[-4000:])
            raise
        finally:
            # Terminate groups even when their parent already exited.
            for process in reversed(processes):
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            for process in processes:
                try:
                    process.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
            for log in logs:
                log.close()
    print("Private Scene sound playback/control/cleanup passed")


if __name__ == "__main__":
    main()
