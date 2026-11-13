//! `menu::link_player_names`: hangs each player's name on that player's car.
//!
//! For each of the four players (the cars): its +0x18 = the player's name (in
//! the player names, kept as the player's number), and the name's 0x20 bytes copied to its +0x124.
//! Callers: `saves::load_savegame` (twice) and the main menu's set-up `menu::set_up_front_end`.
//! Refuses a missing player. Rust callers run its `entry` (`routes.rs`).

use crate::errors::BadInput;
use crate::menu::owner::Menus;
use crate::menu::player_names::{cars, name_bytes};

pub fn link_player_names(menus: &Menus) -> Result<(), BadInput> {
    let play = &menus.play;
    for (player, car) in cars(play).iter().enumerate() {
        car.update(|p| {
            p.set_name_player(player);
            p.name_copy = name_bytes(menus, player);
        });
    }
    Ok(())
}
