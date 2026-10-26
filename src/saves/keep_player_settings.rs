//! `saves::keep_player_settings` (0x4719F5, `maybeCopySomeData` in IDA): the players'
//! own settings copied into the saved game's four cars before those are put over the
//! players, so that a loaded game keeps the controls and choices made since: for each of
//! the four players, the bytes +0x1C .. +0x27 (the keys), the dwords +0x2C .. +0x78 and
//! +0x178, into the saved car at esi + n x 0x28C.
//!
//! In:  esi the saved cars (four of 0x28C bytes)
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `saves::load_savegame` (when `dword_755144` is 0). Refuses (before anything) a missing
//! player or esi 0.

use crate::errors::{BadInput, Refused};
use crate::menu::player::{PLAYERS, player_at};
use crate::pointers::GamePtr;

const CAR_LEN: u32 = rc2k_core::saves::cars::SAVED_CAR_LEN as u32;

pub fn keep_player_settings(cars: u32) -> Result<(), Refused> {
    if cars == 0 || (0..PLAYERS).any(|n| player_at(n) == 0) {
        return Err(BadInput("a player missing, or no saved cars").into());
    }
    for n in 0..PLAYERS {
        let (from, to) = (player_at(n), cars.wrapping_add(n * CAR_LEN));
        let player = GamePtr::new(from).bytes("a player's settings")?;
        let mut kept = Vec::new();
        rc2k_core::saves::cars::keep_settings(&player, &mut |at, b| kept.push((at, b)))?;
        for (at, b) in kept {
            GamePtr::new(to.wrapping_add(at as u32)).write(b, "a saved car's settings")?;
        }
    }
    Ok(())
}
