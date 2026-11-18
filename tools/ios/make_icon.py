#!/usr/bin/env python3
"""Writes the iOS app's placeholder icons (our own flat drawing, no game picture):
AppIcon60x60@2x.png (120) and AppIcon60x60@3x.png (180) into the folder given."""
import struct, sys, zlib


def png(size):
    rows = []
    for y in range(size):
        row = bytearray([0])
        for x in range(size):
            u, v = x / size, y / size
            # a dark blue field with a lighter diagonal road and a yellow stripe
            d = abs((u - v) - 0.0)
            if d < 0.07:
                px = (250, 205, 40)
            elif d < 0.24:
                px = (70, 78, 96)
            else:
                px = (22, 30, 58)
            row += bytes(px)
        rows.append(bytes(row))
    raw = b"".join(rows)

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


out = sys.argv[1]
for name, size in (("AppIcon60x60@2x.png", 120), ("AppIcon60x60@3x.png", 180)):
    open(f"{out}/{name}", "wb").write(png(size))
