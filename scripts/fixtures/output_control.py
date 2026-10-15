#!/usr/bin/env python3
"""Resize/disconnect/reconnect outputs in the private mixed-output fixture only."""
import ctypes as c
import os
import subprocess
import sys


def update_x11(name, width=800, height=600, action='resize'):
    # Delete/recreate the synthetic RandR monitor while holding the server,
    # so the engine only observes the final monitor configuration.
    class Monitor(c.Structure):
        _fields_ = [
            ('name', c.c_ulong), ('primary', c.c_int), ('automatic', c.c_int),
            ('noutput', c.c_int), ('x', c.c_int), ('y', c.c_int),
            ('width', c.c_int), ('height', c.c_int),
            ('mwidth', c.c_int), ('mheight', c.c_int),
            ('outputs', c.POINTER(c.c_ulong)),
        ]

    x = c.CDLL('libX11.so.6')
    randr = c.CDLL('libXrandr.so.2')
    x.XOpenDisplay.argtypes = [c.c_char_p]
    x.XOpenDisplay.restype = c.c_void_p
    x.XDefaultRootWindow.argtypes = [c.c_void_p]
    x.XDefaultRootWindow.restype = c.c_ulong
    x.XInternAtom.argtypes = [c.c_void_p, c.c_char_p, c.c_int]
    x.XInternAtom.restype = c.c_ulong
    for method in ['XGrabServer', 'XUngrabServer', 'XCloseDisplay']:
        getattr(x, method).argtypes = [c.c_void_p]
    x.XSync.argtypes = [c.c_void_p, c.c_int]
    randr.XRRDeleteMonitor.argtypes = [c.c_void_p, c.c_ulong, c.c_ulong]
    randr.XRRSetMonitor.argtypes = [c.c_void_p, c.c_ulong, c.POINTER(Monitor)]
    display = x.XOpenDisplay(None)
    if not display:
        raise RuntimeError('Private X11 display unavailable')
    try:
        root = x.XDefaultRootWindow(display)
        atom = x.XInternAtom(display, name.encode(), 0)
        index = ['FixtureA', 'FixtureB', 'FixtureC'].index(name)
        monitor = Monitor(atom, 0, 0, 0, index * 800, 0, width, height, 200, 150, None)
        x.XGrabServer(display)
        if action != 'reconnect':
            randr.XRRDeleteMonitor(display, root, atom)
        if action != 'disconnect':
            randr.XRRSetMonitor(display, root, c.byref(monitor))
        x.XUngrabServer(display)
        x.XSync(display, 0)
    finally:
        x.XCloseDisplay(display)


if __name__ == '__main__':
    if os.environ.get('VARPAPER_TEST_MIXED_OUTPUTS') != '1':
        raise RuntimeError('Only use this helper in the owned output fixture')
    name, *args = sys.argv[1:]
    action = args[0]
    if os.environ['XDG_SESSION_TYPE'] == 'wayland':
        if action in ['disconnect', 'reconnect']:
            command = ['disable'] if action == 'disconnect' else ['enable']
        else:
            width, height, scale = args
            command = ['mode', f'{width}x{height}', 'scale', scale]
        subprocess.run(['swaymsg', 'output', name, *command], check=True)
    else:
        if action in ['disconnect', 'reconnect']:
            update_x11(name, action=action)
        else:
            width, height, _scale = args
            update_x11(name, int(width), int(height))
