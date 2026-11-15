//! `menu::first_player_b`: the first player's entry of a group (0x63: any group but 0) in
//! standings table B (`menu::standings::first_player`); none: the fatal error 0x8070A0AF
//! "Unable to locate player (find1stplayer)".
//!
//! Callers: the event code. Refuses a table without its end entry, before anything; raises the
//! fatal error.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The first player of `group` in table B.
pub fn first_player_b(
    group: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<Found, Refused> {
    first_player(
        tables,
        Which::B,
        group,
        0x8070A0AF,
        "Unable to locate player (find1stplayer)",
    )
}
