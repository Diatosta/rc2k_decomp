# The front end

*Category: [Game flow](Game-flow)*

"The front end" is everything the player sees outside a race: the main menu, the options, the
car and stage choices, the results and standings, the service area. This page explains how a
menu screen works, how the screens are strung together into events, and how the launch
switches reach a race without showing a menu.

Most of this lives in `src/menu/` (the screens and their flow), `src/screens/` (routines that
lay a screen out), `src/service/` (the service area) and `rc2k_core/src/menu/` (the rules, as
plain functions). The state is the `Menus` owner (`src/menu/owner.rs`).

## Getting there

`system::main_loop` calls `menu::run_front_end`. That function builds the fonts
(`gfx::build_fonts`), plays the films (`menu::play_films`: two logos, then the intro), runs the
main menu, and when the main menu returns it stores the players' names and frees the fonts. The
main menu returns only when the player has said yes to leaving the game.

## One loop for every screen

There is one menu loop, `screen_loop` in `src/menu/screen_loop.rs`, and every screen runs it.
What differs from screen to screen is data: a `Screen` value. Each screen's `Screen` is in its
own file, named after the screen with `_loop` at the end (`src/menu/options_loop.rs`,
`src/menu/select_stage_loop.rs`, `src/menu/championship_standings_loop.rs`, ...). `ScreenId`
(`src/menu/screen_id.rs`) names them all, and `run_screen(ScreenId::..., &entry)` runs one.

What the loop does:

1. **Entry.** The screen's `before` steps run once.
2. **Set-up.** The highlight comes back from where the screen left it last time; the screen's
   menu and its lists of drawing functions are put in place; the background picture is loaded;
   one frame is drawn. Set-up runs again after every action that does not leave the screen.
3. **Every frame.** The blink timer moves on (`menu::update_blink`); the platform handles its
   events and the frame is drawn; then `menu::handle_keys` takes one key:
   - an arrow key moves the highlight (`src/menu/navigation.rs`: each item names its
     neighbours, and items that cannot be chosen are stepped over);
   - Enter runs the highlighted item's action (`menu::run_item_action`);
   - Escape leaves the screen.
4. **Leaving.** A screen that asks first shows the "leave?" question
   (`menu::confirm_leaving`, which runs `menu::leave_question_loop`). The highlight is saved
   and the screen's `after` steps run.

### What a `Screen` says

| Field | Meaning |
| --- | --- |
| `menu` | Which menu (its items) the screen shows. |
| `saved_item` | Where the highlight is kept between visits. |
| `lists` | The lists of drawing functions the screen uses each frame. |
| `before`, `set_up_first`, `prepare`, `after_lists`, `background`, `after_first_frame` | Steps run at fixed points of entry and set-up. |
| `asking` | Present if leaving the screen is asked about. |
| `after`, `leaving`, `after_restored` | Steps run on the way out. |
| `keys`, `frames` | Per-screen differences in key handling and in what runs each frame. |
| `result` | What the loop returns to its caller, for screens that answer something. |

A step (`Step`) is a small thing to do: call a named game function, write a variable, set the
menu's sound, or run a function written out in the screen's own file (`Step::Do`). Reading a
screen's file top to bottom tells you everything that is special about it.

### Menus and items

