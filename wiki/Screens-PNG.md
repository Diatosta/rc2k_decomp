# Screens (PNG)

*Category: [Data formats](Data-formats)*

The menus' backgrounds and information pages are PNG files. The game reads them with a small
PNG decoder of its own, which handles only the kind of PNG its files are. This page says
exactly what that is, so that a replacement picture is accepted.

PNG's own numbers are big-endian (most significant byte first), unlike the game's other
files.

## Where the files are

| Folder | What is in it |
| --- | --- |
| `files/g/graphics/screens/` | The menu screens' backgrounds, 800 × 600. |
| `files/g/graphics/screens/maps/` | The stage maps shown before a stage, 401 × 502. |
| `files/g/graphics/screens/cspecs/` | The cars' specification pages, 800 × 600. |
| `files/g/slides/` | The cars' slideshow pictures, 640 × 480 (a few are narrower). |

If a `.png` named by the game does not exist, the same name with `.pcx` is tried (see
[Textures and pictures (PCX)](Textures-PCX)). A file that exists but is not a PNG the game
accepts is also tried as a PCX, which fails for a real PNG: no picture is loaded then.

## What the game accepts

| Property | Accepted |
| --- | --- |
| Signature | The standard eight bytes, `89 50 4E 47 0D 0A 1A 0A`. |
| First chunk | `IHDR`, straight after the signature, 13 bytes long. |
| Width | 2 .. 800. |
| Height | 1 .. 600. |
| Bit depth | 8 only. |
| Colour type | 2 (red, green, blue) or 3 (palette). Type 0 (grey) is decoded as palette numbers without a palette of its own, so the colours are not dependable. Types 4 and 6 (with alpha) are refused. |
| Interlacing | None (the interlace byte must be 0). |
| `IDAT` | Exactly **one** `IDAT` chunk, holding the whole picture. |
| `PLTE` | For colour type 3: a full palette of 256 entries (`0x300` bytes). The game always reads 256 entries. |
| Row filters | All five standard filters (0 .. 4). |
| Other chunks | Skipped. Check sums are not looked at. |
| End | An `IEND` chunk; the chunks are walked until it is found. |

Two of these trip up ordinary programs:

- **One `IDAT` chunk.** Many programs split the picture's data over several `IDAT` chunks of
  a few kilobytes. The game refuses such a file. Save with a single chunk (some tools call
  this "no chunking" or let you set the chunk size above the file's size), or rewrite the
  file with a script that joins the chunks' data into one.
- **No alpha, no 16 bits.** Save as plain 24-bit colour.

A PNG has no see-through parts for the game; every pixel is drawn.

## How a picture is used

The picture is decoded to 32-bit pixels, `0x00RRGGBB`. A full-screen picture whose width is
not the screen's width is stretched to the screen, so an 800 × 600 picture also works when
the menus run at another size.

Widths other than these are used by the pages that draw a picture inside a screen (the maps
and the specification pages, for example); keep a replacement at the size of the file it
replaces.

## Layout reminder

For someone writing a file by hand, the least the game needs:

| Bytes | Content |
| --- | --- |
| 8 | Signature. |
| 4, 4 | `00 00 00 0D`, `IHDR`. |
| 13 | Width (4), height (4), bit depth `08`, colour type `02`, compression `00`, filter `00`, interlace `00`. |
| 4 | Check sum of the chunk. |
| 4, 4, n, 4 | Length n, `IDAT`, the zlib stream of the rows, check sum. |
| 4, 4, 4 | `00 00 00 00`, `IEND`, check sum. |

Before compression each row is one filter byte followed by width × 3 bytes (red, green,
blue), so the data is (width × 3 + 1) × height bytes.

## A made-up example

A 2 × 2 picture, top row red and green, bottom row blue and white, every row with filter 0,
is these 14 bytes before compression:

```
00  FF 00 00  00 FF 00
00  00 00 FF  FF FF FF
```

## Code

`src/gfx/read_png.rs` (the checks before decoding), `src/gfx/decode_png.rs`,
`rc2k_core/src/gfx/png.rs` (chunks, inflating, filters), `src/gfx/lay_out_png.rs`,
`src/gfx/read_picture_file.rs` (PNG first, then PCX).
