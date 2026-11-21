# Textures and pictures (PCX)

*Category: [Data formats](Data-formats)*

Most of the game's pictures are PCX files: every texture, the sprites and font sheets, the
masks, some full-screen pictures, and the screenshots the game writes. This page says which
kinds of PCX the game reads, how it turns them into pixels, and what a file must look like to
be accepted.

Numbers are little-endian. "Word" means an unsigned 16-bit number.

## Where the files are

| Files | What they are |
| --- | --- |
| `files/a/TXnnnnn.PCX`, `files/b/...`, `files/s/extr/...`, `files/g/frnt/...` | Textures of scenery, cars, extras and front-end models. Which object uses which is on [Object and texture tables](Object-and-texture-tables). |
| `files/m/*.PCX`, `files/s/data/*.PCX` | Pictures of the effects (see [Effects data](Effects-data)) and loading and title pictures. |
| `files/d/fnt/*.PCX` | Font sheets and the fonts' palette: see [Fonts](Fonts). |
| `files/d/gfx/BACKDROP.PCX`, `CHATZONE.PCX` | The installer's and the network lobby's backgrounds. |
| `files/g/graphics/sprites/*.PCX`, `files/g/graphics/screens/*.PCX` | The menus' sprite sheets and a screen picture. |
| `savedata/snapshot/NNN.pcx`, `NNN.pcx` | Screenshots the game writes: see [Screenshots](Screenshots). |

A picture named with `.png` whose file is missing is looked for with `.pcx` instead, so a PCX
can stand in for any [PNG screen](Screens-PNG).

## What the game accepts

Two kinds, told apart by the header's plane count:

| Planes | Kind | Pixels |
| --- | --- | --- |
| 1 | 8 bits a pixel with a palette of 256 colours | each byte is a palette number |
| 3 | 24 bits a pixel | three planes of 8 bits: red, green, blue |

Any other plane count is refused. The game does not look at the header's first four bytes
(maker, version, coding, bits a plane); it always treats the data as run-length coded with 8
bits a plane. Write them as the standard has them all the same: `0A 05 01 08`.

## Layout of a file

| Part | Offset | Size |
| --- | --- | --- |
| Header | `0` | `0x80` (128) |
| Rows, run-length coded | `0x80` | varies |
| Palette (1 plane only) | end − `0x300` | `0x300` (768) |

