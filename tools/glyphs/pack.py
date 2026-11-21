#!/usr/bin/env python3
"""Packs the button pictures of Kenney's "Input Prompts" pack (CC0) into the atlases the game
draws its button icons from.

    python3 tools/glyphs/pack.py <folder of the unpacked pack> [<output folder, assets/glyphs>]

For each family (Xbox Series, PlayStation Series, Steam Deck) it takes the white outline pictures
of the pack's Double size (128 x 128, white on transparent) of the buttons the game names, and
writes
  <family>.png  one atlas, 8-bit grey: each pixel is the picture's alpha (the pictures are
                white, so the alpha is all there is), cells of 128 x 128 in rows of 8; the game
                scales them down with an area filter to the size it draws;
  <family>.idx  one line for each picture: "name x y width height" in the atlas.
The names are the same in every family (see NAMES); a picture the family has not got is left out
of the index (the game then draws the button's number instead). Needs Pillow.
"""
import os
import sys

from PIL import Image

CELL = 128
COLUMNS = 8

# name: (Xbox, PlayStation, Steam Deck) file stem inside the family's Double folder, without
# "_outline.png"; None for one the family has not got; "xbox:" takes the Xbox family's picture.
NAMES = {
    "south": ("xbox_button_a", "playstation_button_cross", "steamdeck_button_a"),
    "east": ("xbox_button_b", "playstation_button_circle", "steamdeck_button_b"),
    "west": ("xbox_button_x", "playstation_button_square", "steamdeck_button_x"),
    "north": ("xbox_button_y", "playstation_button_triangle", "steamdeck_button_y"),
    "dpad_up": ("xbox_dpad_up", "playstation_dpad_up", "steamdeck_dpad_up"),
    "dpad_down": ("xbox_dpad_down", "playstation_dpad_down", "steamdeck_dpad_down"),
    "dpad_left": ("xbox_dpad_left", "playstation_dpad_left", "steamdeck_dpad_left"),
    "dpad_right": ("xbox_dpad_right", "playstation_dpad_right", "steamdeck_dpad_right"),
    "l1": ("xbox_lb", "playstation_trigger_l1", "steamdeck_button_l1"),
    "r1": ("xbox_rb", "playstation_trigger_r1", "steamdeck_button_r1"),
    "l2": ("xbox_lt", "playstation_trigger_l2", "steamdeck_button_l2"),
    "r2": ("xbox_rt", "playstation_trigger_r2", "steamdeck_button_r2"),
    "l3": ("xbox_ls", "playstation_button_l3", "xbox:xbox_ls"),
    "r3": ("xbox_rs", "playstation_button_r3", "xbox:xbox_rs"),
    "start": ("xbox_button_menu", "playstation5_button_options", "steamdeck_button_options"),
    "back": ("xbox_button_view", "playstation5_button_create", "steamdeck_button_view"),
    "guide": ("xbox_guide", None, "steamdeck_button_guide"),
    "touchpad": (None, "playstation5_touchpad", None),
}
# The stick pictures (solid, not outlined): ls_up, ls_down, ls_left, ls_right, ls_horizontal,
# ls_vertical and the same for the right stick, every family having them.
for _side in ("l", "r"):
    for _dir in ("up", "down", "left", "right", "horizontal", "vertical"):
        NAMES[f"{_side}s_{_dir}"] = tuple(
            f"{prefix}_stick_{_side}_{_dir}!" for prefix in ("xbox", "playstation", "steamdeck")
        )
FAMILIES = [
    ("xbox", "Xbox Series"),
    ("playstation", "PlayStation Series"),
    ("steamdeck", "Steam Deck"),
]


def load(pack, folder, stem):
    # a stem ending in "!" is a picture with no outline version
    suffix = "" if stem.endswith("!") else "_outline"
    path = os.path.join(pack, folder, "Double", stem.rstrip("!") + suffix + ".png")
    image = Image.open(path).convert("RGBA")
    if image.size != (CELL, CELL):
        sys.exit(f"{path}: {image.size}, not {CELL} x {CELL}")
    # The pictures are white: nothing but the alpha matters.
    for r, g, b, a in image.get_flattened_data() if hasattr(image, "get_flattened_data") else image.getdata():
        if a > 8 and (r, g, b) != (255, 255, 255):
            sys.exit(f"{path}: not white ({r}, {g}, {b}, {a})")
    return image.getchannel("A")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    pack = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "..", "assets", "glyphs"
    )
    os.makedirs(out, exist_ok=True)
    for column, (key, folder) in enumerate(FAMILIES):
        cells = []
        for name, stems in NAMES.items():
            stem = stems[column]
            if stem is None:
                continue
            if stem.startswith("xbox:"):
                cells.append((name, load(pack, "Xbox Series", stem[5:])))
            else:
                cells.append((name, load(pack, folder, stem)))
        rows = (len(cells) + COLUMNS - 1) // COLUMNS
        atlas = Image.new("L", (COLUMNS * CELL, rows * CELL), 0)
        lines = []
        for n, (name, alpha) in enumerate(cells):
            x, y = n % COLUMNS * CELL, n // COLUMNS * CELL
            atlas.paste(alpha, (x, y))
            lines.append(f"{name} {x} {y} {CELL} {CELL}")
        atlas.save(os.path.join(out, key + ".png"), optimize=True)
        with open(os.path.join(out, key + ".idx"), "w") as f:
            f.write("\n".join(lines) + "\n")
        print(key, len(cells), "pictures")


main()
