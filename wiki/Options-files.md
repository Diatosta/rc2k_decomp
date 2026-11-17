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

A value in `var/rc2k.toml` that is not valid (a word where a number goes, a number out of range,
a text too long) does not stop the game: that one value keeps its built-in value, the other
values of the file are read, and a notice names the values that were reset (also logged as a
`notice:` line). A file that is not TOML at all, or is of a newer version, still stops the program
with the reason shown. The file is not rewritten until the options are next saved, and then the
reset values are written.

When the game saves its options it writes `var/rc2k.toml`. It saves when the program ends (also when
the window is closed) and when an options page, a player's details or controller pages, or a review
menu after a stage is left; not on every key press. It leaves `var/ral.dat` alone.

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
| `[display]` | The screen modes of the menus and of the races (packed numbers) and the mipmapping switch: [Display](#display). |
| `[sound]` | The sound modes (packed numbers), the master and music volumes, and the sliders of the sound pages: [Sound](#sound). |
| `[network]` | This machine's name, the most players, the host's address, the port and the saved connection: [Network](#network). |
| `[pages]` | The graphics page's eleven switches, the audio page's six, a few counters and the arcade levels won: [Pages](#pages). |
| `[cars]` | `first_numbers`, `second_numbers`: the cars the player can choose, and the arcade's secret cars still to be won: [Cars](#cars). |
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

### Display

`[display]` holds the screen choices of the graphics page. The two screen modes are one number
each, written in decimal in the file; to read one, turn it into hexadecimal (`3282946` is
`0x321802`). The digits of a mode, from the right:

| Digits | Meaning |
| --- | --- |
| last two (`0x..__`) | The size, one bit of the game's table of sizes: `0x01` 640 x 480, `0x02` 800 x 600, `0x04` 960 x 720, `0x08` 1024 x 768, `0x10` 1152 x 864, `0x20` 1280 x 1024, `0x40` 1600 x 1200, `0x80` 1920 x 1280. |
| third from the right (`0x.._.`) | The pixel kind: `0x200` 16 bits a pixel, `0x800` 32 bits. This program always draws at 32 bits: a 16-bit mode is taken as the same size at 32 bits. |
| `0x1000`, `0x2000` | The display card: `0x1000` the primary card, `0x2000` the secondary one. This program has one card, so the primary one is used. |
| `0x20000`, `0x40000` | The buffering the mode was tried with: `0x20000` double, `0x40000` triple (the "buffering" row of the graphics page). |
| `0x100000`, `0x200000` | Marks the game puts on a mode that can draw in 3D; every mode the page tries carries both (`0x300000`). |

| Key | What it does |
| --- | --- |
| `menu_mode` | The screen mode of the menus. `3282946` is `0x321802`: size `0x02` (800 x 600), 32 bits (`0x800`), primary card (`0x1000`), double buffering (`0x20000`), the 3D marks (`0x300000`). The menus are always drawn at 800 x 600 (the race's pixel kind) and shown scaled in the window, which has the race's size: what this field holds no longer changes the window. When a race mode is chosen on the graphics page the menus take the same pixel kind at 800 x 600. `0` means none chosen yet: the first start then picks 800 x 600 at 32 bits for both modes (see `videomodemain` in [Settings files](Settings-files)). |
| `race_mode` | The screen mode of the races, in the same form. `3414144` is `0x341880`: size `0x80` (1920 x 1280 in the game's table of sizes), 32 bits, primary card, triple buffering (`0x40000`), the 3D marks. |
| `mipmapping` | The graphics page's mipmapping switch: 0 off, 1 on. Mipmapping draws a texture in smaller pre-reduced copies when it is far away; the choice is held to what the card can do (this program's value is 0 or 1 whatever is written). All three renderers (software, OpenGL, GPU API) draw with it. |
| `render_scale` | The graphics page's "3D resolution": the percent of the screen's width and height the race's 3D is drawn at before it is enlarged or (above 100) scaled down over the screen, 50, 75, 100, 125, 150, 175 or 200 (a value between goes down to the one below; 0 or over 200 is an error). Left out for the default, 100. The menus' pictures, the dashboard and the texts are always drawn at full size; the menus' 3D models are drawn, on the GPU renderers, at the size their frame has in the window times this percent (the software rasteriser at the frame's size). |
| `renderer` | The advanced page's renderer, swapped in at once on the desktop and tried first at start: `"gpu"` (the GPU first, the normal order) or `"software"`. Left out until chosen. `--renderer` and `RC2K_RENDERER` outrank it; when it cannot start, the normal order runs and the file is not changed. |
| `draw_threads` | The advanced page's number of parts of a frame drawn at once, 1 or more (more than the machine can draw at once counts as the most). Left out for the most. |
| `quit_pictures` | The advanced page's "buttons in the quit question" switch (it also covers the tyre change's prompt): `false` draws the race's quit question as the game's own line, without the controller's pictures and their dark box, and the tyre change's prompt without the button's picture. Left out when on (the default). |
| `frame_limit` | The graphics page's "frame limit": `"off"` (nothing in the program waits for a frame's time; a display that waits for its refresh still does), `"display"` (one frame for each refresh; with `smooth_frames` off, every second or third on a display faster than 100 Hz) or a number of frames a second (with `smooth_frames` off a number over 100 counts as 100). Left out for the default, `"display"`. The race's speed does not depend on it. |
| `race_size` | `[width, height]`: the race's size when it is none of the game's own eight (the graphics page's "resolution" offers the monitor's modes and the usual sizes beside them). The `race_mode` beside it then holds 800 x 600 (`0x02`), which the original's file reads. Left out when the size is one of the game's. |
| `custom_width`, `custom_height` | A size set by hand, any shape (320 to 16384 across, 240 to 16384 down; both keys or neither): the race uses it from the start, whatever was chosen before, and the page lists it as "custom W x H". Take both keys out to go back to the page's choice. Left out when unset. |
| `window_mode` | The graphics page's "screen" row (Alt+Enter toggles between a window and full screen): `"window"` (the default, left out), `"fullscreen"` (the display set to the mode nearest the race's size) or `"borderless"` (over the whole desktop without a mode change, the frame shown scaled in it, its shape kept). |
| `hud_box` | The graphics page's "HUD area": the box, centred on the screen, the HUD's pieces keep to: `"4:3"`, `"16:9"` or `"21:9"` (never wider than the screen). Left out for the whole screen, the default. The 3D always fills the screen. |
| `smooth_frames` | The advanced page's "smooth frames" switch: on (the default), the race's frames between two of the simulation's ticks are drawn blended between the last two, so a display faster than 100 Hz shows motion in every frame (the picture is up to a tick, 10 ms, behind the simulation); `false` draws the last tick as it is and holds the frames to 100 a second at most. Left out when on. |
| `split_layout` | The layout of a split screen of three or four players, as its place in the layouts the F2 key cycles through (with four players quarters, rows, columns; with three one wide view above two, rows, columns, quarters). `0`, the first, is the default and is left out. An older file's `split_many` is read and ignored. |

### Sound

`[sound]` holds the sound modes and the volumes. A volume is a number from 0 (silent) to 64
(`0x40`, full): the sliders of the sound pages move it by one step at a time. Two kinds of
numbers live here: modes (packed, see below) and levels.

A **sound mode** is one number, written in decimal; turn it into hexadecimal to read it
(for example `70177` is `0x11221`). Its parts, from the right:

| Digits | Meaning |
| --- | --- |
| last three (`0x___`) | The quality, which decides the sample format the sounds are made ready in. Its last digit says how many times the sample rate is halved: the four qualities of the advanced audio page are `0x212` (lowest, a quarter of the rate), `0x222` (a quarter), `0x221` (half) and `0x220` (full rate); the middle digit is the sample size. |
| `0x1000`, `0x2000` | The sound device. `0x1000` is DirectSound, the only one now. `0x2000` was Aureal's A3D; an options file that names it is read as the same mode with DirectSound's 3D sound in its place. |
| `0x10000`, `0x20000`, `0x40000` | How the sound is made. `0x10000`: each voice has its own buffer. `0x20000`: static buffers. `0x40000`: the game's own mixer (Dolby). |
| `0x100000` | 3D sound. |
| `0x400000` | 16 channels (without it, 8). |
| `0x800000` | The card's own 3D buffers are used (set with 3D sound when the card has free ones). |
| `0x1000000` | EAX (3D sound with a reverb room chosen per stage). |
| `0x2000000` | Dolby (always with the game's own mixer: a Dolby mode is `0x2040000` plus the quality). |

| Key | What it does |
| --- | --- |
| `menu_sound_mode` | The sound mode of the menus. The game sets it to `0x11221` (`70177`: DirectSound, own buffers, half the sample rate) whenever the audio options are applied; 0 means not yet chosen. |
| `race_sound_mode` | The sound mode of the races, built from the advanced audio page (`[pages] audio`) and what the machine has. `0x11221` is the plain mode; with 3D sound it carries `0x100000` (`0x111221`), with the card's own buffers `0x911221`, with EAX also `0x1000000` (`0x1911221`). 0 means not yet chosen. |
| `master_volume` | The master volume the audio thread holds, 0 .. 64. A fresh game holds 65, which counts as full. It follows the slider `general_volume`. |
| `general_volume` | The sound page's "general volume" slider, 0 .. 64: it is sent to the audio thread as the master volume, at start and at each step. |
| `unread_option_f0` | A sound option the old file has room for; nothing reads it (a fresh game holds 1). Kept so the old file can be written back as it was. |
| `race_music_on` | The race music: 0 off, 1 on. The music of the races (tune 8) plays only while it is 1; the sound page shows it as an off / on row. |
| `tune_turn` | Which tune the music plays next: a counter the audio thread keeps for the music's order and saves, so the tunes go on from where they stopped. |
| `music_volume` | The volume the menus' music is started at, 0 .. 64. At the start of the front end it is set from `menu_music_volume`. |
| `race_music_volume` | The volume the races' music is started at, 0 .. 64. At the start of the front end it is set from `race_music_level`. |
| `menu_music_volume` | The "menu music" slider, 0 .. 64 (a fresh game 64). Stepping it sets `music_volume` and the music's volume at once. |
| `race_music_level` | The "race music" slider, 0 .. 64: stepping it sets `race_music_volume`. |
| `film_volume` | The volume page's "film volume" slider, 0 .. 64 (an optional field, only in `rc2k.toml`; missing it is 64, full): the intro and logo films play at this volume. |
| `sound_in_background` | The sound page's "sound when not in front" row (an optional field, only in `rc2k.toml`; missing it is `false`). Off, the sounds go quiet while the window is not in front, as the original's DirectSound did; on, they go on. |
| `menu_effects_volume` | The "menu effects" slider, 0 .. 64: the menus' own sounds (clicks, moving between items); a sound plays at each step so that it can be heard. |
| `race_effects_volume` | The "race effects" slider, 0 .. 64: the level of the race's sound effects (those whose sample record names this level); a race sound plays at each step. |
| `menu_voices_volume` | The "menu voices" slider, 0 .. 64: the spoken sounds of the menus (the announcer, the car's names); a sample plays at each step. |
| `codriver_voice_volume` | The "co-driver's voice" slider, 0 .. 64: the level of the pace notes the co-driver reads in a race. |
| `engine_volume` | The "car engine" slider, 0 .. 64: the level of the cars' engine sounds. |

### Network

`[network]` holds the settings of the network pages. This program plays on TCP/IP (and, when Valve's library is there, Steam), host
and guests running the same program ([Network play](Network-play)).

| Key | What it does |
| --- | --- |
| `machine_name` | The name this machine gives itself in a network game: it names the session when hosting, and is the host's name in the lobby. Up to 63 characters. When it is empty at start, the computer's name is put in; if that is empty too, a default name is made. |
| `most_players` | The most players a hosted session takes (the host's limit; the network page's "most players" row). In one language edition of the game's texts it is held to 4. |
| `host_address` | The address a joining player connects to: a host name or an IP address, up to 63 characters. Empty: look for games on the local network. |
| `port` | The port used with the address, on both TCP and UDP. 0 means the default, 47624 (`0xB9E8`). Every machine in a game must use the same one. |
| `password` | The password a session this machine hosts is locked by, up to 63 characters; empty or missing: no password. Only in this file (the old `ral.dat` has no room for it), and written as plain text, so keep the file to yourself. A joining player is asked for a locked session's password when joining, and it is not kept. |
| `last_host` | The address of the last host this machine joined (an IP address), probed again by a search that has no address typed, so a host over a network with no broadcast (Tailscale) is found again. Optional; missing or empty: none. |
| `saved_connection` | The connection (a DirectPlay service provider, written as a GUID) used last time. At the start of a network game the program looks for a listed connection with this GUID and takes it without asking; otherwise it asks, then saves the one chosen. This program lists TCP/IP, whose GUID is `36E95EE0-8577-11CF-960C-0080C7534E82`, and Steam when available (a GUID of its own); without Steam it is always the first; a different value in an old file is not found and is replaced when a connection is chosen. |

### Pages

`[pages]` holds what the options pages and the front end keep: the graphics switches, the
audio switches, and a few counters. Each of the first two is a list in a fixed order; the
rows below give each place (`switches[0]` is the first) and its default for a fresh game.

**`switches`**: the graphics page, eleven values.

| Place | Name | What it does |
| --- | --- | --- |
| `switches[0]` | mipmapping | Mipmapping, 0 off, 1 on (default 0). A copy of `[display] mipmapping`. |
| `switches[1]` | buffering | Buffering, 0 double, 1 triple (default 0). The screen mode is tried with the matching buffering bits (`0x20000` or `0x40000`). |
| `switches[2]` | mode changed | A flag flipped each time the resolution row of the graphics page is stepped (default 0). Nothing the player sees. |
| `switches[3]` | modes switch | The display card row, 0 primary, 1 secondary (default 1). This program has one card, so the row is not offered and the value changes nothing. |
| `switches[4]` | lens flare | The sun's lens flare in races, 0 off, 1 on (default 1). |
| `switches[5]`, `switches[6]` | two unused percents | Two levels from 0 to 100 (default 100 each) that only code no longer reachable steps; nothing reads them. |
| `switches[7]` | reflections | The shine on shiny faces (the cars' bodies, windows), 0 off, 1 on (default 1). |
| `switches[8]` | car shadows | The shadow under each car: 0 "reduced" (one pass), 1 "full" (two passes) (default 1). |
| `switches[9]` | car detail | Which cars are drawn with their see-through windows: 0 none, 1 only your own car, 2 all cars (default 1). |
| `switches[10]` | headlights | Whether the cars' headlights light the road and scenery: 0 only the players' cars, 1 all cars (default 0). |

**`audio`**: the advanced audio page, six values (a fresh game: `[3, 0, 0, 0, 0, 1]`).

| Place | Name | What it does |
| --- | --- | --- |
| `audio[0]` | audio quality | 0 lowest .. 3 highest (default 3): the sample rate and size of the sounds (see the qualities under Sound). |
| `audio[1]` | DirectSound 3D | 3D sound: 0 off, 1 on. Turning it on turns Dolby off. |
| `audio[2]` | EAX | The reverb rooms: 0 off, 1 on. Turning it on turns 3D sound on and Dolby off. |
| `audio[3]` | retired A3D | The original's switch for Aureal's A3D sound. There is no A3D any more: no page shows it, nothing reads it, and a file loaded at start has it set to 0. Kept so that a file is written back as it was read. |
| `audio[4]` | Dolby | The game's own mixer: 0 off, 1 on. Turning it on turns 3D sound and EAX off. |
| `audio[5]` | channels | How many sounds can be heard at once: 0 eight, 1 sixteen (default 1). |

| Key | What it does |
| --- | --- |
| `services` | How many service-area visits the championship under way has been through, counted up at each service and reset when an event begins. |
| `event_kept` | A word the front end hands to the event's races and takes back after each race (0 when the event's races begin; a fresh game 1). What it stands for could not be told from the code. |
| `race_settings` | The first word of the race's settings: 1 once the A8 game's "new simulation" is unlocked (by the "Enable A8 Game" or "Reverse Sim Tracks" cheat), else 0. |
| `arcade_levels_won` | Six values, one byte for each place of the arcade's level table, 1 when open, 0 when not. The n-th belongs to arcade level n, and winning level n opens place n+1 (the next level) and, the first time, shows the level's secret car and adds it to the cars; so the sixth only records that level 5 was won. The cheat "Unlock Arcade" opens every level whatever these say. |

### Cars

`[cars]` holds the two lists of cars the front end keeps. A number is the game's car number
(the same one as `car_number` of a player); each list ends at its first empty place, and the
file leaves the empty places (-1) out (see the values above).

| Key | What it does |
| --- | --- |
| `first_numbers` | The cars the player can choose in the single events and arcade (up to 40). It starts with the standard cars; the secret cars win on the arcade, and the cars the cheats give, are added at its end. |
| `second_numbers` | The secret cars the arcade can still give (up to 20): winning an arcade level for the first time shows its secret car, takes it out of this list and adds it to `first_numbers`. Empty: all won. |

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
| `sensitivity_level` | The controller page's former "axis mode" item, which is gone: 0 analogue joystick, 1 digital joystick or gamepad, 2 two-axis wheel with pedals, 3 three-axis wheel with pedals. Kept in saved games; a controller no longer uses it (the player's profile for it says which axes do what). |
| `pedal_axis_codes` | The axes learned for the pedals in the three-axis wheel mode. Kept, unused. |
| `controller_mode` | The controller page's former "H shifter" switch. Kept in saved games as it was; nothing reads it now: the switch is the controller profile's `shifter`. |
| `end_zone_percent`, `centre_dead_zone_percent` | The controller page's two former dead zone items, gone (each axis has its dead zone in its profile, set on the axis pages): how much of the stick's travel at the far end counts as full, and how much around the centre counts as nothing, in percent. |
| `strength` | The controller page's force feedback strength, 0 .. 100 (0: off). |
| `binding_buttons`, `binding_directions` | The old record's bindings, eleven each, in this order: accelerate, brake, steer left, steer right, gear up, gear down, handbrake, camera, pause, lights, reset car. Kept as saved games carry them; the race reads the profiles (below) instead. |
| `controller_sensitivity` | The player details page's steering sensitivity, 0 .. 100. |

A player who chose a controller also has a `[players.player_n.controller_device]` table, which
says which controller it is: its `name`, its `serial` number when it reports one, and `nth`,
which of the controllers of that name it is (counted in the order they are listed). The program
works `controller` out from it at start and whenever a controller is plugged in or out, so the
player keeps their controller whatever order the controllers come in ([Controllers](Controllers)).

A player also has a list `[[players.player_n.profiles]]`, one for each controller they have used
(made the first time, with the defaults described in [Controllers](Controllers)):

| Key | Meaning |
| --- | --- |
| `device` | The controller, as in `controller_device`: `name`, `serial` (if any) and `nth`. |
| `wheel`, `full_rotation_degrees` | A steering wheel, and its rotation lock to lock in degrees (default 900). |
| `button_icons` | What the controller's buttons are shown as: `"xbox"`, `"play_station"`, `"steam_deck"` or `"numbers"`; left out for the default, automatic (SDL's gamepad type and USB ids; numbers for a joystick). [Controllers](Controllers#button-pictures). |
| `shifter` | The controller's "H shifter" switch (default false, left out when false): this controller's hat and buttons are a gear shifter's. |
| `axes` | A list of tables, axis 0 first, each with `dead_zone` and `range` (per mille of the travel), `sensitivity` (-100 .. 100) and `invert`. An axis past the list has the defaults (no dead zone, range 1000, linear). The axes are SDL's; a gamepad's are the left stick x, y, the right stick x, y, the left trigger and the right trigger. |
| `bindings.steering_axis`, `accelerate_axis`, `brake_axis` | An axis: `kind = "centred"` and `axis`; or `kind = "pedal"`, `axis` and `rest_high` (the end it rests at has the large values); or `kind = "half"`, `axis` and `positive` (the side of the axis used). |
| `bindings.left`, `right`, `accelerate`, `brake`, `gear_up`, `gear_down`, `handbrake`, `camera`, `pause`, `lights`, `reset` (an old file's `wipers` is read and passed over: the race reads no wipers key) | A button (`button = 5`, from 0) or a hat direction (`hat = 0`, `direction = "up"`, `"right"`, `"down"` or `"left"`). A key or one of these gives all of the control at once. |
| `bindings.gear_1` .. `gear_7`, `reverse` | The H shifter's gears (the controller profile's `shifter` on): a button or a hat direction as above, the input held while the shifter is in that gear. Absent: not bound; with none bound the shifter reads the buttons and hat the way it did before the gears could be bound ([Controllers](Controllers#the-h-shifter)). |
| `bindings.menu_enter`, `menu_back`, `menu_up`, `menu_down`, `menu_left`, `menu_right`, `menu_third`, `menu_tab` | What the menus take from this controller: a button or a hat direction as above. Absent: the controller's own default for that control (a gamepad's face buttons, cross and shoulders; a wheel's second and third buttons and its hat); present, it replaces that default ([Controllers](Controllers)). |

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
  file again. A later version can add keys without older builds losing them. A key the program
  knows is written as the option now is: one back at its default (a window, no password, no size
  set by hand) is taken out of the file.
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
| `0x0B48` | `0x4` | `sound.general_volume` | The "general volume" slider. |
| `0x0B4C` | `0x4` | `sound.menu_music_volume` | The "menu music" slider. |
| `0x0B50` | `0x4` | `sound.race_music_level` | The "race music" slider. |
| `0x0B54` | `0x4` | `sound.menu_effects_volume` | The "menu effects" slider. |
| `0x0B58` | `0x4` | `sound.race_effects_volume` | The "race effects" slider. |
| `0x0B5C` | `0x4` | `sound.menu_voices_volume` | The "menu voices" slider. |
| `0x0B60` | `0x4` | `sound.codriver_voice_volume` | The "co-driver's voice" slider. |
| `0x0B64` | `0x4` | `sound.engine_volume` | The "car engine" slider. |
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
