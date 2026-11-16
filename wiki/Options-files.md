# Options files

*Category: [Data formats](Data-formats)*

The options are everything the game remembers between sessions that is not a saved game or a
best time: screen and sound choices, network settings, the four players with their names,
controls and cars, and the records of the championship's stages.

There are two files with the same contents:

| File | What it is |
| --- | --- |
| `var/rc2k.toml` | This project's options file: text, in [TOML](https://toml.io). Read and written by default. |
| `var/ral.dat` | The 1999 game's options file: binary. Read when the new file does not exist yet, or on request. |

The switches in `var/ral.cfg` are a different thing: see [Settings files](Settings-files).

## Which file is used

At start-up:

1. If `var/rc2k.toml` exists, it is read. `var/ral.dat` is not touched.
2. Otherwise, if `var/ral.dat` exists, it is read and `var/rc2k.toml` is written from it at
   once. This is how an existing installation of the original game is taken over.
3. Otherwise the game starts with its built-in values.

When the game saves its options (on leaving, and when the window is closed), it writes
`var/rc2k.toml`. It leaves `var/ral.dat` alone.

Two command-line switches change this:

| Switch | Effect |
| --- | --- |
| `--load-old-options` | Read `var/ral.dat` at start-up even if `var/rc2k.toml` exists. |
| `--save-old-options` | Also write `var/ral.dat` whenever the options are saved, so that the original game sees them. |

`var/rc2k.toml` is written whole to a temporary name beside it (`rc2k.toml.tmp`) and then put
in place, so a crash while saving cannot leave half a file.

## `var/rc2k.toml`

A TOML file, in UTF-8. The first key is `version`, then one table for each area:

```toml
version = 1

[display]
menu_mode = 3282946
race_mode = 3414144
mipmapping = 1

[sound]
master_volume = 64
...
```

| Table | Holds |
| --- | --- |
| `[display]` | `menu_mode`, `race_mode`, `mipmapping`. |
| `[sound]` | The sound modes, volumes and levels. |
| `[network]` | `machine_name`, `most_players`, `host_address`, `port`, `saved_connection`. |
| `[pages]` | The options pages' switches (`switches`, `audio`) and a few counters. |
| `[cars]` | `first_numbers`, `second_numbers`: the two lists of cars the player can choose from. |
| `[players]` | `records` (four) and `names` (four). |
| `[records]` | The tyre-change record, the championship's and the events' record tables, the rallies' best times. |
| `[kept]` | Values the old file carries that nothing uses; kept so that the old file can be written back exactly. |

Every key has the name given in the third column of the old file's table below; `display.menu_mode`
there is the key `menu_mode` in the table `[display]`.

Three kinds of value appear:

| Kind | Written as | Example |
| --- | --- | --- |
| A number (u32) | an integer | `port = 0` |
| A list of numbers | an array of integers, of a fixed length | `audio = [3, 1, 0, 0, 0, 1]` |
| A run of bytes (a name, a record, a table) | a string of hexadecimal digits, two for each byte, lower case, of a fixed length | `saved_connection = "00000000000000000000000000000000"` |

Texts such as the players' names are runs of bytes too: the name "Ann" in a `0x20`-byte field is
`"416e6e00"` followed by 56 more zeros. The characters are in the Windows-1252 set, as in the
old file.

`players.records`, `players.names`, `records.events` and `records.rallies` are arrays of such
strings (4, 4, 12 and 6 of them). `records.events` is in the events' own order, 0 .. 11, which is
not the order the old file stores them in (see below).

Rules for reading:

- A key that is missing keeps the game's built-in value. An empty file is valid.
- A key the program does not know is ignored, and is still there after the program writes the
  file again. A later version can add keys without older builds losing them.
- A list or byte string of the wrong length, or a file that is not valid TOML, stops the
  program with a message in `rc2k.log`.
- A `version` higher than this program knows (1) stops it too: the file is from a newer build.

## `var/ral.dat`

All numbers are little-endian. The file is always `0xB66A` (46698) bytes:

| Part | Offset | Size |
| --- | --- | --- |
| Head | `0x0000` | `0x10` |
| The options, 69 values one after another | `0x0010` | `0xB656` |
| End marker, the text `####` | `0xB666` | 4 |

