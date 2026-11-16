# Threads and shared state

*Category: [Game flow](Game-flow)*

The program runs several threads. Some are the 1999 game's own design (input, audio, network);
others were added by this project to use today's multi-core processors (drawing helpers, the
renderers' threads). This page says what each thread does, what it shares with the others, and
what stops two of them from tripping over each other.

It is the page to read before changing anything that more than one thread touches.

## The threads

| Thread | Started by | Runs | Does |
| --- | --- | --- | --- |
| Game | the program's `main` | `standalone::run` | Everything not listed below: start-up, the menus, the simulation, issuing the drawing, file reading and writing. Also SDL's event handling. |
| Input | `threading::thread_slot::start_workers` | `input::input_thread` | Every 20 ms: counts the key clock, reads the players' devices, delivers force-feedback changes. |
| Audio | `threading::thread_slot::start_workers` | `audio_thread` (`src/audio/thread.rs`) | Every 20 ms: feeds the music streams, takes the game's sound commands, updates the voices. |
| Network receive | `threading::thread_slot::start_workers` | `net::receive_thread` | Waits for a signal that messages have arrived, takes them, and files race messages in each player's inbox. |
| Network connections | a session, when one is hosted or joined | `src/net/dp_session/` | The sockets: accepting machines, reading and writing frames. |
| Shutdown | `threading::request_quit_in` and its siblings (`src/threading/request_quit.rs`) | `threading::shutdown_thread` | Waits up to three seconds for the input, audio and network threads to end. |
| Drawing helpers | `src/scene/workers.rs` (a thread pool) | parts of a frame | Draw a view, a mirror or the player's car at the same time as the game thread draws another part. |
| Rasteriser | `src/render/software.rs` (a thread pool) | tiles of the picture | The software renderer's triangle filling, split over the cores. |
| GL and GL batches | `src/render/gles.rs` | the handheld's renderer | One holds the OpenGL ES context and talks to the driver; one prepares batches a part ahead (`src/render/gl_batching.rs`). |
| Sound mixer | SDL | `src/platform/sound.rs` | SDL calls in when the output needs more sound. |
| Helpers | various | | A watchdog for the fault reporter (Windows), a safety net after the window is closed (`src/platform/events.rs`), the handheld's profiler when asked for. |

### Thread slots

The first three worker threads and the shutdown thread have **slots**
(`src/threading/thread_slot.rs`): the threads are the `SlotThread` enum (`Input`, `Audio`,
`NetReceive`, `Shutdown`), and each has a small record with its id, its handle and whether its
routine is running. `SlotThread::start` starts a thread, named `input`, `audio`, `net receive`
or `shutdown` (the names a debugger, a profiler and the log show); `SlotThread::run` is what it
runs.
The threads themselves, and the events and waits they use, are built on the standard library
(`src/platform/sync.rs`).

A slot thread is given `SlotOwners`: the owners it may work with. It contains no state that
belongs to the game thread alone, so such state cannot even be named from a slot thread.

## Quitting

One flag ends everything: `flags.quitting` in the session state (`System` owner). It is set by
`request_quit`, which also starts the shutdown thread. Every loop looks at the flag:

- the input, audio and network threads leave their loops and end;
- the menus' screen loop, the race loop and the film player leave and unwind back to
  `system::main_loop`, which saves the options and best times.

Closing the window sets the same flag (`src/platform/events.rs`). If some loop fails to notice
within a grace period, the program ends anyway, without saving, rather than hang.

A fatal error also goes this way: the report is noted, quitting is requested, and the report
is shown when the program has shut down (`src/fatal/`).

## What the threads share

The game's state lives in the twelve owners (see [State and memory](State-and-memory)). They
are created once and live for the whole run, so every thread can hold plain references to
them. What matters is the *type* of each field, because the type decides who may touch it.

### 1. Atomic cells: anyone, any time

Values that more than one of the original's threads read or write are **atomic cells**:
`Dword`, `Word`, `Byte`, `Flag`, `Float`, `Int`, `Bytes<N>` (`src/memory/session.rs`). Each is a
thin wrapper round a standard atomic integer. Reading or writing one is always safe; there is
no lock and no tearing of a single value.

Examples: the quit flag, the window-active flag, the key table and the pending key, the key
clock, a device's readings, the sound mode in use.

This matches how the 1999 game worked: its threads simply read and wrote the same memory. The
atomic types make that well-defined instead of a data race. What they do not give is
consistency *between* two values. Where that matters, there is a lock (below).

### 2. Game-thread state: one thread only

Most of the game's state is only ever touched by the game thread: the menus, the saves, the
installer, most of the race. It is made of **plain cells** (`PlainDword`, `PlainFloat`,
`Plain<T>` and so on: ordinary `Cell`s, cheaper than atomics) wrapped in `GameThread<T>`.

