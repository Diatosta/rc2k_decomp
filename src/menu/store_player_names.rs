//! `menu::store_player_names` (0x470649): copies each player's name into that player's car
//! and writes it to the game's log.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players: the 0x20 bytes of the player's name to its +0x124,
//! and that copy as a line of the log (`logfile::log_write_l1_nl`). A name that fills its
//! 0x20 bytes is logged with what follows it in the player.
//! One caller, `sub_47486C`, after the main menu is left.
//! Shadow mode: the original first, then the Rust version without the log lines on the
//! cars put back. Self-test: a redirected copy whose log calls note the text's address.
//! Refuses a missing player.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::menu::player_names::{CAR_NAME, NAME_LEN, cars, name};

// ---- Rust version ----

/// `log` gets the address of each line's text.
pub unsafe fn store_player_names(log: &mut dyn FnMut(*const u8)) -> Result<(), BadInput> {
    unsafe {
        for (player, car) in cars()?.into_iter().enumerate() {
            let copy = (car as usize + CAR_NAME) as *mut u8;
            std::ptr::copy(name(player), copy, NAME_LEN);
            log(copy);
        }
        Ok(())
    }
}
