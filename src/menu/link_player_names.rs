//! `menu::link_player_names` (0x47059E): hangs each player's name on that player's car.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players (`g_cars`): its +0x18 = the address of the player's
//! name in `g_playerNames`, and the name's 0x20 bytes copied to its +0x124.
//! Callers: `saves::load_savegame` (twice) and the main menu's set-up `sub_47C63F`. Refuses a
//! missing player. Rust callers run its `entry` (`routes.rs`); the registers above are the
//! original's, for reference.

use crate::errors::BadInput;
use crate::menu::player_names::{CAR_NAME, CAR_NAME_POINTER, cars, name, name_bytes};
use crate::pointers::GamePtr;

// ---- Rust version ----

pub fn link_player_names() -> Result<(), BadInput> {
    for (player, car) in cars()?.into_iter().enumerate() {
        GamePtr::new(car + CAR_NAME_POINTER as u32)
            .write::<u32>(name(player), "a player's name pointer")?;
        GamePtr::new(car + CAR_NAME as u32)
            .write_bytes(&name_bytes(player), "a player's name copy")?;
    }
    Ok(())
}
