# Module: audio

*Category: [Module reference](Modules)*

The audio thread: the side that carries out what the game asks for. The game thread never
touches a sound buffer itself; it queues commands (see [`sound`](Module-sound)) and this module
takes them, about fifty times a second, and plays, moves and stops sounds. It also loads the
sample files, feeds the music streams and mixes the channels the game mixes itself.

## State

The owner is `Audio` (`src/audio/owner.rs`). What the audio thread shares with the game thread
is `AudioShared` in `src/audio/store.rs`; the sample records are in `src/audio/sample_records.rs` (their starting table in `sample_table.rs`). The bytes of the
loaded samples are kept by `src/audio/sample_data.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `audio_thread` | `src/audio/thread.rs` | The thread's routine: opens the sound device, then loops until the game quits. |
| `take_commands` | `src/audio/take_commands.rs` | Takes the commands the game queued and carries each out. |
| `update_voices` | `src/audio/update_voices.rs` | Each turn: updates every playing voice (volume, pitch, place). |
| `load_sounds` | `src/audio/load_sounds.rs` | Loads a list of sample files. |
| `read_sample` | `src/audio/samples.rs` | Reads one sample file and prepares it for playing. |
| `start_stream`, `feed_stream`, `stop_stream` | `src/audio/stream.rs` | Music and speech played from a file piece by piece. |
| `mix_channels`, `write_mix` | `src/audio/mixer.rs` | The game's own mixer: channels mixed into one buffer. |
| `set_mode` | `src/audio/set_mode.rs` | Switches the sound mode (stereo, 3D, the surround mode). |
| `open_sound`, `close_sound` | `src/audio/sound_device.rs` | Opens and closes the sound device. |
| `close_all` | `src/audio/close_all.rs` | Stops and frees everything at the end. |

## How it is laid out

A flat folder, one function per file. Groups by name: `buffer_*.rs` and `make_buffer.rs` (sound
buffers), `mix*.rs` and `mixer*.rs` (the mixer), `*_voice*.rs` (voices), `open_*` / `release_*`
(optional 3D sound extensions the original supported; here they report "not present").
`voices.rs` holds the voice and channel records.

## Calls and callers

It plays through `platform::sound` (SDL underneath) and reads files through
[`files`](Module-files). Its commands come from [`sound`](Module-sound); `physics`, `race`,
`drive` and `menu` reach it only for a few queries. The thread is started by `threading`.

## Related pages

[Sound and music](Sound-and-music), [Threads and shared state](Threads-and-shared-state),
[Sound samples](Sound-samples), [Music](Music).
