# Save code

*Category: [Data formats](Data-formats)*

The player's state as one piece of text, and as a QR code of it: for moving a player to another
machine, keeping a copy, or sending it to a friend. It is made and read on the **Save code** page
of the options menu (below "advanced"; the options of the pause menu in a race have no such page), and in the source by `rc2k_core::saves::code`.

## The page

- **Left**, the code as text, in the menus' font, from its first letter. Page Up and Page Down scroll it
  by a page (on a pad, the left and right shoulder buttons), and a pad's right stick up and down line by
  line, repeating while it is held, when it is longer than the screen is high (a code with saved games in it runs to thousands of letters,
  which no smaller font would make readable). Its length is under it.
- **Right**, the QR code: black on white, a module a whole number of pixels wide (as large as the
  screen allows), with the four modules' quiet zone. A code that one QR code cannot hold is shown as
  numbered **parts**, one at a time, "2 / 3" at the top right; Left and Right step through them, round
  the ends (the text on the left stays the whole code). Under the text a line says which it is.
- **copy** puts the whole text in the system's clipboard. **paste** reads the clipboard. **camera**
  opens a camera (SDL's camera API) and looks for a QR code shown on another screen, with a small live
  picture of what the camera sees; Back (Escape) stops. The camera button is dimmed when SDL sees no
  camera; the copy and paste buttons are dimmed on the handheld, whose console build has no clipboard.
  The buttons are chosen with Up and Down (and Left and Right when the code is one QR code).
- **The camera's pictures** are opened in the best uncompressed format the camera lists (RGB, packed
  YUV such as YUY2 and UYVY, planar NV12 or I420; MJPEG only when nothing else is offered, as SDL decodes it
  itself), at the size nearest 1280 x 720 that gives at least 10 frames a second (a QR code on another
  screen fills about half the picture's height, and its modules need about four pixels each; a 640 x 480
  picture reads only a code held close). Every frame is turned into RGBA by the game's own conversion
  (`rc2k_core::gfx::camera_frame`, reading each row at the frame's own pitch; a format it does not read goes
  through SDL's pixel conversion) and shown as a real picture, **all of it**, in the frame's own proportions,
  scaled to the box by averaging (no crop, no mosaic). A camera that faces the user (a front camera, a
  laptop's webcam) is shown mirrored, as in a mirror.
- **Looking for the code** happens on a thread of its own, one frame at a time (a large picture takes
  tenths of a second), so the picture keeps its pace. The frame is read as the camera sent it; when that
  gives nothing it is read mirrored left to right, then with light and dark swapped, so a code seen in a
  mirror or on a dark-on-light screen reads too. A code turned (even upside down) or slightly skewed reads.
  The reader is sensitive to the picture: what does not read in one frame often reads in the next, which
  differs a little.
- **Several cameras**: the page names the camera in use ("Camera 1 / 2: name") and, when there are more,
  says "Left/Right: another camera" (the keyboard's Left and Right, a pad's D-pad, and Tab or a pad's
  shoulders go to the next one). The back-facing cameras of a phone come first and are the first one
  opened; user-facing ones last. The next camera is opened with the same choice of format, and the picture
  and the reading go on at once.
- **The camera reads parts in any order.** The page says "2 of 3 read" and which ("Read: 1, 3");
  when the last is in, the confirmation follows. A part of another code (its check number differs) is
  not taken and the page says so; a part shown again is ignored. Paste takes the whole text only (a
  part's text alone is refused with a message that says it is one part).
- A code that comes in (pasted or seen) is **decoded and checked whole before anything else happens**.
  A bad one gives a line under the buttons ("That is not a usable save code: ...") and nothing is
  changed. A good one opens a question: what the code holds (when it was made, by which program
  version, how many best times and car set-ups, which saved games) beside what is here, and "yes,
  replace" or "no, keep" (no is chosen first; Escape is no). Yes writes the state the way the game
  saves it (the best times into its tables and `var/hst.dat`, the set-up and saved game files, and the
  game's lists of their names read again), so the game goes on with it at once.

## What it holds

In this order in the code:

| Section | In the code | Replaces |
| --- | --- | --- |
| Saved games | Each slot's name and state (29,276 bytes, rearranged for packing: below) | The same slots; other slots stay |
| Car set-ups | Each slot's name and six values | The same slots; other slots stay |
| Best times | Only the entries a player made (below), with the tyre change record | The importing game's players' entries; its made-up ones stay |

**The settings are not in it** (the owner's decision: a code must not override them). An import never
changes the options or `var/rc2k.toml`. The rally times and stage records, which the game keeps in its
options file, are ranked in like the other best times; the file is saved with its own settings as they
are. Not in it either: the replays (they are files), and what belongs to the machine.

A section that is not in the code is left as it is.

### Saved games

A saved game's state is a 28-byte head (the event's numbers), the four players' cars (0x28C bytes each)
and 180 driver records of 0x94 bytes, nine tenths of it. The records are written **column after column**
(every record's byte 0, then every record's byte 1, and so on), the first four bytes (a driver number
that counts up) as the difference to the record above. That is a pure rearrangement: the same bytes come
back, and deflate finds long runs where a field hardly changes from driver to driver. A saved game
becomes about 45 % shorter this way. The rest of the file (the head with its time, and the zero padding
up to 0x8000 bytes) is rebuilt by the importing game.

### Best times

The game fills its lists with made-up entries (a driver of its own and a time to match). They are not
carried. An entry is the player's when its player number is not 0, or its name is none of the 180
drivers' names (the best scores clear the number once shown). The importing game ranks those entries
into its own lists as a new best time is ranked (its fields, its ordering: lower times first, the
levels' scores highest first, a tie going above the old entry where the game moves a new entry up past
equals), over its made-up entries, and drops its own players' entries from them. The rally times in
the options file are done alike, and the stage records held by a name that is one of the four players'
(only those, to the same stage: its place, tag and course must match).

## The text

`RC2K:`, the frame in base 45, `:`. Base 45 (RFC 9285) is used because its letters are the QR code's
alphanumeric characters, which a QR code holds at 5.5 bits each: version 40 at the lowest error
correction holds 4,296 of them, against 2,953 bytes for base 64 text in byte mode. Line breaks and
tabs inside the text are ignored, so a wrapped paste reads; spaces are letters.

The frame is, in bytes: the version (2); flags (0); the CRC-32 (the zip and PNG one) of the version, the
flags and the rest, little-endian, 4 bytes; the payload, raw deflate (level 10, `miniz_oxide`). The
payload, before deflating, is:

| Part | Layout |
| --- | --- |
| Head | made (u32, seconds from 1970), program (length-first text, at most 32 bytes), sections (a byte: 2 best times, 4 set-ups, 8 saved games; 1 is not used) |
| Saved games | a count and, each, slot, 32-byte name, the state's 29,276 bytes rearranged as above |
| Set-ups | a count and, each, slot, 32-byte name, six values |
| Best times | a count and, each, table (0 .. 5: the scores, the stages' records, the levels, the courses, the other group's stage times, the time trial's), list, the entry's bytes (as in `var/hst.dat`); the rally times (rally, list, 156 bytes); the stage records (table, event, record number, 164 bytes); the tyre change's holder (a flag, 32 bytes, time) |

Numbers marked "count" or "list" are 7-bit groups, low first (a LEB128 number).

**Version 1** codes (the first release) still read: their payload began with the settings (a length-first
text, flag 1) and their saved games were not rearranged. The settings are read past and ignored.

### Parts

A text longer than 409 letters is cut into 2 to 99 pieces of nearly one length, each its own QR code
(small ones, of about 380 letters of the text, 57 to 61 modules a side: a camera at 1280 x 720 reads such
a code from a screen held at a normal distance, where one of the largest, 177 modules, never reads; only
a text that would need more than 99 such parts gets fewer, larger parts, up to 4,296 letters each):
`RC2P:`, the part's number and the count (two digits each, from 01), the check number of the whole text
(the CRC-32 of its letters, eight hex digits), `:`, the piece, `:`. The pieces joined in order give the
whole text, whose check number must be the one every part carries: parts of two different codes never mix,
a part that was read wrongly is caught, and the parts may be read in any order. More than 99 parts is
more than the page shows as QR codes (the text and clipboard still hold everything).

### Reading is checked

A code is untrusted input and is read into typed values, never into memory by layout: the frame's
version and flags; the check number before anything is unpacked; the unpacked size (at most 4 MiB, so
a bomb is refused); the payload to its last byte; counts, slots (under 100, none twice); names (ended,
no control characters); set-up values; every best time's player (0 to 4), nationality, group, car and
"new" mark; the best times' lists against the importing game's. The question is asked only when all of
it holds.

## What fits a QR code

Nothing is left out any more: what does not fit one QR code goes into more of them. Sizes measured, in
letters of the whole text, from 25 real saved games of a played championship (nothing else in the code):

| Saved games | Letters | QR codes | Of those letters in version 1's layout |
| --- | --- | --- | --- |
| 1 | 1,875 | 5 | 3,384 |
| 2 | 2,139 | 6 | 4,155 |
| 4 | 4,227 | 12 | 7,859 |
| 8 | 11,538 | 31 | 19,259 |
| 25 | 35,223 | 93 | 60,665 |

A player's best times add about 100 letters each, split times included (a hundred of them about 9,800
letters, about 26 more QR codes). A code with no saved games and no best times of its own is small (the car
set-ups and the tyre record).