The menus themselves are data. `rc2k_core::menu::tables` holds every menu and every item:
where the item is on the screen, its text, its font, which item is above, below, left and right
of it, and which function is its action. The game side keeps only what changes while playing
(an item's text, font, sprite and whether it is skipped) in `src/menu/menus_store.rs`.

An item's action is named in the table by an enum value; `src/menu/table_routes.rs` says which
Rust function each name runs. Typical actions:

- open another screen (the action calls `run_screen`, so screens nest);
- flip or step a setting (`menu::flip_gearing`, `menu::raise_sensitivity`, ...);
- start an event (`src/menu/start_event.rs`, `src/menu/arcade_levels.rs`);
- begin typing a name.

Because an action that opens a screen is a nested call, the menus form a tree of calls. Escape
on the inner screen returns to the outer one, which sets itself up again and carries on.

### Drawing a menu frame

`menu::draw_frame` draws a menu once: the background picture, then the menu's items
(`menu::draw_menu`), then the screen's draw list (`menu::run_draw_list`), a list of small
layout routines from `src/screens/`. Those routines place boxes, tables and captions at fixed
positions, using `src/text/` for strings and `src/gfx/` for pixels. Screens with a 3D part (the
car carousel on the main menu and the car choice) also draw a small scene; see
[Drawing a frame](Drawing-a-frame).

### Typing a name

There is one line of text entry, used for player names, saved-game names, replay names and
set-up names. Its state is in `src/menu/text_entry.rs`: the length so far, the longest allowed,
and a copy of the text as it was, which Escape brings back. Each use has a small editor that
runs once a frame while the entry is going on and takes the keys before the menu does:
`menu::edit_player_name`, `menu::edit_savegame_name`, `menu::edit_replay_name`,
`service::edit_setup_name`. Enter ends the entry and keeps the text.

## Events

An **event** is a whole competition: a championship, a single rally, a time trial or an arcade
game. Each is a function that walks through its screens and races in order. These functions
are the **event flows**.

The flows are written once, in `rc2k_core/src/menu/`, against a trait called `Game`
(`rc2k_core/src/menu/flow.rs`). `Game` is everything a flow needs from the outside: read and
write a variable, run a named screen or step, ask what a screen answered. In the program,
`menu::flow_game::Game` (`src/menu/flow_game.rs`) implements it on the real menus. In the
tests, a scripted `Game` plays the part of the player, so the order of screens in an event can
be tested without drawing anything.

| Event | Flow | Notes |
| --- | --- | --- |
| Single rally | `rounds::single_rally`, `single_rally_split`, `single_rally_two` .. `single_rally_four` | One function per number of players; `_split` is two players on one screen; `single_rally_split_many` (3 or 4) is the larger one, which the players menu's "3 players on one screen" / "4 players on one screen" items (`Seats::SplitThree`, `SplitFour`) start, and `--players 3` / `4` start. |
| Time trial | `rounds::time_trial_one`, `time_trial_split`, `time_trial_two` .. `time_trial_four` | Has its own service step before the stage (`menu::time_trial_service`). |
| Championship | `championship::run`, started through `starters::start` | The game-side driver is `src/menu/run_championship.rs`; it can also continue from a saved game (`saves::load_savegame`). |
| Arcade | `arcade_event::arcade_event` | Levels of six stages; a failed race retries the level. Game side: `src/menu/arcade_event.rs`, `arcade_steps.rs`. |

### The shape of an event

A single rally shows the pattern; the others add to it.

1. **Start.** The drivers and players are made ready (`menu::begin_event`): times cleared, cars
   repaired, set-ups at their defaults.
2. **Cars.** Each player chooses a car on the car screen (`src/menu/select_car_loop.rs`).
3. **Round after round:**
   - the stage is chosen or shown (`src/menu/select_stage_loop.rs`,
     `src/menu/next_stage_loop.rs`);
   - `menu::run_event_races` makes the computer drivers' times (`menu::calc_cpu_times`) and
     runs the race for each player in turn (see [A race, step by step](A-race-step-by-step));
   - the results pages are shown (`src/menu/overall_stage_results_loop.rs` and its relatives);
   - best times are checked and entered (`src/menu/keep_stage_best.rs`,
     `src/menu/enter_record_entry.rs`).
4. **End.** A "yes" to leaving, or the last stage, ends the event and the flow returns to the
   menu it was started from.

A championship is divided into sections. For each section the flow walks a list of records and
does what each record's kind says: a welcome page, a stage, a service, the section's standings
and points, or the championship's end. After a stage come the inspection
(`menu::inspection_pass`: a car too damaged is out) and a review menu from which the player can
save the championship (`src/menu/save_championship_loop.rs`).

### The service area

Between stages the car can be repaired and its set-up changed, against a budget of service
time. `menu::service_pass` runs it for each player: the inspection's screen, then the service
page, `service::service_area_loop` (it is an ordinary screen: `menu::screen_loop` with the
service area's own `Screen`).

On the page the player marks repairs (`service::toggle_repair_choice`; the total is kept by
`service::update_repair_total`) and edits six values: tyres, gear ratio, ride height,
suspension stiffness, brake balance and steering sensitivity (`src/service/pages.rs` and the
`edit_*` and `step_*` files). The first four cost service time; the last two change at once.
A set-up can be saved to and loaded from a slot (`service::write_setup_slot`,
`service::read_setup_slot`; see [Car setups](Car-setups)).

On leaving, `service::apply_repairs` carries the chosen work out, showing each repair being
done. What the set-up values do to the car is described in [Car physics](Car-physics).

## Starting a race from the command line

`rc2k --stage langton --car felicia` starts a single rally without showing a menu; `--mode`
starts a time trial, the arcade game or a new championship the same way. The switches
are listed in [Running and options](Running-and-options). Here is the path they take:

1. `rc2k_core::menu::launch` reads the switches (and the `--launch` file, if given) into a
   `Launch` value. `menu::launch::asked` reads it once and keeps it.
2. `menu::run_front_end` sees that a launch was asked for and skips the films.
3. The main menu's set-up has a step, `menu::launch::run_once`, that runs the launch the first
   time the main menu is set up.
4. That step matches the names against the game's stages, cars and gearboxes. Names are
   compared in English whatever language the game runs in, without case, spaces or punctuation;
   a whole name wins over a part of one. A name that fits nothing, or several things, ends the
   program with a message that lists the choices.
5. It then runs the ordinary flow of the event (`--mode`: `rounds::single_rally` or
   `single_rally_split`, `rounds::time_trial_one` or `time_trial_split`) on a wrapper around the
   menus' `Game`, called `Launched`. The wrapper answers the car choices and the stage choice
   (the single rally's or the time trial's stage screen) from the launch the first time the flow
   asks, leaving exactly what the car and stage screens leave when a choice is confirmed.
   Everything else passes through to the real menus. The time trial's service area is answered
   once (`menu::time_trial_service::time_trial_service_unshown`): each player's figures and
   entry kind as the area leaves them, with the launch's tune put into the car first
   (`menu::launch::apply_tune`, through `service::apply_repairs` run on a clock that does not
   wait).
   The arcade (`--mode arcade`) starts the level as its item on the level screen does
   (`menu::arcade_event`), and a championship through its start menu's starter
   (`menu::start_event`); their flows are not `flow::Game`s, so each takes the launch's answer
   to the car choice as a closure (`menu::launch::CarAnswer`), and their stages stay the
   event's own.
6. With `--seed`, the random numbers are reseeded as the stage begins to load
   (`menu::launch::stage_loading`).
7. After the race the flow goes on as usual: results, then the menus. With `--then quit` the
   program leaves through its normal save-and-close instead (for the arcade and the
   championships, at the end of the first race: `menu::launch::race_over`, called by
   `race::single_rally_race`).

So a launched race is the menus' own race. Nothing about the race, its results or the menus
after it is a special case.

`rc2k --replay 0` takes the same path up to step 3; the step then runs the replay viewer
(`race::replay_viewer::launched_replay`) as the replays menu's item runs it, with the slot
given instead of chosen on the loading screen, and plays the replay at once. A slot is checked
first (the file's head and index, as the viewer checks them when it opens one); a file given by
its path gets a slot number of its own (`rc2k_core::race::viewer::GIVEN_FILE`) that
`race::replay_path` turns into that path. With `--then quit` the viewer returns when the play
stops and the program leaves through its normal save-and-close.

## Where to look

| To change or understand | Start in |
| --- | --- |
| What is special about one screen | its `src/menu/..._loop.rs` |
| The loop itself, `Screen`, `Step` | `src/menu/screen_loop.rs` |
| A menu's items, positions, links | `rc2k_core/src/menu/tables.rs` (data in `tables_data.rs`), `src/menu/menus_store.rs` |
| Which function an item runs | `src/menu/table_routes.rs` |
| Keys on a menu | `src/menu/handle_keys.rs`, `src/menu/navigation.rs` |
| The order of screens in an event | `rc2k_core/src/menu/rounds.rs`, `championship.rs`, `arcade_event.rs` |
| What a flow can ask of the game | `rc2k_core/src/menu/flow.rs`, `src/menu/flow_game.rs` |
| Results, points and standings | `src/menu/standings.rs`, `award_stage_points.rs`, `rc2k_core/src/menu/standings.rs` |
| The service area | `src/service/`, `src/menu/service_pass.rs` |
| The launch switches | `rc2k_core/src/menu/launch.rs`, `src/menu/launch.rs` |
| Screen layouts | `src/screens/` |

Next: [A race, step by step](A-race-step-by-step). See also [Module-menu](Module-menu),
[Module-screens](Module-screens), [Module-service](Module-service),
[Saved games](Saved-games), [Best times](Best-times).
