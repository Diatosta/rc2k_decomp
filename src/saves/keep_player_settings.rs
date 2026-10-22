//! `saves::keep_player_settings` (0x4719F5, `maybeCopySomeData` in IDA): the players'
//! own settings copied into the saved game's four cars before those are put over the
//! players, so that a loaded game keeps the controls and choices made since: for each of
//! the four players, the bytes +0x1C .. +0x27 (the keys), the dwords +0x2C .. +0x78 and
//! +0x178, into the saved car at esi + n x 0x28C.
//!
//! In:  esi the saved cars (four of 0x28C bytes)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `saves::load_savegame` (when `dword_755144` is 0). Refuses (before anything) a
//! missing player or esi 0. Self-test: a buffer of the test's (0xAA) from the game's
//! players, compared whole.
//!
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::menu::player::{PLAYERS, player_at};

const CAR_LEN: u32 = rc2k_core::saves::cars::SAVED_CAR_LEN as u32;

pub unsafe fn keep_player_settings(cars: u32) -> Result<(), Refused> {
    if cars == 0 || (0..PLAYERS).any(|n| player_at(n) == 0) {
        return Err(BadInput("a player missing, or no saved cars").into());
    }
    for n in 0..PLAYERS {
        // Converted: `rc2k_core::saves::cars::keep_settings`.
        let (from, to) = (player_at(n), cars.wrapping_add(n * CAR_LEN));
        let player = unsafe { crate::bridge::Raw::new(from as *const u8) };
        crate::bridge::read_through(rc2k_core::saves::cars::keep_settings(
            &player,
            &mut |at, b| unsafe { *(to.wrapping_add(at as u32) as *mut u8) = b },
        ));
    }
    Ok(())
}
