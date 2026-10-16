#!/usr/bin/env python3
"""Read one callback marker pixel from the fixture's private X11 root."""
import ctypes
import ctypes.util
import os
import sys


def main():
    if os.environ.get('VARPAPER_TEST_X11') != '1':
        raise RuntimeError('Only run in the owned Xvfb package fixture')
    x11 = ctypes.CDLL(ctypes.util.find_library('X11'))
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
    x11.XDefaultRootWindow.restype = ctypes.c_ulong
    x11.XGetImage.argtypes = [ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int,
                             ctypes.c_int, ctypes.c_uint, ctypes.c_uint,
                             ctypes.c_ulong, ctypes.c_int]
    x11.XGetImage.restype = ctypes.c_void_p
    x11.XGetPixel.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int]
    x11.XGetPixel.restype = ctypes.c_ulong
    x11.XDestroyImage.argtypes = [ctypes.c_void_p]
    x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    display = x11.XOpenDisplay(None)
    if not display:
        raise RuntimeError('Private X11 display unavailable')
    try:
        image = x11.XGetImage(display, x11.XDefaultRootWindow(display),
                             32, 32, 1, 1, ctypes.c_ulong(-1), 2)
        if not image:
            raise RuntimeError('Could not read callback marker')
        try:
            pixel = x11.XGetPixel(image, 0, 0)
            # Runner uses Xvfb's 24-bit TrueColor visual.
            sys.stdout.buffer.write(bytes((pixel >> 16 & 255,
                                           pixel >> 8 & 255, pixel & 255)))
        finally:
            x11.XDestroyImage(image)
    finally:
        x11.XCloseDisplay(display)


if __name__ == '__main__':
    main()
