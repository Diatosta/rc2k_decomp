//! `menu::init_standings_b` (0x4A4644): the 200 entries of standings table B emptied (their
//! first five dwords -1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A44A0`. Refuses a null table, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn init_standings_b() -> Result<(), Refused> {
    let first = TABLE_B_RVA.need("no standings table")?;
    // Converted: `rc2k_core::menu::standings::EMPTY_B`.
    use rc2k_core::menu::standings::{EMPTY_B, ROOM};
    for n in 0..ROOM {
        for (k, v) in EMPTY_B.into_iter().enumerate() {
            first
                .add(B_ENTRY * n + 4 * k as u32)
                .write(v, "a standings entry")?;
        }
    }
    Ok(())
}
