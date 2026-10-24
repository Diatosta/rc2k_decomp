//! `menu::clear_players_288` (0x4A969C): the four players' +0x288 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_473126`. Refuses a missing player, before anything. Self-test: the
//! original and the Rust version on players of the test's; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_players_288() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    for n in 0..4 {
        put(player(n) + 0x288, 0);
    }
    Ok(())
}
