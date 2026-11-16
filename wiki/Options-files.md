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

A TOML file, in UTF-8. The first key is `version` (now 2), then one table for each area:

```toml
version = 2

[display]
menu_mode = 3282946
race_mode = 3414144
mipmapping = 1

[network]
machine_name = "my-pc"
saved_connection = "36E95EE0-8577-11CF-960C-0080C7534E82"

[players.player_1]
name = "Ann"
nationality = 0
car_number = 12
...
```

| Table | Holds |
| --- | --- |
| `[display]` | `menu_mode`, `race_mode`, `mipmapping`. |
| `[sound]` | The sound modes, volumes and levels. |
| `[network]` | `machine_name`, `most_players`, `host_address`, `port`, `saved_connection`. |
| `[pages]` | The options pages' switches (`switches`, `audio`), a few counters and `arcade_levels_won`. |
| `[cars]` | `first_numbers`, `second_numbers`: the two lists of cars the player can choose from. |
| `[players.player_1]` .. `[players.player_4]` | One table for each player: name, nationality, car, views, keys, controller and its bindings, the car's set-up and wear, the results so far. |
| `[records]` | The tyre-change record, `championship`, `events` and `rallies`. |
| `[kept]` | Values the old file carries that nothing uses; kept so that the old file can be written back exactly. |

The values are written readably:

| Kind | Written as | Example |
| --- | --- | --- |
| A number (u32) | an integer | `port = 0` |
| A time or other value that can be -1 | a signed integer | `first = -1` |
| A list of numbers | an array of integers, of a fixed length | `audio = [3, 1, 0, 0, 0, 1]` |
| A text (a name, an address) | a string; the game's Windows-1252 characters are the same characters in the file | `name = "Ann"` |
| A GUID | the usual text, `XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX` (braces and lower case are read too) | `saved_connection = "36E95EE0-8577-11CF-960C-0080C7534E82"` |
| A float (the car's impact marks, 0 .. 1) | a number | `impact_marks = [0.0, 0.25, ...]` |

A text ends at its first NUL in the old file's buffers, and what a buffer holds after that
is not kept (the game never reads it): the file holds the text only. A text that is longer than
its buffer (`machine_name` and `host_address` 64 characters, the others 32) is an error.

The two lists of car numbers leave out their "none" entries at the end (`0xFFFFFFFF` in the old
file); reading puts them back, so `first_numbers = [13, 21, 18]` is those three cars and 37
empty places. A "none" between two numbers is written as -1.

### A player

`[players.player_1]` .. `[players.player_4]` each hold one player's settings and the state of
their car in the event under way. The keys below are grouped by what they are for; in the file
they come in the order of the game's player record ([Saved games](Saved-games#the-player-record)
describes its bytes). Choices are numbers, counted from 0 unless the table says otherwise.

**Who the player is**

| Key | What it does |
| --- | --- |
| `name` | The player's name, as typed on the player details page. |
| `name_slot` | Which of the game's four name slots holds this player's name, 1 .. 4 (0: none). The game keeps the names apart from the players and links each player to one; normally player n has slot n. |
| `name_copy` | A copy of the name kept in the player's record itself (made when names are linked to players, and written to the game's log); the game refreshes it from `name`. |
| `nationality` | The country chosen on the player details page; its flag is shown beside the name. |
| `codriver_sex` | The co-driver's voice for the pace notes: 0 male, 1 female. |
| `race_number` | The number on the car's doors, 1 .. 90. |

**The car and the views**

| Key | What it does |
| --- | --- |
| `car_number` | The car the player drives, a number in the game's car list. |
| `gearing` | The gearbox: 0 manual, 1 semi-automatic, 2 fully automatic. |
| `view` | The camera in a race: 0 bumper, 1 bonnet, 2 driver, 3 interior, 4 .. 6 chase 1 .. 3. |
| `second_view` | The camera in split screen: 0 bonnet, 1 bumper, 2 .. 4 chase 1 .. 3. |
| `setup_tyre`, `setup_brake_balance`, `setup_ride_height`, `setup_suspension_stiffness`, `setup_steering_sensitivity`, `setup_gear_ratio` | The car's set-up from the service area's set-up pages. |

**The keyboard and the controller**

