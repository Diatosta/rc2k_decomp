# Saved games

*Category: [Data formats](Data-formats)*

A saved game holds a championship in progress: where the event stands, the four players with
their cars, and all the computer drivers with their times and points. It is written from the
"save game" screen between stages and read back by "load game".

All numbers are little-endian. "u32" is an unsigned 32-bit number, "i32" a signed one.

## Where the files are

| File | What it is |
| --- | --- |
| `savedata/savegame/svgameNN` | A saved game, `NN` = slot `00` .. `99`. |

The files have no extension. Each is exactly `0x8000` (32768) bytes. At start-up the game reads
all hundred slots to list their names.

## Overall layout

| Part | Offset | Size | Notes |
| --- | --- | --- | --- |
| Head | `0x0000` | `0x30` | Marker, version, time, name. |
| Event | `0x0030` | `0x1C` | Seven u32s: where the event stands. |
| Players | `0x004C` | 4 × `0x28C` | The four player records. |
| Drivers | `0x0A7C` | 180 × `0x94` | The computer drivers' records. |
| Rest | `0x728C` | `0xD74` | Not used. Whatever the game's buffer held; usually zeros. |

## The head

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `R5SG` |
| `0x04` | 4 | u32 | 0 |
| `0x08` | 4 | u32 | Version, `0x00051122`. The game ignores a file with any other value. |
| `0x0C` | 4 | u32 | Date and time the file was written, in the form described under [Car setups](Car-setups). |
| `0x10` | `0x20` | text | The saved game's name, padded with zero bytes. |

## The event

| Offset | Meaning |
| --- | --- |
| `0x30` | Number of players. |
| `0x34` | Stages done so far. |
| `0x38` | The stage's number within its section. |
| `0x3C` | The section (a rally of the championship). |
| `0x40` | The kind of event. |
| `0x44` | The event's mode. |
| `0x48` | The class mode. |

When a game is loaded, the game looks up the entry of the championship's table that these
numbers point at to decide where to carry on: at the next stage, at the first stage of the next
rally, or (after the last stage) nowhere. If the entry is of no kind it knows, it stops with the
error "Corrupt SaveGame".

## The player record

