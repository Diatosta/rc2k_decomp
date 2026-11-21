#!/usr/bin/env python3
"""Makes assets/icon.icns, the macOS bundle's icon, from assets/icon.png (512 x 512): each size
the Finder and the Dock ask for as a PNG inside the icns container. Needs Pillow. Run again when
the icon changes:  python3 tools/mac/make_icns.py"""
import io, os, struct
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
# icns types holding PNG data, and their sizes in pixels (the @2x ones are double their points).
TYPES = [(b'icp4', 16), (b'icp5', 32), (b'ic11', 32), (b'ic12', 64), (b'ic07', 128),
         (b'ic08', 256), (b'ic13', 256), (b'ic09', 512), (b'ic14', 512)]

source = Image.open(os.path.join(ROOT, 'assets', 'icon.png')).convert('RGBA')
chunks = b''
for kind, size in TYPES:
    png = io.BytesIO()
    source.resize((size, size), Image.LANCZOS).save(png, 'PNG', optimize=True)
    data = png.getvalue()
    chunks += kind + struct.pack('>I', 8 + len(data)) + data
with open(os.path.join(ROOT, 'assets', 'icon.icns'), 'wb') as out:
    out.write(b'icns' + struct.pack('>I', 8 + len(chunks)) + chunks)
