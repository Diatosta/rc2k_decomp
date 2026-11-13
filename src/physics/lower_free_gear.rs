//! `physics::lower_free_gear` (0x43C58E): from a gear down, the first lower gear whose
//! `lost_gear_bits` bit (`physics::gear_bit`) is clear: gear - 1, - 2, ..; found with that gear, or
//! not found when the count reaches 0 (neutral) first.
//!
//! Takes: the gear
//!
//! Callers (2): both in `physics::manual_gearbox`, the gearbox under `drive::race_step`. Refuses
//! what `gear_bit` refuses (a bit past the dword: from gear 0 or above 32 the search would run
//! there), nothing written.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::gear_bit::bits_at;
use crate::session::PlainDword;

/// On the bits at `bits`: the free lower gear (None when neutral is reached first: CF).
// Converted: `rc2k_core::physics::gearbox::GearBits::lower_free`.
pub fn lower_free_gear_at(bits: &PlainDword, gear: i32) -> Result<Option<i32>, Refused> {
    let s = { bits_at(bits) }.lower_free(gear, 0)?;
    Ok(s.found.then_some(s.gear))
}

pub fn lower_free_gear(car: &Car, gear: i32) -> Result<Option<i32>, Refused> {
    lower_free_gear_at(&car.lost_gear_bits, gear)
}
