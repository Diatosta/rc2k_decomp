# Module: menu

*Category: [Module reference](Modules)*

The front end: everything between the opening films and a race. It holds the loop every menu
screen runs, a description of each screen, the handling of keys and menu items, the players'
details, and the **event flows**, the functions that walk through a championship, a single
rally, a time trial or an arcade game screen by screen and race by race. It is the largest
module by number of files.

## State

The owner is `Menus` (`src/menu/owner.rs`): the screens' variables, the players, the
championship's standings, results, cheats, the car and stage lists. Its parts are the many
`*_store.rs` files (`players_store.rs`, `champ_store.rs`, `results_store.rs`, `menus_store.rs`
for the menus' item tables, `texts_store.rs` ...). The service area's state is a field of it
(`Menus::service`).

## Main entry points

| Item | File | What it does |
| --- | --- | --- |
| `run_front_end` | `src/menu/run_front_end.rs` | Fonts, films, then the main menu, until the player leaves the game. |
| `play_films` | `src/menu/play_films.rs` | The opening films. |
| `screen_loop`, `Screen` | `src/menu/screen_loop.rs` | The loop every screen runs, and the description of a screen. |
| `..._loop.rs` | e.g. `src/menu/main_menu_loop.rs`, `options_loop.rs`, `select_car_loop.rs` | One file per screen: what is particular to it. |
| `handle_keys` | `src/menu/handle_keys.rs` | Reads a key and moves the highlight or runs an item. |
| `run_item_action` | `src/menu/run_item_action.rs` | Runs the highlighted item's action. |
| `draw_menu` | `src/menu/draw_menu.rs` | Draws a screen's menu. |
| `start_event` | `src/menu/start_event.rs` | Sets up and starts a championship. |
| `run_championship_in` | `src/menu/run_championship.rs` | A championship from first rally to last. |
| `arcade_event` | `src/menu/arcade_event.rs` | The arcade game. |
| `run_event_races` | `src/menu/run_event_races.rs` | One round of races, each player in turn. |
| `Game` | `src/menu/flow_game.rs` | The game's side of the event flows whose logic is in `rc2k_core::menu::flow`. |
| `asked`, `run_once` | `src/menu/launch.rs` | The launch switches (`--stage` and the rest, `--replay`): starting straight into a race or a replay. |
| `confirm_leaving` | `src/menu/confirm_leaving.rs` | The "leave?" question. |
| `MenuId` | `src/menu/menus_store.rs` | A menu's item table, by name. |

## How it is laid out

Flat, one function per file. `*_loop.rs` is a screen; `*_store.rs` is state; `show_*` and
`*_pass.rs` are the results and standings sequences; `edit_*` and `begin_*_name` are text
entry; `sort_standings_*` and `award_*` are the championship's arithmetic. `screen_loop/`
holds the loop's tests.

## Calls and callers

It draws with [`screens`](Module-screens), [`gfx`](Module-gfx) and [`text`](Module-text), reads
keys from [`input`](Module-input), saves through [`saves`](Module-saves), and starts races in
[`race`](Module-race). It is entered from `system::main_loop`.

## Related pages

[The front end](The-front-end), [How the game runs](How-the-game-runs),
[Running and options](Running-and-options), [Built-in tables](Built-in-tables).
