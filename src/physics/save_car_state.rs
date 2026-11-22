//! The counterpart of `physics::restore_saved_car`: `clear_place_count` = 0; `position` goes into
//! `saved_place`; the crash matrix (0x40 bytes) into `saved_matrix`; `probe_count` dwords of
//! `probe_point_face` (the probe points' faces) into `probe_saved_faces`.
//!
//! Refuses (before writing) a count above 0x40 (unsigned; `probe_saved_faces` has 0x40 before
//! `probe_face_flip`).

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::contact_store::ProbeLists;
use crate::physics::tables_store::PhysicsTables;
use rc2k_core::physics::saved_car::save_count;

/// Saves the car's state (`restore_saved_car`'s, the other way). The core is
/// `rc2k_core::physics::saved_car::save_count` (at most 0x40 faces: the list's room).
pub(super) fn save_car_state(
    car: &Car,
    tables: &mut PhysicsTables,
    probes: &mut ProbeLists,
) -> Result<(), Refused> {
    let n = save_count(car.probe_count.get())?;
    tables.clear_place_count = 0;
    for (to, from) in tables.saved_place.iter_mut().zip(car.position.bits()) {
        *to = from;
    }
    tables.saved_matrix = car.crash_matrix.bits().map(f32::from_bits);
    for (to, from) in probes.probe_saved_faces[..n]
        .iter_mut()
        .zip(&car.probe_point_face[..n])
    {
        *to = from.get();
    }
    Ok(())
}
