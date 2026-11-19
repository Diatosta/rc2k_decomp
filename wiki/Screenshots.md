# Screenshots

*Category: [Data formats](Data-formats)*

The game can save what is on the screen as a picture. All of its screenshots are PNG files with
24-bit colour, written by one routine. (The original wrote PCX files with the same names; this
version writes PNG, which any picture program opens. Old `.png` screenshots stay where they are
and are not counted.) This page says where they go and what the files hold. Reading PCX files
in general (textures, fonts) is on [Textures (PCX)](Textures-PCX).

## Where the files are

| File | Made by |
| --- | --- |
| `NNN.png` in the game's folder | The screenshot key (numeric keypad 0), in the menus and in a race. Only when `screensnapshot=1` is set in `var/ral.cfg` (see [Settings files](Settings-files)). |
| `savedata/snapshot/NNN.png` | The replay viewer's snapshot button. |
| `rpl/NNNNNN.png` | The replay viewer's picture sequence: one file for each picture drawn while it is switched on (a developers' feature; its keys work only with `wrreplayenable=1` in `var/ral.cfg`). |

`NNN` is a three-digit number. The screenshot key starts at `001` each time the program is
started, so a new session writes over the last one's files. The viewer's snapshots carry on
from the highest number already in the folder. Both go back to `000` after `999`.

In the web build every screenshot is also downloaded by the browser, under the file's name
(`NNN.png`), because the files in the browser's own storage are out of the player's reach.

The picture has the size of the screen mode the game is drawing in at that moment (640 × 480,
for example).

## On a phone

On iOS and Android a screenshot (the key's and the replay viewer's snapshot button; not the
replay's picture sequence) is also put into the phone's photo library once it is written, so the
Photos or Gallery app shows it. iOS adds it to the camera roll and asks for the add-only photo
permission the first time. Android (10 and later) puts it in the Gallery under Pictures / Rally
Championship (older versions skip it). The outcome is a line in the log; a failure never stops
the game. Code: `src/platform/ffi/photo_library.rs`, `tools/android/app/java/org/rc2k/rally/Photos.java`.

## The file

A PNG file of the screen's size, 8 bits for each of red, green and blue (colour type 2), no
alpha, no interlacing, every row stored unfiltered, the pixels deflated in one `IDAT` chunk
(`rc2k_core/src/gfx/png_write.rs`, with tests). There is no palette: the three colours of every
pixel are stored. The pixels are the ones the game's pixel conversion gives for the screen's
format, as the original's were (a 16-bit screen mode keeps the 5-6-5 rounding).

Code: `src/gfx/take_screenshot.rs`, `src/gfx/write_screen_pcx.rs`, `src/gfx/shots.rs`,
`rc2k_core/src/gfx/png_write.rs`,
`src/race/viewer_snapshot.rs`, `src/race/find_snapshot_number.rs`,
`src/race/replay_screenshot.rs`.
