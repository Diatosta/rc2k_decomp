# Settings files

*Category: [Data formats](Data-formats)*

`var/ral.cfg` is a small text file of switches that the game reads once at start-up. It is the
1999 game's way of turning features on and off without a menu: the log, screenshots, the frame
counter, and a set of physics tuning switches among them. The game never writes it; you make and
edit it by hand. Without the file every switch has its built-in value.

`var/name.cfg`, also optional, gives the computer drivers other names.

The options chosen in the menus are not here: see [Options files](Options-files). This
project's own switches are on the command line: see [Running and options](Running-and-options).

## The text format

The same format is used by `var/ral.cfg`, `var/name.cfg` and the install records
(`var/ral.zog`, `lang.cfg`, `files/d/gfx/sku`; see [Install records](Install-records)).

- The file is plain text, one setting to a line, as `key=value` or `key value`. Lines may end
  in CR LF or LF. At most `0xFFFF` (65535) bytes are read.
- The key must be the first thing on its line (spaces and tabs before it are passed over). Keys
  are matched without regard to case.
- Between key and value there may be spaces or tabs, one `=`, and more spaces or tabs.
- There is no comment sign. A line that starts with anything that is not a known key is simply
  never looked up, so lines beginning with `;` or `#` work as comments.
- The game looks each key up from the top of the file, so if a key appears twice the first one
  counts.
- A file that ends with a key and nothing at all after it, not even a line break, is refused.

A **number** is decimal, or hexadecimal after `$` or `0x`; it may have `+` or `-` in front. It
ends at the first character that is not a digit. Numbers are 32 bits and wrap round.

A **text** value runs to the end of the line or to a tab. If it starts with `"` it runs to the
next `"` instead and may hold tabs.

## `var/ral.cfg`

The "advanced" page of the options menu shows eight of these switches (`framecounter`, `nofmv`, `screensnapshot`, `nonlinearsteering`, `noautoneutral`, `noprecipitation`, `nomusic`, `noforcefeedback`; `nojoy` is read but not shown; its other rows, the drawing threads, the renderer and the quit question's pictures, are kept in `rc2k.toml`) and writes a changed one back: its first line is replaced by `key=number` (the line's ending and every other line stay), a key that was not there is added at the end. See [Differences from the original](Differences-from-the-original).


Every value is a number. Most switches are 0 (off, the default) or 1 (on).

The switches most often wanted:

| Key | Effect |
| --- | --- |
| `logfile` | How much the game writes to `var/logfile.txt`: 0 nothing, 1 the main messages (the default), 2 or more everything. See [Log files](Log-files). |
| `screensnapshot` | 1 turns the screenshot key on. See [Screenshots](Screenshots). |
| `framecounter` | 1 shows the frame rate in races. |
| `noreplay` | 1 stops the game recording a replay of each race. |
| `replaykeys` | 1 turns on the replay viewer's editing keys. |
| `wrreplayenable` | 1 turns on the replay viewer's developer keys (saving a sequence of pictures); `wrreplayrate` sets the sequence's picture rate. |
| `usesavegamekeys` | 1 makes a loaded saved game bring its own controls with it. See [Saved games](Saved-games). |
| `seasoncode` | A number stamped into each replay's sealed part. See [Replay files](Replay-files). |
| `key` | Replaces the first half of the key that seals best times. See [Best times](Best-times). |

The physics switches. They change how the cars drive; each replay records the values it was
made with and plays back with them (see [Replay files](Replay-files)). With `fizpatch` on, best
times are marked as driven with changed physics.

| Key | Default | Effect |
| --- | --- | --- |
| `fizpatch` | 0 | 1 puts the values below in force. |
| `patchmaxdrawdistance` | 12 | How far the course is drawn. |
| `patchstickygrass` | 0 | 1 makes grass hold the car back more. |
| `patchgravity` | 100 | Gravity, in percent. |
| `patchdamages` | 100 | How easily the car is damaged, in percent. |
| `patchbrakegrip` | 100 | Grip under braking, in percent. |
| `patchaccelgrip` | 100 | Grip under acceleration, in percent. |
| `patchslidegrip` | 100 | Grip when sliding sideways, in percent. |
| `patchgripgrass` | 100 | Grip on grass, in percent. |
| `patchgriptarmac` | 100 | Grip on tarmac, in percent. |
| `patchgripsnow` | 100 | Grip on snow, in percent. |
| `patchgripmud` | 100 | Grip on mud, in percent. |
| `patchgripgravel` | 100 | Grip on gravel, in percent. |

