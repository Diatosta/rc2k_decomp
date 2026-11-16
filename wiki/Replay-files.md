# Replay files

*Category: [Data formats](Data-formats)*

A replay is a recording of one race: which stage and cars, and what every car did. The game
writes one while you drive and plays it back in the replay viewer. This page describes the file
byte by byte, so that a tool can read one, cut one, or list what a folder of replays holds.

All numbers in the file are little-endian. Offsets and sizes are written in hexadecimal
(`0x430`) with the decimal value where it helps. "u32" is an unsigned 32-bit number, "i32" a
signed one, "f32" a 32-bit float.

## Where the files are

Paths are relative to the game's folder (the one that holds `files` and `var`).

| File | What it is |
| --- | --- |
| `var/tmprpl` | The last race's replay. Written during every race, overwritten by the next. |
| `savedata/replay/replayNN` | A saved replay, `NN` = slot `00` .. `99`. Made from the last race's replay or from the viewer. |
| `files/replay/replayNN` | Replays shipped with the game (the viewer knows them as slots 100 and up). The folder may be empty. |

All three have the same format. The files have no extension. The game names them with
backslashes and looks names up without regard to case, as Windows does; on Linux and macOS the
program does that matching itself.

## How a replay works

The game's physics runs in fixed steps, 100 a second, and gives the same result every time for
the same inputs. So a replay does not store where the car is at every moment. It stores, for
every second of the race:

- the complete state of the cars at the start of that second, and
- the players' inputs (steering, pedals, buttons) for each of that second's 100 steps.

To play a second, the viewer loads the state and then runs the real physics with the recorded
inputs. Because every second starts from a stored state, the viewer can jump to any second, and
seconds can be cut out or reordered.

One second of recording is one **block**, and one hundredth of a second of simulation is one
**step** (see the [Glossary](Glossary)). In the source the 16 bytes of inputs recorded for a
step are called a "frame".

## Overall layout

| Part | Offset | Size | Notes |
| --- | --- | --- | --- |
| Header | `0` | `0x430` (1072) | Fixed size. |
| Blocks | `0x430` | varies | One after another, each stored plain (`0x2810` bytes) or packed (smaller). |
| Index | end − `0x3850` | `0x3850` (14416) | Fixed size, always the last bytes of the file. Says where each block is. |

To read a replay: read the header, read the last `0x3850` bytes as the index, then use the index
to find each block.

