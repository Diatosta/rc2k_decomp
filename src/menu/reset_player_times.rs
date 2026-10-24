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
use crate::framework::{game_write, rva};
use crate::menu::standings::*;

const FLAG_RVA: usize = rva(0x75592C); // dword_75592C
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_player_times() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    // Converted: `rc2k_core::menu::event::PLAYER_TIMES`.
    for n in 0..4 {
        put_all(player(n), &rc2k_core::menu::event::PLAYER_TIMES);
    }
    unsafe { game_write(FLAG_RVA, 0u32) };
    Ok(())
}
