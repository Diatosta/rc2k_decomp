//! `physics::higher_free_gear` (0x43C5A0): from a gear up, the first higher gear whose
//! `lost_gear_bits` bit (`physics::gear_bit`) is clear: gear + 1, + 2, .. up to `gear_count`
//! (unsigned); found with that gear, or not found with one past the count when none is.
//!
//! Takes: the gear
//!
//! Caller: `physics::manual_gearbox`, the gearbox under `drive::race_step`. Refuses what
//! `gear_bit` refuses (a bit past the dword: a count of 31 or more lets the search run
//! there), nothing written.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::gear_bit::bits_at;
use crate::physics::tables_store::PhysicsTables;
use crate::session::PlainDword;

/// On the bits at `bits` and the count `count`.
// Converted: `rc2k_core::physics::gearbox::GearBits::higher_free`.
pub fn higher_free_gear_at(
    bits: &PlainDword,
    count: u32,
    gear: i32,
) -> Result<Option<i32>, Refused> {
    let s = { bits_at(bits) }.higher_free(gear, count)?;
    Ok(s.found.then_some(s.gear))
}

pub fn higher_free_gear(
    car: &Car,
    tables: &PhysicsTables,
    gear: i32,
) -> Result<Option<i32>, Refused> {
    higher_free_gear_at(&car.lost_gear_bits, tables.gear_count.get(), gear)
}
