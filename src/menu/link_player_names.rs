//! `menu::link_player_names` (0x47059E): hangs each player's name on that player's car.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players (`g_cars`): its +0x18 = the player's name (in
//! `g_playerNames`, kept as the player's number), and the name's 0x20 bytes copied to its +0x124.
//! Callers: `saves::load_savegame` (twice) and the main menu's set-up `sub_47C63F`. Refuses a
//! missing player. Rust callers run its `entry` (`routes.rs`); the registers above are the
//! original's, for reference.

use crate::errors::BadInput;
use crate::menu::owner::menus;
use crate::menu::player_names::{cars, name_bytes};

// ---- Rust version ----

pub fn link_player_names() -> Result<(), BadInput> {
    let play = &menus().play;
    for (player, car) in cars(play).iter().enumerate() {
        car.update(|p| {
            p.set_name_player(player);
            p.name_copy = name_bytes(player);
        });
    }
    Ok(())
}