`0x28C` (652) bytes for each of the four players, whether or not they take part. The same
record is used in the [options files](Options-files), which keep each player's own settings
between sessions. Every field is a u32 unless the table says otherwise.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x000` | 4 | Not used. |
| `0x004` | 4 | Nationality (a number into the game's list of countries). |
| `0x008` | 4 | Car (model number, 0 .. 33). |
| `0x00C` | 4 | View. |
| `0x010` | 4 | Second view. |
| `0x014` | 4 | In player 0's record: the section the event started at. |
| `0x018` | 4 | A reference to the player's name that only means something while the game runs. Ignored on reading; the name is at `0x124`. |
| `0x01C` | `0xC` | Twelve key codes: the player's keyboard controls. |
| `0x028` | 4 | Race number (the number on the car), 1 .. 90. |
| `0x02C` | 4 | Controller: 0 the keyboard, n the n-th input device. |
| `0x030` | 4 | Gearbox, 0 .. 2. |
| `0x034` | 4 | Controller sensitivity level, 0 .. 3. |
| `0x038` | 4 | Four bytes: the axes learned for the pedals. |
| `0x03C` | 4 | Controller mode. |
| `0x040` | 4 | The stick's end zone, in percent. |
| `0x044` | 4 | The stick's centre dead zone, in percent. |
| `0x048` | 4 | Force feedback strength, 0 .. 100. |
| `0x04C` | 4 | Not used. |
| `0x050` | `0x2C` | The controller's bindings, eleven u32s. |
| `0x07C` | `0xC` | Not used. |
| `0x088` | 4 | The last stage's time, in hundredths of a second. |
| `0x08C` | 4 | The event's total time. |
| `0x090` | 4 | The car's class, 0 .. 3. |
| `0x094` | 4 | The stage's figure (used by the time trial). |
| `0x098` | `0x1C` | Stage points: one u32 for each of six rallies, then their total. |
| `0x0B4` | `0x1C` | Overall points, the same way. |
| `0x0D0` | 4 | How the player stands in the event: 1 taking part, 6 out. |
| `0x0D4` | `0x38` | Damage to the car's fourteen parts, a u32 each. |
| `0x10C` | 4 | Set-up: tyres. |
| `0x110` | 4 | Set-up: brake balance. |
| `0x114` | 4 | Set-up: ride height. |
| `0x118` | 4 | Set-up: suspension stiffness. |
| `0x11C` | 4 | Set-up: steering sensitivity. |
| `0x120` | 4 | Set-up: gear ratios. |
| `0x124` | `0x20` | The player's name (text, padded with zero bytes). |
| `0x144` | `0x10` | Four u32s: the player's places in the standings. |
| `0x154` | 4 | Co-driver's voice: 0 male, 1 female. |
| `0x158` | `0x24` | Nine display choices, 0 or 1 each: local map, rear mirror, rev counter, car information, message bar, progress bar, driver information, pace notes, metric units. |
| `0x17C` | `0x7C` | Wear of the car's parts, 31 u32s. |
| `0x1F8` | `0x14` | Wear of the tyres, 5 u32s. |
| `0x20C` | `0x40` | The player's best line: sixteen checkpoint times. |
| `0x24C` | `0x20` | Eight f32s, 0 .. 1: marks of hits on the body, by direction. |
| `0x26C` | 8 | Not used. |
| `0x274` | 4 | In player 0's record: a second start section (one of the championship kinds uses it). |
| `0x278` | 4 | The car's state bits. |
| `0x27C` | 4 | A word left by the last race. |
| `0x280` | 4 | Steering sensitivity of the controller, 0 .. 100. |
| `0x284` | 4 | i32: the in-race display's bits; −1 at an event's start. |
| `0x288` | 4 | Money spent in the service area, times 100. |

When a saved game is loaded, each player's keys (`0x01C`), controller settings and bindings
(`0x02C` up to `0x07C`) and units choice (`0x178`) are taken from the player's current settings
and not from the file, unless `usesavegamekeys` is set in
`var/ral.cfg` (see [Settings files](Settings-files)). The advanced page's "controls from saved games" row (on its second part, the "more" button) sets it; a change counts from the next load.

## A computer driver's record

`0x94` (148) bytes, 37 u32s. There are always 180.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x00` | 4 | The driver's number, which picks the name. |
| `0x04` | 4 | 1. |
| `0x08` | 4 | Class, 0 .. 3. |
| `0x0C` | 4 | The last stage's time, in hundredths of a second. |
| `0x10` | 4 | The event's total time. |
| `0x14` | 4 | Nationality. |
| `0x18` | 4 | Car. |
| `0x1C` | 8 | 0. |
| `0x24` | 4 | 1 while the driver is in the event, 4 once dropped out. |
| `0x28` | 4 | 0. |
| `0x2C` | `0x30` | Twelve figures for the car's condition; 100 each at an event's start. |
| `0x5C` | `0x1C` | Overall points: six rallies, then their total. `0xFFFFFFFF` for a rally not driven yet. |
| `0x78` | `0x1C` | Stage points, the same way. |

## How the game reads and writes one

Writing: the head is made with the current time and the name typed in, the event's seven
numbers, the four players and the 180 drivers are copied after it, and the whole `0x8000` bytes
are written.

Reading: up to `0x8000` bytes are read. The file is accepted if it begins with `R5SG` and has
the version at `0x08`; the name goes into the slot list. The rest is only looked at when the
slot is previewed or loaded.

To make a file by hand, fill everything from `0x728C` on with zeros and write 0 at `0x018` of
each player record.

## A made-up example

The start of a saved game named "after wales", written on 3 May 2027 at 14:07, for one player
in the second rally:

| Offset | Bytes | Meaning |
| --- | --- | --- |
| `0x00` | `52 35 53 47` | `R5SG` |
| `0x08` | `22 11 05 00` | version `0x00051122` |
| `0x0C` | `87 1B B5 7E` | 2027, month 5, day 3, 14:07 |
| `0x10` | `61 66 74 65 72 20 77 61 6C 65 73 00 ...` | `after wales` |
| `0x30` | `01 00 00 00` | 1 player |
| `0x34` | `07 00 00 00` | 7 stages done |
| `0x38` | `01 00 00 00` | stage 1 of its section |
| `0x3C` | `01 00 00 00` | section 1 |
| `0x4C` | 652 bytes | player 1's record; its name at `0x4C` + `0x124` = `0x170` |

Code: `src/saves/write_savegame_slot.rs`, `src/saves/gather_savegame.rs`,
`src/saves/read_savegame_slot.rs`, `src/saves/load_savegame.rs`,
`rc2k_core/src/saves/savegame.rs`, `rc2k_core/src/menu/player.rs` (the player record),
`src/menu/drivers_store.rs` (the driver record).