The head:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `R5CI` |
| `0x04` | 4 | u32 | 0 |
| `0x08` | 4 | u32 | Version, `0x00050004`. |
| `0x0C` | 4 | u32 | Date and time written, in the form described under [Car setups](Car-setups). |

A file that does not begin with `R5CI` and this version is passed over as if it were not there.
One that does, but has no `####` at `0xB666`, stops the game with the error "ral.dat out of
date".

The values have no gaps or alignment between them, which is why some offsets are odd. The
"Key" column is the value's name in `var/rc2k.toml`. A size of `0x4` is one u32.

| Offset | Size | Key | Meaning |
| --- | --- | --- | --- |
| `0x0010` | `0x4` | `kept.runs` | Times the game was started. |
| `0x0014` | `0x4` | `display.menu_mode` | The menus' screen mode. |
| `0x0018` | `0x4` | `display.race_mode` | The race's screen mode. A mode saved at 16 bits a pixel (both modes) is taken as the same size at 32 bits: the screen always has 32. |
| `0x001C` | `0x4` | `display.mipmapping` | Mipmapping, 0 or 1. |
| `0x0020` | `0x4` | `sound.menu_sound_mode` | The menus' sound mode. |
| `0x0024` | `0x4` | `sound.race_sound_mode` | The race's sound mode. A mode of the original's A3D option (bit `0x2000`) is read as the same mode with 3D sound instead (bit `0x1000`), from either file; with it the A3D switch of `pages.audio` is read as the 3D sound switch. |
| `0x0028` | `0x4` | `sound.master_volume` | Master volume. |
| `0x002C` | `0x4` | `sound.unread_option_f0` | Kept, not used. |
| `0x0030` | `0x4` | `sound.race_music_on` | Music in races, 0 or 1. |
| `0x0034` | `0x4` | `sound.tune_turn` | Which tune plays next. |
| `0x0038` | `0x4` | `sound.music_volume` | Music volume. |
| `0x003C` | `0x4` | `sound.race_music_volume` | Race music volume. |
| `0x0040` | `0x40` | `network.machine_name` | This machine's name in network games (text). |
| `0x0080` | `0x4` | `network.most_players` | Most players in a game hosted here. |
| `0x0084` | `0x40` | `network.host_address` | The host's address last typed in (text). |
| `0x00C4` | `0x4` | `network.port` | Network port. |
| `0x00C8` | `0x10` | `network.saved_connection` | The saved connection, 16 bytes. |
| `0x00D8` | `0x4` | `kept.saved_games_a` | Kept, not used. |
| `0x00DC` | `0x4` | `kept.saved_games_b` | Kept, not used. |
| `0x00E0` | `0x4` | `kept.set_ups` | Kept, not used. |
| `0x00E4` | `0x20` | `records.tyre_change_holder` | Name of the holder of the tyre-change record (text). |
| `0x0104` | `0x4` | `records.tyre_change_time` | The tyre-change record. |
| `0x0108` | `0x10` | `kept.options` | Four u32s, kept, not used. |
| `0x0118` | 4 × `0x28C` | `players.records` | The four player records. |
| `0x0B48` | `0x4` | `sound.master_volume_shown` | Master volume as the options page shows it. |
| `0x0B4C` | `0x4` | `sound.music_level` | Music level. |
| `0x0B50` | `0x4` | `sound.other_volume` | A second music level. |
| `0x0B54` | `0x4` | `sound.menu_sounds_c` | A menu sounds level. |
| `0x0B58` | `0x4` | `sound.menu_sounds_a` | A menu sounds level. |
| `0x0B5C` | `0x4` | `sound.speech_level` | Speech level. |
| `0x0B60` | `0x4` | `sound.sounds_a` | An effects level. |
| `0x0B64` | `0x4` | `sound.sounds_b` | An effects level. |
| `0x0B68` | `0x2C` | `pages.switches` | Eleven u32s, the graphics options page: mipmapping, buffering, two screen-mode switches, lens flare, two unused percentages, reflections, car shadows, car detail, headlights. |
| `0x0B94` | `0x18` | `pages.audio` | Six u32s, the audio options page: quality, 3D sound, EAX, the A3D switch of the original (not used: 0 after a load), Dolby, channels. |
| `0x0BAC` | `0x4` | `pages.services` | Services counted. |
| `0x0BB0` | `0x4` | `pages.event_kept` | A value kept between an event's races. |
| `0x0BB4` | `0x4` | `pages.race_settings` | The first word of the race settings. |
| `0x0BB8` | `0x6` | `pages.arcade_levels_won` | The arcade levels won, a byte each. |
| `0x0BBE` | `0xA0` | `cars.first_numbers` | 40 u32s: the first list of car numbers; `0xFFFFFFFF` for an empty place. |
| `0x0C5E` | `0x50` | `cars.second_numbers` | 20 u32s: the second list of car numbers. |
| `0x0CAE` | `0x2A48` | `records.championship` | The championship's record table: 66 records of `0xA4`. |
| `0x36F6` | 12 × `0x47C` | `records.events` | Twelve event record tables, 7 records of `0xA4` each. In the file they are stored in the order of events 5, 4, 3, 2, 1, 0, 6, 7, 8, 9, 10, 11. |
| `0x6CC6` | 6 × `0xC30` | `records.rallies` | The six rallies' best times: 20 entries of `0x9C` each. |
| `0xB5E6` | 4 × `0x20` | `players.names` | The four players' names (text). |

