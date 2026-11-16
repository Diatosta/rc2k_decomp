# Glossary

*Category: [About](About)*

The words these pages use, each defined once. Where the source code uses a different word for
the same thing, it is given in brackets. Some words have two meanings, in different parts of
the game; both are listed.

## The game

| Word | Meaning |
| --- | --- |
| stage | One timed run from a start to a finish. The game has 36. |
| course | A stage's ground: the faces driven on, the scenery's places, the checkpoints ([Course files](Course-files)). |
| rally (section) | A group of stages; the championship is a row of them. Saved games call a rally a section. |
| stage set | One of the stage screen's two lists, simulation and arcade. The arcade set runs each stage the other way. |
| event | A whole competition chosen in the menus: a championship, a single rally, a time trial, an arcade game. |
| event flow | The function that walks an event's screens and races in order ([The front end](The-front-end)). |
| front end | The menus, as opposed to a race. |
| screen | One menu screen: a `Screen` description run by the menus' loop. |
| service area | The screens between stages where the car is repaired and set up. |
| set-up | A car's adjustable values: tyres, gear ratios, brake balance, ride height and so on ([Car set-ups](Car-setups)). |
| class | A car's group, 0 .. 3. |
| checkpoint | A place along the course that a car must pass; stage and split times are taken there. |
| pace notes | The co-driver's calls and their signs on the screen. |
| computer driver | The code that drives a car no player drives. |
| slot | A numbered save file: saved games, replays and car set-ups each have slots `00` and up. |
| game's folder | The folder that holds `files`, `var` and `savedata`. The program is started in it. |
| download folder | The files of the game's CD on disk; the installer installs from it. |
| installation record | The file that says a folder is an installed game ([Install records](Install-records)). |
| launch | Starting straight into a race from the command line; a launch file holds the same choices as lines of text ([Running and options](Running-and-options)). |

## The simulation and replays

| Word | Meaning |
| --- | --- |
| step (tick, tenth) | One advance of the simulation: one hundredth of a second, whatever the frame rate. |
| frame | One picture drawn. (In the replay code a "frame" is a block; these pages do not use it so. On the network pages a frame is one length-prefixed piece of data on a connection.) |
| block | One second of a replay: the cars' whole state and 100 steps of inputs ([Replay files](Replay-files)). |
| car block | The game's record of one car. |
| work car | The working copy of a car that a step's physics runs on, copied back when the step is done. |
| force record | A force together with the point it acts on. |
| gauge | One of a car's damage or wear figures. |
| sealed | Enciphered with a checksum, as a replay's header part and the best-time entries are. |
| held race | A race made repeatable for tests: fixed random numbers, the computer driver, the clock stepped by a fixed amount ([Testing](Testing)). |
| replay check | The comparison, while a replay plays, of the state the physics reached with the state stored in the next block. |
| remote build | A build with the `remote` feature, which `tools/remote.py` can drive. |

## The code

| Word | Meaning |
| --- | --- |
| owner | One of the twelve structs that between them hold all of the game's state ([State and memory](State-and-memory)). |
| store | A struct that groups related variables inside an owner (files named `*_store.rs`). |
| cell | A small type holding one value, read and written through a shared reference. Every field of a store is one. |
| handle | A typed number that names an entry of a table, used where the original kept an address. |
| place | A buffer together with an offset into it. |
| refusal | A function's way of giving up: a `FatalError` (the original's own fatal errors) or a `BadInput` (a value the code cannot handle sensibly). Either ends the program with its reason in the log. |
| platform layer | `src/platform/`: everything between the game and the system it runs on. |
| slot (thread) | The game's record of one of its threads. |
| pending key | The latest key press, waiting for the game to take it. |
| reading, binding | The value a device gives for a control, and what a control is bound to ([Input](Input)). |

## Drawing

| Word | Meaning |
| --- | --- |
| view | What is drawn for one player: the scene from that player's camera. |
| mirror | A view's rear view, drawn as a small scene of its own. |
| part | One piece of a frame (a view or a mirror) prepared on its own thread. |
| drawing scope | The stretch of a frame in which the parts are prepared at once. |
| fan | A run of finished, screen-space vertices drawn as triangles sharing the first vertex: the unit the game's 3D code hands on. |
| render state | A numbered drawing setting (blending, depth test and so on). |
| command list | The frame's drawing, noted in order for the renderer. |
| backend (renderer) | What draws the command list: the GPU renderer, the OpenGL ES renderer or the software rasteriser ([Drawing a frame](Drawing-a-frame)). |
| batch | Fans that share their settings, drawn in one go. |
| canvas | The memory picture the game's 2D drawing writes into. |
| virtual screen | The screen size the game's 2D positions are worked out in, scaled to the window. |
| glyph, glyph table | One character's picture, and a font's set of them in memory ([Fonts](Fonts)). |
| sheet, cell (font) | A font's picture file, and the box on it that holds one glyph. |
| mark | The code's word for any race effect: skid marks, spray, rain, drops on the windscreen ([Effects data](Effects-data)). |

## Game data

| Word | Meaning |
| --- | --- |
| packed | Stored compressed. `.SQ` files use the game's own packing ([Packed files](Packed-files-SQ)); replay blocks use a run-length coding. |
| cell (course) | One square of the course's ground plan; each face and point belongs to one. |
| face, point | A polygon of the course or of an object, and a corner of one. |
| marker | A course entry that is not ground: an object, a checkpoint, a camera, a sound or a lamp. |
| object kind | A type of 3D object, with a record in one of the object tables ([Object and texture tables](Object-and-texture-tables)). |
| object slot | One of a stage's numbered places for object kinds; the stage's `LOCALDAT` says which kind fills each. |
| lexicon | The language file: every text the game shows, by number ([Language files](Language-files)). |
| built-in table | Data that was inside the original program and is now Rust source ([Built-in tables](Built-in-tables)). |
| u8, u16, u32, i32, f32 | Unsigned 8-, 16- and 32-bit numbers, a signed 32-bit number, a 32-bit float; all little-endian. |

## Sound

| Word | Meaning |
| --- | --- |
| sample | A sound loaded from a `.wav` file; its sample record describes it. |
| voice | A playable copy of a sample. |
| channel | A place a voice plays in. |
| stream | Music played from its file a piece at a time. |
| cue | A few spoken sounds played in a row, as the co-driver's calls are. |
| sound mode | One number that stands for the sound's quality, channels and kind of output. |

## Network

| Word | Meaning |
| --- | --- |
| session | A network game that one machine hosts and others join. |
| host | The machine that made the session. |
| lobby | The screens where the players of a session gather before a race. |
| inbox | A player's race messages, filed by the receive thread for the game thread. |
| sync round | The host's exchange with every player that makes sure all are ready before a race starts. |
