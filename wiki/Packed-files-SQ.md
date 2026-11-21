# Packed files (.SQ)

*Category: [Data formats](Data-formats)*

Two kinds of file in Rally Championship 2000 have the extension `.SQ`. They share the
extension and the first two letters of their marker, and nothing else:

| Marker | Files | What it is |
| --- | --- | --- |
| `SQSL` | `files/levels/LEVELnn/CDAT00C.SQ` | A packed file: a general container for data made of 32-bit numbers. This page. |
| `SQSM` | `files/maps/MAPnn.SQ` | The in-race map. Not packed at all: see [Map files](Map-files). |

This page describes the `SQSL` packing. What is inside a packed course file is on
[Course files](Course-files).

All numbers are little-endian. A "dword" is a 32-bit number. File names are matched without
regard to case.

## The idea

The packed data is treated as a list of dwords, never as bytes. Course data is full of zeros,
small numbers and values that repeat a little further on, so each dword is written as a short
code of bits: "zero", "the same as the dword *n* places back", or the number itself in as few
bits as it fits.

## Layout

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| `0x00` | 4 | text | `SQSL` |
| `0x04` | 4 | u32 | Not read. `0` in the game's files. |
| `0x08` | 4 | u32 | Not read. `0x00050000` in the game's files (the same number the map files carry as their version). |
| `0x0C` | 4 | u32 | The number of dwords the file unpacks to. The unpacked size in bytes is 4 times this. Must not be 0. |
| `0x10` | rest | dwords | The bit stream. |

The file's length is a multiple of 4.

## The bit stream

The stream is read one dword at a time, and within each dword from the highest bit (bit 31)
down to the lowest. A code may run across from one dword into the next.

Each code makes one dword of output. Read bits until one of these patterns is complete:

| Code (bits, in reading order) | Followed by | The dword it makes |
| --- | --- | --- |
| `0` | nothing | 0 |
| `10` | 8 bits, a number *n* | a copy of the output dword *n* + 1 places back (*n* = 0 is the one just written) |
| `1100` | 12 bits, a number *n* | a copy of the output dword *n* + 1 places back |
| `1101` | 17 bits | that number (0 .. `0x1FFFF`) |
| `1110` | 1 sign bit, then 27 bits | a signed 28-bit number: the 27 bits, with bits 27 .. 31 all set to the sign bit |
| `1111` | 32 bits | that number |

Numbers inside a code are written highest bit first.

Repeat until the count at `0x0C` is reached. Bits left over in the last dword are not used.
A few of the game's files have one whole spare dword after the end of the stream.

A copy must refer to a dword that has already been written; the game refuses a file whose
copy reaches back before the start of the output, or whose stream ends early.

## Unpacking, in short

```
out = []
repeat count times:
    if bit() == 0:            out.append(0)
    else if bit() == 0:       n = bits(8);  out.append(out[len(out) - 1 - n])
    else if bit() == 0:
        if bit() == 0:        n = bits(12); out.append(out[len(out) - 1 - n])
        else:                 out.append(bits(17))
    else if bit() == 0:       s = bit(); v = bits(27); out.append(v | 0xF8000000 if s else v)
    else:                     out.append(bits(32))
```

## Packing

Any stream that unpacks to the right dwords is a valid file; the game does not care which
codes were chosen. A simple packer takes each dword in turn and writes the first of these
that applies:

1. `0` if the dword is 0;
2. `10` and the distance if the same value was written at most `0x100` (256) dwords back;
3. `1100` and the distance if it was written at most `0x1000` (4096) dwords back;
4. `1101` if the value is below `0x20000`;
5. `1110` if the value fits 28 bits as a signed number (its top five bits are all the same);
6. `1111` otherwise.

Then pad the last dword of the stream with zero bits and write the header with the dword
count. Data whose length is not a multiple of 4 must be padded first.

## How the game reads one

The game reads the whole file into the start of a large buffer, moves it to the buffer's end,
and unpacks from there back into the start, so no second buffer is needed. A file that does
not open or does not begin with `SQSL` counts as "not there".

Packed files are a second choice. The game first looks for the unpacked file under a
developer's folder name (`srclvls\LEVELnn\CDAT00C` for a course); only when that is missing
does it read `files\levels\LEVELnn\CDAT00C.SQ` and unpack it. A modder can therefore test an
unpacked course by putting it at the first path, without packing it.

## A made-up example

Six dwords, `0x00000000 0x00000007 0x00000007 0x00000000 0xFFFFFFFE 0x12345678`, pack to:

| Dword | Code | Bits |
| --- | --- | --- |
| `0x00000000` | zero | `0` |
| `0x00000007` | 17-bit number | `1101` `00000000000000111` |
| `0x00000007` | copy, 1 back (*n* = 0) | `10` `00000000` |
| `0x00000000` | zero | `0` |
| `0xFFFFFFFE` | signed 28-bit: sign 1, low 27 bits `0x7FFFFFE` | `1110` `1` `111111111111111111111111110` |
| `0x12345678` | 32-bit number | `1111` `00010010001101000101011001111000` |

That is 101 bits, so the stream takes four dwords (the last padded with zeros), and the
file is:

```
53 51 53 4C  00 00 00 00  00 00 05 00  06 00 00 00    SQSL, 0, 0x50000, 6 dwords
00 1E 00 68  FF FF FF 77  B3 A2 91 78  00 00 00 C0    the stream
```

The first stream dword is `0x68001E00`: in bits, `0` `1101` `00000000000000111` `10`
`00000000`, which are the first three codes.

Code: `rc2k_core/src/files/packed.rs` (the unpacking), `src/files/read_packed_file.rs`,
`src/files/read_level_file.rs` (the choice between the unpacked and the packed file).
