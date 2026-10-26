//! `menu::first_player_a` (0x4A2023): the first player's entry of group ebx (0x63: any group but
//! 0) in standings table A from edi on (`menu::standings::first_player`); none: the
//! fatal error 0x8070A0A0 "Unable to locate player (find1stplayer)".
//!
//! In:  edi the first entry, ebx the group
//! Out: ecx the group's computer drivers before it, edx its +0x44, edi the entry; the
//! rest kept
//!
//! Callers: the event code. Refuses a table without its end entry, before anything; raises the
//! fatal error as the original.

use crate::errors::Refused;
use crate::memory::blocks::Place;
use crate::menu::standings::*;

/// The first player of `group` (ebx) in the table from `from` (edi).
pub fn first_player_a(from: Place, group: u32) -> Result<Found, Refused> {
    first_player(
        from,
        group,
        A_ENTRY,
        0x1C,
        0x44,
        0x8070A0A0,
        "Unable to locate player (find1stplayer)",
    )
}
