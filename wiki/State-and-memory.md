# State and memory

*Category: [Game flow](Game-flow)*

A 1999 game written in C and assembly keeps its state in hundreds of global variables and
fixed-size buffers, and passes addresses around freely. This program keeps the same state, but
as typed Rust values with an owner each. This page explains the vocabulary you meet all over
the source: **owners**, **stores**, **cells**, **handles** and **places**. It also says where
unsafe code is allowed and why.

If you only want to find a variable, skip to "Finding a variable" near the end.

## Owners

All of the game's state is in twelve structs, the **owners** (`src/owners.rs`):

| Owner | Defined in | Area |
| --- | --- | --- |
| `Race` | `src/race/owner.rs` | The race in progress. |
| `Menus` | `src/menu/owner.rs` | The front end. |
| `Scene` | `src/scene/owner.rs` | The 3D scene and drawing work. |
| `Gfx` | `src/gfx/owner.rs` | The screen, pictures, fonts, the 3D layer's state. |
| `Sound` | `src/sound/owner.rs` | The game's side of sound. |
| `Audio` | `src/audio/owner.rs` | The audio thread's side. |
| `Input` | `src/input/owner.rs` | Keys, devices, force feedback. |
| `Net` | `src/net/owner.rs` | Network play. |
| `Saves` | `src/saves/owner.rs` | Options, saved games, best times. |
| `Strings` | `src/strings/owner.rs` | The language's texts. |
| `System` | `src/system/owner.rs` | Session flags, clocks, the machine's facts. |
| `Install` | `src/install/owner.rs` | The installer. |

`Owners::build` makes all twelve once, at start-up, and they live until the program ends.
`Owners` itself is just twelve references and is cheap to copy.

There are no global variables for game state. A function gets the state it works on as
arguments: either the whole `Owners` (written `o` in the code), or, more often lower down,
just the owner or the part of an owner it needs. This makes a function's reach visible in its
signature: `fn update_blink(menus: &Menus, system: &System)` cannot touch the race.

A big owner is divided into groups. `Race`, the largest, has `physics`, `course`, `frame`,
`replay`, `shared`, `hud` and `effects`, each in its own file (`src/race/owner_physics.rs` and
so on).

What statics remain are of three kinds: constant tables (the game's built-in data, see
[Built-in tables](Built-in-tables)), the platform's and renderers' own bookkeeping (caches,
counters, the open window), and the debug window's state. None of them is the game's state.

## Stores

An owner is built from **stores**. A store is a struct that groups variables that belong
together, with a doc comment on each field. They live in files named `..._store.rs` beside the
code that uses them: `src/physics/car_store.rs` (a car), `src/drive/finish_store.rs` (the
finish sequence), `src/menu/event_store.rs` (the event in progress), and about a hundred more.

A store's file is the place to learn what a group of variables means. Because the meanings
were worked out from behaviour, the comments say so where a meaning is only a best reading.

## Cells

A field of a store is not a bare `u32`. It is a **cell**: a small type that holds one value
and lets it be read and written through a shared reference (`.get()`, `.set(v)`). The game's
code changes its state from many places, often while other parts hold a reference to the same
store, and cells are what make that legal in Rust without locks.

There are two families (`src/memory/session.rs`):

| | Atomic cells | Plain cells |
| --- | --- | --- |
| Types | `Dword`, `Int`, `Float`, `Flag`, `Word`, `Byte`, `Bytes<N>` | `PlainDword`, `PlainInt`, `PlainFloat`, `PlainFlag`, `PlainWord`, `PlainByte`, `PlainBytes<N>`, `Plain<T>` |
| Made of | A standard atomic integer | A standard `Cell` |
| May be used by | Any thread | One thread at a time |
| Used for | State the input, audio or network thread also reads | Everything else |

A float cell stores the float's bits, so `Float` and `Dword` are the same size and a value can
be read either way, which the original code sometimes does.

Plain cells are faster, but nothing stops two threads from using one at once except the
wrapper they sit in:

- `GameThread<T>`: only the game thread may reach `T`.
- `DrawShared<T>`: game-thread state that is lent read-only to the drawing helpers while a
  frame's parts are drawn.
- `PartCell<T>` (`src/memory/part_cell.rs`): state that belongs to one drawing part at a time.

These wrappers, and the checks a debug build makes on them, are described in
[Threads and shared state](Threads-and-shared-state).

### Records with a fixed layout

Most stores can have any layout; the compiler arranges them. A few records must keep an exact
byte layout, because they are written to a file or copied as bytes: a car's state in a replay
block, a saved game, the options file's records.

Two derive macros from `rc2k_derive` serve both kinds:

- `#[derive(Cells)]` lets a record of cells be copied, compared and visited field by field,
  whatever its layout. This is how a car block is copied into the work car and back.
- `#[derive(ByteView)]` goes on a record with a fixed layout (`#[repr(C)]`) and adds a view of
  it as bytes: "the cells that make up bytes `0x134` to `0x140`". This is how a replay block
  or a saved game is filled from and written to a file without any pointer casts
  (`src/memory/cells.rs`).

Both macros check at compile time that every field is a cell or made of cells.

## Handles instead of addresses

