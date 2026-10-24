//! `menu::init_standings_b` (0x4A4644): the 200 entries of standings table B emptied (their
//! first five dwords -1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A44A0`. Refuses a null table, before anything.
use crate::errors::{BadInput, Refused};
use crate::framework::game_read;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn init_standings_b() -> Result<(), Refused> {
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    if first == 0 {
        return Err(BadInput("no standings table").into());
    }
    // Converted: `rc2k_core::menu::standings::EMPTY_B`.
    use rc2k_core::menu::standings::{EMPTY_B, ROOM};
    for n in 0..ROOM {
        for (k, v) in EMPTY_B.into_iter().enumerate() {
            put(first + B_ENTRY * n + 4 * k as u32, v);
        }
    }
    Ok(())
}
