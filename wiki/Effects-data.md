# Effects data

*Category: [Data formats](Data-formats)*

The folder `files/m/` holds what Rally Championship 2000 needs for its weather and dirt
effects: rain, snow, dust and spray behind the cars, drops and flakes on the windscreen, the
dirt that builds up at the edges of the glass. Most of it is pictures
([Textures (PCX)](Textures-PCX)). This page covers the files that are not pictures.

| File | Size | Read by the game |
| --- | --- | --- |
| `files/m/MARKINFO` | `0x8000` (32768) | Yes: where each effect's texture lies on its sheet. |
| `files/m/SPLINE.DAT` | `0xC00` (3072) | Yes: blending weights for smooth curves. |
| `files/m/MASKnn.RAW` | `0x20000` (131072) each | No. |
| `files/m/TABLE` | `0x10000` (65536) | No. |
| `files/m/MUDFILE` | `0x10000` (65536) | No (see [Mud files](Mud-files)). |

All numbers are little-endian. "u16" is an unsigned 16-bit number, "f32" a 32-bit float.
File names are matched without regard to case.

## MARKINFO

The effects' textures are small pieces of a few shared picture sheets (pictures 900 .. 915).
`MARKINFO` says where on its sheet each piece is. It is 256 records of `0x80` bytes with no
header; record *n* is at *n* × `0x80`.

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u16, u16 | Corner 0 of the piece: u, v. |
| `0x04` | u16, u16 | Corner 1. |
| `0x08` | u16, u16 | Corner 2. |
| `0x0C` | u16, u16 | Corner 3. |
| `0x10` | `0x70` bytes | Not used by the effects. A few records of the game's file have values up to `0x17`; the rest is zero. |

Coordinates are in 1/256ths of a pixel: `0x6000` is pixel 96, `0x7FFF` just short of pixel
128. In the game's file the corners are those of an upright rectangle, in most records in
the order (low u, low v), (high u, low v), (low u, high v), (high u, high v). A record of all zeros is
an unused number.

The file is used in two places:

- It is one of the three files that make up the game's texture table, where it supplies the
  records of texture numbers 14400 .. 14655, replacing whatever `files/a/TEXTINFO` has
  there (see [Object and texture tables](Object-and-texture-tables); the record shape is the
  same).
- The effects code keeps its own copy of the 256 records. When the effects are set up, each
  corner pair is multiplied by a scale factor, and a slope is worked out for each record
  from its corners (the piece's height over its width).

Which record belongs to which effect is decided by tables built into the game.

## SPLINE.DAT

Some effects move their points along smooth curves through a few control points.
`SPLINE.DAT` is a ready-made table of the weights for that: 256 entries of 12 bytes, no
header.

| Offset in entry | Type | Meaning |
| --- | --- | --- |
| `0x00` | f32 | The weight of point C. |
| `0x04` | f32 | The weight of point B. |
| `0x08` | f32 | The weight of point A. |

Given three points A, B and C (whole-number x, y, z) and an entry number 0 .. 255, the game
makes the blended point

```
P = A × weight_A + B × weight_B + C × weight_C
```

for each of x, y and z. In the game's file the entries go round a closed loop: entry 0 is
all on A, entry 128 all on C, and entry 255 is nearly back at entry 0, with B pulling the
curve outwards in between; the three weights of an entry add up to about 1.

The file is read once, the first time a stage's effects are set up. If it is shorter than
`0xC00` bytes the missing entries are zero.

## MASKnn.RAW

Twenty-three files, `MASK00.RAW` .. `MASK22.RAW`, numbered like the mask pictures
`MASKnn.PCX` beside them. Each is a 256 × 128 picture with no header, 4 bytes a pixel, in
which all four bytes of a pixel are the same: 0 or `0xFF`.

The game does not read them. It reads the pictures `MASKnn.PCX` and `BMASKnn.PCX` from the
same folder and builds the same kind of block from them in memory (white pixels with the
mask as their transparency). The `.RAW` files have the size and shape of such a block.

## TABLE

A file of `0x10000` bytes without a header. The game does not read it, and what it holds is
not known.

## A made-up example

A `MARKINFO` record for a piece that covers pixels 32 .. 63 across and 96 .. 127 down on
its sheet:

```
00 20 00 60   corner 0: u 0x2000 (32),    v 0x6000 (96)
FF 3F 00 60   corner 1: u 0x3FFF (63.99), v 0x6000
00 20 FF 7F   corner 2: u 0x2000,         v 0x7FFF (127.99)
FF 3F FF 7F   corner 3: u 0x3FFF,         v 0x7FFF
... zeros to 0x80
```

A `SPLINE.DAT` entry that puts the point halfway between A and C, pulled a little towards
B: the three floats 0.45, 0.1, 0.45, that is

```
66 66 E6 3E  CD CC CC 3D  66 66 E6 3E
```

Code: `src/effects/blocks.rs` (the corner records and their scaling), `src/race/level_init.rs`
(the texture table), `src/effects/splines.rs` and `src/effects/blend_points.rs`
(`SPLINE.DAT`), `src/effects/load_mask_pictures.rs` (the masks the game does read).
