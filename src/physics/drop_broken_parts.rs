//! `physics::drop_broken_parts` (0x462386): seven parts of the work car checked against
//! their limits: when `g_car` +0xA3C + 4k is not 0x80000000 and not below (unsigned)
//! `dword_71BA2C` + 4k, `physics::fill_slot_on_bit` with bit 9, 0xD, 0xE, 0x10, 0x11, 0xC,
//! 0xF (the first only with `dword_62DAD0` set). A reading: damaged parts falling off.
//!
//! Out: as the last `fill_slot_on_bit` leaves them; no flags read
//!
//! Caller: `sub_43B635` (0x43B98B), every physics step. Refuses what `fill_slot_on_bit`
//! refuses; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
use crate::race::owner::Race;
use rc2k_core::physics::parts::{BROKEN_BITS, breaks, broken_checked};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::parts::{BROKEN_BITS, broken_checked, breaks}`.
pub fn drop_broken_parts(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    for (k, &bit) in BROKEN_BITS.iter().enumerate() {
        if !broken_checked(k, || scene.lights.work.marks_mask.get() != 0) {
            continue;
        }
        if !breaks(race.shared.car.part_damage_limits[k].get(), || {
            race.shared.work.part_break_limits[k].get()
        }) {
            continue;
        }
        fill_slot_on_bit(race, bit, scene)?;
    }
    Ok(())
}