| Key | What it does |
| --- | --- |
| `key_scan_codes` | The twelve keys of the "redefine keyboard" page, in this order: steer left, steer right, accelerate, brake, handbrake, gear up, gear down, lights, wipers (no page item; unused), camera, pause, reset car. Each is a key number: the PC keyboard's scan code (for example 44 is Z, 45 is X, 57 the space bar), with the arrow keys and other extended keys numbered 0x60 .. 0x7A by the game. |
| `controller` | The controller the player drives with: 0 the keyboard, n the n-th controller in the list of those plugged in. The program sets it from `controller_device` (below); changing it by hand has no lasting effect. |
| `sensitivity_level` | The controller page's "axis mode": 0 analogue joystick, 1 digital joystick or gamepad, 2 two-axis wheel with pedals, 3 three-axis wheel with pedals. It decides which axes steer, accelerate and brake. |
| `pedal_axis_codes` | The axes learned for the pedals in the three-axis wheel mode (brake, then accelerate in the last two bytes): the axis number, plus 16 when the pedal moves the axis up. |
| `controller_mode` | The controller page's "ACT LABS RS Shifter" switch: 1 reads the gear lever of that shifter (its buttons and hat as gears). |
| `end_zone_percent`, `centre_dead_zone_percent` | The controller page's two dead zones: how much of the stick's travel at the far end counts as full, and how much around the centre counts as nothing, in percent. |
| `strength` | The controller page's force feedback strength, 0 .. 100 (0: off). |
| `binding_buttons`, `binding_directions` | The controller's bindings from the "program controller" page, eleven each, in this order: accelerate, brake, steer left, steer right, gear up, gear down, handbrake, camera, pause, lights, reset car. A button is its number (from 1), a direction 1 the stick's y, 2 its z, 4 left, 8 right; 0 is none. |
| `controller_sensitivity` | The player details page's steering sensitivity, 0 .. 100. |

A player who chose a controller also has a `[players.player_n.controller_device]` table, which
says which controller it is: its `name`, its `serial` number when it reports one, and `nth`,
which of the controllers of that name it is (counted in the order they are listed). The program
works `controller` out from it at start and whenever a controller is plugged in or out, so the
player keeps their controller whatever order the controllers come in ([Controllers](Controllers)).

**On-screen displays** (the visual preferences page; each 1 shown, 0 hidden)

| Key | What it shows |
| --- | --- |
| `map_area` | The local map. |
| `rear_mirror` | The rear-view mirror. |
| `rev_counter` | The rev counter. |
| `car_info` | The car's information panel. |
| `message_display` | The message bar. |
| `progress_bar` | The progress bar along the top of the screen. |
| `driver_info` | The driver's information panel. |
| `pace_notes` | The pace note arrows. |
| `metric_units` | The units: 1 metric (km/h), 0 imperial (mph). |

**The event under way** (reset when an event starts; kept so that it can go on after the
program is closed)

