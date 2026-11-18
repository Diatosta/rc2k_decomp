#!/usr/bin/env python3
"""Writes the app's launcher icons (the project's own icon, assets/icon.png, scaled) as
mipmap-<density>/ic_launcher.png under the folder given."""
import os, sys
from PIL import Image

SIZES = {"mdpi": 48, "hdpi": 72, "xhdpi": 96, "xxhdpi": 144, "xxxhdpi": 192}
ICON = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "icon.png")

root = sys.argv[1]
icon = Image.open(ICON).convert("RGB")
for name, size in SIZES.items():
    folder = os.path.join(root, f"mipmap-{name}")
    os.makedirs(folder, exist_ok=True)
    icon.resize((size, size), Image.LANCZOS).save(os.path.join(folder, "ic_launcher.png"))
