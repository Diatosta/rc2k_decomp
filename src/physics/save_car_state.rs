//! `physics::save_car_state` (0x43DBD5): the counterpart of `physics::restore_saved_car`:
//! `dword_614738` = 0; `dword_71A10C` .. `114` into `dword_6141E0` .. `E8`;
//! `g_crashMatrix` (0x40 bytes) into `unk_6141EC`; `dword_71A238` dwords of
//! `dword_71ADC8` (the probe points' faces) into `dword_6D2814`.
//!
//! Caller: `physics::car_physics_step` (0x43B719), every physics step. Refuses a count above 0x40
//! (unsigned; `dword_6D2814` has 0x40 before `dword_6D2914`), before writing.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::saved_car::save_count;

/// The Rust version (`restore_saved_car`'s, the other way). Converted:
/// `rc2k_core::physics::saved_car::save_count` (at most 0x40 faces: the list's room).
pub fn save_car_state(
    car: &Car,
    tables: &PhysicsTables,
    contact: &ContactWork,
) -> Result<(), Refused> {
    let n = save_count(car.probe_count.get())?;
    tables.clear_place_count.set(0);
    for (to, from) in tables.saved_place.iter().zip(&car.position) {
        to.set(from.get());
    }
    for (to, from) in tables.saved_matrix.iter().zip(&car.crash_matrix) {
        to.set_bits(from.bits());
    }
    for (to, from) in contact.probe_saved_faces[..n]
        .iter()
        .zip(&car.probe_point_face[..n])
    {
        to.set(from.get());
    }
    Ok(())
}