| Key | What it holds |
| --- | --- |
| `entry` | How the player stands in the event: 1 taking part, 2 below the scoring places, 6 out. |
| `time`, `total` | The last stage's time and the event's total so far, in hundredths of a second (-1 or a very large value: none yet). |
| `class` | The car's class, 0 .. 3, which decides the standings group. |
| `figure` | The figure the results pages show for the player (the time trial's result). |
| `stage_points`, `overall_points` | The points of the event's six sections and their total (-1: not driven yet). |
| `number_standings_a`, `number_all_a`, `number_standings_b`, `number_all_b` | The player's places in the two standings tables (in their class and among all), -1 when not placed. |
| `damage` | The damage of the car's fourteen parts, as the service area shows it. |
| `parts_wear`, `tyre_wear` | The wear of the car's parts and of its tyres, carried from stage to stage. |
| `impact_marks` | How hard the car has been hit from each of eight directions, 0 .. 1; a body repair clears them. |
| `state_flags` | The car's repair state (which repairs were made; -1 after a full repair). |
| `best_line_checkpoint_times` | The time trial's best run: the time at each of up to sixteen checkpoints, in hundredths (-1 or 0: none). |
| `service_spent` | What the service area has spent so far in the event (money × 100). |
| `saved_start_section`, `saved_start_section_a8` | The section of the championship, and of the A8 championship, a saved game goes on from. |
| `result_word`, `hud_bits` | Values the last race left for the next (its result, and which on-screen items it showed); the game sets them itself. |

**Kept, unused**: `kept_binding_bits` and the `kept_unused_*` / `kept_unread_*` keys are parts of
the record that nothing reads; they are written so that the old file can be written back exactly.

### Kept

`[kept]` holds values the old file `ral.dat` has room for but the game does not use for
anything a player sees; they are in the new file so that the old one can be written back as it
was.

| Key | What it is |
| --- | --- |
| `runs` | How many times the game has been started (counted up at each start; nothing else reads it). |
| `saved_games_a`, `saved_games_b` | Two words of the saved games' settings that the game writes and never reads. |
| `set_ups` | A word of the car set-ups that the game writes and never reads. |
| `options` | Four more words the old file carries and nothing reads. |

### Records

| Key | What it holds |
| --- | --- |
| `tyre_change_holder`, `tyre_change_time` | The tyre-change record. When a tyre must be changed at the roadside during a stage (after a puncture), the change is timed: a gauge fills while the button is held and the press times it. The fastest change is kept here: who made it and the time in hundredths of a second. A fresh install holds a made-up driver's 210.00 s (`21000`). |
| `championship` | The championship's 66 stage records (below). |
| `events` | `event_00` .. `event_11`: the twelve single events' tables, 7 stage records each. |
| `rallies` | `rally_0` .. `rally_5`: each rally's 20 best times (below). |

A **stage record** is one step of an event: a stage, a service area or the end of a rally. Most
of it is the event's fixed description, the same in every copy of the game; it is in the file
because it also holds the stage's two record holders, which change as people set records.

| Key | What it holds |
| --- | --- |
| `tag` | What the step is: 0 a stage, 1 a service area, 3 or 4 the end of a rally, 5 a welcome screen; -1 ends a table. |
| `track`, `pack_id` | Which track, in which of the game's track lists. |
| `service_figure` | For a service area, the service figure; 1 on a stage. |
| `start_hhmm` | The time of day the stage starts at, as hours × 100 + minutes (`1430` is 14:30): it sets the light. |
| `service_time` | The time allowed in a service area. |
| `class_times` | Target times for the four classes of car. |
| `race_stage`, `name_string`, `event_slot`, `number` | Which stage the race loads, the number of its name among the game's texts, which ten best-time entries belong to it, and its number. |
| `marker_points` | Where the stage's two markers go on the map screen (x, y each). |
| `fastest_name`, `fastest_nation`, `fastest_car`, `fastest_speed` | The stage's fastest driver: name, nationality, car, top speed. **These change when a record is beaten.** |
| `time_trial_name`, `time_trial_nation`, `time_trial_car`, `time_trial_speed` | The same for the stage's time trial record. **These change too.** |

A tool that edits the file should change only the record holders and leave the rest of each
stage record as it is.

A **best time** in `rallies` is one entry of a rally's best-times list:

| Key | What it holds |
| --- | --- |
| `first` | 0 for an entry; -1 ends the list. |
| `name`, `nationality`, `car`, `group` | Who drove it, from where, in what car, in which class. |
| `player` | The player who set it (1 .. 4), or 0 for one of the game's made-up drivers. |
| `name_id` | For a made-up driver, the number of their name among the game's texts. |
| `key` | What the list is sorted by: the time in hundredths of a second. |
| `rally_time` | The rally's total time, in hundredths. |
| `mark`, `figure`, `entered` | 1 for the entry just made (shown highlighted), a figure shown with it, whether it takes part. |
| `cheats` | The cheats that were on when it was driven, one bit each. |
| `sealed_time` | The time again, enciphered, so that an edited `key` can be told ([Best times](Best-times#the-sealed-time)). |
| `checkpoint_times` | The time at each checkpoint, in hundredths (-1: the stage has no such checkpoint). |
| `kept_unread_48` | Unused. |

Rules for reading:

- A key that is missing keeps the game's built-in value (in a player's table, too). An empty
  file is valid. The lists of tables (`championship`, an event's, a rally's) are taken whole:
  one is read as written, with every key of each entry.
- A key the program does not know is ignored, and is still there after the program writes the
  file again. A later version can add keys without older builds losing them.
- A list of the wrong length, a text too long for its buffer, a GUID that is not one, or a file
  that is not valid TOML, stops the program with a message in `rc2k.log`.
- A `version` higher than this program knows (2) stops it too: the file is from a newer build.
- A file of version 1 (the first, which held the names, records and tables as strings of
  hexadecimal digits) is not read: it counts as not there, so the options come from
  `var/ral.dat` if that exists and else from the built-in values, and the next save writes the
  file anew (nothing of the old file is kept).

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
"Key" column is the value's name in the program's model of the options; it is the key in `var/rc2k.toml` too, but for the players (`players.records`, `players.names`: see above) and the record tables (`records.*`). A size of `0x4` is one u32.

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
version = 2

[sound]
race_music_on = 1

[players.player_1]
name = "Ann"
```

The first player is named "Ann". When the game next saves its options, it writes the file out
in full, with every key.

And the first bytes of a `var/ral.dat` written on 3 May 2027 at 14:07 by a game started 17
times:

```
52 35 43 49  00 00 00 00  04 00 05 00  87 1B B5 7E
11 00 00 00  ...
```

Code: `rc2k_core/src/options/` (`model.rs`: every value and the old file's order;
`old_file.rs`, `toml_file.rs`: the two formats, `document.rs`: the new file's readable form of the values; `files.rs`: which file is used when),
`src/saves/options_files.rs`, `src/saves/options_state.rs` (where each value lives in the
game), `src/records/stages_store.rs` (the record of a record table).
