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
use crate::memory::session::{DwordLike, PlainDword};
use crate::race::owner::Race;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

fn get(v: &[&dyn DwordLike], k: usize) -> u32 {
    v[k].get()
}

fn put(v: &[&dyn DwordLike], k: usize, x: u32) {
    v[k].set(x);
}

/// On the variables at `v` (`game`'s order).
fn choose_at(
    v: &[&dyn DwordLike],
    listed: &mut [u32; 8],
    players: &[usize],
    heard_cars: u32,
) -> Result<(), Refused> {
    let mut player_cars = [0u32; 8];
    for (to, &car) in player_cars.iter_mut().zip(players) {
        *to = car as u32;
    }
    let setup = ShownSetup {
        player_cars,
        player_count: players.len().min(8) as u32,
        players: get(v, 2),
        replay_mode: get(v, 4),
        layout: get(v, 5),
        network: get(v, 7),
        local_car: get(v, 8),
        heard_cars,
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
    *listed = shown.listed;
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
        &race.course.number_vars.get().replay_mode,
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
pub fn choose_shown_cars_in(race: &Race, net: &crate::net::owner::Net) -> Result<(), Refused> {
    let network = PlainDword::new(u32::from(net.race_slot.is_on()));
    // A replay of the own format names players in its layouts (`rc2k_core::race::replay_v2`).
    let (players, heard) = if race.viewer_buffers().v2.is_set() {
        let v2 = race.replay.v2().lock();
        let ext = v2.ext.as_ref();
        (
            ext.map(rc2k_core::race::replay_v2::Extension::player_cars)
                .unwrap_or_default(),
            // Only while the sound objects are set up for every car.
            ext.and_then(rc2k_core::race::replay_v2::Extension::heard_kinds)
                .filter(|_| crate::race::heard_cars::is_on(&race.shared))
                .map_or(0, |kinds| kinds.len() as u32),
        )
    } else {
        (Vec::new(), 0)
    };
    let mut listed = race.frame.frame_work.borrow().listed_car_sounds;
    choose_at(&game(race, &network), &mut listed, &players, heard)?;
    race.frame.frame_work.borrow_mut().listed_car_sounds = listed;
    Ok(())
}
