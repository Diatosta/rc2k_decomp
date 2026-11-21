//! `menu::first_player_unplaced`: the first player's entry that has a group (`group` not 0) in
//! standings table B, counting the computer drivers with a group before it; none: the fatal
//! error 0x8070A0AE "Unable to locate player (find1stplayerinochamp)".
//!
//! Refuses a table without its end entry, before anything; raises the fatal error.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The first player of any group in table B; its `computers` is the player's number.
pub fn first_player_unplaced(
    tables: &crate::saves::best_times::BestTables,
) -> Result<Found, Refused> {
    let found = first_player(
        tables,
        Which::B,
        rc2k_core::menu::standings::ANY_GROUP,
        0x8070A0AE,
        "Unable to locate player (find1stplayerinochamp)",
    )?;
    let player = with_b(tables, |t| t.entries[found.entry].player)?;
    Ok(Found {
        computers: player,
        ..found
    })
}