### The header

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x00` | 1 | `0x0A`. Not checked. |
| `0x01` | 1 | Version, `5`. Not checked. |
| `0x02` | 1 | Coding, `1` (run-length). Not checked. |
| `0x03` | 1 | Bits a plane, `8`. Not checked. |
| `0x04` | 2 | Left edge. |
| `0x06` | 2 | Top edge. |
| `0x08` | 2 | Right edge. |
| `0x0A` | 2 | Bottom edge. |
| `0x0C` | 4 | Dots an inch, two words. Not read. |
| `0x10` | `0x30` | A 16-colour palette. Not read. |
| `0x41` | 1 | Planes: `1` or `3`. |
| `0x42` | 2 | Bytes in one plane of one row. |
| the rest | | Not read. |

The picture's width is right − left + 1 and its height bottom − top + 1.

The bytes-a-row value may be larger than the width (the standard rounds it up to an even
number). The extra bytes of each row are decoded and thrown away. In the game's own files it
always equals the width.

### The rows

After the header come the rows, top to bottom. A row is its planes one after another, each
"bytes a row" long: for one plane just the palette numbers; for three planes all the row's
red bytes, then its green bytes, then its blue bytes.

The bytes are run-length coded:

| Byte | Meaning |
| --- | --- |
| `0x00` .. `0xBF` | This byte itself, once. |
| `0xC1` .. `0xFF` | A count: the next byte, (count − `0xC0`) times. Runs are 1 to 63 long. |
| `0xC0` | A run of none. Never write it: the game takes it as a run that never ends. |

A byte value of `0xC0` or more must therefore be written as a run, even a run of one
(`C1 F3`).

Runs and rows:

- In a one-plane file a run may carry on from one row into the next.
- In a three-plane file a run may carry on from one plane into the next plane of the same
  row, but **not into the next row**: the game repeats 0 from the start of the next row for
  what is left of such a run. End every run at the end of a row.

A file that ends early is filled up with zeros (palette entry 0, or black).

### The palette

A one-plane file ends in its palette: the last `0x300` bytes of the file, 256 entries of
three bytes, red, green, blue, each 0 .. 255. The game finds it by counting back from the end
of the file. The standard puts a marker byte `0x0C` before the palette; the game does not
look for it, but write it for other programs' sake.

## What the pixels become

Every picture is decoded to 32-bit pixels, `0x00RRGGBB` (stored blue byte first), and
converted from there to whatever the screen or the texture needs.

### Size limits

A picture is decoded into a buffer of a fixed size, and anything outside it is cut off (the
right-hand columns and the bottom rows):

| Use | Buffer | A different size |
| --- | --- | --- |
| Texture | `0x100` × `0x100` (256 × 256) | Smaller pictures fill the top left corner; the rest of the 256 × 256 is white. Larger ones are cut. |
| Full-screen picture | 800 × 600 | A picture whose width is not the screen's is stretched to the screen. Larger ones are cut first. |
| Sprite and font sheets | 800 × 600 | Used at their own size. |

Textures in the game's files are 256 × 256, 256 × 128 and 128 × 128.

### See-through textures

A PCX has no alpha channel. Textures get their see-through parts in one of two ways, chosen
by the texture's record (see [Object and texture tables](Object-and-texture-tables)):

**By colour.** Each pixel is looked at once:

- a pixel that is magenta, counting only the top five bits of each of red, green and blue (red
  and blue 248 or more, green below 8), becomes see-through;
- a pixel that is black by the same test (all three below 8) is nudged to the darkest colour
  that is not zero and stays solid, because zero itself is kept for "see-through";
- every other pixel is solid as it is.

So magenta means "nothing here", and black can be drawn freely.

**By a second picture.** The texture names a second PCX of the same size, a mask. Each
pixel's opacity is the inverse of the mask pixel's blue value: a black mask pixel is solid, a
white one fully see-through. Magenta has no special meaning then. If the mask cannot be read,
the texture falls back to the colour rule.

When a texture is made smaller for distant drawing, each new pixel is the average of four,
and see-through pixels are left out of the colour average.

### See-through sprites and overlays

- A picture laid over another (an overlay): pure magenta, `FF 00 FF`, is see-through.
- Sprite and font sheets: see [Fonts](Fonts). The backdrop is the colour the sheet's four
  corners share, and magenta inside a sprite is see-through.

### Colour grading

Textures can be brightened or made more or less colourful as they are read, by two settings
(scale and saturation, both `0x100` for "leave alone"). This changes what is drawn, not what
a file must contain.

## Writing a PCX the game reads

1. Header: `0A 05 01 08`, the four edges (0, 0, width − 1, height − 1), planes 1 or 3, bytes
   a row = the width. Zeros elsewhere are fine.
2. Rows from the top, planes in the order red, green, blue, run-length coded, every run
   ended at the end of its row, no `C0` count bytes, values of `0xC0` and up always as runs.
3. For one plane: `0C`, then the 768 palette bytes, as the last bytes of the file.
4. For a texture, keep to 256 × 256 or smaller, and use magenta only where the texture is
   to be see-through.

## A made-up example

A 4 × 2 picture with a palette, the top row all colour 5, the bottom row colours 1, 2, 2,
200:

```
0A 05 01 08  00 00 00 00  03 00 01 00  ...      header: edges 0,0 .. 3,1
... 01 at 0x41, 04 00 at 0x42 ...               1 plane, 4 bytes a row
C4 05                                            row 0: colour 5 four times
01 C2 02 C1 C8                                   row 1: 1, then 2 twice, then 200 once
0C                                               palette marker
(768 bytes)                                      palette
```

The file is `0x80` + 2 + 5 + 1 + `0x300` = `0x388` (904) bytes long.

The same picture in 24 bits would have, for row 0, the red bytes of its four pixels, then
the green, then the blue, for example `C4 FF C8 00` for four pixels of pure red (red plane
`FF` × 4, then green and blue together as one run of eight zeros, which is allowed because
it stays inside the row).

## Code

`src/gfx/read_pcx.rs`, `src/gfx/read_pcx_8bit.rs`, `src/gfx/read_pcx_24bit.rs`,
`src/gfx/lay_out_pcx.rs`, `rc2k_core/src/gfx/pcx.rs` (the decoder and the screenshot
writer), `src/gfx/read_picture_file.rs` (PNG or PCX), `src/gfx/texture_pixels.rs` and
`rc2k_core/src/gfx/pixels.rs` (see-through colours, masks, grading),
`src/gfx/load_picture_to_texture.rs`.
