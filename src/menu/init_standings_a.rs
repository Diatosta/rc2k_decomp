//! `menu::init_standings_a` (0x4730D8): the 200 entries of standings table A emptied: +0
//! and +4 -1, +8 .. +0x10 0, +0x14 / +0x18 3600370 (a time beyond any), +0x1C 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A2B75` and two of the game's. Refuses a null table, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn init_standings_a() -> Result<(), Refused> {
    let first: u32 = { TABLE_A_RVA.get() };
    if first == 0 {
        return Err(BadInput("no standings table").into());
    }
    // Converted: `rc2k_core::menu::standings::EMPTY_A`.
    use rc2k_core::menu::standings::{EMPTY_A, ROOM};
    for n in 0..ROOM {
        let e = first + A_ENTRY * n;
        for (k, v) in EMPTY_A.into_iter().enumerate() {
            put(e + 4 * k as u32, v)?;
        }
    }
    Ok(())
}
