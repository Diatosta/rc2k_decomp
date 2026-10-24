//! `menu::link_player_names` (0x47059E): hangs each player's name on that player's car.
//!
//! In:  nothing
//! Out: every register preserved
//! For each of the four players (`g_cars`): its +0x18 = the address of the player's
//! name in `g_playerNames`, and the name's 0x20 bytes copied to its +0x124.
//! Callers: `saves::load_savegame` (twice) and the main menu's set-up `sub_47C63F`.
//! Refuses a missing player.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::BadInput;
use crate::menu::player_names::{CAR_NAME, CAR_NAME_POINTER, NAME_LEN, cars, name};

// ---- Rust version ----

pub unsafe fn link_player_names() -> Result<(), BadInput> {
    unsafe {
        for (player, car) in cars()?.into_iter().enumerate() {
            ((car as usize + CAR_NAME_POINTER) as *mut u32).write_unaligned(name(player) as u32);
            std::ptr::copy(name(player), (car as usize + CAR_NAME) as *mut u8, NAME_LEN);
        }
        Ok(())
    }
}
