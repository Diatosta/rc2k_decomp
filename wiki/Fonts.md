# Fonts

*Category: [Data formats](Data-formats)*

The game's text is drawn with bitmap fonts. Most fonts are not stored ready-made: at start-up
the game loads a picture with all the letters on it (a **sheet**), cuts the letters out, and
builds a **glyph table** in memory. Two small files hold glyph tables ready-made. This page
covers the sheets, how they are cut, the glyph table's layout, and how text bytes pick
glyphs.

Numbers are little-endian.

## The files

| File | What it is |
| --- | --- |
| `files/d/fnt/PAL_00.PCX` | The fonts' palette. Only the file's palette is used; the picture has no glyphs. |
| `files/d/fnt/FNT13_00.PCX`, `FNT21_00.PCX`, `FNT23_00.PCX`, `FNT27_00.PCX` | The menus' font sheets. The number is the font's height in pixels. |
| `files/d/fnt/FNT22_00.PCX`, `FNT22_01.PCX` | The race's font, as a pair: `_00` is the smoothing sheet, `_01` the glyph sheet. Loaded when a stage loads. |
| `files/d/gfx/JADE.BUN` | A ready-made glyph table: the small font of the fatal error screen. |
| `files/d/gfx/INSTALL.BUN` | A ready-made glyph table with the installer's progress bar: see [Installer data](Installer-data). |

The sheets are one-plane PCX files (8 bits a pixel with a palette): see
[Textures and pictures (PCX)](Textures-PCX). The menus' sprite sheets
(`files/g/graphics/sprites/`) are cut by the same builder.

## A sheet

A sheet is a picture with one glyph after another drawn on a plain background. Each glyph
sits in its own **cell**: a filled rectangle of a second colour, with the letter drawn on it.

| Colour | How the game finds it | Meaning |
| --- | --- | --- |
| Backdrop | The palette number the sheet's four corner pixels share. If they differ, 0. | Between cells. |
| Transparent | The first palette entry that is pure magenta (`FF 00 FF`). If there is none, the backdrop. | Inside a cell: no ink here. |
| Anything else | | Ink. |

So in the menus' sheets the backdrop is one palette number, each cell is a magenta rectangle,
and the letter's pixels are other palette numbers.

### How the cells are found

The builder scans the sheet row by row from the top, left to right. At the first pixel that
is not backdrop it grows a rectangle around it until no pixel on the rectangle's border is
anything but backdrop. That rectangle is one cell. The cell is then painted over with
backdrop and the scan carries on.

Two things follow:

- Cells must be separated from each other by at least one pixel of backdrop on every side.
- Glyphs are numbered in the order their cells are met: by the row of the cell's top edge,
  then left to right. Cells of one text line must therefore start on the same pixel row.

What is kept of a cell:

- The ink's bounding box, and where it sits in the cell: the blank columns to its left and the
  blank rows above it. The rows above place the letter on the line, so the cells of one font
  normally all have the font's height and each letter is drawn at its true height inside its
  cell.
- For a cell with ink, the cursor afterwards moves by the **ink's width**, not the cell's.
  Blank columns left of the ink shift where it is drawn without making the glyph wider, so
  cells are best made exactly as wide as their ink.
- A cell with no ink at all is a blank glyph that moves the cursor by the cell's width (the
  space is one).

Characters a font does not have are still there as tiny cells, so that the numbering holds.

The menus' sheets hold `0xE0` (224) cells, for the characters `0x20` .. `0xFF` in order.

### Colours

A glyph's pixels are palette numbers. Only the low four bits are used as colour: a font is
drawn with a set of sixteen colours, and the same glyphs can be drawn with different sets
(white text, yellow highlighted text and so on). The sets come from `PAL_00.PCX`: its palette
of 256 colours is sixteen sets of sixteen.

### The race font's smoothing sheet

`FNT22_00.PCX` has the same layout of cells as `FNT22_01.PCX`. The low four bits of each of
its pixels become the top four bits of the glyph sheet's pixel in the same place: an opacity
of 0 .. 15 for soft edges. If the `_00` file is missing, the builder makes the opacity itself
from each glyph's shape (a pixel counts for more the more of its neighbours are ink).

## A glyph table

This is what the builder makes in memory, and what the `.BUN` files hold on disk.