## The header

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x000` | 4 | text | `REPL` |
| `0x004` | 4 | u32 | 0 |
| `0x008` | 4 | u32 | Version, `0x00050400`. The game refuses any other value. |
| `0x00C` | 4 | u32 | Date and time the file was made (see below). |
| `0x010` | `0x20` | text | The replay's name, padded with zero bytes. `tmpreplay` in `var/tmprpl`; the name typed in when a replay is saved. |
| `0x030` | `0x40` | 16 × u32 | The race's settings (next table). |
| `0x070` | `0x100` | bytes | The sealed part (see "The sealed part"). |
| `0x170` | 4 | u32 | 6 when the physics switches that follow are present. |
| `0x174` | `0x34` | 13 × u32 | The physics switches the race was recorded with, in the order of their keys in `var/ral.cfg`: `fizpatch`, `patchmaxdrawdistance`, `patchstickygrass`, `patchgravity`, `patchdamages`, `patchbrakegrip`, `patchaccelgrip`, `patchslidegrip`, `patchgripgrass`, `patchgriptarmac`, `patchgripsnow`, `patchgripmud`, `patchgripgravel` (see [Settings files](Settings-files) and "Compatibility"). |
| `0x1A8` | 1 | u8 | The first car's race number (the number on its door). |
| `0x1A9` | `0x287` | bytes | Not used; zero. |

The date and time is one number, to the minute:

| Bits | Field |
| --- | --- |
| 0 .. 5 | minute |
| 6 .. 10 | hour |
| 11 .. 15 | day |
| 16 .. 19 | month |
| 20 and up | year (in full, e.g. 2026) |

The race's settings, each a u32:

| Offset | Meaning |
| --- | --- |
| `0x030` | Stage number. |
| `0x034` | Game mode (0 .. 3). |
| `0x038` | Number of players (1 or 2). |
| `0x03C` | Number of cars in the race. |
| `0x040`, `0x044`, `0x048`, `0x04C` | Car model of cars 0 .. 3 (0 .. 33). Only as many as there are cars mean anything. |
| `0x050` | 1 for a one-player race, 2 for two players. |
| `0x054`, `0x058` | The co-driver's voice for car 0 and car 1 (0 or 1). |
| `0x05C` | Arcade mode: the start place. |
| `0x060` | Arcade mode: the number of arcade cars. |
| `0x064` | Arcade mode: the random seed. |
| `0x068`, `0x06C` | Game-over flag of player 1 and player 2. |

When the viewer opens a replay it copies these settings into the race's own variables and loads
the stage from them, exactly as a race would.

### The sealed part

The `0x100` bytes at `0x070` are enciphered. They hold a checksum, the player's name, the date
and the car's set-up at the time of the race. The game writes them and never deciphers them
again: nothing in playback depends on them, and a reader that only wants the race can skip
them. The checksum doubles as the key that scrambles the stage time in the index, which ties a
time to the replay it was driven in.

The plain bytes, before enciphering (offsets from the start of the sealed part):

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x00` | 4 | u32: the checksum of the `0xFC` bytes that follow (below). |
| `0x04` | 1 | `6`. |
| `0x05` | 4 | u32: the date and time of the race, in the header's form. A saved replay keeps the race's date here while the header at `0x00C` gets the date of saving. |
| `0x09` | 4 | u32: the `seasoncode` setting of `var/ral.cfg` (see [Settings files](Settings-files)); 0 unless set. |
| `0x0D` | `0x20` | The player's name, padded with zero bytes. |
| `0x2D` | 1 | A number from the language file in use. |
| `0x2E` | 4 | u32: the cheats in use, one bit each; 0 for none. |
| `0x32` | 1 | The player's car model. |
| `0x33` | 1 | The stage number. |
| `0x34` | 1 | The direction the stage is driven in. |
| `0x35` | 1 | The kind of run (which event the race is part of). |
| `0x36` | 1 | The car's race number. |
| `0x37` | 1 | The view the player started with. |
| `0x38` | 8 | Not written; whatever was there before. |
| `0x40` | 1 | Gearbox. |
| `0x41` | 1 | Gear ratios. |
| `0x42` | 1 | Tyres. |
| `0x43` | 1 | Brake balance. |
| `0x44` | 1 | Steering sensitivity. |
| `0x45` | 1 | Ride height. |
| `0x46` | 1 | Suspension stiffness. |
| `0x47` | 1 | The first of the thirteen physics switches (the header's u32 at `0x174`), as one byte. |
| `0x48` | `0x18` | The other twelve physics switches in the header's order, cut down to: two of 1 byte, one of 2 bytes, one of 4 bytes, then eight of 2 bytes. |
| `0x60` | `0xA0` | Not written; zero in practice. |

The set-up bytes are the values of the service area's set-up screen (see
[Car setups](Car-setups)).

**The checksum.** Read the plain bytes from `0x04` to the end as 63 u32s. Start with 0; for each
one in turn, add it (wrapping at 32 bits), then rotate the sum left by 5 bits. The result goes
at `0x00`.

**The cipher** is DES, used with its own way of numbering bits, on 8 bytes at a time, each 8
bytes mixed with the 8 before (cipher block chaining). The key is the 8 ASCII bytes
`halfmoon`. To encipher the `0x100` bytes: take them 8 at a time, in order; xor each 8 with the
enciphered 8 bytes before it (the first with zeros); encipher; the result replaces them. To
decipher: decipher each 8 bytes, then xor with the *enciphered* 8 bytes before it.

To work on 8 bytes, read them as two u32s, `a` then `b`, and form the 64-bit number
`N = a × 2^32 + b`. Bit 0 is the lowest bit of `N`. The cipher is described with *pick tables*:
to apply one to a number, go through its entries in order; each entry is a bit number, and that
bit of the number becomes the next bit of the result, the first entry's bit ending up highest.

The key's sixteen subkeys:

1. Form `N` from the key's 8 bytes. `C` = `N` through PC1-C, `D` = `N` through PC1-D (28 bits
   each).
2. For each of the sixteen rounds, rotate `C` and `D` left within their 28 bits, by 1 place in
   rounds 1, 2, 9 and 16 and by 2 places in the others. The round's subkey is the number
   `C × 2^32 + D` through PC2 (48 bits).

Enciphering 8 bytes:

