//! `menu::first_player_unplaced` (0x4A4B45): the first player's entry with a group (+0x10
//! not 0) in standings table B from edi on, counting the computer drivers with a group
//! before it; none: the fatal error 0x8070A0AE "Unable to locate player
//! (find1stplayerinochamp)".
//!
//! In:  edi the first entry
//! Out: edx its +0x70, ecx its player (+4), edi the entry; the rest kept
//!
//! Callers: the event code. Refuses a table without its end entry, before anything; raises the
//! fatal error as the original.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The first player of any group in table B (the original's edi, its start); its
/// `computers` is the player's number (ecx = the entry's player). The caller's ebx is kept.
pub fn first_player_unplaced() -> Result<Found, Refused> {
    // Converted: `rc2k_core::saves::rankings::first_player` (any group), ecx the player.
    let found = first_player(
        Which::B,
        rc2k_core::menu::standings::ANY_GROUP,
        0x8070A0AE,
        "Unable to locate player (find1stplayerinochamp)",
    )?;
    let player = with_b(|t| t.entries[found.entry].player)?;
    Ok(Found {
        computers: player,
        ..found
    })
}
