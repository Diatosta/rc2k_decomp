# Data formats

*Category: Data formats*

Every file Rally Championship 2000 reads or writes, with a page for each format. The pages
describe the files closely enough to write a tool that reads one, and one that writes one the
game accepts.

Conventions on all of these pages:

- Numbers in the files are little-endian unless a page says otherwise.
- Offsets and sizes are in hexadecimal (`0x430`), with the decimal value in brackets where it
  helps.
- "u8", "u16", "u32" are unsigned numbers of 8, 16 and 32 bits; "i32" is signed; "f32" is a
  32-bit float.
- Paths are relative to the game's folder (the one that holds `files` and `var`). The game
  writes them with backslashes and matches names without regard to case, as Windows does; on
  Linux and macOS the program does that matching itself. On disc most names are in upper case.
- Examples use made-up values. No data of the game itself is reproduced.

Words such as "stage", "block" and "step" are defined in the [Glossary](Glossary).

## Files the game writes and reads back

| Page | Files | What they hold |
| --- | --- | --- |
| [Replay files](Replay-files) | `var/tmprpl`, `savedata/replay/replayNN`, `files/replay/replayNN` | The recording of a race. |
| [Saved games](Saved-games) | `savedata/savegame/svgameNN` | A championship in progress. |
| [Car set-ups](Car-setups) | `savedata/carsetup/carsetNN` | A saved car set-up. |
| [Screenshots](Screenshots) | `savedata/snapshot/NNN.png`, `NNN.png` in the game's folder | Pictures (PNG) taken in the replay viewer and in a race. |
| [Options files](Options-files) | `var/rc2k.toml`, `var/ral.dat` | The player's options: this version's file and the original's. |
| [Save code](Save-code) | (text and a QR code) | The player's state as text, from the options menu. |
| [Best times](Best-times) | `var/hst.dat` | Best times and scores. |
| [Settings files](Settings-files) | `var/ral.cfg`, `var/name.cfg` | Switches as `key=value` lines; the computer drivers' names. |
| [Install records](Install-records) | `var/install.cfg`, `var/ral.zog`, `lang.cfg`, `files/d/gfx/sku` | What was installed and in which language. |
| [Log files](Log-files) | `var/logfile.txt`, `rc2k.log`, `var/imgui.ini` | The game's log, the program's log, the debug window's layout. |

## Files the game only reads

| Page | Files | What they hold |
| --- | --- | --- |
| [Packed files (SQ)](Packed-files-SQ) | `*.SQ` | The packing used by course data and maps: the container only. |
| [Course files](Course-files) | `files/levels/LEVELnn/CDAT00C.SQ`, `LOCALDAT` | A stage's course and its texture lists. |
| [Map files](Map-files) | `files/maps/MAPnn.SQ` | The in-race map. |
| [Object files](Object-files) | `files/a/OBJnnnn.RO3`, `.LWO`, and those in `files/b`, `files/s/extr`, `files/g/frnt` | 3D shapes: scenery, cars, front-end models. |
| [Object and texture tables](Object-and-texture-tables) | `OBJSINFO`, `CARSINFO`, `EXTRINFO`, `FRNTINFO`, `TEXTINFO` | Records per object kind and per texture. |
| [Textures and pictures (PCX)](Textures-PCX) | `*.PCX` | Textures, sprites, masks, some screens. |
| [Screens (PNG)](Screens-PNG) | `files/g/graphics/screens/*.png`, `files/g/slides/*.png` | Menu backgrounds, stage maps, car specification pages, the cars' slideshows. |
| [Fonts](Fonts) | `files/d/fnt/FNTnn_nn.PCX`, `PAL_00.PCX`, `files/d/gfx/JADE.BUN` | Glyph sheets, their palette, and the error screen's font. |
| [Mud files](Mud-files) | `files/b/MUDFILE.nnn`, `files/m/MUDFILE` | Where mud gathers on each car. |
| [Effects data](Effects-data) | `files/m/MARKINFO`, `SPLINE.DAT`, `MASKnn.RAW`, `TABLE` | Weather, spray and windscreen effects. |
| [Sound samples](Sound-samples) | `files/d/snd/**.wav`, `files/speech/**.wav` | Effects and the co-driver's and announcer's speech. |
| [Music](Music) | `files/music/NN.wav` | The streamed tunes. |
| [Films](Films) | `files/fmv/INTRO.MPG`, `MFLOGO.MPG`, `ALLOGO.AVI` | The intro and the two logos. |
| [Language files](Language-files) | `files/locale/*.loc`, `*.kim`, `*.txt` | The language's set-up, keyboard layout and texts. |
| [Stage times data](Stage-times-data) | `files/s/time/TIMES.0`, `TIMES.TXT` | The computer drivers' stage times. |
| [Installer data](Installer-data) | `files/d/gfx/INSTALL.BUN`, `INSTALL.PAL`, `BACKDROP.PCX` | The installer's pictures, and what it copies. |

## Not files

| Page | What it covers |
| --- | --- |
| [Network messages](Network-messages) | What a network game sends between machines: this project's own protocol. |
| [Built-in tables](Built-in-tables) | Data that was inside the original program and is now Rust source: where each table lives. |

## How the game finds a file

A path in the code such as `files\d\snd\car\horn.wav` is turned into a real path in three
steps:

1. A leading `;3` or `;4` is a marker for the original's CD-or-disk choice. It is removed.
   (Every file is on disk in this version.)
2. Each `%` is replaced by the language's folder prefix: the `language=` line of the install
   record (see [Install records](Install-records)). A record without that line gives an empty
   prefix, so `files\%speech\m` is `files/speech/m`.
3. The result is looked up under the game's folder, matching each part without regard to case.

Paths that start `var\` or `save` are used as they are.

Two endings are swapped when the file is missing: a `.png` that does not exist is looked for
as `.pcx`, and an `.avi` as `.mpg`.

Code: `src/files/resolve_path.rs`, `rc2k_core/src/files/paths.rs`,
`src/files/swap_missing_extension.rs`.
