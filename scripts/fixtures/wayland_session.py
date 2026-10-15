"""Run native Wayland fixtures without connecting to the user's compositor."""

import os
import pathlib
import subprocess
import tempfile
import time

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
        finally:
            niri.terminate()
            try:
                niri.wait(timeout=10)
            except subprocess.TimeoutExpired:
                niri.kill()
                niri.wait()
