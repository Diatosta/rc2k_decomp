# Module: the small modules

*Category: [Module reference](Modules)*

The small folders and the single files at the top of `src/`, together on one page.

## `errors.rs`: how functions fail

`src/errors.rs` defines the two ways a function of the game refuses to go on, and `Refused`,
which is either of them. Most functions return `Result<_, Refused>`.

| Type | Meaning |
| --- | --- |
| `FatalError` | A condition the game itself treats as fatal, with its error code and message. |
| `BadInput` | The code met a value it cannot handle sensibly (an index outside a table, a damaged file). The program ends with the reason in its log rather than carry on with wrong data. |

## `fatal`: the fatal-error report

Ends the program when an error reaches the top, and shows the game's error report.

| Item | File | What it does |
| --- | --- | --- |
| `raise`, `refused`, `end` | `src/fatal/raise.rs` | Raise a fatal error; turn a `Refused` into the program's end; the one way to stop. |
| `report` | `src/fatal/report.rs` | Builds the report's text. |
| `show_report_at_exit` | `src/fatal/show_report_at_exit.rs` | Shows it as the program leaves. |

## `logfile`: the two logs

The game's own log, `var/logfile.txt`, written as the 1999 program wrote it, and the
program's log, `rc2k.log`.

| Item | File | What it does |
| --- | --- | --- |
| `start_log` | `src/logfile/start_log.rs` | Starts the game's log with its heading lines. |
| `log_text` | `src/logfile/write_line.rs` | Writes a text to the game's log, under its lock and at its level. |
| `log_int`, `log_float`, `log_hex8` | files of the same names | Numbers into the game's log. |
| `log` | `src/logfile/program_log.rs` | Writes a line to `rc2k.log`. |

See [Log files](Log-files).

## `math`: vectors, random numbers and exact floats

Small vector helpers (`normalise_vector`, `rotate_point_about_y`), the game's random number
streams (`src/math/next_random.rs`, `seed_random.rs`), and the arithmetic that reproduces the
original's floating point. The 1999 program computed on the x87 unit, whose rounding differs
from ordinary `f32` / `f64` arithmetic; replays depend on every result matching to the last
bit. `X` in `src/math/x87_float.rs` is a float that rounds as that unit did, and
`src/math/x87.rs` holds the operations built on it. Used heavily by
[`physics`](Module-physics), [`effects`](Module-effects) and [`scene`](Module-scene). The
portable sine, cosine and square-root estimates are in `rc2k_core::math`.

## `records`: the stage tables

The stages' built-in records: which stages make up which rally, their names' text numbers,
lengths and record holders. `src/records/stages_store.rs` holds the tables
(`StageTables`, `Record`); `find_stage_record` and `find_record_in` look a stage up;
`seal_time` scrambles a time so that a stored record is tied to its entry. Used by
[`menu`](Module-menu) and [`screens`](Module-screens). See [Built-in tables](Built-in-tables).

## `settings`: `var/ral.cfg`

Reads the settings files: lines of `key=value`. `read_settings` (`src/settings/read_settings.rs`)
reads the file and `apply_settings` stores each known key; `apply_feature_switches` applies the
switches that turn features on and off; `read_driver_names` reads the optional file of
computer drivers' names. See [Settings files](Settings-files).

## `threading`: the game's threads

Starts and ends the worker threads the original design has (input, audio, network).

| Item | File | What it does |
| --- | --- | --- |
| `start_threads` | `src/threading/start_threads.rs` | Starts them together at start-up. |
| `ThreadSlot`, `run_routine` | `src/threading/thread_slot.rs` | A thread's record, and what each runs. |
| `request_quit_with` | `src/threading/request_quit.rs` | Asks the threads to end. |
| `shutdown_thread`, `wait_for_threads` | files of the same names | Waits for them at the game's end. |

See [Threads and shared state](Threads-and-shared-state).

## The single files

| File | What it is |
| --- | --- |
| `src/owners.rs` | `Owners`: one reference to each module's owner, built once by `Owners::build` and passed down the call chain. |
| `src/standalone.rs` | Start-up and shutdown: `run` sets up the platform, builds the owners and walks the start-up steps to `system::main_loop`. |
| `src/bin/rc2k.rs` | `main`: picks the log's name and calls `standalone::run`. |
| `src/lib.rs` | The list of modules, and the crate-wide rule that unsafe code is denied outside the places that allow it one by one. |
| `src/remote.rs` | Only in builds with the `remote` feature: lets `tools/remote.py` drive the game (keys, pad, screenshots, held races) for automated runs. |
| `src/test_support/` | Only in tests: owners for a test thread, snapshots of the game's state, recorded vectors to compare against. |

## Related pages

[How the game runs](How-the-game-runs), [State and memory](State-and-memory),
[Code conventions](Code-conventions), [Testing](Testing).
