//! `menu::clear_result_slots` (0x470DDA): the four players' fourteen result slots
//! (+0x98 .. +0xCC) and every driver's (+0x5C .. +0x90) emptied: -1, but 0 in the seventh
//! and the last (a reading: two groups of six places and a count).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a missing player or a drivers' table without its end entry,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_result_slots() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    let mut drivers = driver_table()?;
    // Converted: `rc2k_core::menu::event::result_slots`.
    use rc2k_core::menu::event::result_slots;
    for n in 0..4 {
        put_all(player(n), &result_slots(0x98));
    }
    drivers.store_all(&result_slots(0x5C));
    Ok(())
}
