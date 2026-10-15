"""Run the engine fixture on three private Wayland or X11 outputs.

Requires an already built Web host and explicit VARPAPER_CEF_RUNTIME.
Wayland needs sway/swaymsg; X11 is launched by xvfb-run with -noreset.
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

root = Path(__file__).resolve().parents[2]
backend = sys.argv[1]
if backend not in ['wayland', 'x11']:
    raise RuntimeError('Expected wayland or x11')
if backend == 'x11' and not os.environ.get('VARPAPER_TEST_OWNED_XVFB'):
    subprocess.run([
        'xvfb-run', '-a', '-s', '-screen 0 2400x1200x24 -noreset',
        'env', 'VARPAPER_TEST_OWNED_XVFB=1', sys.executable, __file__, 'x11',
    ], check=True)
    sys.exit(0)
if not os.environ.get('VARPAPER_CEF_RUNTIME'):
    raise RuntimeError('Set VARPAPER_CEF_RUNTIME to an existing CEF runtime; this fixture never downloads it')
with tempfile.TemporaryDirectory(prefix='varpaper-mixed-') as tmp:
    runtime = Path(tmp)
    env = dict(os.environ, XDG_SESSION_TYPE=backend, LIBGL_ALWAYS_SOFTWARE='1',
               SDL_AUDIODRIVER='dummy', VARPAPER_TEST_CEF_SOFTWARE='1',
               VARPAPER_TEST_MIXED_OUTPUTS='1', XDG_RUNTIME_DIR=tmp,
               VARPAPER_WEB_HOST=str(root / 'build/native-web/varpaper-web-host'),
               VARPAPER_TEST_OUTPUT_CONTROL=str(root / 'scripts/fixtures/output_control.py'))
    env['VK_ICD_FILENAMES'] = str(Path(env['VARPAPER_CEF_RUNTIME']) / 'vk_swiftshader_icd.json')
    compositor = None
    log = (runtime / 'compositor.log').open('w+')
    try:
        if backend == 'wayland':
            for name in ['WAYLAND_DISPLAY', 'SWAYSOCK', 'NIRI_SOCKET']:
                env.pop(name, None)
            config = runtime / 'config'
            config.write_text('output HEADLESS-1 mode 320x240 pos 0 0 scale 1\n'
                              'output HEADLESS-2 mode 400x300 pos 320 0 scale 1\n'
                              'output HEADLESS-3 mode 640x480 pos 720 0 scale 2\n')
            env.update(WLR_BACKENDS='headless', WLR_HEADLESS_OUTPUTS='3', WLR_RENDERER='pixman')
            compositor = subprocess.Popen(['sway', '-c', str(config)], env=env, stdout=log, stderr=log)
            deadline = time.monotonic() + 20
            while True:
                sockets = [p for p in runtime.glob('wayland-*') if p.is_socket()]
                ipc = list(runtime.glob('sway-ipc.*.sock'))
                if sockets and ipc:
                    break
                if compositor.poll() is not None or time.monotonic() > deadline:
                    log.seek(0)
                    raise RuntimeError('Private sway failed:\n' + log.read())
                time.sleep(.1)
            env.update(WAYLAND_DISPLAY=str(sockets[0]), SWAYSOCK=str(ipc[0]))
        else:
            env.pop('WAYLAND_DISPLAY', None)
            for name, geometry in [('FixtureA', '320/100x240/100+0+0'),
                                   ('FixtureB', '400/100x300/100+320+0'),
                                   ('FixtureC', '640/100x480/100+720+0')]:
                subprocess.run(['xrandr', '--setmonitor', name, geometry, 'none'], env=env, check=True)
        subprocess.run(['cargo', 'test', '-p', 'wayvid-engine',
                        'mixed_outputs_first_frames_scale_resize_pause_and_frame_limit',
                        '--locked', '--offline', '--', '--test-threads=1', '--nocapture'],
                       cwd=root, env=env, check=True, timeout=180)
    finally:
        if compositor:
            compositor.terminate()
            try:
                compositor.wait(timeout=10)
            except subprocess.TimeoutExpired:
                compositor.kill()
                compositor.wait()
        log.close()