The other keys, grouped by what their names and the variables they set say they are for. All of
them are read and stored. Those in the second half of the table dealt with hardware or with
parts of Windows that this program does not use, and have nothing left to act on. The effect of
each key in this table was not tried one by one for this page.

| Keys | What they are for |
| --- | --- |
| `nofmv` | 1 skips the films. |
| `noaudio`, `nosound` | 1 turns all sound off (two names for one switch). |
| `nomusic` | 1 turns the music off. |
| `noprecipitation` | 1 turns rain and snow off. |
| `allsecretcars`, `secretcar` | 1 makes every car available (two names for one switch). |
| `sensiblesaving` | Changes how the screens between stages offer saving. |
| `noautoneutral`, `nonlinearsteering` | Driving aids: the automatic neutral, and steering that is finer near the centre. |
| `nojoy`, `noforcefeedback`, `joyffgain`, `joylowerdz`, `joyupperdz`, `mergejoy`, `joymode`, `joycode`, `alt2axcode` | Controllers: off altogether, force feedback off and its strength, the sticks' dead zones, one player on two devices, axis layouts. |
| `nonetwork`, `inetport` | Network play off, and its port. |
| `videomodemain`, `videomodegame` | The screen modes of the menus and of the race (the options files hold the same two values). The screen always has 32 bits a pixel: a 16-bit mode is taken as the same size at 32 bits, and a first start picks 800 × 600 at 32 bits. |
| `lexcontrol`, `sparksmode`, `textflashmode` | Developers' switches for the texts and the menus. |
| `notaskswitch` | Windows task switching. |
| `noprichange`, `nocdaudio`, `forceno3dcard`, `ignoreprimary`, `ignoresecondary`, `ignoremmx`, `ignorekatmai`, `ignoreamdk3d`, `ignorea3d` | Read and not used: the process's priority is left alone, there is no CD audio, there is one display card, and the game's arithmetic has one path whatever the processor; the game has no A3D sound. |
| `altmemenable`, `altmemheapsize`, `wavheapsize`, `bundleheapsize`, `imageheapsize`, `gamegfxheapsize` | Sizes of the 1999 game's memory pools. |
| `audiobasefreq`, `ignoreds3d`, `ignoredsv`, `ignoreeax` | Sound hardware: the mixing rate, 3D sound systems. |
| `forcedx5`, `agpmode`, `force3dtrans`, `force3dalpha`, `force3dshadow`, `force3dfogging`, `v3driverfix`, `nopngdecode` | Display cards and drawing work-arounds. |

A made-up `var/ral.cfg`:

```
; my settings
logfile=2
screensnapshot=1
framecounter 1

fizpatch=1
patchgravity=80
patchgripmud=$50
```

The log is at its fullest, screenshots and the frame counter are on, gravity is at 80 percent
and grip on mud at `0x50` (80) percent.

## `var/name.cfg`

Up to 180 lines with the keys `name000` .. `name179`, one for each computer driver. A value is
up to three words separated by spaces or tabs:

| Word | Meaning |
| --- | --- |
| 1 | The driver's name. Put it in double quotes if it has spaces. At most 30 characters are kept. |
| 2 | Optional: the driver's nationality, a number below `0x28` (40). |
| 3 | Optional: the driver's car, a number below `0x22` (34). |

A number that is too large is ignored; a missing word ends the entry. Drivers without a line
keep the game's own names.

```
name000="Ann Example" 3 12
name001=Bob
```

Code: `src/settings/read_settings.rs` (the keys of `var/ral.cfg` and where each goes),
`rc2k_core/src/files/settings.rs` (the text format and the number parser, with tests),
`src/settings/read_driver_names.rs`.