The original stored addresses inside its data: a menu item pointed at its font, a car slot
pointed at its car block, a texture record pointed at its pixels. Stored addresses are what
make old code fragile, and they are gone. In their place are **handles**: small typed numbers
that only mean something to the table they index.

| Handle | Names | Resolved by |
| --- | --- | --- |
| `ScreenId` | A menu screen | `src/menu/screen_id.rs` |
| `MenuId` | A menu (its items) | `src/menu/menus_store.rs` |
| `FontId` | A font | `src/gfx/typefaces.rs` |
| `TextureId` | One level of a texture | `src/render/mod.rs` |
| a car block's number | A car block | `src/race/blocks_store.rs` |
| a voice's or channel's number | An audio voice or channel | `src/audio/voices.rs` |

A handle that is out of range is refused where it is used (see "Errors" below); it cannot read
someone else's memory.

## Buffers and places

Text and byte buffers are `Bytes<N>` or `PlainBytes<N>`: a fixed number of byte cells. The
game's texts are bytes in the Windows-1252 encoding, ending in a zero byte, as in the original
(file names and menu texts both).

Many old routines took "a pointer to where the next character goes". Their Rust versions take
a **place**: a buffer together with an offset into it (`BytePlace`, `src/memory/place.rs`). A
place can be moved along and written through, and writes past the buffer's end are refused
rather than landing in a neighbour.

Larger memory comes in three forms:

| Need | Type | Notes |
| --- | --- | --- |
| A table that used to be a heap block (the best times, for one) | `Owned<T>` (`src/memory/owned.rs`) | A typed value, lent for one step with `.with(...)`. Counted on the game's memory lists so the game's own memory report stays the same. |
| Pixels: the screen, textures | `PoolBytes` (`src/memory/pool.rs`) | Blocks of bytes reused through a pool; reads and writes are checked against the length. |
| The course, objects, sample data | `Vec`s and typed structs inside their owner | Loaded at stage load, dropped at its end. |

## Errors

A function that can fail returns `Result<_, Refused>` (`src/errors.rs`). `Refused` is one of:

- `FatalError { code, message }`: a failure the original game also treated as fatal, with the
  original's error code. The program shows the game's error report and ends.
- `BadInput(reason)`: the function was given something it will not work with: an index past a
  table, a count that makes no sense, a damaged file. The original would have carried on and
  read or written the wrong memory. Here the reason travels up the call chain and the program
  ends with it in `rc2k.log`.

There are no silent fallbacks. If a value cannot be handled, the program says so and stops,
which is how most of the original's latent memory bugs were found (see
[Fixed bugs](Fixed-bugs)).

## Where unsafe code is allowed

The game crate is compiled with unsafe code **denied** (`src/lib.rs`). A file or item that
needs it must opt in with an `allow` and a stated reason, and each unsafe block carries a
comment saying why it is sound. `rc2k_core` **forbids** unsafe code outright; it cannot be
switched on there.

The opt-ins are few and of two kinds:

| Where | Why |
| --- | --- |
| `src/platform/ffi/` | Calls into C libraries: SDL, OpenGL ES, Dear ImGui, the operating system. |
| `src/platform/`, `src/render/` | Handing memory to those libraries; processor-specific instructions in the rasteriser. |
| `src/memory/` | The building blocks themselves: the `Sync` promises of `GameThread`, `DrawShared` and `PartCell`, the byte views, the page pool, copies through atomics. |
| `src/owners.rs` | Building the owners on a large temporary stack. |

Everything else, which is nearly all of the game (menus, race, physics, scene, sound, network,
installer, saves), is safe Rust on top of those blocks. `tools/count_unsafe.py` counts the
unsafe blocks, so a change that adds one shows up.

## Finding a variable

1. Guess the area and open its owner (`src/<area>/owner.rs`). For the race, pick the group
   (`owner_physics.rs`, `owner_course.rs`, `owner_frame.rs`, `owner_replay.rs`,
   `owner_shared.rs`, `owner_views.rs`).
2. Follow the field to its store's file and read the field comments.
3. To see who uses it, search for the field's name. Names are specific, so a plain text search
   works well.

Going the other way, from code to state: a line such as
`race.shared.view.get().race_cars.get()` reads *owner* `race`, *group* `shared`, *store*
`view` (a `GameThread`, hence the first `.get()`), *field* `race_cars` (a cell, hence the
second).

The debug window can show many stores live while the game runs; see
[The debug window](The-debug-window).

## Where to look

| To change or understand | Start in |
| --- | --- |
| The list of owners and how they are built | `src/owners.rs` |
| Cell types and the thread wrappers | `src/memory/session.rs` |
| Copying and byte views of records | `src/memory/cells.rs`, `rc2k_derive/src/lib.rs` |
| Places in buffers | `src/memory/place.rs` |
| Owned tables | `src/memory/owned.rs` |
| Pixel memory | `src/memory/pool.rs` |
| Errors | `src/errors.rs`, `src/fatal/` |
| A car's fields, as a large worked example of a store | `src/physics/car_store.rs` |

Next: [The installer](The-installer). See also [Module-memory](Module-memory),
[Code conventions](Code-conventions), [Glossary](Glossary).
