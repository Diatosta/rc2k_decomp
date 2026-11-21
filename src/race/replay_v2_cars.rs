//! The cars of a replay of the own format (`rc2k_core::race::replay_v2`), made when a race's
//! recording begins: every car of the race with its model, its driver's name, whether a player
//! or the computer drives it, and how it is recorded. In a network race the cars of the other
//! machines (their players and the computer drivers) are recorded by their motion at each step,
//! as this machine showed them; every car this machine drives keeps its inputs and its state at
//! each second.

use crate::net::owner::Net;
use crate::race::owner::Race;
use rc2k_core::race::replay_v2::{CarEntry, Extension, MAX_CARS, NAME_LEN, flag};

/// A name cut at its end mark and held to the file's room.
fn held(name: &[u8]) -> Vec<u8> {
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    name[..end.min(NAME_LEN - 1)].to_vec()
}

/// The race's cars: the players' (local ones named by the front end, the network's by their
/// records), then the computer's.
pub(super) fn cars_of_race(race: &Race, net: &Net, menus: &crate::menu::owner::Menus) -> Extension {
    let view = race.shared.view.get();
    let cars = (view.race_cars.get() as usize).clamp(1, MAX_CARS);
    let network = net.race_slot.is_on();
    let players = if network {
        net.race_slot.players() as usize
    } else {
        view.players.get() as usize
    };
    let mut entries = Vec::with_capacity(cars);
    for car in 0..cars {
        let kind = race.shared.vars.get().race_car_kinds[car].get();
        let player = car < players;
        let (flags, name) = if player && network {
            let own = car as u32 == net.race_slot.player();
            let name = net
                .vars
                .player_block(car as u32)
                .map(|block| held(&block.name.read()))
                .unwrap_or_default();
            let flags = flag::PLAYER
                | if own {
                    flag::LOCAL | flag::OWN
                } else {
                    flag::STATES
                };
            (flags, name)
        } else if player {
            let name = held(&crate::menu::player_names::name_bytes(menus, car));
            let own = if car == 0 { flag::OWN } else { 0 };
            (flag::PLAYER | flag::LOCAL | own, name)
        } else {
            let flags = flag::COMPUTER | if network { flag::STATES } else { flag::LOCAL };
            (
                flags,
                format!("Computer {}", car + 1 - players).into_bytes(),
            )
        };
        entries.push(CarEntry { kind, flags, name });
    }
    Extension { cars: entries }
}
