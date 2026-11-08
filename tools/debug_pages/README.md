# The debug UI's display page, from IDA (2026-10-30)

How `src/debug/display_rows.rs` was made from `debug::draw_wear_bars` (0x429B19):

1. `parse_wear.py` (IDA, `py_exec_file`): the function's 9,204 instructions into its 875
   items (bar, label, figure, surface name) in `wear_items.json`; every instruction matched.
2. `dump_rows.py`, `names.py` (IDA): each figure with its label, place and IDA operand name.
3. `groups.py` (plain Python): the names and groups written by hand, duplicates dropped,
   the Rust table written.

The IDA scripts read and write their JSON in the folder named by `RC2K_DEBUG_PAGES` (else
IDA's working folder); `groups.py` runs from the repository's root, with the JSON files there.
