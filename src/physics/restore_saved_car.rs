//! `physics::restore_saved_car` (0x43DC25): puts a saved state back into the work copy:
//! `dword_614738` + 1; the three dwords `dword_6141E0` .. `E8` into `dword_71A10C` ..
//! `114`; the 0x40 bytes at `unk_6141EC` into `g_crashMatrix` (0x71A08C); and
//! `dword_71A238` dwords from `dword_6D2814` into `dword_71ADC8` (the copy's +0xDF0).
//! What the pieces are (a place, the car's matrix, a list) is a reading.
//!
//! In:  nothing
//! Out: eax = `dword_6141E8`, ecx = 0, esi = 0x6D2814 + 4 n, edi = 0x71ADC8 + 4 n (n the
//!      count); flags the `add`'s (no caller reads them)
//!
//! Callers (7): `sub_43BDD1`, `sub_45D737`, `sub_45D7B6` (twice), `sub_45E3BF`,
//! `sub_45EFF3`, `sub_45F703`, under `sub_4454D2` (most followed by `sub_43F1A8`). Refuses
//! a count above 0x7C (unsigned: the list would run past the work copy's end, 0x71AFB8),
//! before writing.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::physics::tables_store::PHYSICS;
use rc2k_core::physics::saved_car::{restore_count, restored};

/// The Rust version. Converted: `rc2k_core::physics::saved_car::{restore_count, restored}`.
pub fn restore_saved_car() -> Result<(), Refused> {
    let n = restore_count(CAR.probe_count.get())?;
    PHYSICS
        .clear_place_count
        .set(restored(PHYSICS.clear_place_count.get()));
    for (to, from) in CAR.position.iter().zip(&PHYSICS.saved_place) {
        to.set(from.get());
    }
    for (to, from) in CAR.crash_matrix.iter().zip(&PHYSICS.saved_matrix) {
        to.set_bits(from.bits());
    }
    // By address: up to 0x7C dwords from a list of 0x40 (the original copies on into what
    // follows each).
    crate::session::copy_on(
        &CONTACT_WORK.probe_saved_faces,
        &CAR.probe_point_face,
        n as usize,
        "the saved faces",
    )?;
    Ok(())
}
