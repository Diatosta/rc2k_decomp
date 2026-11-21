# Car setups

*Category: [Data formats](Data-formats)*

A car setup is the set of choices made on the service area's set-up screen: tyres, brake
balance, ride height, suspension stiffness, steering sensitivity and gear ratios. The screen can
save a setup under a name and load it again later, for any car.

All numbers are little-endian. "u32" is an unsigned 32-bit number.

## Where the files are

| File | What it is |
| --- | --- |
| `savedata/carsetup/carsetNN` | A saved setup, `NN` = slot `00` .. `99`. |

The files have no extension. A **slot** is one numbered file (see the [Glossary](Glossary)); the
game lists all hundred at start-up and shows the names of the ones that exist.

## Layout

The file is always `0x48` (72) bytes.

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `CRSU` |
| `0x04` | 4 | u32 | 0 |
| `0x08` | 4 | u32 | Version, `0x00050000`. |
| `0x0C` | 4 | u32 | Date and time the file was written (below). |
| `0x10` | `0x20` | text | The setup's name, padded with zero bytes. |
| `0x30` | 4 | u32 | Tyres. |
| `0x34` | 4 | u32 | Brake balance. |
| `0x38` | 4 | u32 | Ride height. |
| `0x3C` | 4 | u32 | Suspension stiffness. |
| `0x40` | 4 | u32 | Steering sensitivity. |
| `0x44` | 4 | u32 | Gear ratios. |

The first `0x30` bytes have the same shape in every file the game keeps: four letters, a zero,
a version, the time, and (in the slot files) a name. See [Saved games](Saved-games) and
[Replay files](Replay-files).

The date and time is one number, to the minute:

| Bits | Field |
| --- | --- |
| 0 .. 5 | minute |
| 6 .. 10 | hour |
| 11 .. 15 | day |
| 16 .. 19 | month |
| 20 and up | year (in full, e.g. 2026) |

The six values are the positions of the set-up screen's items. Tyres and gear ratios are a
choice from a short list, counted from 0. The other four are positions on a slider.

## How the game reads and writes one

Reading: the game reads `0x48` bytes and accepts the file if it begins with `CRSU` and has the
version `0x00050000` at `0x08`. Nothing else is checked. The name goes into the list of slots;
the six values are applied when the player loads the slot.

Writing: the whole file is built in memory and written in one go, with the current date and the
name typed in. Saving into a slot that exists replaces it.

## A made-up example

A setup named "wet gravel", written on 3 May 2027 at 14:07:

| Offset | Bytes | Meaning |
| --- | --- | --- |
| `0x00` | `43 52 53 55` | `CRSU` |
| `0x04` | `00 00 00 00` | 0 |
| `0x08` | `00 00 05 00` | version `0x00050000` |
| `0x0C` | `87 1B B5 7E` | `0x7EB51B87`: 2027, month 5, day 3, 14:07 |
| `0x10` | `77 65 74 20 67 72 61 76 65 6C 00 ...` | `wet gravel`, then zeros to `0x30` |
| `0x30` | `01 00 00 00` | tyres: choice 1 |
| `0x34` | `28 00 00 00` | brake balance `0x28` (40) |
| `0x38` | `50 00 00 00` | ride height `0x50` (80) |
| `0x3C` | `19 00 00 00` | suspension stiffness `0x19` (25) |
| `0x40` | `32 00 00 00` | steering sensitivity `0x32` (50) |
| `0x44` | `00 00 00 00` | gear ratios: choice 0 |

Code: `src/service/read_setup_slot.rs`, `src/service/write_setup_slot.rs`,
`rc2k_core/src/saves/kept.rs` (the file's bytes, with tests).
