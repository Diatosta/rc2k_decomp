//! `menu::player_entered` (0x4A95EB): eax kept when player ecx's +0xD0 is 1 (a reading: the
//! player takes part), else eax 0.
//!
//! In:  ecx the player, eax a value
//! Out: edi the player, eax as above; the rest kept
//!
//! Caller: `sub_473126` (four times). Refuses a player past 3 or missing, before
//! anything. Self-test: the original and the Rust version on players of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::Regs;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn player_entered(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if g.ecx >= 4 || player(g.ecx) == 0 {
        return Err(BadInput("a player past 3 or missing").into());
    }
    g.edi = player(g.ecx);
    // Converted: `rc2k_core::menu::event::if_entered`.
    g.eax = rc2k_core::menu::event::if_entered(get(g.edi + 0xD0), g.eax);
    Ok(g)
}
