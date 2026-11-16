# Course files

*Category: [Data formats](Data-formats)*

A course is the 3D world of one stage of Rally Championship 2000: the ground, the road, where
the scenery stands, the checkpoints along the route. Each course has a folder with two files:

| File | What it is |
| --- | --- |
| `files/levels/LEVELnn/CDAT00C.SQ` | The course itself, packed. |
| `files/levels/LEVELnn/LOCALDAT` | The stage's settings: which textures and objects it uses, its light, fog and sky. |

`nn` is the course's two-digit number, 00 .. 99. The game looks for all hundred and
remembers which exist; which stage of which rally uses which course is in the game's
built-in stage tables. The same number names the stage's map, `files/maps/MAPnn.SQ`
([Map files](Map-files)).

All numbers are little-endian. "u32" is an unsigned 32-bit number, "i32" a signed one.
File names are matched without regard to case.

## The course: CDAT00C.SQ

The file is packed with the scheme on [Packed files (.SQ)](Packed-files-SQ). Everything
below describes the unpacked bytes. (If an unpacked copy exists at
`srclvls\LEVELnn\CDAT00C`, the game reads that instead.)

### Overall layout

| Part | Offset | Size |
| --- | --- | --- |
| Header | `0` | `0x100` |
| Cells | `0x100` | cell count × `0x20` |
| Faces | after the cells | face count × `0x40` |
| Points | after the faces | point count × `0x0C` |

The file ends with the last point. The game gives a course `0xF00000` bytes of room; the
space after the points is used for points it makes while drawing.

### Header

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | 4 bytes | `sout` in most of the game's files, zeros in a few. Not checked. |
| `0x04` | u32 | Number of cells. |
| `0x08` | u32 | Number of faces. Must be 1 .. `0xFFFFF`. |
| `0x0C` | u32 | Number of points. |
| `0x10` | u32 | Not read. |
| `0x14` | u32 | A face number kept with the start place below; `0xFFFFFFFF` for none. |
| `0x2C` | u32 | A camera figure of the saved view. |
| `0x4C` | u32 | The saved view's heading. |
| `0x6C` | u32 | The saved view's depth figure. |
| `0x8C`, `0xAC`, `0xCC` | i32 | The saved view's place: x, y, z. |

Everything else in the header is not read. Each of the fields from `0x2C` on is the first
of a row of eight u32, of which the game uses only the first. It loads these values into its
camera variables and overwrites them as soon as a race begins, so they matter little.

### Points

A point is three i32: x, y, z. Coordinates are fixed-point, 65536 (`0x10000`) to a unit, the
same as a car's position in a [replay](Replay-files); y is height. Faces name points by
number, counted from 0.

### Cells

The ground plan is divided into square cells of `0x200000` (2097152, 32 units) a side. A
cell record lists the faces and points that lie in one cell, so that the game can find the
ground under a car or the faces in view without searching the whole course.

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u32 | The cell's key: `((z >> 21) & 0x7FF) << 11 | ((x >> 21) & 0x7FF)`. `0xFFFFFFFF` in the last record, which is empty. |
| `0x04` | u32 | 0. |
| `0x08` | u32 | First face of the cell; `0xFFFFFFFF` if it has none. |
| `0x0C` | u32 | Last face of the cell (included). |
| `0x10` | u32 | First point of the cell; `0xFFFFFFFF` if it has none. |
| `0x14` | u32 | Last point of the cell (included). |
| `0x18` | 8 bytes | Not read. |

Every face and every point belongs to exactly one cell, and each cell's faces and points are
a run of consecutive numbers. A point is filed under the cell it lies in. Each key appears
once.

After loading, the game sorts the cell records into 256 groups by a small hash of the key
(`(key & 0xF) + ((key >> 11) & 0xF) * 16`), keeping the order within a group, and looks a
cell up by walking its group. The order of the records in the file therefore does not
matter.

### Faces

A face is `0x40` (64) bytes. Its first byte says what kind of thing it is:

