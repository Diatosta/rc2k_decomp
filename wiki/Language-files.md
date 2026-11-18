# Language files

*Category: [Data formats](Data-formats)*

Everything that depends on the language is in three small files in `files/locale/`: the
texts, a short set-up record, and the keyboard layout used when typing names. This page
describes all three.

Numbers are little-endian. Text is one byte a character, in the code page of the language
(Windows-1252 for the western European versions).

## The files

| File | Size | What it is |
| --- | --- | --- |
| `files/locale/*.txt` | about 80 kB | Every text the game shows: a plain text file, one message a line. |
| `files/locale/*.loc` | `0x30` (48) bytes | The language's set-up: its name, its units. |
| `files/locale/*.kim` | up to `0x400` (1024) bytes | The keyboard layout. Optional. |

The files are named after the language, but the game does not care about the name: it takes
the **first** file with each ending that it finds in the folder. An installation has one
language, so one file of each.

The path is written `files\%locale\` in the code; the `%` is explained on
[Data formats](Data-formats).

## The texts: `.txt`

The game calls this file the lexicon. At every start it is read and "compiled" into a
compact table in memory; nothing is written back.

### Lines

Each line is one message:

```
label<tab or spaces>text of the message        ; comment
```

| Part | Rule |
| --- | --- |
| Label | Any run of characters without blanks. It must start in the first column: a line that begins with a blank is an error. The label is for the people editing the file; the game never looks a message up by it. |
| Blanks | One or more tabs or spaces. |
| Text | To the end of the line, or to a `;`. Blanks at the end are dropped. |
| Comment | From `;` to the end of the line. A line that starts with `;` is all comment. |
| Empty lines | Skipped. |

Lines end with CR LF or LF.

**Messages are known by their position.** The first message in the file is number 0, the
next number 1, and so on; the code asks for texts by these numbers. The file must hold
exactly `0x757` (1879) messages. One more or one fewer and the game stops at start-up with
an error. So a translation changes the texts but must keep every line in its place.

### Special characters in a text

| Written | Becomes |
| --- | --- |
| `\n` | A new line. |
| `\s` | A space (one that is kept, for example at the start or end of a message). |
| `\t` | A tab. |
| `\1` .. `\9` | The byte 1 .. 9. In drawn text these are small moves to the left (see [Fonts](Fonts)). |
| `\#hh` | The byte with the hexadecimal value `hh`. |
| `\` and any other character | That character itself: `\;` is a semicolon, `\\` a backslash, `\{` a brace. |
| `{ ... }` | Everything between the braces as one piece, blanks included. `{}` alone is an empty message. A group must close on its own line and cannot hold another `{` or a bare `;`. |
| `~ddd` | Three decimal digits: extended glyph number `ddd`, for languages with more characters than one byte can hold. It becomes the two bytes of `0x1001` + 2 × `ddd`, high byte first. |

Runs of blanks between words are stored as a single space. To keep several, or to keep
blanks at the edges, use `\s` or braces.

### Limits

| Limit | Value |
| --- | --- |
| Messages | exactly `0x757` |
| A word (a run of letters and digits, or one group) | at most `0x7F` (127) bytes as written |
| A message | fewer than `0x80` (128) pieces, counting each word, each punctuation mark and each space between them |
| The compiled table | `0xFFD0` (65,488) bytes |

The compiled table stores each different word once and each message as a list of word
numbers, so a long file with much repeated wording still fits.

### Messages the code reads for itself

A few messages are not shown but used as data. Two matter to a translator:

| Number | Content |
| --- | --- |
| 2 | The language's letters that have an upper and a lower case and are not A .. Z, as pairs: lower, upper, lower, upper ... Its length must be even. |
| 3 | For sorting and comparing names: each plain letter followed by its accented forms, which are then treated as that plain letter. |

## The set-up record: `.loc`

`0x30` bytes, always.

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x00` | 4 | `LOCL`. Anything else stops the game with an error. |
| `0x04` | 4 | Not read. |
| `0x08` | 1 | The version's kind, which chooses the notice shown at start-up. `7` marks the Japanese version: it gets a larger memory for its fonts, and on a system set to Japanese only a kind 7 file is accepted. The other values' meanings are not known in detail. |
| `0x09` | 6 | Not read by the game. The last three appear to be the date, decimal and thousands separators as characters. |
| `0x0F` | 1 | Units: `0` imperial (miles), `1` metric (kilometres). The game starts with these units; the options screen can change them. |
| `0x10` | `0x20` | The language's name as text, padded with zero bytes. Not read by the game. |

A missing `.loc` file also stops the game with an error.

## The keyboard layout: `.kim`

It maps keys to characters, but names and chat no longer use it: they take the characters SDL's
text input reports ([Input](Input#text-entry)). The layout is still read for the yes/no
questions' letters and the names of keys on the key pages. Without a `.kim` file a built-in
layout is used.

The file is read into a block of `0x400` bytes; a shorter file simply ends early.

### Head

| Offset | Size | Meaning |
| --- | --- | --- |
| `0x00` | 4 | `KEYB`. Anything else stops the game with an error. |
| `0x08` | 1 | The keyboard's kind. Only `0x0C` has an effect: three entries of the key names table are then replaced by fixed message numbers. |
| `0x0A` | 1 | Layout mode. `2`: the file has two layouts and the player switches between them with Alt and a Shift key (left Shift the first, right Shift the second). |
| `0x0B` | 1 | Which layout is in use at the start: `0` or `1`. |
| `0x10` | `0x20` | The layout's name as text. Not read. |
| `0x30` | `0x14` | Layout 0: five u32 offsets (below). |
| `0x44` | `0x14` | Layout 1: five u32 offsets. All zero if there is only one layout. |

Each layout is five offsets, **counted from `0x30`**, not from the start of the file. An
offset of 0 means "no such table".

| Offset number | Table |
| --- | --- |
| 0 | The dead keys list. |
| 1 | Characters without Shift. |
| 2 | Characters with Shift. |
| 3 | Characters with AltGr. |
| 4 | Key names. |

### The tables

Keys are numbered by their keyboard scan code, `0x00` .. `0x7F` (`0x02` is the key marked
"1" on most keyboards, `0x10` is "Q" on a QWERTY board, `0x39` the space bar).

**Character tables** (offsets 1, 2, 3): `0x80` bytes each. Byte n is the character key n
types, or 0 for none.

**Key names** (offset 4): `0x80` u16 values. Value n is the number of the message (in the
`.txt` file) that names key n, as shown on the controls screen; 0 for a key with no name.

**Dead keys** (offset 0): entries of three bytes, ended by an entry whose first byte is 0.

| Byte | Meaning |
| --- | --- |
| 0 | The key's scan code. |
| 1 | The accent it starts when pressed without Shift, as a small number; 0 for none. |
| 2 | The accent it starts with Shift. |

A dead key types nothing by itself. The next key pressed is looked up together with the
accent in a list built into the program (accent, plain letter → accented letter), and that
character is typed. The accent numbers are the ones that list uses (small numbers from 1
up).

### A made-up example

A layout file with one layout, no dead keys and no AltGr table:

```
0x000  4B 45 59 42  00 00 00 00  01 00 00 00  00 00 00 00    "KEYB", kind 1, mode 0, layout 0
0x010  "Example", zeros to 0x30
0x030  00 00 00 00                                             no dead keys
0x034  28 00 00 00                                             plain table at 0x30 + 0x28 = 0x58
0x038  A8 00 00 00                                             shifted table at 0xD8
0x03C  00 00 00 00                                             no AltGr table
0x040  28 01 00 00                                             key names at 0x158
0x044  zeros (no second layout)
0x058  0x80 bytes: plain characters by scan code
0x0D8  0x80 bytes: shifted characters
0x158  0x100 bytes: 0x80 message numbers
```

The file is `0x258` (600) bytes long.

## The game's own texts: `.rc2k.toml`

The pages this version adds to the original's (the controller axes page, the extra rows of the
controller buttons page, the H shifter's switch, the button icons' and engine vibration settings and the hints beside the pictures) have texts that the language's `.txt` does not
hold. They are in a file of their own for each language, in the same folder as the language's
files, named after its `.loc` file with `.rc2k.toml` in place of the ending: `PORTUGES.LOC`
goes with `portuges.rc2k.toml`, `ENGLISH.LOC` with `english.rc2k.toml`. The repository has them
in `assets/locale/`; English is also built into the program.

The game reads the file at start, after the language's own files. A missing file, or a text the
file does not have, is shown in English (a missing file is noted in `rc2k.log`); a key the game
does not know is noted there and ignored; a file that is not UTF-8 TOML text is passed over as
a whole, the texts shown in English and the reason (the line and what is wrong) noted in
`rc2k.log`.

The file is TOML in UTF-8, a `key = "text"` for each text; a key's dots stand for tables, so
`axis_page.dead_zone = "dead zone"` may also be written `dead_zone = "dead zone"` under
`[axis_page]`. The texts are converted to Windows-1252, the code page of the game's own. The
built-in English file, `assets/locale/english.rc2k.toml`, lists every key with its comment.
**To add a language**, copy it, translate the texts and name it after the language's `.loc`
file; no code changes.

Where the files go:

- the builds (`./build.sh`) put every language's file in `dist/<target>/locale/`, beside the
  program;
- the installer copies the one of the language chosen from there into the installed
  `files/locale`, as it copies the language's own files (a download folder that already has the
  file in its language folder gets it copied with that folder);
- `./build.sh <target> --deploy` copies the one of the language already installed in the game
  folder it deploys to.

## Code

`src/strings/load_language.rs` (the order of loading, the `.loc` and `.kim` checks),
`src/strings/compile_lexicon.rs`, `rc2k_core/src/strings/lexicon.rs` (the `.txt` compiler),
`rc2k_core/src/strings/language.rs` (the `.loc` record), `rc2k_core/src/strings/own.rs` and
`src/strings/own_texts.rs` (the `.rc2k.toml` files), `src/install/copy_own_texts.rs` (the
installer's copy),
`rc2k_core/src/input/layouts.rs` (the `.kim` file), `rc2k_core/src/input/keyboard.rs` (keys
to characters, dead keys), `rc2k_core/src/strings/table.rs` (reading the compiled table).
