#!/usr/bin/env python3
"""Check native Linux bundle contents and links without launching the desktop app."""
import argparse
import json
import os
from pathlib import Path
import re
import subprocess


def check(bundle, web_runtime=None, library_dirs=(), manifest=None):
    bundle = bundle.resolve(strict=True)
    source = Path(__file__).resolve().parents[1]
    expected = json.loads((manifest or source / 'native/web/runtime.json').read_text())
    host = bundle / 'libexec/varpaper-web-host'
    required = ['wayvid-gui', 'lib/libwayvid_gui.so', 'data/licenses/LICENSE-Scene-reference']
    if os.uname().machine == 'x86_64':
        required += ['libexec/varpaper-web-host', 'data/runtime.json',
                     'data/licenses/CEF-LICENSE', 'data/licenses/JSON-LICENSE']
    for relative in required:
        path = bundle / relative
        if not path.is_file():
            raise RuntimeError(f'Missing bundle artifact: {relative}')
    if host.exists():
        actual = json.loads((bundle / 'data/runtime.json').read_text())
        if actual != expected:
            raise RuntimeError('Bundled Web manifest differs from the pinned SDK/IPC manifest')
    if host.exists() and not os.access(host, os.X_OK):
        raise RuntimeError('Bundled Web helper is not executable')
    required_elf = {bundle / 'wayvid-gui', bundle / 'lib/libwayvid_gui.so'}
    if host.exists():
        required_elf.add(host)

    forbidden = {'libEGL.so', 'libGLESv2.so', 'libvk_swiftshader.so',
                 'vk_swiftshader_icd.json',
                 'resources.pak', 'chrome_100_percent.pak', 'chrome_200_percent.pak',
                 'snapshot_blob.bin', 'v8_context_snapshot.bin'}
    elf_count = 0
    for path in sorted(bundle.rglob('*')):
        # Flutter itself installs data/icudtl.dat; only that ICU location is
        # part of the base application, independent of the optional CEF copy.
        if (path.name in forbidden or path.name.startswith(('libvarpaper_scene.so', 'libcef.so'))
                or (path.name == 'icudtl.dat' and path != bundle / 'data/icudtl.dat')
                or (path.suffix == '.pak' and 'locales' in path.relative_to(bundle).parts)):
            raise RuntimeError(f'Forbidden base-bundle artifact: {path.relative_to(bundle)}')
        if not path.is_file():
            continue
        with path.open('rb') as binary:
            if binary.read(4) != b'\x7fELF':
                if path in required_elf:
                    raise RuntimeError(f'Native bundle artifact is not ELF: {path}')
                continue
            data = binary.read()
        elf_count += 1
        if b'libvarpaper_scene.so' in data or b'/linux-wallpaperengine' in data:
            raise RuntimeError(f'Obsolete Scene dependency/reference path in {path}')
        dynamic = subprocess.check_output(['readelf', '-d', str(path)], text=True)
        is_host = path == host
        if 'libcef.so' in dynamic and not is_host:
            raise RuntimeError(f'Main application links optional Chromium: {path}')
        if is_host and 'Shared library: [libcef.so]' not in dynamic:
            raise RuntimeError('Web helper does not link the pinned CEF library')
        if is_host and re.search(r'\((?:RPATH|RUNPATH)\)', dynamic):
            raise RuntimeError('Web helper embeds a library search path instead of using its installed component')
        env = dict(os.environ, LD_LIBRARY_PATH=os.pathsep.join(
            str(path.resolve(strict=True)) for path in [bundle / 'lib', *library_dirs]))
        if is_host and web_runtime:
            env['LD_LIBRARY_PATH'] = f'{web_runtime.resolve(strict=True)}:{env["LD_LIBRARY_PATH"]}'
        result = subprocess.run(['ldd', str(path)], env=env, text=True,
                                stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
        missing = re.findall(r'^\s*(\S+) => not found$', result.stdout, re.MULTILINE)
        if is_host and web_runtime is None:
            # libcef is deliberately absent until the user installs Web support.
            missing = [name for name in missing if name != 'libcef.so']
        if missing or result.returncode:
            raise RuntimeError(f'Unresolved native dependencies in {path}:\n{result.stdout}')
    print(f'Linux bundle verified: {elf_count} ELF files; Scene integrated; Web runtime excluded')
    if host.exists() and web_runtime is None:
        print('Web helper libcef.so is optional; pass --web-runtime to also verify its installed runtime links')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    parser.add_argument('--web-runtime', type=Path)
    parser.add_argument('--manifest', type=Path,
                        help='Pinned source manifest when running outside the source checkout')
    parser.add_argument('--library-dir', type=Path, action='append', default=[],
                        help='Additional system library directory, e.g. /app/lib inside Flatpak')
    args = parser.parse_args()
    check(args.bundle, args.web_runtime, args.library_dir, args.manifest)
