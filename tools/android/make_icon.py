#!/usr/bin/env python3
"""Writes the app's launcher icons (our own placeholder: a rally-red disc with a chevron
road) as mipmap-<density>/ic_launcher.png under the folder given. No game pictures."""
import os, sys
from PIL import Image, ImageDraw

SIZES = {"mdpi": 48, "hdpi": 72, "xhdpi": 96, "xxhdpi": 144, "xxxhdpi": 192}

def draw(size):
    big = size * 4
    img = Image.new("RGBA", (big, big), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([0, 0, big - 1, big - 1], fill=(200, 38, 38, 255))
    d.ellipse([big * 0.06, big * 0.06, big * 0.94, big * 0.94], fill=(30, 30, 36, 255))
    # a road narrowing to the horizon, with a dashed middle line
    d.polygon([(big * 0.18, big * 0.9), (big * 0.82, big * 0.9), (big * 0.54, big * 0.3), (big * 0.46, big * 0.3)], fill=(90, 90, 100, 255))
    for i, (y0, y1, w) in enumerate([(0.78, 0.88, 0.035), (0.62, 0.7, 0.026), (0.48, 0.54, 0.018)]):
        d.polygon([(big * (0.5 - w), big * y1), (big * (0.5 + w), big * y1), (big * (0.5 + w * 0.7), big * y0), (big * (0.5 - w * 0.7), big * y0)], fill=(240, 240, 240, 255))
    return img.resize((size, size), Image.LANCZOS)

root = sys.argv[1]
for name, size in SIZES.items():
    folder = os.path.join(root, f"mipmap-{name}")
    os.makedirs(folder, exist_ok=True)
    draw(size).save(os.path.join(folder, "ic_launcher.png"))
