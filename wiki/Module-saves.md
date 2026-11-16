# Module: saves

*Category: [Module reference](Modules)*

What the game keeps between sessions: the options, the best times and scores, the saved
championships, and the lists of saved games and replays the menus show. It reads these files
at start-up and writes them when the player leaves, saves a game, or beats a time.

## State

The owner is `Saves` (`src/saves/owner.rs`): the best-times tables (`best_times_store.rs`),
the file buffers, and the preview of a saved game shown on the load screen (`preview_store.rs`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `load_options`, `save_options` | files of the same names | The options: read at start-up, written on leaving. |
| `load`, `save`, `load_old`, `save_old` | `src/saves/options_files.rs` | The two options files: this version's own, and the original's. |
| `capture`, `apply` | `src/saves/options_state.rs` | Copies the options between the game's variables and the saved model. |
| `load_game_best_times` | `src/saves/load_best_times.rs` | Reads the best times, or fills in the defaults. |
| `save_best_times.rs` | `src/saves/save_best_times.rs` | Writes them. |
| `default_*.rs` | `src/saves/default_stage_times.rs` ... | The times and scores a fresh install starts with. |
| `gather_savegame`, `write_savegame_slot` | files of the same names | Collects a championship's state and writes it into a slot. |
| `read_savegame_slot_in`, `load_savegame` | `src/saves/read_savegame_slot.rs`, `src/saves/load_savegame.rs` | Reads a slot; puts a saved championship back. |
| `read_replay_slot` | `src/saves/read_replay_slot.rs` | Reads a saved replay's name and details for the list. |
| `save_game_data` | `src/saves/save_game_data.rs` | Everything saved when the game is left. |

## How it is laid out

A small flat folder, one function per file. The layouts of the files and the typed tables are
in `rc2k_core/src/saves/` and `rc2k_core/src/options/`. `golden/` holds expected bytes for
tests.

## Calls and callers

It reads and writes through [`files`](Module-files) and copies to and from
[`menu`](Module-menu)'s state. It is called by `menu`, [`screens`](Module-screens) and
`system` (start-up and leaving).

## Related pages

[Options files](Options-files), [Best times](Best-times), [Saved games](Saved-games),
[Replay files](Replay-files).