`GameThread<T>` is the promise "only the game thread reads or writes this". Its `get` returns
the inside; in debug builds it checks that the caller really is the game thread
(`set_game_thread` marks it at start-up) and stops the program if not.

### 3. Locks: where several values must agree

| Shared thing | Protected by | Used by |
| --- | --- | --- |
| The sound command queue | a mutex (`src/sound/queue.rs`) | game thread (send), audio thread (take) |
| The network inboxes | a lock; the game copies them out under it (`net::copy_received`) | receive thread (store), game thread (apply) |
| A table Rust owns in place of a heap block, such as the best times | `Owned<T>::with` lends it for one step (`src/memory/owned.rs`) | mostly the game thread |
| The game's log file | a lock in `src/logfile/` | every thread that logs |
| The handles of threads and events | a table under a mutex (`src/platform/sync.rs`) | all |

The rule of thumb in the code: hold a lock for a short copy in or out, never across a wait or
a call into another subsystem.

### 4. Records shared by raw copy

A few records are copied whole between the audio thread and the game thread (a voice, a
channel, a stream player): `Shared<T>` in `src/memory/session.rs`. Byte-wise copies between
shared memory go through `src/memory/shared_memory.rs`, which copies through atomics so that
a copy racing a write is still defined behaviour.

## Drawing on several threads

The drawing helpers are the newest threads and have the strictest rules, because they run the
game's own drawing code on state the game thread also uses. The picture must come out the same
as on one thread. See [Drawing a frame](Drawing-a-frame) for what is drawn; here is how the
sharing is kept safe.

A frame's parallel stretch is a **drawing scope**, opened by `draw_parts` in
`src/scene/workers.rs`. Inside a scope:

| Kind of state | Type | Rule |
| --- | --- | --- |
| A part's own working state | `PartCell<PartScratch>`, `PartCell<ViewScratch>` (`src/memory/part_cell.rs`) | Claimed by exactly one thread for the scope. A debug build stops if it is claimed twice, or reached by the game thread while a helper holds it. |
| State every part reads | `DrawShared<T>` | Lent read-only to all parts as a `PhaseRead` for the length of the scope. A debug build stops at any write to it during the scope. |
| Everything else game-thread | `GameThread<T>` | Not reachable from a helper. |
| The noted drawing | each part's own command list | Joined on the game thread, in the parts' fixed order, after the scope. |

Outside a scope, `DrawShared` state is ordinary game-thread state and the game thread writes
it freely (between frames, during the simulation).

The outcome is that a helper thread can only *read* shared things and *write* its own. What it
needs to pass on, it returns, and the game thread applies it in order. `RC2K_DRAW_SERIAL` runs
the same parts one after another on the game thread, which separates "the copies are wrong"
from "the threads interfere" when hunting a difference.

## The renderers' threads

By the time a command list reaches a renderer it is plain data, moved (not shared) to whoever
draws it:

- the software rasteriser cuts the target into tiles and gives each tile to one pool thread;
  tiles do not overlap, so no pixel has two writers;
- on the handheld, the game thread sends each part's commands through a channel to the batching
  thread, which sends batches on to the GL thread. Only the GL thread touches the GL context.

## The audio output

SDL calls the mixer in `src/platform/sound.rs` on its own thread. The mixer reads the playing
buffers; the audio thread changes them (volume, position, new data). The buffers' table is
under a lock, held briefly by each side.

## Practical rules

If you add or change shared state:

1. **Find out who touches it.** If only the game thread: a plain cell inside the owner's
   `GameThread` part. If any other thread: an atomic cell.
2. **Two values that must agree** across threads need a lock, or must be packed into one
   atomic value.
3. **Never reach game-thread state from a new thread.** Pass what the thread needs in, and
   take results back as return values or through a channel.
4. **In drawing code**, write only to the `PartScratch` and `ViewScratch` you were handed.
   Anything a view produces for later goes into the view's outputs.
5. **Run a debug build.** The checks described above are debug-only and catch most slips at
   once.

## Where to look

| To change or understand | Start in |
| --- | --- |
| Starting and ending the game's threads | `src/threading/` |
| Threads, events and waits underneath | `src/platform/sync.rs` |
| Atomic and plain cells, `GameThread`, `DrawShared` | `src/memory/session.rs` |
| Per-part state and its checks | `src/memory/part_cell.rs` |
| The drawing scope and helper threads | `src/scene/workers.rs` |
| The sound command queue | `src/sound/queue.rs` |
| Network inboxes | `src/net/store_received.rs`, `copy_received.rs` |
| Quitting | `src/threading/request_quit.rs`, `src/platform/events.rs` |

Next: [State and memory](State-and-memory). See also [the small modules](Module-small-modules),
[Module-memory](Module-memory), [Module-platform](Module-platform).
