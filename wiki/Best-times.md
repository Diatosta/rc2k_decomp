# Best times

*Category: [Data formats](Data-formats)*

`var/hst.dat` holds the game's best lists: the ten best times of every stage in each kind of
event, and the best scores. The game reads it at start-up and writes it when it saves its
options. When the file is missing, the lists start out filled with made-up drivers.

Two more sets of records are kept elsewhere, in the [options files](Options-files): the record
holders of each stage of the championship, and the six rallies' best times.

All numbers are little-endian. "u32" is an unsigned 32-bit number.

## Overall layout

| Part | Offset | Size | Holds |
| --- | --- | --- | --- |
| Head | `0x00000` | `0x10` | Marker, version, time. |
| Best scores | `0x00010` | `0xBB8` (3000) | 5 groups × 10 entries of `0x3C`. |
| Championship stage times | `0x00BC8` | `0xDB60` (56160) | 36 stages × 10 best times of `0x9C`. |
| Arcade level scores | `0x0E728` | `0x15E0` (5600) | 7 lists × 10 entries of `0x50`. |
| Arcade stage times | `0x0FD08` | `0xDB60` | 36 stages × 10 best times. |
| Stage times, second set | `0x1D868` | `0xDB60` | 36 stages × 10 best times (the stage screen's other set of stages). |
| Time trial stage times | `0x2B3C8` | `0xDB60` | 36 stages × 10 best times. |

The file is `0x38F28` (233256) bytes. The four tables of stage times have one block of ten for
each stage the championship's table lists, which is 36 in the game as shipped; the sizes above
follow from that.

Every list of ten is kept in order, best first. A new result is written over the tenth entry
and moved up to its place.

## The head

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `R5HS` |
| `0x04` | 4 | u32 | 0 |
| `0x08` | 4 | u32 | Version, `0x00050004`. |
| `0x0C` | 4 | u32 | Date and time written, in the form described under [Car setups](Car-setups). |

## A best score

`0x3C` (60) bytes. The five groups are the four classes of car and, last, the overall list.

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | u32 | A reference to the name that only meant something while the game ran. Ignore it; write 0. |
| `0x04` | 4 | u32 | The player, 1 .. 4, who made the entry; 0 for a made-up driver. |
| `0x08` | 4 | u32 | Nationality. |
| `0x0C` | 4 | u32 | Car. |
| `0x10` | 4 | u32 | Group (the car's class). |
| `0x14` | 4 | u32 | 1 for an entry made in the event just finished; otherwise 0. |
| `0x18` | `0x20` | text | The name. A name that fills all `0x20` bytes has no zero byte after it. |
| `0x38` | 4 | u32 | The score. Higher is better. |

## A list entry

`0x50` (80) bytes. The arcade level scores are lists of these (lists 0 .. 5 are the six arcade
levels; the seventh belongs to the "class times" page), and every best time begins with one.

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | u32 | 0 in the file. |
| `0x04` | 4 | u32 | The player, 1 .. 4; 0 for a made-up driver. |
| `0x08` | 4 | u32 | Nationality. |
| `0x0C` | 4 | u32 | For a made-up driver, the number of the name among the game's texts; for a player, a reference with no meaning in the file. |
| `0x10` | 4 | u32 | Car. |
| `0x14` | 4 | u32 | What the list is ordered by: the time in hundredths of a second, or the score. |
| `0x18` | 4 | u32 | A rally's total time, in the rallies' lists (which are in the options files); otherwise 0. |
| `0x1C` | 4 | u32 | Group (the car's class). |
| `0x20` | `0x20` | text | The name. |
| `0x40` | 4 | u32 | 1 for the entry just made; otherwise 0. |
| `0x44` | 4 | u32 | A figure shown with some entries. |
| `0x48` | 4 | u32 | Not used. |
| `0x4C` | 4 | u32 | In the record lists, whether the entry takes part. |

## A best time

`0x9C` (156) bytes: a list entry, then:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x50` | 4 | u32 | The cheats that were on when the time was driven, one bit each. A made-up driver's entry has `0xB` here. |
| `0x54` | 8 | bytes | The time, sealed (below). Zeros in a made-up driver's entry. |
| `0x5C` | `0x40` | 16 × u32 | The time at each checkpoint, in hundredths of a second; `0xFFFFFFFF` for checkpoints the stage does not have. |

### The sealed time

The 8 bytes at `0x54` repeat the time in a form that is awkward to edit by hand. Before
enciphering they are:

| Byte | Meaning |
| --- | --- |
| 0 .. 3 | u32: the time, at most `0x22550FF`. With certain cheats on it is stored negated (as 0 − time); with others, or with the physics switches on (`fizpatch` in `var/ral.cfg`), negated and with bit 30 cleared. |
| 4 | The car. |
| 5 | The kind of run. |
| 6 | Which of the table's stages the entry belongs to, counted from 0. |
| 7 | A check byte: start with `0x5A`; for each of bytes 0 .. 6 add the byte with all its bits flipped; keep the low 8 bits. |

These 8 bytes are enciphered as one block with the cipher described under
[Replay files](Replay-files#the-sealed-part), without chaining, with the key `darkstar`. (The
`key` setting in `var/ral.cfg` replaces the first four bytes of the key; a file made with it
can only be checked with the same setting.)

To check an entry: decipher the 8 bytes, compute the check byte, and compare the time with the
u32 at `0x14`.

## How the game reads and writes it

Reading: the file is taken if it begins with `R5HS` and has the version at `0x08`; the six
tables are then read one after another in the sizes above. Without the file, every list is
filled with made-up drivers and times.

Writing: the head with the current time, then the six tables, in one go.

## A made-up example

A player's entry at the top of a stage's time trial list: "Ann", player 1, nationality 3, car
12, a time of 1:35.20 (9520 hundredths, `0x2530`), four checkpoints.

| Offset | Bytes | Meaning |
| --- | --- | --- |
| `0x00` | `00 00 00 00` | 0 |
| `0x04` | `01 00 00 00` | player 1 |
| `0x08` | `03 00 00 00` | nationality 3 |
| `0x0C` | `00 00 00 00` | no meaning in the file |
| `0x10` | `0C 00 00 00` | car 12 |
| `0x14` | `30 25 00 00` | 9520 hundredths |
| `0x1C` | `01 00 00 00` | group 1 |
| `0x20` | `41 6E 6E 00 ...` | `Ann` |
| `0x40` | `01 00 00 00` | just made |
| `0x50` | `00 00 00 00` | no cheats |
| `0x54` | 8 bytes | the sealed time |
| `0x5C` | `60 09 00 00` `C0 12 00 00` `20 1C 00 00` `E0 24 00 00` | checkpoints at 2400, 4800, 7200 and 9440 hundredths |
| `0x6C` | `FF FF FF FF` × 12 | no more checkpoints |

The 8 bytes sealed, before enciphering, for car 12, run kind 3 and stage 22 of the table:
`30 25 00 00 0C 03 16 D9`. The check byte is `0x5A` + `0xCF` + `0xDA` + `0xFF` + `0xFF` +
`0xF3` + `0xFC` + `0xE9` = `0x6D9`, of which the low 8 bits are `0xD9`.

Code: `src/saves/load_best_times.rs`, `src/saves/save_best_times.rs`,
`src/saves/best_times.rs`, `rc2k_core/src/saves/` (`tables.rs`: the sizes; `scores.rs`,
`entries.rs`: the entries; `defaults.rs`: the made-up lists), `src/records/seal_time.rs` and
`rc2k_core/src/files/records.rs` (the sealed time).