| First byte | What the face is |
| --- | --- |
| `0x00` .. `0x0F` | A piece of ground or scenery surface that is drawn and driven on. |
| `0xFF` | A marker: an object standing on the course, a checkpoint, a camera, a sound or a lamp. The second byte says which. |
| other | Not drawn and not used as a marker (`0x80` occurs a few times in the game's files). |

#### A drawn face (`0x00` .. `0x0F`)

A face is a quadrilateral made of two triangles (points 0, 1, 2 and points 3, 2, 1), or a
single triangle when point 3 is the same as point 2. It can carry two textures, the second
blended over the first.

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u8 | How the first texture is drawn: bits 0 .. 2 are a drawing order figure and bit 3 a back-face switch (the exact effect of each value is not known). |
| `0x01` | u8 | The first texture: one of the stage's 256 textures, which `LOCALDAT` lists (below). |
| `0x02` | u16 | Switches. Bits 1, 3, 4 and 5 hide the face in some views; bit 2 says the face takes light from nearby lamps (the lamp numbers are then at `0x28` and `0x2C`); bit 9 is set by the game at load for a face with two points alike; bit 12 marks one special surface for the wheels. |
| `0x04` | 4 × u32 | The face's four points. |
| `0x14` | 4 × u8 | Each corner's light, an index into the stage's light table. |
| `0x18`, `0x1C` | 2 × u32 | Filled in by the game at load (the x of the two triangles' normals). |
| `0x20` | u8 | How the second texture is drawn, as `0x00`. |
| `0x21` | u8 | The second texture; 0 for none. |
| `0x22` | 4 × u8 | Each corner's blend: how strongly the second texture shows there. |
| `0x26` | 2 bytes | Not known. |
| `0x28` | 4 × u8 | Each corner's light on the back of a two-sided face; or, with switch bit 2, each corner's first lamp. |
| `0x2C` | 4 × u8 | Each corner's blend on the back; or, with switch bit 2, each corner's second lamp. |
| `0x30` | 4 × u32 | Filled in by the game at load (the y and z of the two normals). |

A tool that writes a course can leave the fields the game fills in as zero.

#### A marker (`0xFF`)

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u8 | `0xFF` |
| `0x01` | u8 | What it marks (next table). |
| `0x04` | u32 | The point where it stands. |
| `0x11` .. `0x13`, `0x30` .. `0x3B` | bytes | Working state (an object that has been hit, an animation's clock). The game clears these when the course loads. |

| Second byte | Marks |
| --- | --- |
| `0x02` .. `0x37` | An object standing on the course (a tree, a sign, a spectator ...). The number is one of the stage's object slots; `LOCALDAT` says which object kind, and so which [object file](Object-files), fills each slot. |
| `0x38` | A replay camera of the second set. |
| `0x3B` | A lamp that lights the faces around it. Byte `0x11` is its strength. |
| `0x3C` | A replay camera of the first set. |
| `0x3D` | A checkpoint of the route driven backwards. |
| `0x3E` | A checkpoint of the route driven forwards. |
| `0x3F` | A sound spot (a stream, a crowd, birds ...). Byte `0x08` is which sound; the game plays 1 .. `0x15` and ignores other values. |

The other bytes of an object, camera, lamp or sound marker are not described here: their
meaning is not known beyond what the tables say.

#### Checkpoints: the route

The route of a stage is a chain of checkpoint markers, a few metres apart, from the start to
past the finish. There are two chains, one for each direction the stage can be driven
(`0x3E` forwards, `0x3D` backwards). Stage times, the distance display, the co-driver, the
computer drivers and "wrong way" all work from the chain.

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x04` | u32 | The checkpoint's point. |
| `0x0A`, `0x0B` | u8 | Two figures for the computer drivers about the bend here. |
| `0x0C` | u32 | The face number of the next checkpoint in the chain. |
| `0x26` | u8 | 0: an ordinary checkpoint. 1: the first of the chain. 2: a timed place (the start line, a split, the finish); the game counts them in order along the chain. |
| `0x19` .. `0x1B`, `0x20`, `0x28` .. `0x2B` | 8 × u8 | Eight pace-note slots (in the order `0x28`, `0x29`, `0x2A`, `0x2B`, `0x19`, `0x1A`, `0x1B`, `0x20`). |
| `0x2C`, `0x2E` | u16 | The checkpoint's place on the stage's map, x then y; both 0 if it has none. In the game's files the values are four to a map pixel. Only some checkpoints have one; the game works out the places in between. |

To follow the route: collect the faces with bytes `FF 3E`, find the one whose byte `0x26` is
1, and follow `0x0C` from face to face. The chain ends when the link names a face that is
not a checkpoint of this direction or has been visited already.

The co-driver's calls are not stored as text. Each checkpoint has eight slots, so a place on
the route is "checkpoint × 8 + slot". A slot holds 0 for nothing or a code: a value from 1
to `0x63` other than `0x62` begins a note of that kind (`0x63` is a corner, whose sharpness
and direction the game measures from the route itself); `0x62` or `0x64` ends the note. When
a stage loads, the game walks the route, groups notes that follow closely, and builds the
spoken calls from them.

## The stage's settings: LOCALDAT

`LOCALDAT` is not packed. It is a block of `0x206C` (8300) bytes of u32 values that the
game reads straight over its settings for the stage; a shorter file sets only the fields it
reaches and leaves the rest at their defaults (a few of the game's files stop at `0x2038`).
If `srclvls\LEVELnn\LOCALDAT` exists it is read over the first.

Every field is a u32 or a table of u32 unless noted.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x0000` | `0x100` | Not used by name. |
| `0x0100` | `0x400` | The stage's 256 textures: for each, a texture number (see below). 0 for none. |
| `0x0500` | `0x100` | The object kind in each of the 64 object slots, by slot number (slot 0 is not used). The kind is the number in an object file's name ([Object files](Object-files)). A marker face names a slot. The game replaces the last eight (`0x05E0` on, slots `0x38` .. `0x3F`) with fixed kinds of its own. |
| `0x0600` | 3 × 4 | The dark colour of the stage's light: red, green, blue. |
| `0x060C` | 3 × 4 | The lit colour. |
| `0x0618` | 3 × 4 | The fog's colour. |
| `0x0630`, `0x0634`, `0x06A8` | 4 each | Light level and strength for cars and objects. |
| `0x0654`, `0x0658`, `0x065C`, `0x066C`, `0x06A4`, `0x06AC` | 4 each | Fog distances. |
| `0x0664` | 4 | The sky's height. |
| `0x0674` | 4 | How dark the cars' shadows are. |
| `0x0680`, `0x0684`, `0x0688` | 4 each | The sky's turn, the sun's direction and the sun's height. |
| `0x06D0` | `0x8C8` | The animation frames, one byte an entry: entry *texture* × 8 + *frame* is the texture (0 .. 255) shown in place of an animated texture in frame 0 .. 7. |
| `0x0F98` | `0x900` | One byte for each texture (the first 256 are used): 0 for a texture that does not animate, else its speed. |
| `0x1898`, `0x18A4`, `0x18B0` | 3 × 4 each | Three small light tables (the last for the cars). |
| `0x18BC`, `0x18C0` | 4, 8 | The sun glare's colour. |
| `0x18C8` | 3 × `0x100` | Three lists of picture numbers that the stage loads besides its textures. |
| `0x1BC8` | 4 | How many cells away the course is drawn. |
| `0x1BCC` | `0x400` | A tint for each of the 256 textures. |
| `0x1FCC` .. `0x204B` | 4 each | The draw distance: the current values, then six tables of five entries (one per draw-distance setting) for the reach and the fog distances. |
| `0x204C` | 3 × 4 | The colour of this stage's mud: red, green, blue. |
| `0x2058` | 4 | The scale of the shine on cars. |
| `0x205C`, `0x2064` | 4, 8 | Figures for the roughness of the surface under the wheels. |

Fields not listed are not read, or their meaning is not known.

A **texture number** is a picture number times 16 plus a place 0 .. 15. The picture number
names the file `TXnnnn.PCX` ([Textures (PCX)](Textures-PCX)), and the place picks one of the
sub-textures the picture is cut into, whose corners are in the picture's texture table
([Object and texture tables](Object-and-texture-tables)).

For a few stages the game changes one texture after loading the file (a built-in list); a
tool that reads `LOCALDAT` sees the values before that change.

## A made-up example

The smallest course: one square of ground with one checkpoint on it.

Header: `73 6F 75 74` (`sout`), then cell count 2, face count 2, point count 4, and zeros up
to `0x100`.

Points (at `0x1C0`), a square 16 units a side at height 0:

| Point | x | y | z |
| --- | --- | --- | --- |
| 0 | `0x00000000` | 0 | `0x00000000` |
| 1 | `0x00100000` | 0 | `0x00000000` |
| 2 | `0x00000000` | 0 | `0x00100000` |
| 3 | `0x00100000` | 0 | `0x00100000` |

All four lie in cell (0, 0), key 0.

Cells (at `0x100`):

| Record | Key | Faces | Points |
| --- | --- | --- | --- |
| 0 | `0x00000000` | 0 .. 1 | 0 .. 3 |
| 1 | `0xFFFFFFFF` | none (`FFFFFFFF`, `FFFFFFFF`) | none |

Faces (at `0x140`):

| Face | Bytes that are not zero |
| --- | --- |
| 0 | `0x00`: `00` (drawn); `0x01`: `05` (the stage's texture 5); `0x04`: points 0, 1, 2, 3; `0x14`: `80 80 80 80` (corner lights) |
| 1 | `0x00`: `FF`; `0x01`: `3E` (checkpoint, forwards); `0x04`: point 0; `0x0C`: `FFFFFFFF` (no next); `0x26`: `01` (first of the chain) |

The unpacked file is `0x100` + 2 × `0x20` + 2 × `0x40` + 4 × `0x0C` = `0x1F0` bytes. A real
stage has a chain of hundreds of checkpoints, and a `LOCALDAT` that gives texture 5 a
picture.

Code: `src/race/course_data.rs` and `rc2k_core/src/race/loading.rs` (reading the course),
`rc2k_core/src/race/faces.rs` (the face's fields), `rc2k_core/src/physics/course_records.rs`
(cells), `src/race/best_line.rs` and `rc2k_core/src/race/checkpoints.rs` (the route),
`src/race/pace_notes.rs` (pace notes), `src/race/stage_prefs.rs` and
`src/race/prefs_store.rs` (`LOCALDAT`), `src/race/find_present_courses.rs`.
