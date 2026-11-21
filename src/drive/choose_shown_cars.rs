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
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

/// Chooses from the race's variables (the views, the players, the replay's layout, the split)
/// and writes the shown cars, the views' count, the split and the views' blocks.
fn choose_at(
    race: &Race,
    network: bool,
    listed: &mut [u32; 8],
    players: &[usize],
    heard_cars: u32,
) -> Result<(), Refused> {
    let (view, vars) = (race.shared.view.get(), race.shared.vars.get());
    let mut player_cars = [0u32; 8];
    for (to, &car) in player_cars.iter_mut().zip(players) {
        *to = car as u32;
    }
    let setup = ShownSetup {
        player_cars,
        player_count: players.len().min(8) as u32,
        players: view.players.get(),
        replay_mode: race.course.replay_mode(),
        layout: view.view_switch.get(),
        network: u32::from(network),
        local_car: view.local_car.get(),
        heard_cars,
    };
    let split = vars.split.get();
    let mut shown = ShownCars {
        split,
        ..ShownCars::default()
    };
    shown.choose(&setup, 0)?;
    view.view_cars[0].set(shown.cars[0]);
    view.view_cars[1].set(shown.cars[1]);
    view.views.set(shown.views);
    if shown.split != split {
        vars.split.set(shown.split);
    }
    // The views' blocks kept by handle (`blocks_store::block_ref`).
    // (the cars are below eight: `choose` refuses a network car above 7)
    view.view_blocks[0].set(crate::race::blocks_store::BLOCK_TAG | shown.cars[0]);
    view.view_blocks[1].set(crate::race::blocks_store::BLOCK_TAG | shown.cars[1]);
    *listed = shown.listed;
    Ok(())
}

/// Chooses the shown cars from the race's players, replay layout and network game.
pub fn choose_shown_cars_in(race: &Race, net: &crate::net::owner::Net) -> Result<(), Refused> {
    let network = net.race_slot.is_on();
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
    choose_at(race, network, &mut listed, &players, heard)?;
    race.frame.frame_work.borrow_mut().listed_car_sounds = listed;
    Ok(())
}
