# Module: screens

*Category: [Module reference](Modules)*

The routines that lay out the parts of a menu screen: where each piece of text and each box
goes, in which font, with which text. The [`menu`](Module-menu) module decides which screen is
shown and what its items do; this module draws what the screen shows around its menu, such as
a stage's details, a results table, the service area's boxes or the list of saved games.

## State

No owner of its own. It reads the menus' state (`Menus`), the fonts in `Gfx` and the texts in
`Strings`, and keeps a few text buffers and the controls page's values
(`src/screens/text_buffers_store.rs`, `src/screens/controls_store.rs`), which are parts of
`Menus`.

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `draw_menu_items`, `draw_menu_labels` | files of the same names | A menu's items and fixed labels. |
| `draw_stage_info` | `src/screens/draw_stage_info.rs` | A stage's details: length, surface, records. |
| `draw_stage_body` | `src/screens/stage_panel.rs` | The panel the stage screens share. |
| `draw_championship_standings` | file of the same name | The championship table. |
| `draw_players_results.rs`, `draw_player_stage_result.rs` | `src/screens/` | Results after a stage. |
| `draw_service_area` | `src/screens/draw_service_area.rs` | The service area's boxes, repair times and set-up markers. |
| `draw_control_bindings` | file of the same name | The controls screen's list of keys. |
| `draw_savegame_names`, `draw_savegame_preview` | files of the same names | The load and save screens. |
| `draw_message_box`, `draw_leave_question` | files of the same names | Boxes over a screen. |
| `GameFont`, `GameText`, `GameWork` | `src/screens/game_screen.rs` | How the layout rules in `rc2k_core::screens` reach the game's fonts, texts and drawing. |

## How it is laid out

Flat, mostly `draw_*.rs`, one drawing routine per file. The positions and rules themselves are
plain functions in `rc2k_core/src/screens/`; a file here usually gathers the values and lets
the rule draw through the traits in `game_screen.rs`.

## Calls and callers

It draws with [`text`](Module-text) and [`gfx`](Module-gfx), gets texts from
[`strings`](Module-strings) and stage records from `records`. It is called by `menu` (through
the screens' draw lists) and [`service`](Module-service).

## Related pages

[The front end](The-front-end), [Fonts](Fonts), [Language files](Language-files).
