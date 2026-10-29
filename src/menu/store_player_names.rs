//! `menu::store_player_names` (0x470649): copies each player's name into that player's car
//! and writes it to the game's log.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players: the 0x20 bytes of the player's name to its +0x124, and that copy
//! as a line of the log (`logfile::log_write_l1_nl`). A name that fills its 0x20 bytes is logged
//! with what follows it in the player. One caller, `sub_47486C`, after the main menu is left.
//! Refuses a missing player.

use crate::errors::BadInput;
use crate::menu::player_names::{CAR_NAME, cars, name_bytes};
use crate::pointers::GamePtr;

// ---- Rust version ----

/// `log` gets each line's text.
pub fn store_player_names(log: &mut dyn FnMut(&[u8])) -> Result<(), BadInput> {
    for (player, car) in cars()?.into_iter().enumerate() {
        let copy = car + CAR_NAME as u32;
        GamePtr::new(copy).write_bytes(&name_bytes(player), "a player's name copy")?;
        log(&crate::files::cstring(
            GamePtr::new(copy),
            "a player's name copy",
        )?);
    }
    Ok(())
}
