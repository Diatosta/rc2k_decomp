//! `physics::drop_broken_parts` (0x462386): seven parts of the work car checked against
//! their limits: when `g_car` +0xA3C + 4k is not 0x80000000 and not below (unsigned)
//! `dword_71BA2C` + 4k, `physics::fill_slot_on_bit` with bit 9, 0xD, 0xE, 0x10, 0x11, 0xC,
//! 0xF (the first only with `dword_62DAD0` set). A reading: damaged parts falling off.
//!
//! In:  nothing
//! Out: as the last `fill_slot_on_bit` leaves them; no flags read
//!
//! Caller: `sub_43B635` (0x43B98B), every physics step.
//! Refuses what `fill_slot_on_bit` refuses; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): damage about the limits, the switch.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::fill_race_slot::g;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
use rc2k_core::physics::parts::{BROKEN_BITS, breaks, broken_checked};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::parts::{BROKEN_BITS, broken_checked, breaks}`.
pub fn drop_broken_parts() -> Result<(), Refused> {
    for (k, &bit) in BROKEN_BITS.iter().enumerate() {
        if !broken_checked(k, || g(0x62DAD0) != 0) {
            continue;
        }
        if !breaks(g(0x71AA14 + 4 * k), || g(0x71BA2C + 4 * k)) {
            continue;
        }
        fill_slot_on_bit(bit)?;
    }
    Ok(())
}
