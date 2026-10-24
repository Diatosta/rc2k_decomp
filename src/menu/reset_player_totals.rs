//! `menu::reset_player_totals` (0x471FB8): the counted players' (`dword_755984`) +0x8C and
//! +0x88 3600000.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_471F0D`. Refuses a player count of 0 or past 4 or a missing player, before
//! anything. Self-test: the original and the Rust version on the game's players; put
//! back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_player_totals() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::event::PLAYER_TOTALS`.
    for p in counted_players()? {
        put_all(p, &rc2k_core::menu::event::PLAYER_TOTALS);
    }
    Ok(())
}
