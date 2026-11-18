# Sound and music

*Category: [Game flow](Game-flow)*

The game's sound is split in two. The game thread decides *what* should be heard and sends
short commands. A separate audio thread carries them out: it loads samples, starts and stops
voices, and keeps the music fed. This page follows a sound from the game's decision to the
loudspeaker, then covers 3D placing, music, the co-driver's speech and the sound modes.

| Folder | Side | Owner |
| --- | --- | --- |
| `src/sound/` | The game's side: which sound, how loud, where, when. | `Sound` |
| `src/audio/` | The audio thread: samples, voices, channels, streams. | `Audio` |
| `src/platform/sound.rs` | The output: a small mixer on one SDL audio stream. | |
| `rc2k_core/src/sound/`, `rc2k_core/src/audio/` | The rules and the command format, as plain functions. | |

## Commands

Everything the game wants from the audio thread is a `Command`
(`rc2k_core/src/audio/commands.rs`):

| Command | Meaning |
| --- | --- |
| Start | Start, change or stop one sound: its number, volume, pan or place, pitch. |
| Room | Choose the reverb room (for the EAX mode). |
| Load | Load a list of sounds. |
| Free | Give every voice back. |
| Mode | Change the sound mode (menus and races use different ones). |
| StopAll | Stop every playing voice. |
| Volume | The master volume. |
| Stream, Music | Start, stop or change one of the two streamed players. |
| EndOfTurn | "That is all for now." |

