# Installer data

*Category: [Data formats](Data-formats)*

The installer is part of the game's program and draws its own screens. It needs three small
files for that, and it has a built-in list of what to copy. This page describes both. The
files the installer **writes** (the install record) are on
[Install records](Install-records); how to use the installer is on
[Installing the game](Installing-the-game).

Numbers are little-endian.

## The installer's pictures

| File | Size | What it is |
| --- | --- | --- |
| `files/d/gfx/BACKDROP.PCX` | an 800 × 600 PCX | The background of every installer page. An ordinary picture: see [Textures and pictures (PCX)](Textures-PCX). |
| `files/d/gfx/INSTALL.PAL` | `0x300` (768) bytes | A palette for the progress bar. |
| `files/d/gfx/INSTALL.BUN` | under `0x3000` (12,288) bytes | The progress bar's picture, as a glyph table. |

All three must be present in the folder the installer runs from; a missing one stops it with
an error.

### `INSTALL.PAL`

256 colours of three bytes each: red, green, blue, each 0 .. 255. No header. Entry n is at
byte 3 × n.

### `INSTALL.BUN`

A glyph table with two glyphs, in the layout described on [Fonts](Fonts) ("A glyph table"):
two u32 offsets, then two records of run-length coded rows, one byte a pixel.

| Glyph | Content |
| --- | --- |
| 0 | The left half of the full progress bar, `0xC8` (200) pixels wide. |
| 1 | The right half, the same size. |

Each pixel byte is an entry of `INSTALL.PAL`. A glyph record stores widths and counts in
single bytes, so a glyph can be at most 255 pixels wide, which is why the 400-pixel bar is in
two halves.

The installer draws the two glyphs side by side at a fixed place on the 800 × 600 screen and
cuts them off on the right at the progress made so far: 4 pixels of bar for each percent. The
percentage is written beside it with the game's own font.

The file must be smaller than `0x3000` bytes; only that much is read.

## What the installer copies

The installer takes the game from the folder it is started in: the contents of the game's CD,
or of a download of it, with the program put beside them (this version does not ask for a
disc). It recognises such a folder by the game's data being there (`files/d/gfx/JADE.BUN`)
with no install record (`var/install.cfg` or `var/ral.zog`).

What is copied is decided by a short list built into the program, not by a file. In words:

| From the source folder | Copied |
| --- | --- |
| Top level | The original program file, and every `*.txt` and `*.pdf` (the readme and the manual). |
| `files/a`, `files/b`, `files/d`, `files/m`, `files/maps`, `files/levels`, `files/replay`, `files/s` | Whole, with everything under them. |
| `files/g` | `frnt`, `graphics` and `slides`, whole. |
| `files/fmv`, `files/music` | Whole. |
| `files/locale` | The chosen language's folder. |
| `files/speech` | `m`, `f`, `cars` and `rally`, for the chosen language. |
| `var` | `ral.cfg` only. |

It also makes the empty folders `savedata/carsetup`, `savedata/replay`, `savedata/savegame`
and `savedata/snapshot`, copies itself (the `rc2k` program and, on Windows, its DLLs) into
the new folder, and writes the install record.

The original installer offered smaller installations that left the films, music, slides and
announcer's speech on the CD. The list still marks those folders, but this version always
installs everything.

A source folder with several languages has a `lang.cfg` file naming them and, for each, a
folder prefix; `files/locale` and `files/speech` then exist once per language under that
prefix. See [Install records](Install-records) for `lang.cfg`.

### The list's coding

For the curious, the built-in list is a string of bytes read from left to right:

| Byte | Meaning |
| --- | --- |
| `01` name | Enter the folder `name` (make it in the destination). |
| `02` | Go back up one folder. |
| `03` pattern | Copy the files matching `pattern` (`*.txt`) in the current folder. |
| `04` name | Copy the folder `name` and everything under it. |
| `08` n | What follows, up to the matching `09`, only for an installation of kind n or larger. |
| `06`, `09` | Nothing. |
| anything else | The end. |

Names are plain text and end at the next byte below `0x20`. A `%` in a name is replaced by
the language's folder prefix. The uninstaller runs a second list of the same kind that names
what to remove (everything, or everything but `savedata` and `var`).

## Code

`src/install/open_installer.rs` (loading the three files), `src/install/install_frames.rs`
(the progress bar), `rc2k_core/src/install/script.rs` (the list's coding),
`src/install/run_script.rs`, `src/install/list_files.rs`, `src/install/install_files.rs`
(copying), `src/install/portable.rs` (recognising a source folder), `src/install/vars.rs`
(the built-in lists and file names).
