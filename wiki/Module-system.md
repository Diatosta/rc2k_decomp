# Module: system

*Category: [Module reference](Modules)*

The program's outer shell and its small services: the main loop that chooses between the
installer and the game, the command line, the clocks, sleeping and waiting, and what the game
wants to know about the machine it runs on.

## State

The owner is `System` (`src/system/owner.rs`): session flags (quitting, whether the window is
active), start-up values, the clocks, facts about the machine and processor, and the file work
buffers that [`files`](Module-files) uses.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `main_loop` | `src/system/main_loop.rs` | Runs the installer or the front end, then saves the game's data. |
| `read_command_line` | `src/system/read_command_line.rs` | The game's own command-line switches. |
| `load_game_data` | `src/system/load_game_data.rs` | Loads options, best times and the lists of saved games at start-up. |
| `test_processor` | `src/system/test_processor.rs` | Notes what the processor is. |
| `tick_count` | `src/system/tick_count.rs` | The millisecond clock. |
| `sleep` | `src/system/sleep.rs` | Sleeps. |
| `pause` | `src/system/pause.rs` | The screens' fixed pauses: the window's events go on meanwhile, and a key ends one early (`pause` takes the key, `pause_keeping_key` leaves it for the menu, `hold` ignores keys). |
| `wait_or_key` | `src/system/wait_or_key.rs` | Waits for a time or a key, whichever is first. |
| `packed_local_time` | `src/system/packed_local_time.rs` | The date and time as one number (as stored in replays and saves). |
| `checked_reads.rs` | `src/system/checked_reads.rs` | Bounds-checked reads used by loaders. |

## How it is laid out

A small flat folder, one function per file. Several files keep services the 1999 program
asked Windows for (`read_windows_version`, the processor's test); here they
answer from the standard library or with fixed values. What only served 1999's machines (the
error mode, the process's priority, the drive's serial, the search for a running copy, for 3D
cards and for a PC-98 keyboard) is gone.

## Calls and callers

It calls [`platform`](Module-platform), [`install`](Module-install), [`saves`](Module-saves),
[`menu`](Module-menu) and [`strings`](Module-strings). Its clocks and flags are read almost
everywhere; `main_loop` is called from start-up (`src/standalone.rs`).

## Related pages

[How the game runs](How-the-game-runs), [Running and options](Running-and-options),
[Threads and shared state](Threads-and-shared-state).