The game side queues commands under a lock (`src/sound/queue.rs`); the audio thread takes them
oldest first (`src/audio/take_commands.rs`). When the game must be sure something has
happened, for instance that a stage's sounds are loaded before the race starts, it calls
`sound::wait_for_commands`: it queues an EndOfTurn and sleeps until the audio thread has taken
every command and finished that turn. The audio thread is told that someone waits and takes the
queue at once instead of at its next turn, so a wait lasts as long as the commands' work (a
stage's sounds to load, say) and no longer. Every 10 ms of a long wait the window's events are
handled. Commands sent without a wait (a race's sounds) are taken at the audio thread's own
pace.

The simplest sender is `sound::play_sound`: one Start command. `sound::play_menu_sound` picks
a menu's click or confirm sound and sends it.

## The audio thread

`audio_thread` (`src/audio/thread.rs`) is started with the game's other threads. It opens the
sound device (`audio::start`), then takes a **turn** every 20 ms until the game quits (sooner
when the game waits for its commands, see above):

1. feed the music streams (`src/audio/stream.rs`);
2. take the queued commands;
3. update the voices (`src/audio/update_voices.rs`).

The 1999 game also restored sound buffers that Windows had taken away for another program.
That cannot happen here; a buffer that cannot be played is noted in `rc2k.log` and stays silent.

### Samples, voices and channels

These three words are the heart of `src/audio/voices.rs`:

- A **sample record** describes one sound: its file, its base volume and rate, how many copies
  may sound at once, its priority. A sound number picks a record.
- A **voice** is one loaded, playable copy of a sample. A sound that can overlap itself (a
  gravel spray, say) has several voices that share the same sample data.
- A **channel** is one of a small number of places in the output. A voice needs a channel to
  be heard. When all are in use, a new sound takes the channel of the playing voice with the
  lowest priority below its own, or is not played.

A Start command (`src/audio/start_voice.rs`) finds a free voice of the sound, gets it a
channel and puts it on the playing list. It does not touch the output itself: it marks what
changed (volume, pan, frequency, place, play, restart), and the turn's update applies the
marks. A voice that is stopped goes on a stopping list for the next turn to finish off.

Samples are WAV files (see [Sound samples](Sound-samples)). `src/audio/samples.rs` reads one
and prepares it for the sound mode's quality; lower qualities halve the sample rate to save
memory. The sample bytes are held in `src/audio/sample_data.rs`.

### The output

The original used DirectSound. Here `src/platform/sound.rs` provides the same operations as
plain functions on numbered objects: a device, a primary buffer, secondary buffers (one per
voice, with copies sharing data), and each buffer's 3D settings. A small software mixer, called
by SDL when it wants more sound, plays every playing buffer at its frequency, volume and pan
into 44.1 kHz 16-bit stereo.

## A race's sounds

Once per frame the race gives its sounds a turn: `sound::race_turn` (`src/sound/race_turn.rs`).
It runs at most once per audio turn, and does this:

| Step | What | File |
| --- | --- | --- |
| Cars | For each car heard: engine by revs, throttle, a sound that follows the speed, the surface under the wheels, gear changes, hits and scrapes. | `car_sounds.rs`, `revs_sound.rs`, `throttle_sound.rs`, `speed_sound.rs`, `surface_sounds.rs`, `car_gear_sounds.rs`, `car_hit_sound.rs` |
| Cues | The co-driver's queued speech, one sound after the other. | `play_cue_queues.rs`, `play_cue_queue.rs` |
| Stage | The stage's own sounds: the start's beeps, ambience. | `stage_sounds.rs` |
| Spots | Sounds with a fixed place in the world: the nearest are kept, far ones merged or dropped. | `spot_sounds.rs`, `spot_voices.rs` |
| End | An EndOfTurn command. | `send_end_of_turn.rs` |

The physics does not play sounds itself. During a step it leaves notes on the car (a hit and
how hard, a gear change, the surface, a cue for the co-driver), and the sound turn reads them
(`src/physics/car_sounds.rs`, `queue_sound_cue.rs`, `src/physics/engine_sound.rs`).

Each car has a **sound object** holding its engine state and the sounds it is making. A
player's car uses the full set of sounds for its model; the other cars use a smaller shared
set.
The engine sounds come in pairs of sample records, the first player's and the second's; the
cars after the first two (the third and fourth players of a three or four player split screen, and the cars
of a replay of more than two cars) have copies of the records (numbers after the game's own, a block of copies for
each further pair of cars) so that each car's engine has a voice of its own. A replay of more than two cars
that the original's format could not hold is heard with every car, from the followed car in one view.

### Placing a sound

`sound::play_placed_sound` plays a sound that has a place relative to the listener. The
listener's state is worked out each frame from the viewed car and its camera
(`drive::set_listener`).

- In the plain stereo modes, the place becomes a volume (quieter with distance) and a pan from
  the angle to the left or right.
- In the 3D modes, the three coordinates are sent as they are, and the output's 3D settings do
  the rest: `src/platform/sound.rs` works out distance loss and left-right balance for each
  placed buffer.

Pitch follows the engine's revs for the engine sounds, and the sample's own rate otherwise.

## Music

Music files are WAV files played as a **stream**: only a second or so is in memory at a time
(see [Music](Music)). `src/audio/stream.rs` opens the file, makes a looping buffer and, each
turn, writes into it what has been played since the last turn. After the file's end it writes
silence and stops.

There are two stream players sharing that code, driven by the Stream and Music commands. The
game side picks the tune: the menus' music, a race's music chosen at stage load
(`src/race/music.rs`), and the other tunes (the `play_tune_*` files in `src/sound/`). `sound::stop_music` stops it; `sound::set_music_volume` follows the options.

The original could also play CD audio tracks. There is no CD here, and those calls are gone.

## The co-driver

The co-driver's calls are WAV files per language and per voice (a male and a female one; see
[Sound samples](Sound-samples)). At stage load, `sound::load_race_sounds` names the speech
files for the chosen voice and tells the audio thread to load them with the race's other
sounds.

During the race:

1. The stage's **pace notes** attach a short list of calls to checkpoints
   (`race::init_pace_notes`).
2. Each physics step, `physics::pace_notes_step` looks ahead along the checkpoints, further at
   higher speed, and queues the calls it reaches (`physics::queue_pace_note`).
