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
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::fill_race_slot::g;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit_in;
use crate::screens::harness::Left;
use rc2k_core::physics::parts::{BROKEN_BITS, breaks, broken_checked};

/// The steps on `r`.
// Converted: `rc2k_core::physics::parts::{BROKEN_BITS, broken_checked, breaks}`.
pub fn drop_broken_parts_in(r: &mut Regs) -> Result<(), Refused> {
    for (k, &bit) in BROKEN_BITS.iter().enumerate() {
        if !broken_checked(k, || g(0x62DAD0) != 0) {
            continue;
        }
        let limit = || {
            r.eax = g(0x71BA2C + 4 * k);
            r.eax
        };
        if !breaks(g(0x71AA14 + 4 * k), limit) {
            continue;
        }
        r.ecx = bit;
        fill_slot_on_bit_in(r)?;
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn drop_broken_parts(entry: &Regs) -> Result<Left, Refused> {
    let mut r = *entry;
    match drop_broken_parts_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => Err(e),
    }
}
