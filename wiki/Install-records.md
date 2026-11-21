# Install records

*Category: [Data formats](Data-formats)*

A handful of small text files tell the game whether and how it is installed, which languages
its data has, and which edition of the game the data is. All of them use the text format of
`var/ral.cfg`, described under [Settings files](Settings-files#the-text-format): one
`key=value` to a line.

| File | Written by | Says |
| --- | --- | --- |
| `var/install.cfg` | this project's installer | That this folder is an installation. |
| `var/ral.zog` | the installer (the original's and this one) | Where the game's data is and how much of it was copied. |
| `lang.cfg` | nobody; it comes with the game's data | The languages on the disc. |
| `files/d/gfx/sku` | nobody; it comes with the game's data | The edition. |

Paths are relative to the folder the program runs in. For the installer that is the folder of
the CD's files, which is where `lang.cfg` matters.

## How the program tells an installation from a download folder

When it starts, the program looks at the folder it runs in:

| The folder has | The program |
| --- | --- |
| the game's data (`files/d/gfx/jade.bun`) and neither `var/ral.zog` nor `var/install.cfg` | takes it for the CD's files and opens the installer |
| `var/ral.zog` or `var/install.cfg` | takes it for an installation and starts the game |

So an existing installation of the original game, which has a `var/ral.zog`, is played as it
is.

## `var/install.cfg`

The 1999 game kept its installation status in the Windows registry. This project keeps it in a
file in the game's folder, so that an installed folder can be copied or moved and still be a
working installation, on any system.

```
; Rally Championship
; INSTALLATION STATUS (the standalone's, in place of the registry)

status=1
```

| Key | Value |
| --- | --- |
| `status` | A number, 0 .. 255. 1 means installed. |

The installer writes the file with CR LF line ends. When reading, spaces around the key, the
`=` and the number are allowed. The uninstaller removes the file with the rest of the game.

The original's registry entry also named the folder the game was installed from. That is not
kept: asked for it, the program answers with the folder it is running in.

## `var/ral.zog`

```
; Rally Championship
; INSTALLATION CONFIGURATION

cdrom=D:\
language=english
installation=3
```

| Key | Value |
| --- | --- |
| `cdrom` | A folder: where the files that were not copied to the hard disc are read from. In 1999 this was the CD drive; here it is the folder the game was installed from. |
| `language` | Optional. The name of the language's folder, which stands in for the `%` sign in the paths of language-dependent files. |
| `installation` | The kind of installation, 0 .. 3 (a value outside that is brought into it). It decides which files are looked for under `cdrom` and which in the game's folder: 3 means everything is in the game's folder; 2 is the original's large installation; lower values leave more on the CD. |

A folder name gets a `\` added if it does not end in `\` or `/`. Each must be shorter than
`0xFE` (254) characters.

This project's installer always makes a full installation and writes `installation=3`, so
`cdrom` is recorded but nothing is read from it afterwards. The lines are written in the order
shown, with CR LF line ends, after the two comment lines and an empty line. A key that is
missing leaves the game's built-in value.

## `lang.cfg`

Lists up to nine languages for the installer's language page.

```
lang1=English:english
lang2=Deutsch:german
```

| Key | Value |
| --- | --- |
| `lang1` .. `lang9` | `name:folder`: the language's name as the installer shows it, a colon, and the name of its folder in the game's data. |

A value must be shorter than `0x80` (128) characters. Entries without a colon, or that begin
with one, are dropped; the rest are numbered in the order of their keys. The folder name is
what ends up as `language` in `var/ral.zog`.

With no more than one language listed, the installer does not show its language page.

## `files/d/gfx/sku`

One key, `sku`, naming the edition of the game the data belongs to.

```
sku=dvd:2:1.4
```

| Part of the value | Meaning |
| --- | --- |
| `oem:` | Optional mark: an OEM edition. |
| `dvd:` | Optional mark: a DVD edition. The two marks may come in either order. |
| a number | The edition's number. |
| `:` and a number | Optional: a second number (one byte). |
| `.` and a number | Optional: a third number (one byte). |

Without the file, or without the key, the game keeps its built-in edition.

Code: `src/install/record.rs` (`var/install.cfg`, and the test for a download folder),
`src/install/read_record.rs`, `src/install/write_record.rs`, `src/install/read_languages.rs`,
`src/install/read_sku.rs`, `rc2k_core/src/install/config.rs` (the rules of the three older
files, with tests).
