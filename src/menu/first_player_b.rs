//! `menu::first_player_b` (0x4A4B16): the first player's entry of group ebx (0x63: any group but
//! 0) in standings table B from edi on (`menu::standings::first_player`); none: the
//! fatal error 0x8070A0AF "Unable to locate player (find1stplayer)".
//!
//! In:  edi the first entry, ebx the group
//! Out: ecx the group's computer drivers before it, edx its +0x70, edi the entry; the
//!      rest kept
//!
//! Callers: the event code. Refuses a table without its end entry, before anything;
//! raises the fatal error as the original. Self-test: `standings::test_first_player`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn first_player_b(entry: &Regs) -> Result<Regs, Refused> {
    first_player(
        entry,
        B_ENTRY,
        0x10,
        0x70,
        0x8070A0AF,
        "Unable to locate player (find1stplayer)",
    )
}
