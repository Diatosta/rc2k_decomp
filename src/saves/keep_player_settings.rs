//! `saves::keep_player_settings` (`maybeCopySomeData` in IDA): the players'
//! own settings copied into the saved game's four cars before those are put over the
//! players, so that a loaded game keeps the controls and choices made since: for each of
//! the four players, the bytes +0x1C .. +0x27 (the keys), the dwords +0x2C .. +0x78 and
//! +0x178, into the saved car at the buffer offset + n x 0x28C.
//!
//!
//! Caller: `saves::load_savegame` (when `use_savegame_keys` is 0). Refuses (before anything) a missing
//! player or a null buffer.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::player::PLAYERS;
use crate::pointers::Store;

const CAR_LEN: usize = rc2k_core::saves::cars::SAVED_CAR_LEN;

/// `file` is the savegame file; the saved cars start at byte `cars`.
pub fn keep_player_settings(menus: &Menus, file: &dyn Store, cars: usize) -> Result<(), Refused> {
    for n in 0..PLAYERS {
        // The saved cars lie in the savegame file's buffer (`saves::load_savegame`'s cursor).
        let to = cars + n as usize * CAR_LEN;
        let player = crate::menu::standings::player(menus, n).get().to_le_bytes();
        let mut kept = Vec::new();
        rc2k_core::saves::cars::keep_settings(&player, &mut |at, b| kept.push((at, b)))?;
        for (at, b) in kept {
            file.store(to + at, &[b])?;
        }
    }
    Ok(())
}
