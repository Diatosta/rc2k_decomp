# Screenshots

*Category: [Data formats](Data-formats)*

The game can save what is on the screen as a picture. All of its screenshots are PCX files with
24-bit colour, written by one routine. This page says where they go and exactly what the files
hold. Reading PCX files in general (textures, fonts) is on [Textures (PCX)](Textures-PCX).

Numbers of more than one byte are little-endian.

## Where the files are

| File | Made by |
| --- | --- |
| `NNN.pcx` in the game's folder | The screenshot key (numeric keypad 0), in the menus and in a race. Only when `screensnapshot=1` is set in `var/ral.cfg` (see [Settings files](Settings-files)). |
| `savedata/snapshot/NNN.pcx` | The replay viewer's snapshot button. |
| `rpl/NNNNNN.pcx` | The replay viewer's picture sequence: one file for each picture drawn while it is switched on (a developers' feature; its keys work only with `wrreplayenable=1` in `var/ral.cfg`). |

`NNN` is a three-digit number. The screenshot key starts at `001` each time the program is
started, so a new session writes over the last one's files. The viewer's snapshots carry on
from the highest number already in the folder. Both go back to `000` after `999`.

The picture has the size of the screen mode the game is drawing in at that moment (640 × 480,
for example).

## Layout

| Part | Offset | Size |
| --- | --- | --- |
| Header | `0x00` | `0x80` (128) |
| Picture data | `0x80` | to the end of the file |

There is no palette at the end of the file: the three colours of every pixel are stored.

### The header

| Offset | Size | Value | Meaning |
| --- | --- | --- | --- |
| `0x00` | 1 | `0x0A` | PCX marker. |
| `0x01` | 1 | `5` | Version. |
| `0x02` | 1 | `1` | Run-length coded. |
| `0x03` | 1 | `8` | Bits per pixel in each plane. |
| `0x04` | 2 + 2 | 0, 0 | Left and top. |
| `0x08` | 2 | width − 1 | Right. |
| `0x0A` | 2 | height − 1 | Bottom. |
| `0x0C` | 2 + 2 | `0x012C`, `0x012C` | 300 dots per inch, across and down. |
| `0x10` | `0x30` | | The 16-colour palette, not used by the picture: sixteen greys, each three equal bytes, 0 .. 15. |
| `0x40` | 1 | 0 | |
| `0x41` | 1 | `3` | Number of planes. |
| `0x42` | 2 | width | Bytes in one plane of one row. |
| `0x44` | `0x3C` | zeros | |

### The picture data

The rows follow from the top of the picture down. A row is `3 × width` bytes before coding: all
its red bytes, then all its green bytes, then all its blue bytes.

Each row is run-length coded on its own, in the usual PCX way:

| Byte `c` | What follows | Result |
| --- | --- | --- |
| `0xC1` .. `0xFF` | one byte | that byte, `c` − `0xC0` times (1 .. 63) |
| `0x00` .. `0xBF` | nothing | the byte `c` itself, once |

A single byte of `0xC0` or more is written as a run of one (`0xC1`, then the byte). Runs do not
stop at the end of a plane: a run may begin in a row's red bytes and end in its green ones. A
run never passes the end of a row.

To read a file: read `width` and `height` from the header, then for each row decode until you
have `3 × width` bytes.

## A made-up example

A picture 4 pixels wide and 1 high, with the pixels (red, green, blue) =
(10, 20, 30), (10, 20, 30), (10, 20, 200), (10, 21, 200).

The row before coding is the red plane `0A 0A 0A 0A`, the green plane `14 14 14 15`, the blue
plane `1E 1E C8 C8`. Coded:

```
C4 0A  C3 14  15  C2 1E  C2 C8
```

- `C4 0A`: four bytes of `0A`;
- `C3 14`: three bytes of `14`;
- `15`: one byte, below `0xC0`, as itself;
- `C2 1E`: two bytes of `1E`;
- `C2 C8`: two bytes of `C8` (a single `C8` would be written `C1 C8`).

The header has 3 at `0x08` (right), 0 at `0x0A` (bottom), 3 at `0x41` and 4 at `0x42`.

Code: `src/gfx/take_screenshot.rs`, `src/gfx/write_screen_pcx.rs`, `src/gfx/shots.rs`,
`rc2k_core/src/gfx/pcx.rs` (the header and the coding, with tests),
`src/race/viewer_snapshot.rs`, `src/race/find_snapshot_number.rs`,
`src/race/replay_screenshot.rs`.
