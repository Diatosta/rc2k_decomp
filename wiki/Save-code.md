# Save code

*Category: [Data formats](Data-formats)*

The player's state as one piece of text, and as a QR code of it: for moving a player to another
machine, keeping a copy, or sending it to a friend. It is made and read on the **Save code** page
of the options menu (below "advanced"; the options of the pause menu in a race have no such page), and in the source by `rc2k_core::saves::code`.

## The page

- **Left**, the code as text, in the menus' font, from its first letter. It scrolls with Up and Down
  when it is longer than the screen is high (a code with saved games in it runs to tens of thousands of
  letters, which no smaller font would make readable). Its length is at the top right.
- **Right**, the QR code: black on white, a module a whole number of pixels wide (as large as the
  screen allows), with the four modules' quiet zone. A line under the text says what it holds
  (below, "What fits a QR code").
- **copy** puts the text in the system's clipboard. **paste** reads the clipboard. **camera** opens a
  camera (SDL's camera API) and looks for a QR code shown on another screen, with a small live picture
  of what the camera sees; Back (Escape) stops. The camera button is dimmed when SDL sees no camera;
  the copy and paste buttons are dimmed on the handheld, whose console build has no clipboard.
- A code that comes in (pasted or seen) is **decoded and checked whole before anything else happens**.
  A bad one gives a line under the buttons ("That is not a usable save code: ...") and nothing is
  changed. A good one opens a question: what the code holds (when it was made, by which program
  version, the players' names, the runs, how many best times, car set-ups and which saved games) beside
  what is here, and "yes, replace" or "no, keep" (no is chosen first; Escape is no). Yes writes the
  state the way the game saves it (the options into the game and `var/rc2k.toml`, the best times into
  its tables and `var/hst.dat`, the set-up and saved game files, and the game's lists of their names
  read again), so the game goes on with it at once.

## What it holds

| Section | In the code | Replaces |
| --- | --- | --- |
| Settings | The options as the options file's text, only the keys that differ from a new install's | All the settings but those below |
| Best times | Only the entries a player made (below), with the tyre change record | The importing game's players' entries; its made-up ones stay |
| Car set-ups | Each slot's name and six values | The same slots; other slots stay |
| Saved games | Each slot's name and state (29,276 bytes) | The same slots; other slots stay |

Not in it: the replays (they are files), and what belongs to the machine: the screen modes, the
window and renderer choices, the network's name and addresses, the controllers and their bindings.
The importing machine keeps its own.

A section that is not in the code is left as it is (a QR code that had to leave the saved games out
does not touch the saved games here).

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

The frame is, in bytes: the version (1); flags (0); the CRC-32 (the zip and PNG one) of the version, the
flags and the rest, little-endian, 4 bytes; the payload, raw deflate (level 10, `miniz_oxide`). The
payload, before deflating, is:

| Part | Layout |
| --- | --- |
| Head | made (u32, seconds from 1970), program (length-first text, at most 32 bytes), sections (a byte: 1 settings, 2 best times, 4 set-ups, 8 saved games) |
| Settings | length-first UTF-8 text: TOML, `var/rc2k.toml`'s keys that differ from a new install's |
| Best times | a count and, each, table (0 .. 5: the scores, the stages' records, the levels, the courses, the other group's stage times, the time trial's), list, the entry's bytes (as in `var/hst.dat`); the rally times (rally, list, 156 bytes); the stage records (table, event, record number, 164 bytes); the tyre change's holder (a flag, 32 bytes, time) |
| Set-ups | a count and, each, slot, 32-byte name, six values |
| Saved games | a count and, each, slot, 32-byte name, the state's 29,276 bytes |

Numbers marked "count" or "list" are 7-bit groups, low first (a LEB128 number).

### Reading is checked

A code is untrusted input and is read into typed values, never into memory by layout: the frame's
version and flags; the check number before anything is unpacked; the unpacked size (at most 4 MiB, so
a bomb is refused); the payload to its last byte; counts, slots (under 100, none twice); names (ended,
no control characters); set-up values; every best time's player (0 to 4), nationality, group, car and
"new" mark; the settings (every key through the options' own reader, a value out of its range being a
refusal, not a default, the volumes within range, the names clean); the best times' lists against the
importing game's. The question is asked only when all of it holds.

## What fits a QR code

A QR code holds one text of up to 4,296 letters. What the text carries is cut down, in steps, until it
fits, and the page says which step it is on:

1. Everything.
2. Without the saved games (each takes about 3,500 letters: more than a QR code has room for beside
   anything else).
3. And the best times without their checkpoint lines (the sixteen split times; the bulk of an entry,
   numbers that do not pack): a time then has no splits to be compared with.
4. And without the rally times and stage records.
5. And without any best times: the settings and set-ups.

The text on the left is always the whole state, which may be longer than the QR code. Sizes measured
(letters of the whole text; `var` of a played install, with 2 car set-ups):

| State | Letters |
| --- | --- |
| A new install (default settings) | 75 |
| The played install's settings, set-ups and tyre record, no best times of its own | 1,569 |
| And 20 best times of a player | 3,699 (2,103 without split times) |
| And 100 | 11,325 (3,902 without split times) |
| And 300 | 29,859 (8,078 without split times: the QR leaves the best times out) |
| One saved game over the 1,569 | 5,036 |
| 25 saved games over the 1,569 | 62,837 |

A new install's state and a played one's settings fit one QR code; a player with up to about a hundred
best times fits it without split times; saved games never do beside the rest.
