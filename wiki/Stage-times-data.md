# Stage times data

*Category: [Data formats](Data-formats)*

The folder `files/s/time/` of Rally Championship 2000 holds two files about stage times:

| File | Size | What it looks like |
| --- | --- | --- |
| `files/s/time/TIMES.TXT` | `0x2800` (10240) | Text. |
| `files/s/time/TIMES.0` | `0x3CC` (972) | 243 32-bit numbers. |

**The game does not read either file.** No part of the program opens anything in this
folder. They appear to be working data from the game's making, left on the CD.

The times that the game does use come from elsewhere:

- The computer drivers' times for a stage are worked out from figures in the stage tables
  built into the program (a time for each of four car classes per stage), not from a file.
- The best times a player sets are kept in `var/hst.dat`: see [Best times](Best-times).

So changing these files changes nothing in the game. The rest of this page describes their
shape, for the curious. File names are matched without regard to case.

## TIMES.TXT

Plain text with Windows line ends (carriage return, line feed), one line for each of the 36
rally stages, then zero bytes up to the file's fixed length of `0x2800`.

A line is the letter `L` and a two-digit course number (the `nn` of `LEVELnn`, see
[Course files](Course-files)), followed by a handful of three-digit decimal numbers (between 4 and 16 of them),
each with a space before it:

```
L99 041 038 052 047 060 044 039 051 048
```

(a made-up line). What the numbers measure is not known. Their count differs from stage to
stage, which suggests one figure per section of the stage.

## TIMES.0

A list of little-endian 32-bit numbers with no header. The value `0xFFFFFFFF` separates
groups; a group begins with a small number followed by larger ones that rise through the
group, in the manner of running times at successive points of a stage. What exactly the
numbers are is not known, and nothing in the program gives a clue, since it never reads
them.

Code: none. For where the computer drivers' times come from, see
`src/records/stages_store.rs` and `src/menu/calc_cpu_times.rs`.
