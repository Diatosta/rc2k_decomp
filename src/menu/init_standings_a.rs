//! `menu::init_standings_a` (0x4730D8): the 200 entries of standings table A emptied: +0
//! and +4 -1, +8 .. +0x10 0, +0x14 / +0x18 3600370 (a time beyond any), +0x1C 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A2B75` and two of the game's. Refuses a null table, before anything.
//! Self-test: the original and the Rust version on a table of the test's; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn init_standings_a(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_A_RVA) };
    if first == 0 {
        return Err(BadInput("no standings table").into());
    }
    // Converted: `rc2k_core::menu::standings::EMPTY_A`.
    use rc2k_core::menu::standings::{EMPTY_A, ROOM};
    for n in 0..ROOM {
        let e = first + A_ENTRY * n;
        for (k, v) in EMPTY_A.into_iter().enumerate() {
            put(e + 4 * k as u32, v);
        }
    }
    Ok(*entry)
}
