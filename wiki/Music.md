# Music

*Category: [Data formats](Data-formats)*

The music is a set of WAV files that the game plays straight from disk, a piece at a time,
instead of loading them whole. This page says what the files must be and how the game picks
one.

Numbers are little-endian.

## Where the files are

`files/music/NN.wav`, where `NN` is a two-digit tune number: `02.wav`, `03.wav` and so on up
to `15.wav`.

## Which tune plays when

The game asks for tunes by number. The path is always `files\music\` + two digits +
`.wav`.

| Tunes | Use |
| --- | --- |
| `02` .. `09` | Asked for by number at fixed moments: the menus, a stage's loading, results and so on. Each has a tune that follows it when it ends; for some that is the tune itself. |
| `10` .. `15` | The race tunes. They are never asked for by number: tune `08` is followed by "the next race tune", and the game goes round these six in turn, skipping any whose file is missing. |

When a tune ends, the tune that was named to follow it starts; a tune that follows itself
loops for ever. A tune's file being absent is not an error: nothing plays.

Because the tunes are found by number, a tune is replaced by a file of the same name. Fewer
race tunes can be had by deleting some of `10` .. `15`; more than six cannot be added
without changing the code.

## What the game accepts

The same WAV rules as for the effects: see [Sound samples](Sound-samples). In short: RIFF
`WAVE`, PCM, 1 or 2 channels, 8 or 16 bits.

One rule is special to music. Only the first `0xC00` (3072) bytes of the file are read to
find the format and the start of the samples. The `fmt ` chunk and the header of the `data`
chunk must both lie inside those bytes. A file with large chunks of text or pictures before
its samples (some programs write long `LIST` chunks) will not play; write the file with the
plain 44-byte header.

There is no limit on the length of a tune.

The game's own tunes are stereo, 16 bits, 22,050 Hz, with the samples starting at byte
`0x2C` (44).

## How it is played

The audio thread keeps a short looping sound buffer and, every 20 ms, tops it up from the
file with as many bytes as have been played since the last time. When the file runs out the
buffer is filled with silence, and when the silence has played the tune counts as ended.

The volume is the options' music volume; tune `08` and the race tunes after it have their
own volume and their own on / off switch.

## Code

`src/audio/stream.rs` (starting and feeding a stream), `rc2k_core/src/audio/streams.rs` (the
tunes' order and what follows what), `rc2k_core/src/sound/senders.rs` (which tune each
request means), `src/race/music.rs`.