| Part | Size | Content |
| --- | --- | --- |
| Offsets | 4 × the number of glyphs | For each glyph, a u32: where its record starts, counted from the start of the table. |
| Records | varies | One after another, in the same order. |

There is no count. The first offset points just past the offsets, so the number of glyphs is
the first offset divided by 4.

A glyph's record:

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 1 | Blank columns left of the ink: the ink is drawn this far right of the cursor. For a glyph with no ink: the cell's width. |
| 1 | 1 | Blank rows above the ink: the ink is drawn this far below the line's top. |
| 2 | 1 | The ink's width, which is also how far the cursor moves. 0 for a glyph with no ink. |
| 3 | 1 | The ink's height: the number of rows that follow. |
| 4 | | The rows. |

Each row is a list of runs, ended by a zero byte:

| Bytes | Meaning |
| --- | --- |
| count (1) | How many pixels of ink follow. 0 ends the row. |
| gap (1) | How many blank pixels to skip before them. |
| pixels (count) | The ink, one byte a pixel. |

A row with no ink is a single zero byte. Blank pixels at the end of a row are not stored.

This layout is the one for fonts with one byte a pixel, which is what the files hold. (In
memory the menus' sprites use the same scheme with counts, gaps and pixels of two or four
bytes each.)

A glyph with no ink is stored as five bytes, `w t 00 01 00`: its width, the cell's height less one, no ink width, one row, and that row's end.

### The `.BUN` files

A `.BUN` file is a glyph table and nothing else: no header before it, no palette.

| File | Glyphs | Notes |
| --- | --- | --- |
| `JADE.BUN` | `0x60` (96): characters `0x20` .. `0x7F` | Up to 8 × 8 pixels. All ink is pixel value 0; the game draws it in white. The file must be smaller than `0x1400` (5120) bytes. |
| `INSTALL.BUN` | 2 | See [Installer data](Installer-data). |

## From text to glyphs

Text is a string of bytes ended by a zero byte. A font says which byte is glyph 0 (`0x20` in
all the game's fonts) and which is the last (`0xFF`). Bytes below the first are control
codes:

| Byte | Meaning |
| --- | --- |
| `0x00` | End of text. |
| `0x0A` | New line. |
| `0x0E` n | Move right by n pixels (n is a signed byte). |
| `0x0F` n | Move down by n pixels (signed). |
| `0x10` and up, then n | An extended glyph, for languages with more than 224 characters: glyph number ((byte − `0x10`) × 256 + n) ÷ 2 + the number of normal glyphs. |
| other values | Move left by that many pixels. |

After each glyph the cursor moves by the glyph's width (byte 2 of its record, or byte 0 for
a glyph with no ink) plus the font's spacing, 1 pixel in the game's fonts.

### Kerning

The game's fonts have kerning on. It needs no table in the file: as a line is drawn, the game
keeps the outline of the line's right-hand edge, row by row, for the first 64 rows. Each new
glyph is moved left as far as its own left-hand outline allows without touching that edge.
So a glyph's shape decides its spacing, and a pair like "AV" closes up by itself.

For the sheet's author this means spacing between letters comes from the font's spacing
value and the letters' shapes, not from margins drawn into the cells.

## A made-up example

A glyph table with two glyphs: a space 4 pixels wide, and a 3 × 3 plus sign in colour 1 that
sits one row down in its cell.

```
08 00 00 00                offset of glyph 0
0D 00 00 00                offset of glyph 1
04 00 00 01 00             glyph 0: no ink, width 4, from a cell 1 pixel high
00 01 03 03                glyph 1: 0 columns cut, 1 row cut, 3 wide, 3 rows
01 01 01 00                row 0: 1 pixel after a gap of 1: colour 1
03 00 01 01 01 00          row 1: 3 pixels, no gap
01 01 01 00                row 2: as row 0
```

## Code

`src/gfx/build_fonts.rs`, `src/gfx/build_game_fonts.rs` (which sheets, which palettes),
`src/gfx/build_glyphs.rs`, `rc2k_core/src/gfx/glyphs.rs` (finding and coding the cells),
`src/gfx/load_jade.rs`, `rc2k_core/src/strings/text.rs` (text layout, control codes,
kerning), `src/gfx/typefaces.rs` (the font objects), `rc2k_core/src/gfx/blit.rs` (drawing a
glyph).