1. Form `N`, and put it through IP. The upper 32 bits of the result are `L`, the lower `R`.
2. For each subkey in turn: put `R` through E (48 bits) and xor with the subkey. Cut the 48
   bits into eight groups of 6, from the top. The first group goes through S-box 1, the second
   through S-box 2, and so on; each gives 4 bits. Join the eight results, first on top, into
   32 bits, and put them through P. Xor that with `L`. The new `L` is the old `R`; the new `R`
   is the xor's result.
3. Put `R × 2^32 + L` (the halves the other way round) through FP. The upper 32 bits are the
   new `a`, the lower the new `b`.

Deciphering is the same with the subkeys in the opposite order.

The S-boxes are the eight standard S-boxes of DES. Within a group of 6 bits, the top and bottom
bits together choose the row (0 .. 3) and the middle four bits the column (0 .. 15), as in the
standard.

The pick tables:

| Table | Entries |
| --- | --- |
| PC1-C | 6 14 22 30 38 46 54 62 5 13 21 29 37 45 53 61 4 12 20 28 36 44 52 60 3 11 19 27 |
| PC1-D | 0 8 16 24 32 40 48 56 1 9 17 25 33 41 49 57 2 10 18 26 34 42 50 58 35 43 51 59 |
| PC2 | 46 43 49 36 59 55 57 32 45 54 39 50 37 41 48 56 34 52 44 53 33 40 47 58 15 4 25 19 9 1 26 16 5 11 23 8 12 7 17 0 22 3 10 14 6 20 27 24 |
| IP | 6 14 22 30 38 46 54 62 4 12 20 28 36 44 52 60 2 10 18 26 34 42 50 58 0 8 16 24 32 40 48 56 7 15 23 31 39 47 55 63 5 13 21 29 37 45 53 61 3 11 19 27 35 43 51 59 1 9 17 25 33 41 49 57 |
| E | 0 31 30 29 28 27 28 27 26 25 24 23 24 23 22 21 20 19 20 19 18 17 16 15 16 15 14 13 12 11 12 11 10 9 8 7 8 7 6 5 4 3 4 3 2 1 0 31 |
| P | 16 25 12 11 3 20 4 15 31 17 9 6 27 14 1 22 30 24 8 18 0 5 29 23 13 19 2 26 10 21 28 7 |
| FP | 24 56 16 48 8 40 0 32 25 57 17 49 9 41 1 33 26 58 18 50 10 42 2 34 27 59 19 51 11 43 3 35 28 60 20 52 12 44 4 36 29 61 21 53 13 45 5 37 30 62 22 54 14 46 6 38 31 63 23 55 15 47 7 39 |

In PC2, entries of 32 and more pick from `C` (bit number less 32) and the others from `D`.

### The scrambled stage time

The u32 at `0x04` of the index holds the first car's stage time, in steps, at the moment the
recording was closed. A time above `0x22550FF` is stored as `0x22550FF`. If the race was left
before the car finished, the value stored is `0xFFFFFFFF` in place of a time.

To scramble: xor the value with the sealed part's checksum, then rotate the result right by
the number in the checksum's lowest 5 bits. To read it back: rotate left by that number, then
xor with the checksum. The checksum is the first u32 of the deciphered sealed part.

A tool that changes anything in the sealed part must work out the checksum again, encipher the
part again, and scramble the time again with the new checksum.

## The index

The last `0x3850` bytes of the file.

| Offset in index | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `RCIB` |
| `0x04` | 4 | u32 | The first car's stage time, scrambled with the sealed part's checksum (see "The scrambled stage time"). |
| `0x08` | 4 | u32 | Number of blocks, 1 .. 1800. |
| `0x0C` | 4 | u32 | Number of steps recorded. In a recording it is 100 × the number of blocks: the last block is filled up to 100 steps when the file is closed. |
| `0x10` | `0x3840` | 1800 × (u32, i32) | For each block: its offset from the start of the file, and its size. |

A block's size is positive when the block is stored plain (then it is `0x2810`) and **negative**
when it is stored packed: the size is then minus the number of bytes it takes in the file.
Entries past the block count are zero.

1800 blocks is the most a replay can hold: 30 minutes. A recording that reaches it is closed
there.

The blocks are played in the index's order. In a file the game has written they also lie in
that order, the first at `0x430`, each starting where the one before ends, the last ending
where the index begins.

## Packed blocks

A packed block is a run-length coding of the block's `0x2810` bytes. Read one count byte at a
time:

| Count byte `c` | What follows | Result |
| --- | --- | --- |
| `0x01` .. `0x7F` | one byte | that byte, `c` + 1 times |
| `0x80` .. `0xFF` | 256 − `c` bytes | those bytes as they are |
| `0x00` | nothing | the end |

After the end byte the block is padded with zero bytes to a multiple of 4. The size in the
index includes the end byte and the padding.

The game packs every block and keeps the packed form when it is smaller than the plain one,
which in practice is always (a block usually packs to between a tenth and a third of its plain
size).

## A block

A block is `0x2810` (10256) bytes once unpacked.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x0000` | 4 × `0x684` | The state of cars 0 .. 3 at the start of this second, `0x684` (1668) bytes each. |
| `0x1A10` | 8 × `0x78` | Eight loose parts (pieces that have come off a car), `0x78` (120) bytes each. |
| `0x1DD0` | `0x400` | The computer driver's state. |
| `0x21D0` | 100 × `0x10` | The inputs: one entry of 16 bytes for each of the 100 steps. |

### A car's state

The `0x684` bytes are the first part of the game's record of a car: everything the physics
needs to carry on from this moment. There are about 170 fields; the full list, with the offset
of each, is in `src/drive/replay_state.rs` (`car_fields`), and their meanings in
`src/physics/car_store.rs`. The ones a tool is most likely to want:

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x000` | 8 bytes | The block's camera bytes (car 0's record only; see below). |
| `0x008` | u32 | The car's step clock: steps since the stage was loaded. In car 0 it is 100 × the block's number in a recorded race. |
| `0x014` | u32 | Steps since the run began. |
| `0x024` | u32 | Car model. |
| `0x02C` | i32 | Tyres, 0 .. 5. |
| `0x088` | u32 | Stage time so far, in steps (hundredths of a second), counted from the start signal. |
| `0x098` | u32 | The checkpoint the car is at. |
| `0x0A8` | u32 | Not 0 once the car has finished. |
| `0x0B4` | 16 × f32 | The car's orientation, a 4 × 4 matrix. |
| `0x134` | 3 × i32 | Position x, y, z, in 1/65536ths of a unit. |
| `0x140` | 16 × u32 | Split times. |
| `0x198` | 3 × f32 | Velocity. |
| `0x1C4` | f32 | Speed (the length of the velocity). |
| `0x1D4` | i32 | Gear: −1 reverse, 0 neutral, 1 and up. |
| `0x1D8` | f32 | Engine revs. |

The camera bytes are the first 8 bytes of car 0's record. Bytes 0 and 1 choose the replay
camera for car 0 and car 1 during this second; bytes 2 .. 5 are view switches. A recording
leaves them 0 (byte 5 is 2 or 3 in a two-player race); the viewer's camera editing fills them
in when a replay is saved.

### A loose part

| Offset | Type | Meaning |
| --- | --- | --- |
| `0x00` | u32 | Which part; 0 = this slot is empty. |
| `0x04` | 3 × i32 | Its place. |
| `0x10` | 16 × f32 | Its orientation. |
| `0x50` | 3 × f32 | Its motion. |
| `0x5C` | u32 | A height offset used when drawing. |
| `0x60` | u32 | A word copied from the car it came off. |
| `0x64` | u32 | The car's step clock when it came off. |
| `0x68` | 3 × f32, f32 | Its spin, and the spin's length. |

### The computer driver's state

`0x400` bytes that the computer-driven cars' steering works from (where it is aiming, its
wanted speed, its progress along the stage). The fields and their offsets are listed in
`src/drive/replay_state.rs` (`extra_fields`).

### The inputs

100 entries of 16 bytes, one per step. Each entry holds 8 bytes for car 0, then 8 bytes for
car 1. Only these two cars have recorded inputs: they are the players' cars. Any other cars are
computer-driven, and what their driver needs to carry on is the state described above.

| Offset in a car's 8 bytes | Type | Meaning |
| --- | --- | --- |
| 0 | u32 | Key and button bits. |
| 4 | u32 | The three analogue inputs packed into one number (below). |

| Bits of the packed number | Meaning |
| --- | --- |
| 0 .. 3 | not used, 0 |
| 4 .. 12 | throttle, 0 (released) .. 256 (fully pressed) |
| 13 .. 21 | brake, 0 .. 256 |
| 22 .. 31 | steering, signed, −256 .. 256 |

## How the game writes one

1. At the start of a race the header is written to `var/tmprpl` (if replays are on and the disc
   has about 30 MB free).
