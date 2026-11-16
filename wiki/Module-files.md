# Module: files

*Category: [Module reference](Modules)*

Everything about reading and writing files. The game names its files as the 1999 program did,
with backslashes and without regard to case; this module turns such a name into a real path on
whatever system it runs on, and offers the few ways the game reads a file: whole, packed, or
through a small buffered stream.

## State

The work buffers (a 64 KB scratch the loaders share, the open stream) are in
`src/files/scratch.rs` and `src/files/stream.rs`, held by the `System` owner (`System::files`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `resolve_path` | `src/files/resolve_path.rs` | Turns one of the game's file names into a full path. |
| `read_whole_file_into`, `read_whole_file_len` | `src/files/read_whole_file.rs` | Reads a file in one go. |
| `read_file_into` | `src/files/read_file.rs` | Reads part of a file into a buffer. |
| `write_file_from` | `src/files/write_file.rs` | Writes a buffer to a file. |
| `read_packed_file` | `src/files/read_packed_file.rs` | Reads and unpacks a packed (`.SQ`) file. |
| `read_level_file` | `src/files/read_level_file.rs` | Reads a file from the current stage's folder. |
| `open_stream`, `read_stream_block`, `read_stream_byte` | `src/files/open_stream.rs`, ... | A buffered file, as the picture readers use it. |
| `file_exists`, `file_size`, `copy_file` | files of the same names | Small queries and a copy. |
| `handle.rs` | `src/files/handle.rs` | Open files by number, over the standard library. |
| `search.rs` | `src/files/search.rs` | Listing a folder by a wildcard pattern (`first`, `next`, `close`). |
| `directories.rs` | `src/files/directories.rs` | Making, removing and changing folders. |
| `numbered_path` | `src/files/numbered_path.rs` | Builds names like `replay07` from a pattern and a number. |

## How it is laid out

A flat folder, one function per file. The rules that need no file system (path joining,
wildcards, the packed format) are in `rc2k_core/src/files/`.

## Calls and callers

It uses the standard library and `platform::os`. Nearly everything that loads data calls it:
[`install`](Module-install), [`race`](Module-race), [`gfx`](Module-gfx), [`saves`](Module-saves),
[`scene`](Module-scene).

## Related pages

[Data formats](Data-formats), [Packed files (SQ)](Packed-files-SQ).
