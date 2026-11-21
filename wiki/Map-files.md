# Map files

*Category: [Data formats](Data-formats)*

The map shown in a corner of the screen during a race is a scan of a paper map, cut into
small square tiles. Each stage has its own file, `files/maps/MAPnn.SQ`, where `nn` is the
course's two-digit number, the same as in `files/levels/LEVELnn` (see
[Course files](Course-files)). Only the tiles the route passes near are stored.

Despite the extension, a map file is not packed: it has nothing to do with the packing on
[Packed files (.SQ)](Packed-files-SQ) beyond a similar marker.

All numbers are little-endian. File names are matched without regard to case.

## The picture

The whole map is 128 × 128 tiles of 16 × 16 pixels: `0x800` (2048) pixels each way. Tiles
are counted from the top left, row by row; tile (*tx*, *ty*) is entry *ty* × 128 + *tx*.

## Layout

| Part | Offset | Size | Notes |
| --- | --- | --- | --- |
| Header | `0x0000` | `0x10` | |
| Tile table | `0x0010` | `0x8000` | 128 × 128 entries of 2 bytes. |
| Tile pixels | `0x8010` | count × `0x300` | One block of `0x300` (768) bytes for each stored tile. |

### Header

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `SQSM` |
| `0x04` | 4 | u32 | Not read. `0` in the game's files. |
| `0x08` | 4 | u32 | Version, `0x00050000`. Any other value and the game shows no map. |
| `0x0C` | 4 | u32 | The number of stored tiles, 1 .. `0xFFFF`. |

### Tile table

One u16 for each of the `0x4000` (16384) tiles of the whole map, in row order:

| Value | Meaning |
| --- | --- |
| 0 | No tile here; the game draws black. |
| *n* (1 .. count) | The tile's pixels are block *n* − 1 of the tile pixels. |

In the game's files the stored tiles are numbered 1, 2, 3 ... in the order they appear in
the table, and every block is used by exactly one entry. The reader does not need this: it
looks each tile up by its number.

### Tile pixels

A block is one tile: 16 rows of 16 pixels, top row first, 3 bytes a pixel in the order
blue, green, red. Block *n* starts at `0x8010` + *n* × `0x300`.

The file's length is exactly `0x8010` + count × `0x300`.

## How the game uses it

The file is read when a stage loads. The pixels are converted to the screen's own pixel
format and kept in memory; the in-race display then copies the tiles around the car's place
on the map. A course's checkpoints carry their place on this map (see
[Course files](Course-files)), which is how the game knows where on the scan the car is.

If the file is missing, has another marker or another version, the stage loads with an empty
(black) map.

The loader also knows a developer's form that the shipped game does not contain: a full
2048 × 2048 picture `srcmaps\mapNN.pcx` with a second picture `srcmaps\keyNN.pcx` that marks
which tiles are in use. When `srcmaps\mapNN.pcx` exists it is used instead of the `.SQ`
file: every tile within three tiles of a marked one is kept. The `.SQ` file is the result of
that selection, saved.

## A made-up example

A map with two stored tiles, side by side at (5, 3) and (6, 3):

| Offset | Bytes | Meaning |
| --- | --- | --- |
| `0x0000` | `53 51 53 4D` | `SQSM` |
| `0x0004` | `00 00 00 00` | not read |
| `0x0008` | `00 00 05 00` | version `0x00050000` |
| `0x000C` | `02 00 00 00` | 2 tiles |
| `0x0010` | zeros | tiles 0 .. 388: none |
| `0x031A` | `01 00` | entry 3 × 128 + 5 = 389: tile 1 |
| `0x031C` | `02 00` | entry 390: tile 2 |
| `0x031E` | zeros | the rest of the table: none |
| `0x8010` | `0x300` bytes | tile 1: 256 pixels, each blue, green, red |
| `0x8310` | `0x300` bytes | tile 2 |

The file is `0x8010` + 2 × `0x300` = `0x8610` (34320) bytes long.

Code: `src/race/game_map.rs` (both loaders), `src/race/blit_map_tiles.rs` and
`src/race/map_scroll.rs` (the display).