The player record (`0x28C` bytes) is described under
[Saved games](Saved-games#the-player-record). An entry of the rallies' best times (`0x9C` bytes)
is described under [Best times](Best-times#a-best-time).

### A record of a record table

The championship's table and the events' tables are lists of records of `0xA4` (164) bytes. A
record is one step of an event: a stage, a service area, the end of a rally. Most of a record
is the event's fixed description, the same in every copy of the game; the options file carries
it along because the record also holds the stage's two record holders, which do change. Every
field is a u32 unless the table says otherwise.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x00` | 4 | The kind: 0 a stage, 1 a service area, 3 or 4 the end of a rally, 5 a welcome screen. |
| `0x04` | 4 | Which track of the game's track list. |
| `0x08` | 4 | The track list's id. |
| `0x0C` | 4 | A figure for a service area; 1 on a stage. |
| `0x10` | 4 | The time of day the stage starts at, as hours × 100 + minutes. |
| `0x14` | 4 | The time allowed in a service area. |
| `0x18` | `0x10` | Four u32s: target times for the four classes of car. |
| `0x28` | 4 | Not used. |
| `0x2C` | 4 | The time trial record holder's top speed. |
| `0x30` | 4 | The fastest record holder's top speed. |
| `0x34` | 4 | The stage number the race is loaded with. |
| `0x38` | 4 | The number of the stage's name among the game's texts. |
| `0x3C` | 4 | Which ten entries of the best-times tables belong to this stage. |
| `0x40` | 4 | The stage's number. |
| `0x44` | `0x10` | Four i32s: two points (x, y) on the map screen where the stage's markers go. |
| `0x54` | `0x20` | The fastest record holder's name (text). |
| `0x74` | 4 | That holder's nationality. |
| `0x78` | 4 | That holder's car. |
| `0x7C` | `0x20` | The time trial record holder's name (text). |
| `0x9C` | 4 | That holder's nationality. |
| `0xA0` | 4 | That holder's car. |

A tool that edits the options should change only the record holders (`0x2C`, `0x30`, and
`0x54` to the end) and leave the rest of each record as it found it.

## A made-up example

The smallest useful `var/rc2k.toml`, which changes two things and leaves everything else at the
game's own values:

```toml
version = 1

[sound]
race_music_on = 1

[players]
names = [
  "416e6e0000000000000000000000000000000000000000000000000000000000",
  "0000000000000000000000000000000000000000000000000000000000000000",
  "0000000000000000000000000000000000000000000000000000000000000000",
  "0000000000000000000000000000000000000000000000000000000000000000",
]
```

The first player is named "Ann" (`41 6E 6E`). When the game next saves its options, it writes
the file out in full, with every key.

And the first bytes of a `var/ral.dat` written on 3 May 2027 at 14:07 by a game started 17
times:

```
52 35 43 49  00 00 00 00  04 00 05 00  87 1B B5 7E
11 00 00 00  ...
```

Code: `rc2k_core/src/options/` (`model.rs`: every value and the old file's order;
`old_file.rs`, `toml_file.rs`: the two formats; `files.rs`: which file is used when),
`src/saves/options_files.rs`, `src/saves/options_state.rs` (where each value lives in the
game), `src/records/stages_store.rs` (the record of a record table).
