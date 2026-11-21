# Code conventions

*Category: [Building and running](Building-and-running)*

How the code is laid out and the rules it follows. The code began as a function-by-function
port of a 1999 program, and its shape still shows that in places; the conventions below are
what new and reworked code is held to.

## Layout

| Path | What it holds |
| --- | --- |
| `src/` | The game crate, `rc2k_decomp`: state, flow, drawing, sound, input, the platform layer. |
| `rc2k_core/` | The game's rules as plain functions: no state of their own, no platform, no unsafe code. Most unit tests are here. |
| `rc2k_derive/` | Derive macros for the typed records the state is made of. |

Inside `src/` there is one folder per area of the game ([Modules](Modules)).

**One function per file.** Most files hold one function of the game, named after the file:
`src/physics/car_physics_step.rs` holds `car_physics_step`. The comment at the top of the file
says what the function does. To find something, search for file names.

**Rules in `rc2k_core`, state in `src`.** A function in `src/` typically gathers values from
the game's state, calls a function in `rc2k_core` to decide something, and stores the result.
New logic that can be written without the game's state belongs in `rc2k_core`, with tests.

## State: owners and cells

The game has no global variables. Its state lives in twelve structs called **owners**, one per
area (`Race`, `Menus`, `Scene`, `Gfx`, `Sound`, `Audio`, `Input`, `Net`, `Saves`, `Strings`,
`System`, `Install`), each defined in its module's `owner.rs`. They are built once at start-up
and handed down the call chain; `Owners` (`src/owners.rs`) is a small struct of references to
all twelve. A function takes the owners, or the parts of one, that it needs.

Inside an owner, values are held in **cells**: small typed containers that can be read and
written through a shared reference, atomic where more than one thread reaches them
(`src/memory/`). So state is passed as `&`, not `&mut`, and which thread may touch what is
said by the cell's type.

More in [State and memory](State-and-memory) and
[Threads and shared state](Threads-and-shared-state).

## Errors

There are no hidden fallbacks. A function that cannot do its job says so, and the program ends
with the reason rather than carry on with wrong data. The types are in `src/errors.rs`:

| Type | Meaning |
| --- | --- |
| `FatalError { code, message }` | A condition the original game treated as fatal, with its error number. The game's error report is shown. |
| `BadInput(reason)` | A value the code cannot handle sensibly: an index outside a table, a damaged file, a case the original itself went wrong on. The reason goes to `rc2k.log`. |
| `Refused` | Either of the two, for functions that can meet both. |

Most functions return `Result<_, Refused>` (or `Result<_, BadInput>`) and pass errors up with
`?`. Do not swallow an error, substitute a default, or `unwrap` on data that comes from a file
or the network.

## Unsafe code

- `rc2k_core` forbids unsafe code outright.
- The game crate denies it by default. A file may allow it only by saying so at its top, and
  only two kinds of file do: the wrappers around C libraries (`src/platform/ffi/`, and a few
  platform, renderer and scene files) and the handful of memory primitives the cells stand on
  (`src/memory/`).
- Every unsafe block has a `// SAFETY:` comment saying why it is sound; every unsafe function
  has a `# Safety` section saying what its callers must promise. Clippy enforces both.
- Unsafe code is acceptable when it is proven sound and documented at the spot. Speed alone is
  not a reason to add it.

`tools/count_unsafe.py` counts what there is.

## Arithmetic that must match everywhere

Replays and network games only work if every machine computes the same numbers. Two rules
follow:

- Do not call the standard library's `sin`, `cos`, `atan2`, `powf` and their kind: the
  system's maths library may differ in the last bit between machines. Use
  `rc2k_core::math::portable` instead. `clippy.toml` lists these methods as disallowed, so
  clippy catches a slip.
- The physics keeps the floating-point behaviour of the original program (which values are
  held at higher precision, where they are rounded). Do not "simplify" an expression in the
  physics without checking a replay still plays back exactly ([Testing](Testing)).

## Comments

- A comment says what the code does and why, for someone reading the Rust code.
- No addresses, processor registers or disassembler names, and no history of how the code got
  here. Git has the history.
- The top-of-file comment describes the function: what it takes, what it gives or changes,
  when it refuses.
- Where the code deliberately keeps an oddity of the original game, say that it is deliberate.

## Style and lints

- `cargo fmt` formats everything.
- Clippy must give no warnings. The workspace turns on a few extra lints (`Cargo.toml`,
  `[workspace.lints.clippy]`).
- An `allow` goes on the one item that needs it, with its reason, never on a whole module.
- Names are explicit: prefer a long clear name to a short internal one.

## The game's behaviour

- The game is meant to play exactly as it did in 1999. A change to behaviour is either a fix
  of one of the original's bugs, recorded in [Fixed bugs](Fixed-bugs), or a deliberate
  difference, recorded in [Differences from the original](Differences-from-the-original).
- File formats stay compatible: old replays and saves must keep working
  ([Data formats](Data-formats)). New options go in the program's own file, not the
  original's.
- Platform code is not a copy of the 1999 platform: it uses today's libraries where they
  behave the same for the game.

## Keeping the wiki current

A change that alters something the wiki describes (a switch, a build step, a file format, a
module's job) updates the wiki page in the same commit. The wiki's source is the `wiki/`
folder of the repository.
