//! `physics::probe_and_keep` (0x45DC12): `physics::probe_ground`, then its three
//! whole-number lists (x, y, z at 0x6D1E14, 0xC0 dwords) copied to 0x6D2C14 and the probe
//! points' faces (`dword_71ADC8`, 0x40 dwords) to 0x6D2F14 (`rep movsd`): by the look of
//! it the step's starting points kept (a reading).
//!
//! In:  nothing
//! Out: ecx 0, esi 0x71ADC8 + 0x100, edi 0x6D2F14 + 0x100; eax, ebx, edx as
//!      `probe_ground` leaves them (with no probe points: kept)
//!
//! Caller: `sub_43B635` (0x43B700), every physics step; next a call (no flags read).
//! Refuses what `probe_ground` refuses (nothing written).
//! Self-test: a copy of the original (its call to a copy of `probe_ground` on the test's
//! ground) against the Rust core, `probe_ground`'s cases.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::ground::*;
use crate::physics::probe_ground::probe_ground_at;

/// The Rust core. The registers it leaves (esi, edi past the copies, ecx 0) are read by
/// none of its callers.
pub unsafe fn probe_and_keep_at(g: &Ground) -> Result<(), Refused> {
    unsafe {
        probe_ground_at(g)?;
        for (from, to, n) in [
            (0x6D1E14u32, 0x6D2C14u32, 0xC0u32),
            (0x71ADC8, 0x6D2F14, 0x40),
        ] {
            for k in 0..n {
                g.put(to + 4 * k, g.get(from + 4 * k));
            }
        }
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn probe_and_keep() -> Result<(), Refused> {
    unsafe { probe_and_keep_at(&Ground::game()) }
}
