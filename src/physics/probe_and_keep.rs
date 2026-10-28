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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::probe_ground::probe_ground_at;

/// The Rust core. The registers it leaves (esi, edi past the copies, ecx 0) are read by
/// none of its callers.
pub fn probe_and_keep_at() -> Result<(), Refused> {
    probe_ground_at()?;
    for k in 0..64 {
        CONTACT_WORK.saved_probe_x[k].set(CONTACT_WORK.probe_x[k].get());
    }
    for k in 0..64 {
        CONTACT_WORK.saved_probe_y[k].set(CONTACT_WORK.probe_y[k].get());
    }
    for k in 0..64 {
        CONTACT_WORK.saved_probe_z[k].set(CONTACT_WORK.probe_z[k].get());
    }
    for k in 0..0x40 {
        CONTACT_WORK.saved_probe_faces[k].set(CAR.probe_point_face[k].get());
    }
    Ok(())
}

/// The Rust version.
pub fn probe_and_keep() -> Result<(), Refused> {
    probe_and_keep_at()
}