3. A call is a **cue**: a few sounds to be said in a row ("left", "four", "into", "right").
   `sound::play_cue_queue` plays a cue's sounds one after another, starting each when the one
   before has had time to finish (`sound::sample_duration`).

Other spoken cues from the physics go through the same queue (`physics::queue_sound_cue`).

Each of the first two players has a pair of voice records (the first player's co-driver speaks
with one, the second's with the other, so both can speak at once). In a split screen of three or
four players the third and fourth players have a pair of their own: copies of the 190 voice
records after the engine sound copies, named by those players' voice settings and loaded
with the race's sounds (`SoundList::Range`), so up to four co-drivers speak at the same time,
each in its own cue queue (`sound::cue_players` says how many); the start's count down is spoken to them too. Their copies take the voice pool past the original's 0x200 voices to 0x240. The pace-note signs of every
player's part are kept per view (`Panel::marks_of_view`). A split screen in the arcade and a
replay keep the co-driver for the first two players only.

## Sound modes

A **sound mode** is one number that says how sound is produced: the quality (sample rate), the
number of channels, and which kind of output. The menus and the race each have their own mode;
`src/audio/choose_modes.rs` builds both from the advanced audio screen's options, and
`src/audio/set_mode.rs` switches between them (the Mode command) when a race starts and ends.

| Option | What it means here |
| --- | --- |
| Quality | Three levels; lower ones halve the samples' rate. |
| Channels | How many sounds can be heard at once. |
| 3D sound | Sounds are placed in 3D by the output instead of by volume and pan. |
| EAX | 3D sound with a reverb room chosen per stage. The room is chosen and kept; no reverb is heard (see "No reverb" below). |
| Dolby | The game mixes all channels itself (`src/audio/mixer.rs`, `mix_channels.rs`) and writes the result to the primary buffer. |

### No reverb

The original asked EAX 1 sound cards for a reverb room (ten presets) and weighed the course
around the car to move it; no such card exists on a current system, so the original is silent
here. The program once had a reverb of its own in the device mixer, driven by that weight, and
took it out: the weight only follows the geometry around the car (and says nothing of places
where a reverb would make sense, such as a large field with a fence beside the road), so the
game's data cannot drive it well. The program no longer works the weight out, and the sound page
has no "reverb" row. The room commands the game sends are still taken and kept, with the EAX
mode's choice, but nothing is heard of them.

The original's page had a sixth option, A3D (Aureal's 3D sound, for sound cards of 1999). No
system has it any more, so the option is gone and Dolby stands in its row. An options file that
still names an A3D mode loads as the same mode with 3D sound in its place (see
[Options files](Options-files)).

The Dolby mode's faults in the original (no music in races, quieter and harsher effects) are
fixed; see [Fixed bugs](Fixed-bugs).

When the window loses the focus the sound goes quiet, as the original's did.

## Where to look

| To change or understand | Start in |
| --- | --- |
| The command format | `rc2k_core/src/audio/commands.rs` |
| Sending a sound | `src/sound/play_sound.rs`, `play_placed_sound.rs`, `src/sound/queue.rs` |
| The audio thread's turn | `src/audio/thread.rs`, `take_commands.rs`, `update_voices.rs` |
| Voices and channels | `src/audio/voices.rs`, `start_voice.rs` |
| Loading samples | `src/audio/load_sounds.rs`, `samples.rs` |
| A car's sounds | `src/sound/car_sounds.rs` and the files it names |
| Speech | `src/sound/load_race_sounds.rs`, `play_cue_queue.rs`, `src/physics/pace_notes_step.rs` |
| Music | `src/audio/stream.rs`, `src/race/music.rs` |
| Sound modes | `src/audio/choose_modes.rs`, `set_mode.rs` |
| The mixer and the output device | `src/platform/sound.rs` |

Next: [Input](Input). See also [Module-sound](Module-sound), [Module-audio](Module-audio),
[Sound samples](Sound-samples), [Music](Music).
