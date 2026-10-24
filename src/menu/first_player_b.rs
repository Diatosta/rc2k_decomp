//! `menu::first_player_b` (0x4A4B16): the first player's entry of group ebx (0x63: any group but
//! 0) in standings table B from edi on (`menu::standings::first_player`); none: the
//! fatal error 0x8070A0AF "Unable to locate player (find1stplayer)".
//!
//! In:  edi the first entry, ebx the group
//! Out: ecx the group's computer drivers before it, edx its +0x70, edi the entry; the
//! rest kept
//!
//! Callers: the event code. Refuses a table without its end entry, before anything; raises the
//! fatal error as the original.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The first player of `group` (ebx) in the table from `from` (edi).
pub unsafe fn first_player_b(from: u32, group: u32) -> Result<Found, Refused> {
    first_player(
        from,
        group,
        B_ENTRY,
        0x10,
        0x70,
        0x8070A0AF,
        "Unable to locate player (find1stplayer)",
    )
}
