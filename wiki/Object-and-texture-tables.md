# Object and texture tables

*Category: [Data formats](Data-formats)*

Next to the object files and texture pictures of Rally Championship 2000 lie a few table
files with no extension. Each is a plain array of fixed-size records, one for every object
kind or every texture the folder could hold, used or not. There is no header: a record's
place in the file is its number.

| File | Records | Record size | One record for each |
| --- | --- | --- | --- |
| `files/a/OBJSINFO` | `0x190` (400) | `0x80` | object kind 0 .. 399 (scenery) |
| `files/b/CARSINFO` | `0xC8` (200) | `0x80` | object kind 400 .. 599 (cars) |
| `files/g/frnt/FRNTINFO` | `0xC8` (200) | `0x80` | object kind 600 .. 799 (front end) |
| `files/s/extr/EXTRINFO` | `0x190` (400) | `0x80` | object kind 800 .. 1199 (extras) |
| `files/a/TEXTINFO` | `0x3E80` (16000) | `0x80` | texture number 0 .. 15999 (pictures 0 .. 999) |
| `files/b/TEXTINFO` | `0x3E80` (16000) | `0x80` | texture number 16000 .. 31999 (pictures 1000 .. 1999) |
| `files/m/MARKINFO` | `0x100` (256) | `0x80` | texture number 14400 .. 14655 (pictures 900 .. 915: the effects) |
| `files/s/data/LDATA` | `0x22` (34) | `0x90` | car model |

All numbers are little-endian. "u16" is an unsigned 16-bit number, "i32" a signed 32-bit
one. File names are matched without regard to case. A record that is all zeros means
"nothing here".

The game reads each file over a table of its own that starts out zeroed (or, for a few
tables, with built-in values). A file that is missing or short simply leaves the rest of the
table as it was.

## Object kinds: OBJSINFO, CARSINFO, FRNTINFO, EXTRINFO

The four files together make one table of 1200 records, indexed by object kind: the number
in an object's file name ([Object files](Object-files)). Record *k* − 400 of `CARSINFO`
belongs to `files/b/OBJ0`*k*`.RO3`, and so on.

A record is `0x80` bytes. The fields the game is known to read:

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x1D` | u16 | Another object kind to draw in this one's place when it is far away (a simpler model); 0 for none. |
| `0x20` | u8 | How the object is turned when drawn. 0: as placed on the course. 1 or 2: always facing the viewer. 4: the object has two shapes, one stored right after the other, and moves between them. 5: turned with the view about the upright axis. |
| `0x22` | u16 | The distance beyond which the object fades out (and is replaced by the kind at `0x1D`, if any); 0 for never. |

In the game's files the first twelve bytes hold three i32, and several other single bytes
up to `0x24` are set; what those mean is not known. Everything from `0x28` on is zero.

## Textures: TEXTINFO

A texture picture (`TXnnnn.PCX`, see [Textures (PCX)](Textures-PCX)) is a sheet that can
hold up to 16 separate textures. A **texture number** is the picture's number times 16 plus
the place 0 .. 15 on the sheet, and `TEXTINFO` has one record for each texture number: record
*n* is at offset *n* × `0x80`.

`files/a/TEXTINFO` covers pictures 0 .. 999, which is every picture a stage's ground and
scenery use. `files/b/TEXTINFO` would cover pictures 1000 .. 1999 (the cars'), but in the
game it is all zeros: cars take their texture coordinates from their object files.

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | 4 × (u16, u16) | The texture's four corners on the sheet, as (u, v) pairs in 1/256ths of a pixel: `0x7F00` is pixel 127. Corner *k* goes with point *k* of a course face, so the order also says how the texture is turned or mirrored on the face. |
| `0x10` | u32 | Copied into the game's texture table; not known. |
| `0x14` | u32 | Copied into the game's texture table; not known. |
| `0x18` | u8 | Copied into the game's texture table; not known. |
| `0x20` | u32 | Copied into the game's texture table; not known. |
| `0x24` | u8 | A surface kind number. The game uses it to choose how a face's second texture is drawn, and when it works out how much of a stage is on each kind of ground. |
| `0x25` | i8 | Mapping. 0: the corners above are used. Above 0: the texture is laid over the ground by position instead, repeating; the coordinates are the corner points' x and z shifted left by (value & 3) + 2 bits. Values below 0 also occur in the game's files; the corners are used as for 0. |
| `0x26` | u8 | Copied into the game's texture table; not known. |
| `0x40` | `0x40` bytes | In a few records, a name as text padded with spaces. Not read. |

When a stage loads, the game looks up the record of each of the stage's 256 textures (listed
by texture number in `LOCALDAT`, see [Course files](Course-files)) and keeps a short copy.
A face's texture coordinates are then the record's corner pairs, doubled for pictures
400 .. 799.

## Effect textures: MARKINFO

`files/m/MARKINFO` has the same record shape for the 256 texture numbers of pictures
900 .. 915, which hold the weather, spray, dust and windscreen effects' textures. Only the
start of each record is used: the four corner pairs, and in some records more 16-bit values
up to `0x17` that the effects code scales along with them.

The game loads it twice: over that part of the texture table (it is read after
`files/a/TEXTINFO` and replaces those 256 records), and into the effects' own corner table.
See [Effects data](Effects-data).

## Car lights: LDATA

`files/s/data/LDATA` gives, for each of the 34 car models, where its lights are. A record
is `0x90` (144) bytes: twelve places of three i32 (x, y, z) in the car's own space. A place
of (0, 0, 0) is not used.

The game copies each record into its table of car models and draws the lights' glow at those
places: headlights, rear lights and the like. A place stands for a pair of lights: one at the place itself and its twin at the same place with x negated. In the
game's files the first four places are at the front of the car, the next five at the back,
and the last three are empty. Which place is which lamp is not known in more detail.

The file is `0x22` × `0x90` = `0x1320` (4896) bytes.

## Files the game does not read

Three more files without an extension lie in these folders: `files/s/data/CARINFO`
(`0x8000` bytes), `files/g/frnt/L` and `files/a/1`. Nothing in the program opens them, and
what they hold is not known.

## A made-up example

A `TEXTINFO` record for a texture that fills the top left quarter of its sheet (pixels
0 .. 127 each way), drawn plainly, surface kind 2:

```
00 00 00 00  00 7F 00 00  00 00 00 7F  00 7F 00 7F    corners (0,0) (127,0) (0,127) (127,127)
00 00 00 00  00 00 00 00  00 00 00 00  00 00 00 00
00 00 00 00  02 00 00 00  00 00 00 00  00 00 00 00    0x24: surface kind 2; 0x25: mapping 0
... zeros to 0x80
```

If this is place 3 of picture 210, it is texture number 210 × 16 + 3 = 3363, at offset
3363 × `0x80` = `0x69180` of `files/a/TEXTINFO`.

Code: `src/scene/read_kind_records.rs` and `rc2k_core/src/scene/kind_records.rs` (object
kinds), `src/race/level_init.rs` and `rc2k_core/src/race/stage_files.rs` (the texture
tables), `rc2k_core/src/race/stage.rs` (`texture_info`), `rc2k_core/src/race/faces.rs`
(`texture_coordinates`), `src/race/stage_prefs.rs` and `src/race/light_records_store.rs`
(`LDATA`).