2. At the first step of each second the cars' state is copied into the block being built; at
   every step the two players' inputs are added to it.
3. At the last step of each second the block is packed and appended to the file, and its place
   is noted in the index (kept in memory).
4. When the race ends, the index is written after the last block and the file is closed.

Saving a replay copies `var/tmprpl` into a slot: a new header with the name typed in, each block
read, given its camera bytes and written again, then a new index. Opening a saved replay in the
viewer copies it over `var/tmprpl` first, so viewing a saved replay replaces the last race's.

Code: `src/race/replay.rs` (open, write, close), `src/drive/replay_state.rs` (what goes into a
block), `src/race/open_viewed_replay.rs` and `src/race/save_viewed_replay.rs` (the viewer's
side), `rc2k_core/src/drive/replay_frames.rs` and `rc2k_core/src/race/replay.rs` (the format's
sizes and rules, with tests).

### Unfinished recordings

The index is only written when a race ends. If the program is stopped in the middle of a race,
`var/tmprpl` is left as a header followed by blocks, with no index, and the game will not open
it. Its blocks can still be read: they start at `0x430` and each packed block ends at its end
byte, rounded up to a multiple of 4.

## Compatibility

The format is the 1999 game's and this project does not change it. A replay recorded by the
original program plays in this one; one recorded here has the same bytes, so the original
should play it too. There is one version number
(`0x00050400`); there are no older or newer layouts to tell apart.

Playback only gives the recorded race if the physics behaves exactly as it did when recording.
Two things in the file serve that:

- The physics switches at `0x174`. The game has a set of tuning switches (gravity and grip
  factors, read from `var/ral.cfg`). A replay carries the values it was recorded with, and the
  viewer puts them in force while the replay plays and restores the player's own afterwards.
- The stored state at every second. If the physics drifts, the error lasts at most one second
  before the next block corrects it. The project uses this as a test: during playback it
  compares the state its physics reached with the state stored in the next block
  (`src/drive/replay_check.rs`), so every old replay is a check that the physics still matches.

## A made-up example

A short replay named "demo", saved on 3 May 2027 at 14:07: one player in car model 5 on stage
12, 2 seconds long.

Header (the fields that are not zero):

| Offset | Bytes | Meaning |
| --- | --- | --- |
| `0x000` | `52 45 50 4C` | `REPL` |
| `0x008` | `00 04 05 00` | version `0x00050400` |
| `0x00C` | `87 1B B5 7E` | `0x7EB51B87`: year 2027, month 5, day 3, hour 14, minute 7 |
| `0x010` | `64 65 6D 6F 00 ...` | `demo` |
| `0x030` | `0C 00 00 00` | stage 12 |
| `0x038` | `01 00 00 00` | 1 player |
| `0x03C` | `01 00 00 00` | 1 car |
| `0x040` | `05 00 00 00` | car model 5 |
| `0x050` | `01 00 00 00` | one-player race |
| `0x070` | 256 bytes | the sealed part |
| `0x170` | `06 00 00 00` | physics switches follow |

Then two packed blocks, say of `0x5A4` and `0x618` bytes, and the index. The file is
`0x430` + `0x5A4` + `0x618` + `0x3850` = `0x483C` bytes long, and the index begins at `0xFEC`:

| Offset in index | Bytes | Meaning |
| --- | --- | --- |
| `0x00` | `52 43 49 42` | `RCIB` |
| `0x08` | `02 00 00 00` | 2 blocks |
| `0x0C` | `C8 00 00 00` | 200 steps |
| `0x10` | `30 04 00 00` `5C FA FF FF` | block 0 at `0x430`, size −`0x5A4` (packed) |
| `0x18` | `D4 09 00 00` `E8 F9 FF FF` | block 1 at `0x9D4`, size −`0x618` (packed) |
| `0x20` .. | zeros | no more blocks |

And a piece of packed data, to show the coding:

```
7F 00  1F 00  FD 12 34 56  02 AA  00  00
```

- `7F 00`: 128 zero bytes;
- `1F 00`: 32 more zero bytes;
- `FD 12 34 56`: 256 − `0xFD` = 3 bytes as they are: `12 34 56`;
- `02 AA`: `AA` three times;
- `00`: the end, followed here by one byte of padding (11 bytes rounded up to 12).

That is 12 bytes in the file for 166 bytes of block. (A real block always unpacks to exactly
`0x2810` bytes.)
