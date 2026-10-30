# The debug UI's display page, from IDA (2026-10-30)

How `src/debug/display_rows.rs` was made from `debug::draw_wear_bars` (0x429B19):

1. `parse_wear.py` (IDA, `py_exec_file`): the function's 9,204 instructions into its 875
   items (bar, label, figure, surface name) in `wear_items.json`; every instruction matched.
2. `dump_rows.py`, `names.py` (IDA): each figure with its label, place and IDA operand name.
3. `groups.py` (plain Python): the names and groups written by hand, duplicates dropped,
   the Rust table written.

The scripts' `D` / output paths were a session scratchpad: change them to run again.
