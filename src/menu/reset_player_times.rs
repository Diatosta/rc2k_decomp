//! `menu::reset_player_times` (0x4813D6): the four players' +0x88 3600000 (an hour in
//! milliseconds: no time yet) and +0xD0 6; `dword_75592C` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126` (three times). Refuses a missing player, before anything.
//! Self-test: the original and the Rust version on the game's players; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_write, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const FLAG_RVA: usize = rva(0x75592C); // dword_75592C
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_player_times(entry: &Regs) -> Result<Regs, Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    // Converted: `rc2k_core::menu::event::PLAYER_TIMES`.
    for n in 0..4 {
        put_all(player(n), &rc2k_core::menu::event::PLAYER_TIMES);
    }
    unsafe { game_write(FLAG_RVA, 0u32) };
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_player_times(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
