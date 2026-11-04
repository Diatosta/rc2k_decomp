//! `menu::clear_result_slots` (0x470DDA): the four players' fourteen result slots
//! (+0x98 .. +0xCC) and every driver's (+0x5C .. +0x90) emptied: -1, but 0 in the seventh
//! and the last (two groups of six sections' points and their total).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a missing player or a drivers' table without its end entry,
//! before anything.
use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub fn clear_result_slots() -> Result<(), Refused> {
    let drivers = drivers()?;
    // Converted: `rc2k_core::menu::event::result_slots`.
    use rc2k_core::menu::event::result_slots;
    for n in 0..4 {
        put_all(player(n), &result_slots(0x98))?;
    }
    drivers.iter().for_each(Driver::clear_results);
    Ok(())
}
