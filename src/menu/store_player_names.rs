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
use crate::menu::player_names::{cars, name_bytes};

// ---- Rust version ----

/// `log` gets each line's text.
pub fn store_player_names(log: &mut dyn FnMut(&[u8])) -> Result<(), BadInput> {
    for (player, car) in cars().iter().enumerate() {
        let name = name_bytes(player);
        car.update(|p| p.name_copy = name);
        // Logged up to its NUL, read on past the copy's room as the original reads.
        let bytes = car.get().to_le_bytes();
        let copy = &bytes[std::mem::offset_of!(rc2k_core::menu::player::Player, name_copy)..];
        let end = copy.iter().position(|&b| b == 0).unwrap_or(copy.len());
        log(&copy[..end]);
    }
    Ok(())
}
