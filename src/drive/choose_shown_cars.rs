//! Which car blocks the two views show, `views` (the views' number, from `players`, by
//! `view_switch` when the replay plays, also setting `split`), this machine's car in a network
//! game (`local_car`, set by `net::set_up_race` from the network player's number; then the
//! second view 0), and the eight car sound slots all -1 but the shown ones' 0 (the first, and
//! the second when there are two views). The layout `view_switch` (1 .. 3) is the replay frame
//! head's byte +5 (`drive::replay_cameras`), used while a replay plays: 1 one view of car 1, 2
//! two views side by side, 3 two views split the other way (`split`, 1 = side by side,
//! `race::draw_race_scene`).
//!
//! Refuses (before anything) a network car above 7 with a session set (it would write past the
//! eight slots).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::session::{DwordLike, PlainDword};
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

fn get(v: &[&dyn DwordLike], k: usize) -> u32 {
    v[k].get()
}

fn put(v: &[&dyn DwordLike], k: usize, x: u32) {
    v[k].set(x);
}

/// On the variables at `v` (`game`'s order).
pub fn choose_at(v: &[&dyn DwordLike], listed: &[PlainDword]) -> Result<(), Refused> {
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
    shown.choose(&setup, 0)?;
    put(v, 0, shown.cars[0]);
    put(v, 1, shown.cars[1]);
    put(v, 3, shown.views);
    if shown.split != split {
        put(v, 6, shown.split);
    }
    // The views' blocks kept by handle (`blocks_store::block_ref`).
    // (the cars are below eight: `choose` refuses a network car above 7)
    put(v, 9, crate::race::blocks_store::BLOCK_TAG | shown.cars[0]);
    put(v, 10, crate::race::blocks_store::BLOCK_TAG | shown.cars[1]);
    for (k, &x) in shown.listed.iter().enumerate() {
        listed[k].set(x);
    }
    Ok(())
}

/// The variables, in the order the core uses them (`listed_car_sounds` apart).
fn game<'a>(race: &'a Race, network: &'a PlainDword) -> Vec<&'a dyn DwordLike> {
    let shared = &race.shared;
    let cells: [&dyn DwordLike; 11] = [
        &shared.view.get().view_cars[0],
        &shared.view.get().view_cars[1],
        &shared.view.get().players,
        &shared.view.get().views,
        &race.course.numbers.get().replay_mode,
        &shared.view.get().view_switch,
        &shared.vars.get().split,
        network,
        &shared.view.get().local_car,
        &shared.view.get().view_blocks[0],
        &shared.view.get().view_blocks[1],
    ];
    cells.to_vec()
}

/// Chooses the shown cars from the race's players, replay layout and network game.
pub fn choose_shown_cars_in(race: &Race) -> Result<(), Refused> {
    let network = PlainDword::new(u32::from(race.shared.net_race.is_on()));
    choose_at(
        &game(race, &network),
        &race.frame.frame_work.get().listed_car_sounds,
    )
}
