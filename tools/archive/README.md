# Archived tools

Scripts of finished conversion phases. They read or rewrite things that are gone (the hook
framework, `src/moved.rs`, `src/pointers.rs`, `src/address_map.rs`, `data/ral_image.bin`,
`GamePtr`, `game_addr`) or generated a file that was typed by hand afterwards, so
running them would undo later work. Kept for the history of how the code was made; none runs
on the current tree. Paths below are relative to `tools/archive/`.

- `extract_image/` - cut RAL.EXE's non-code sections into `data/ral_image.bin` (phase 5, step B); the image is deleted.
- `phase7_fields/` - typed the game's loose variables into stores, fields and constants from IDA names and RVAs (phase 7, 2026-10-25); the address layer it targets is deleted.
- `phase13_runs/` - rewrote the x87 runs, cell lists and race-block tests onto typed cells (phase 13, 2026-10-28); its patterns match `pointers::host` / `GamePtr`, gone.
- `effects_recut.py` - recut the effects' ranges by IDA address (marks / effects, phase 7); read `moved.rs`.
- `frame_census.py` - census of the frame work's places from `moved::MOVED` (F-steps); `moved.rs` is deleted.
- `debug_pages/` - built `debug::display_rows` / `watch_rows` from the debug overlay's IDA lists (2026-10-30); `typed_rows.py` rewrote rows by address, and the rows are typed by hand since.
- `screen_loops/` - wrote the hook file of each menu screen loop from IDA (hook era; emits `game_addr`).
- `clip_asm/` - wrote the clippers' x87 asm into `scene/clip_asm.rs` (phase 7); the asm is now plain Rust.
- `texture_catalogue/` - wrote `scene/texture_steps.rs` from IDA; hand-typed since.
- `carousel_set_ups/` - wrote `scene/carousel_set_ups.rs` from IDA; hand-typed since (its own header says so).
- `strip_hook.py`, `remove_dead.py` - phase 4: cut a hook's machinery from a file, then the unused items clippy reported; the hooks are gone.
- `callgraph/` - call graph of `race()` accessor calls to plan passing `&Race` down (M6 planning); done.
- `menu_tables/add_entries.py` - gave hook files an `entry` function for shared dispatchers (phase 5 typed tables); no dispatchers remain. (`tools/menu_tables/{extract,gen}.py` stay: they generate `rc2k_core/src/menu/tables_data.rs`.)
- `m7_tests/stores_list.py`, `codemod.py`, `t5fx_codemod.py` - M7 step 5 code mods (store lists, tests off old addresses); applied, they read the deleted layer.
