# Sound samples

*Category: [Data formats](Data-formats)*

Every sound effect and every piece of speech is an ordinary WAV file. This page says which
WAV files the game takes, where they live, and how a file is tied to a sound in the game. The
music is also WAV but is played differently: see [Music](Music).

Numbers are little-endian.

## Where the files are

| Folder | What is in it |
| --- | --- |
| `files/d/snd/ambient/`, `weather/` | Background and weather sounds of the stages. |
| `files/d/snd/car/`, `engine/` | The car's own noises and its engine, several files per make of car. |
| `files/d/snd/impacts/`, `surfaces/` | Hits, and tyre noise by road surface. |
| `files/d/snd/misc/` | Other sounds. |
| `files/speech/m/`, `files/speech/f/` | The co-driver, male and female voice. Each has the folders `comment`, `count`, `distance`, `general`, `instruct` and `suffix`, with the same file names in both voices. |
| `files/speech/cars/` | The announcer's line for each car. |
| `files/speech/rally/` | The announcer's lines for the menus, the rallies and the stages. |

In the code the speech paths are written `files\%speech\...`. The `%` is the language's
folder prefix (see [Data formats](Data-formats), "How the game finds a file"), which is how
a version with several languages keeps one set of speech for each.

## How a file is tied to a sound

The game knows its sounds by number. Each number has a record that names the file to load,
how loud it is, whether it loops, and how many copies of it can sound at once. The records
and the file names are built into the program (see [Built-in tables](Built-in-tables)); no
file on disk lists them.

So a sound is replaced by putting another WAV file under the same name. A new sound cannot be
added without changing the code.

The co-driver's calls are put together from several files of those folders, played one
after the other.

The announcer's speech in the menus (`files/speech/cars/`, `files/speech/rally/`) is not
loaded whole but played from disk as a stream, the way the [Music](Music) is. For those
files the size limit below does not apply, and the music's rule about the first `0xC00`
bytes does.

## What the game accepts

| Property | Accepted |
| --- | --- |
| Container | RIFF with the form `WAVE`. |
| Coding | PCM only (format tag 1). Anything else, including compressed WAV, is refused. |
| Channels | 1 or 2. |
| Bits a sample | 8 (unsigned) or 16 (signed). |
| Sample rate | 100 .. 200,000 Hz; values outside are pulled to the nearest limit. |
| Length | The data is cut down to a multiple of 4 bytes and must be at least 4 bytes. |
| File size | An effect is read whole into a work buffer, 1 MB unless `wavheapsize` in `var/ral.cfg` says otherwise (in kilobytes). A longer file is cut off there. |

The game's own effects and speech are all mono, 16 bits, at 22,050 or 44,100 Hz.

How the chunks are read:

1. Bytes 0 .. 3 must be `RIFF` and bytes 8 .. 11 `WAVE`.
2. From byte 12 on, chunks follow: a four-character name, a u32 length, then that many
   bytes. The game steps from chunk to chunk by exactly the length given; it does **not**
   skip the padding byte that the WAV standard puts after a chunk of odd length. Keep every
   chunk before `data` an even length.
3. `fmt `: the format tag (u16) must be 1; then channels (u16), sample rate (u32), two fields
   that are not read, and bits a sample (u16 at offset `0x0E` of the chunk's data).
4. `data`: the samples start here. The search stops at the first `data` chunk. Its length is
   used, or what is left of the file if that is less.
5. Other chunks before `data` are skipped. Chunks after `data` are never looked at.

A file that does not start with `RIFF` .. `WAVE` is not refused: all of it is taken as raw
samples, mono, 16 bits, 22,050 Hz.

Stereo samples are stored left then right for each moment, as usual.

## What happens to a sample after loading

- Its loudness is scaled by the record's volume factor.
- In the lower sound quality settings it is halved in rate (each pair of samples averaged),
  once or twice, and in the 8-bit mode it is cut to 8 bits. A file is never improved, so
  nothing is gained by a rate above 44,100 Hz.
- Whether a sample loops is the record's choice. A looping sample loops over its whole
  length; there are no loop points in the file. Make a looping sound's end meet its start.
  When it does not (the step from the last frame to the first is larger than 95 % of the
  steps inside the sample), the game blends the last 10 ms into the first 10 ms when it
  loads the sample and ends the sample where the blend begins, so the turn of the loop is
  not heard as a click; the sample is 10 ms shorter then. Several of the engine samples
  needed it.
- A looping engine sample (a file in `files/d/snd/engine/`) is not played as a plain loop
  in the sound modes where each sound has a buffer of its own (the Dolby mode's own mixer still loops plainly): at the pitch of high revs a sample about
  a second long comes round every few tenths of a second and the ear hears the same phrase
  again. It is played as two overlapping grains of about 0.1 s, a new one starting every
  0.05 s under a raised-cosine window, each at the voice's pitch. A new grain starts at a
  random place of the sample, moved a little to where its wave best matches what the grain
  before goes on to play, so the engine's tone is the same as the plain loop's and only the
  order its stretches come in varies. This costs about half a percent of one core a voice.
- The pitch of engine and tyre sounds is changed while they play; the file's rate is only the
  starting point.

## A made-up example

The 44 bytes that begin a mono, 16-bit, 22,050 Hz file with 1000 samples (2000 bytes of
data):

```
52 49 46 46  F4 07 00 00  57 41 56 45            "RIFF", 2036, "WAVE"
66 6D 74 20  10 00 00 00                          "fmt ", 16
01 00  01 00  22 56 00 00  44 AC 00 00  02 00  10 00
                                                  PCM, 1 channel, 22050 Hz, 44100 bytes/s, 2 bytes a frame, 16 bits
64 61 74 61  D0 07 00 00                          "data", 2000
```

## Code

`rc2k_core/src/audio/samples.rs` (reading a file, preparing a sample),
`src/audio/samples.rs`, `src/audio/load_sounds.rs` (loading a list of sounds),
`src/audio/names_store.rs` (the file names), `src/audio/voices.rs` (the records),
`src/sound/` (what the game asks to be played).
