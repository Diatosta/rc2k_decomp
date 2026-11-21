# Module: sound

*Category: [Module reference](Modules)*

The game's side of the sound. It decides *what* should be heard: a menu click, a car's engine
at these revs, gravel under the tyres, a spot sound beside the road, the music for this screen.
It does not play anything itself. It puts commands into a queue, and the audio thread
([`audio`](Module-audio)) carries them out.

## State

The owner is `Sound` (`src/sound/owner.rs`): the race's sound state (which voices each car
holds, the cue queues), the menus' sound state, and the part shared with the other threads
(the command queue). Parts are in `src/sound/store.rs` and `src/sound/sets_store.rs`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `send`, `send_and_wait` | `src/sound/senders.rs` | Queue a command for the audio thread. |
| `queue.rs` | `src/sound/queue.rs` | The command queue and its lock. |
| `play_sound_in`, `stop_sound` | `src/sound/play_sound.rs`, `src/sound/stop_sound.rs` | Play or stop a sample. |
| `play_menu_sound` | `src/sound/play_menu_sound.rs` | The menus' clicks. |
| `play_placed_sound` | `src/sound/play_placed_sound.rs` | A sound at a place in the world. |
| `start_stream`, `stop_stream`, `stop_music` | files of the same names | Music and speech streams. |
| `play_tune` | `src/sound/senders.rs` | Starts a piece of music by number. |
| `load_race_sounds`, `set_up_race_sounds` | files of the same names | The sounds a stage needs. |
| `race_sound_tick`, `race_turn` | files of the same names | The race's sounds, once per step and once per audio turn. |
| `car_sounds.rs`, `revs_sound.rs`, `surface_sound.rs` | `src/sound/` | A car's engine, speed and surface sounds. |
| `spot_voices.rs`, `play_spot_voices` | `src/sound/` | Sounds fixed beside the road, shared between the nearest listeners. |
| `set_music_volume`, `apply_settings` | files of the same names | Volumes and the sound options. |

## How it is laid out

Flat, one function per file. Groups: `car_*` and `*_sound.rs` (the cars), `*spot_voices*`
(road-side sounds), `play_tune_*` (music), `*_stream*` (streams), `play_cue_*` and
`drop_cue.rs` (queued cues such as the co-driver's calls).

## Calls and callers

It calls only [`audio`](Module-audio)'s shared state and `math`. It is called by
[`physics`](Module-physics) (the cars), [`menu`](Module-menu), [`race`](Module-race),
[`drive`](Module-drive) and [`input`](Module-input).

## Related pages

[Sound and music](Sound-and-music), [Sound samples](Sound-samples), [Music](Music).
