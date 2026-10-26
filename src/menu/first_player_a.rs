//! `menu::first_player_a` (0x4A2023): the first player's entry of group ebx (0x63: any group but
//! 0) in standings table A from edi on (`menu::standings::first_player`); none: the
//! fatal error 0x8070A0A0 "Unable to locate player (find1stplayer)".
//!
//! In:  edi the first entry, ebx the group
//! Out: ecx the group's computer drivers before it, edx its +0x44, edi the entry; the
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
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn first_player_a(entry: &Regs) -> Result<Regs, Refused> {
    first_player(
        entry,
        A_ENTRY,
        0x1C,
        0x44,
        0x8070A0A0,
        "Unable to locate player (find1stplayer)",
    )
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { first_player_a(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
