"""Run native Wayland fixtures without connecting to the user's compositor."""

import argparse
import os
import pathlib
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--service-boundary', action='store_true')
args = parser.parse_args()

with tempfile.TemporaryDirectory(prefix='varpaper-wayland-') as tmp:
    root = pathlib.Path(tmp)
    runtime = root / 'runtime'
    runtime.mkdir(mode=0o700)
    config = root / 'config.kdl'
    config.write_text('')
    env = dict(
        os.environ,
        XDG_RUNTIME_DIR=str(runtime),
        XDG_SESSION_TYPE='wayland',
        LIBGL_ALWAYS_SOFTWARE='1',
        SDL_AUDIODRIVER='dummy',
        XDG_DATA_HOME=str(root / 'data'),
        XDG_CACHE_HOME=str(root / 'cache'),
        XDG_CONFIG_HOME=str(root / 'config'),
        XDG_STATE_HOME=str(root / 'state'),
        PULSE_SERVER='unix:' + str(root / 'no-pulse'),
    )
    for name in ['WAYLAND_DISPLAY', 'NIRI_SOCKET']:
        env.pop(name, None)
    log_path = root / 'niri.log'
    with log_path.open('w+') as log:
        niri = subprocess.Popen(
            ['niri', '-c', str(config)], env=env, stdout=log, stderr=log,
        )
        try:
            deadline = time.monotonic() + 20
            while True:
                sockets = [p for p in runtime.glob('wayland-*') if p.is_socket()]
                if sockets:
                    break
                if niri.poll() is not None or time.monotonic() > deadline:
                    log.seek(0)
                    raise RuntimeError('Nested niri failed:\n' + log.read())
                time.sleep(.1)
            env['WAYLAND_DISPLAY'] = str(sockets[0])
            env['VARPAPER_TEST_WAYLAND'] = '1'
            env['VARPAPER_TEST_ENGINE_CYCLES'] = '1'
            env['VARPAPER_TEST_NESTED_WAYLAND'] = '1'
            subprocess.run(
                [
                    'cargo', 'test', '-p', 'wayvid-engine',
                    'engine::backend_tests::', '--locked', '--offline', '--',
                    '--test-threads=1', '--nocapture',
                ],
                env=env,
                check=True,
                timeout=300,
            )
            if args.service_boundary:
                test = 'bridge::boundary_tests::wayland_service_scan_preview_and_playback_lifecycle'
                result = subprocess.run(
                    ['cargo', 'test', '-p', 'wayvid_gui', '--locked', '--offline',
                     '--lib', test, '--', '--exact', '--nocapture'],
                    env=env, text=True, stdout=subprocess.PIPE,
                    stderr=subprocess.STDOUT, timeout=120,
                )
                print(result.stdout, flush=True)
                result.check_returncode()
                if f'test {test} ... ok' not in result.stdout:
                    raise RuntimeError('Wayland service boundary fixture was not executed')
        finally:
            niri.terminate()
            try:
                niri.wait(timeout=10)
            except subprocess.TimeoutExpired:
                niri.kill()
                niri.wait()
