# Mud files

*Category: [Data formats](Data-formats)*

On a muddy stage of Rally Championship 2000 the player's car gets dirtier as the race goes
on. The dirt is painted into the car's textures while you drive, and a mud file says where on
those textures mud gathers: a lot behind the wheels and along the sills, little or none on
the roof and glass.

| File | What it is |
| --- | --- |
| `files/b/MUDFILE.0xx` | The mud map of one car. `xx` is the car's two-digit file number (the same number the game uses for the car's other files), so the names run `MUDFILE.000`, `MUDFILE.001` ... |
| `files/m/MUDFILE` | A file of the same size and shape. The game never opens it. |

File names are matched without regard to case.

## Layout

A mud file is exactly `0x10000` (65536) bytes with no header: four maps of `0x4000` (16384)
bytes, one after another.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x0000` | `0x4000` | The map of the car's texture 0. |
| `0x4000` | `0x4000` | The map of texture 1. |
| `0x8000` | `0x4000` | The map of texture 2. |
| `0xC000` | `0x4000` | The map of texture 3. |

A car's body is drawn from four textures of 256 × 256 pixels. Each map is 128 × 128 bytes,
row by row from the top, one byte for every 2 × 2 block of pixels of its texture. The byte
for pixel (x, y) of texture *n* is at

```
n * 0x4000 + (y / 2) * 128 + (x / 2)
```

Each byte is a **strength**, 0 .. `0xFF`:

| Strength | Effect |
| --- | --- |
| 0 | Mud never lands here. |
| 1 .. `0x1F` | Mud thrown up by the wheels can tint these pixels, a little. Splashes do not land. |
| `0x20` .. `0xFF` | The higher, the more often a splash lands and the more strongly thrown mud tints. |

A file shorter than `0x10000` bytes leaves the rest of the maps at 0; a missing file gives a
car that stays clean.

## How the game uses it

When a stage loads, the game reads the mud file of the player's own car, and takes a set of
mud colours from a small picture, tinted towards the stage's own mud colour (set in the
stage's `LOCALDAT`, see [Course files](Course-files)).

During the race the wheels add to two running amounts for each of the four textures, mud
"thrown" and mud "splashed". Every 14 frames the game takes the next of the four textures
and spends its amounts:

- A **throw** takes the next pixel of a fixed shuffled order and moves its colour towards the mud colour by
  (strength + 1) / 2 parts in 256.
- A **splash** picks a random place; if the strength there is at least `0x20` and a random
  byte is not above it, a 4 × 4 patch of pixels is painted with one of sixteen mud patterns.

Only pixels whose top bit is set in the texture take mud. The mud is on the textures, not in
any file: it is gone at the next stage.

## A made-up example

To make mud gather only on the lower half of texture 1 and stick most at the very bottom:
bytes `0x4000` .. `0x5FFF` (rows 0 .. 63 of map 1) are 0, the rows below rise from `0x20` in
row 64 to `0xE0` in row 127, and the other three maps are all 0.

Code: `src/race/mud.rs` (loading), `rc2k_core/src/race/mud.rs` (the map's rules),
`src/race/update_mud_texture.rs`, `src/race/throw_mud_texel.rs`, `src/race/splash_mud.rs`.
