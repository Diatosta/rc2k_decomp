# Modules

*Category: Module reference*

The game's source is the folder `src/`, one folder per area of the game. This page lists them
all, with one line each and a link to a short page per module. Two smaller crates sit beside it:
`rc2k_core/` (the game's rules without state or platform) and `rc2k_derive/` (derive macros).

Two habits hold everywhere and make the folders easy to read:

- Most files hold **one function**, named after the file: `drive::race_loop` is the function
  `race_loop` in `src/drive/race_loop.rs`. A comment at the top of the file says what it does.
- A module's variables live in its **owner** (a struct in the module's `owner.rs`) or in
  `..._store.rs` files that define the owner's parts. See [State and memory](State-and-memory)
  and the [Glossary](Glossary).

For how the modules work together, start at [How the game runs](How-the-game-runs). Which modules may use which: [Code layers](Code-layers).

## The game

| Module | What it is |
| --- | --- |
| [`menu`](Module-menu) | The front end: every menu screen, the event flows (championship, single rally, time trial, arcade), the launch switches. |
| [`screens`](Module-screens) | The routines that lay out and draw the parts of menu screens. |
| [`service`](Module-service) | The service area between stages: repairs and car set-up. |
| [`race`](Module-race) | A stage from loading to tidying up: course data, replays and the viewer, the race's drawing, pace notes, mud, the map. |
| [`drive`](Module-drive) | The race while it runs: the race loop, the simulation step, inputs, the computer driver, recording and playing replays. |
| [`physics`](Module-physics) | The car model: forces, grip, gearbox, engine, contacts, damage, and the chase cameras. |
| [`scene`](Module-scene) | The game's own 3D: objects, textures, lighting, clipping, projection. |
| [`effects`](Module-effects) | Weather, spray, dust, mud particles, windscreen drops, lens flare, shadows. |
| [`hud`](Module-hud) | The in-race display: dials, times, signs, stage bars. |
| [`sound`](Module-sound) | The game's side of the sound: what to play, sent as commands. |
| [`audio`](Module-audio) | The audio thread: carries the commands out, voices, samples, music streams, mixing. |
| [`input`](Module-input) | Keys, controllers, force feedback, the input thread. |
| [`net`](Module-net) | Network play: lobby screens, sessions, the race's messages. |
| [`saves`](Module-saves) | Options, best times, saved games, the replay slots. |
| [`install`](Module-install) | The installer and uninstaller, and the installation's record. |

## Drawing and the platform

| Module | What it is |
| --- | --- |
| [`gfx`](Module-gfx) | 2D graphics: the screen surface, pictures (PCX, PNG), fonts and glyphs, screen modes, screenshots. |
| [`text`](Module-text) | Drawing and measuring text with the game's fonts. |
| [`d3d`](Module-d3d) | The thin 3D drawing layer the scene hands finished triangles to. |
| [`render`](Module-render) | The renderers: the command list and the three backends (GPU, OpenGL ES, software). |
| [`platform`](Module-platform) | Everything between the game and the computer: window, events, sound output, controllers, films, threads, the operating system. |
| [`debug`](Module-debug) | The developers' debug window. |
| [`video`](Module-small-modules) | A replay rendered to a video file (`--render-replay`), and its encoder. |

## Support

| Module | What it is |
| --- | --- |
| [`files`](Module-files) | Paths, file reading and writing, packed files, buffered streams. |
| [`strings`](Module-strings) | The language's texts, the lexicon, number and time formatting. |
| [`clock`](Module-clock) | The game's millisecond clock, sleeping, the packed local time. |
| [`system`](Module-system) | The main loop, the command line, clocks, facts about the machine. |
| [`memory`](Module-memory) | The typed cells and stores the game's data lives in. |
| [Small modules](Module-small-modules) | `errors.rs`, `fatal`, `logfile`, `math`, `records`, `settings`, `threading`, `owners.rs`, `standalone.rs`, `remote.rs`, `first_run.rs`, `single_copy.rs`, `error_dialog`, `video`, `lib.rs`, `bin/`, `test_support`. |

## The other crates

| Crate | What it is |
| --- | --- |
| [`rc2k_core`](Module-rc2k-core) | The game's logic as plain functions and types: no state of its own, no platform, no unsafe code. Most unit tests are here. |
| [`rc2k_derive`](Module-rc2k-core) | The derive macros `Cells`, `ByteView` and `ZeroCells` for the game's records. |
