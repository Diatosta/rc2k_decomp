//! `physics::save_car_state` (0x43DBD5): the counterpart of `physics::restore_saved_car`:
//! `dword_614738` = 0; `dword_71A10C` .. `114` into `dword_6141E0` .. `E8`;
//! `g_crashMatrix` (0x40 bytes) into `unk_6141EC`; `dword_71A238` dwords of
//! `dword_71ADC8` (the probe points' faces) into `dword_6D2814`.
//!
//! In:  nothing
//! Out: eax = `dword_71A114`, ecx 0, esi = 0x71ADC8 + 4 n, edi = 0x6D2814 + 4 n; no flags
//!      read
//!
//! Caller: `sub_43B635` (0x43B719), every physics step. Refuses a count above 0x40
//! (unsigned; `dword_6D2814` has 0x40 before `dword_6D2914`), before writing.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use rc2k_core::physics::saved_car::save_count;

/// The Rust version (`restore_saved_car`'s, the other way). Converted:
/// `rc2k_core::physics::saved_car::save_count` (at most 0x40 faces: the list's room).
pub fn save_car_state() -> Result<(), Refused> {
    let n = save_count(CAR.probe_count.get())?;
    PHYSICS.clear_place_count.set(0);
    for (to, from) in PHYSICS.saved_place.iter().zip(&CAR.position) {
        to.set(from.get());
    }
    for (to, from) in PHYSICS.saved_matrix.iter().zip(&CAR.crash_matrix) {
        to.set_bits(from.bits());
    }
    for (to, from) in crate::race::owner::race().physics.contact.probe_saved_faces[..n]
        .iter()
        .zip(&CAR.probe_point_face[..n])
    {
        to.set(from.get());
    }
    Ok(())
}
