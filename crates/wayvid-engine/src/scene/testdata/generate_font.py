"""Generate an original minimal TrueType test font; no system fonts or packages.

Printable ASCII maps to one rectangular glyph, space maps to an empty glyph.
This deliberately tests layout and rasterization, not typographic appearance.
Run with Python 3 to reproduce minimal.ttf beside this script.
"""
from pathlib import Path
import struct


def pack(fmt, *values):
    return struct.pack('>' + fmt, *values)


def checksum(data):
    padded = data + b'\0' * (-len(data) % 4)
    return sum(struct.unpack('>' + 'I' * (len(padded) // 4), padded)) & 0xffffffff


glyph = (pack('hhhhh', 1, 0, 0, 500, 700) + pack('HH', 3, 0)
         + bytes([1] * 4) + pack('hhhh', 0, 500, 0, -500)
         + pack('hhhh', 0, 0, 700, 0))
glyph += b'\0' * (-len(glyph) % 4)
characters = list(range(32, 127)) + [65535]
segments = len(characters)
search = 2 ** (segments.bit_length() - 1)
format4 = (pack('HHHHHHH', 4, 16 + 8 * segments, 0, 2 * segments,
                2 * search, search.bit_length() - 1, 2 * segments - 2 * search)
           + pack('H' * segments, *characters) + pack('H', 0)
           + pack('H' * segments, *characters)
           + pack('H' * segments, *[(1 if c != 32 else 0) - c & 65535 for c in characters])
           + bytes(2 * segments))
tables = {
    b'head': pack('IIIIHHQQhhhhHHhhh', 0x10000, 0x10000, 0, 0x5f0f3cf5,
                  0, 1000, 0, 0, 0, 0, 500, 700, 0, 8, 2, 0, 0),
    b'hhea': pack('IhhhHhhhhhhhhhhhH', 0x10000, 800, -200, 0, 600,
                  0, 100, 500, 1, 0, 0, 0, 0, 0, 0, 0, 2),
    b'maxp': pack('I' + 'H' * 14, 0x10000, 2, 4, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0),
    b'hmtx': pack('HhHh', 600, 0, 600, 0),
    b'loca': pack('HHH', 0, 0, len(glyph) // 2),
    b'glyf': glyph,
    b'cmap': pack('HHHHI', 0, 1, 3, 1, 12) + format4,
    b'name': pack('HHH', 0, 0, 6),
    b'post': pack('I', 0x30000) + bytes(28),
}
count = len(tables)
search = 2 ** (count.bit_length() - 1)
header = pack('IHHHH', 0x10000, count, search * 16,
              search.bit_length() - 1, count * 16 - search * 16)
records = bytearray()
payload = bytearray()
offsets = {}
for tag, data in sorted(tables.items()):
    offset = 12 + 16 * count + len(payload)
    offsets[tag] = offset
    records.extend(tag + pack('III', checksum(data), offset, len(data)))
    payload.extend(data + bytes(-len(data) % 4))
font = bytearray(header + records + payload)
struct.pack_into('>I', font, offsets[b'head'] + 8, (0xb1b0afba - checksum(font)) & 0xffffffff)
Path(__file__).with_name('minimal.ttf').write_bytes(font)
