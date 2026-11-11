//! `drive::choose_shown_cars` (0x4560EA): which car blocks the two views show (`view_blocks`,
//! `off_63DBB4` = 0x70F3EC + n 0xFE0 for `view_cars`, `dword_63DBAC`), `views` (the
//! views' number, from `players`, by `view_switch` when `replay_mode` is 2, also setting
//! `split`), this machine's car in a network game (`session`, `g_netSession`, set; the car
//! `local_car`, set by `net::set_up_race` from the network player's number; then the second
//! view 0), and the eight of `listed_car_sounds` (each car's device, -1 none) all -1 but the shown ones'
//! 0 (the first, and the second when there are two views). The layout `view_switch` (1 .. 3) is
//! the replay frame head's byte +5 (`drive::replay_cameras`), used while a replay plays
//! (`replay_mode` 2): 1 one view of car 1, 2 two views side by side, 3 two views split the other
//! way (`split` the split, 1 = side by side, `race::draw_race_scene`).
//!
//! Callers (5): race init's loop (`drive::race_cars_step`), `race::step_replay_cars`,
//! `drive::replay_step`, `race::race_keys`,
//! `race::draw_race_scene`. Refuses (before anything) a network car (`local_car`) above 7 with
//! `session` set (it would write past `listed_car_sounds`'s eight).

use crate::errors::Refused;
use crate::net::owner::net;
use crate::race::owner::Race;
use crate::session::Dword;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

fn get(v: &[&Dword], k: usize) -> u32 {
    v[k].get()
}

fn put(v: &[&Dword], k: usize, x: u32) {
    v[k].set(x);
}

/// On the variables at `v` (`game`'s order).
// Converted: `rc2k_core::drive::shown_cars::ShownCars::choose`.
pub fn choose_at(v: &[&Dword]) -> Result<(), Refused> {
    let setup = ShownSetup {
        players: get(v, 2),
        replay_mode: get(v, 4),
        layout: get(v, 5),
        network: get(v, 7),
        local_car: get(v, 8),
    };
    let split = get(v, 6);
    let mut shown = ShownCars {
        split,
        ..ShownCars::default()
    };
    shown.choose(&setup, crate::race::blocks_store::block_addr(0))?;
    put(v, 0, shown.cars[0]);
    put(v, 1, shown.cars[1]);
    put(v, 3, shown.views);
    if shown.split != split {
        put(v, 6, shown.split);
    }
    // The views' blocks kept by handle (`blocks_store::block_ref`).
    put(v, 9, crate::race::blocks_store::handle_of(shown.blocks[0]));
    put(v, 10, crate::race::blocks_store::handle_of(shown.blocks[1]));
    for (k, &x) in shown.listed.iter().enumerate() {
        put(v, 11 + k, x);
    }
    Ok(())
}

/// The variables' addresses, in the order the core uses them.
fn game(race: &Race) -> Vec<&Dword> {
    let shared = &race.shared;
    let mut cells: Vec<&Dword> = vec![
        &shared.view.view_cars[0],
        &shared.view.view_cars[1],
        &shared.view.players,
        &shared.view.views,
        &race.course.numbers.replay_mode,
        &shared.view.view_switch,
        &shared.vars.split,
        &net().vars.session, // dword_52A910: in the network's variables
        &shared.view.local_car,
        &shared.view.view_blocks[0],
        &shared.view.view_blocks[1],
    ];
    cells.extend(race.frame.frame_work.listed_car_sounds.iter());
    cells
}

pub fn choose_shown_cars(race: &Race) -> Result<(), Refused> {
    choose_at(&game(race))
}
