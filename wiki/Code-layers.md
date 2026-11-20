# Code layers

*Category: Module reference*

The modules of `src/` sit in layers. The rule is one line: **a module may use its own layer and
the layers below it, never one above.** The code does not hold the map everywhere yet: the
script `tools/check_boundaries.py` counts every use the wrong way, and each round of checks fails
when the count rises above the recorded figure, which each boundary batch lowers. The script holds
the same table as this page (`LAYERS`), so the two cannot drift apart: change both together.

## Crates

| Crate | Holds | May use |
| --- | --- | --- |
| `rc2k_core` | The game's logic as pure functions over values and slices: formats, rules, arithmetic, film decoding. No unsafe (forbidden), no files, clock, threads, sockets, environment, process. | std's pure parts, `rc2k_derive` |
| `rc2k_derive` | Derive macros for typed cells and byte views. | proc-macro crates |
| `rc2k_decomp` (`src/`) | The program: state (the owners), the platform, drawing, the flow. | `rc2k_core`, `rc2k_derive`, the system crates through `src/platform/` only |

Why `rc2k_core` is strict: it is what the Linux and the Windows unit tests run without a window, what
the fuzzers feed, and what a new port gets for free. Anything in it that opens a file or reads a clock
makes a test depend on the machine.

## Layers inside `src/` (lower may not use higher)

| # | Layer | Modules | What it is | Why it sits here |
| --- | --- | --- | --- | --- |
| 0 | foundation | `errors`, `memory`, `math`, `strings`, `program_log` | Typed cells and stores, the x87-exact arithmetic, text building. | Everything stands on these; they must know nothing of the game's objects, or every module depends on every other through them. |
| 1 | platform | `platform` (with `platform::ffi`, `platform::os`), `threading`, `logfile`, `files`, `clock`, `language` | The only code that talks to the system: SDL, libc, Win32, Android / iOS glue, files, threads, the clock, the log, the loading of the language's files. | A port touches this layer and nothing else. `platform::ffi` is its leaf: wrappers over one foreign API each, using nothing of `platform` above them. |
| 2 | engine | `render`, `gfx`, `text`, `d3d`, `video`, `audio`, `sound`, `input`, `net` | Services over the platform that a game needs: drawing back ends, pictures and fonts, the mixer and its command queue, devices, the wire protocol. | They serve any screen or race; they must not know which one is running. |
| 3 | simulation | `physics`, `scene`, `effects`, `hud`, `drive`, `race`, `records`, `saves`, `settings`, `service` | The race: the car, the course, the 3D scene, weather and marks, the replay, saved state. | Held races and replays are decided here; it must run the same whatever front end started it. |
| 4 | front end | `menu`, `screens`, `save_code`, `install`, `first_run`, `error_dialog`, `single_copy` | What the player navigates: menus, pages, the installer, dialogs. | Uses the simulation (starts races, shows results); the simulation never asks the menus anything. |
| 5 | root | `owners`, `system`, `fatal`, `standalone`, `shell`, `bin`, `lib` | Builds the owners, starts the threads, runs the main loop, ends the program. | The one place that knows every module. |
| 6 | tools | `debug`, `remote` | The debug window and the test driver. | May read anything; the game reaches them only through named doors, so a build without them is the same game. |

## Doors (the allowed exceptions, kept short)

- `crate::owners::*` from any layer: the owner types are how state is passed down.
- `crate::fatal::raise` from any layer: ending the program with a reason.
- `crate::system::pause` from any layer: a pause pumps the window's events and ends on a key, so
  it needs the owners; it stays in `system` and is reached from below by name.
- `crate::shell::frame::*`: showing a frame and pumping the window's events (`show_frame_now`, `run_frame_now`, `pump_messages_now`) need every owner and are called by the menus, the screens and the race loops.
- `crate::debug::{timing, imgui_sdl, switches}` and `crate::remote::*`: the tools' doors.
  `remote` is always compiled and is one door: `src/remote/mod.rs` picks `driver.rs` (the test
  driver, with the `remote` feature) or `off.rs` (constants and no-ops with the same signatures),
  so game code calls `crate::remote::*` with no `cfg`. `TOOL_DOORS` in `tools/check_boundaries.py`
  lists its functions.

## Further rules the script checks

1. **System crates** (`sdl3_sys`, `libc`, `windows_sys`, `dear_imgui_*`, `mdns_sd`) are named only
   under `src/platform/` (SDL's events are decoded in `platform::event`). At the start (2026-11-19): 29 lines in 23 files outside it (`render/gpu*` for the SDL GPU API,
   `debug/*` for Dear ImGui, `net/bonjour.rs`, `net/steam/flat.rs`, `first_run.rs`, `remote/driver.rs`,
   `bin/rc2k.rs`). A `use sdl3_sys::..` line counts once, so `render/gpu.rs` (84 `SDL_` mentions)
   shows as one line.Open: either `render/gpu*` and `debug/imgui_sdl.rs` become allowed
   homes (they ARE back ends), or their SDL calls move behind `platform::ffi::sdl_gpu` / `ffi::imgui`,
   which already exist.
2. **`platform::ffi` looks at nothing above it** (clean: `web`, `arch`, `vibrate` and `process` live in `ffi`; the program's `finish` and fault report reach the signal handlers through `ffi::process::set_handlers`).
3. **`rc2k_core` does no I/O**: no files, sockets, threads, clock reads, environment or process.
   Value types under those paths (`std::time::Duration`, `std::net::Ipv4Addr`) are allowed.
4. An upward use that only **names an owner type** (`crate::menu::owner::Menus` as a parameter) is
   counted apart (`up-owner`, 447) from one that calls code (`up`, 612): the first is cured by passing
   a narrower value, the second by moving code.

## What is deliberately not a rule

- Same-layer uses (race <-> drive, menu <-> screens, physics <-> scene): allowed. Cycles inside a
  layer do not fail the check.
- Test code: `#[cfg(test)]` items, `tests.rs`, `*_tests.rs`, `test_support` are skipped.

## Using the script

    tools/check_boundaries.py            # summary: counts per pair of modules, worst files
    tools/check_boundaries.py --list     # every violation as file:line: [kind] from -> to::item
    tools/check_boundaries.py --matrix   # every module-to-module count, allowed ones too
    tools/check_boundaries.py --max N    # exit 1 above N violations

It finds the repository from its own place (or takes one as its argument). Known limits: it reads
text, not the compiler's view (a glob import or a re-export hides a use; a `crate::x` inside a
string or macro body counts), and a `#[cfg(test)]` on a single `use` line is skipped only when it
sits directly above the item.
