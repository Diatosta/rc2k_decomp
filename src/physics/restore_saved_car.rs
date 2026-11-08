//! `physics::restore_saved_car` (0x43DC25): puts a saved state back into the work copy:
//! `dword_614738` + 1; the three dwords `dword_6141E0` .. `E8` into `dword_71A10C` ..
//! `114`; the 0x40 bytes at `unk_6141EC` into `g_crashMatrix` (0x71A08C); and
//! `dword_71A238` dwords from `dword_6D2814` into `dword_71ADC8` (the copy's +0xDF0).
//! What the pieces are (a place, the car's matrix, a list) is a reading.
//!
//! Callers (7): `physics::find_clear_place`, `physics::crossed_edge`, `physics::wall_contacts`
//! (twice), `physics::face_contact`,
//! `physics::ground_contact`, `physics::corner_hits_car`, under `drive::race_step` (most followed
//! by `physics::feel_ground`). Refuses a count above 0x7C (unsigned: the list would run past the
//! work copy's end, 0x71AFB8), before writing.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::physics::saved_car::{restore_count, restored};

/// The Rust version. Converted: `rc2k_core::physics::saved_car::{restore_count, restored}`.
pub fn restore_saved_car(race: &Race) -> Result<(), Refused> {
    let n = restore_count(race.shared.car.probe_count.get())?;
    race.shared
        .physics_tables
        .clear_place_count
        .set(restored(race.shared.physics_tables.clear_place_count.get()));
    for (to, from) in race
        .shared
        .car
        .position
        .iter()
        .zip(&race.shared.physics_tables.saved_place)
    {
        to.set(from.get());
    }
    for (to, from) in race
        .shared
        .car
        .crash_matrix
        .iter()
        .zip(&race.shared.physics_tables.saved_matrix)
    {
        to.set_bits(from.bits());
    }
    // By address: up to 0x7C dwords from a list of 0x40 (the original copies on into what
    // follows each).
    crate::session::copy_on_in(
        (
            &race.physics.contact,
            &race.physics.contact.probe_saved_faces,
        ),
        (&race.shared.car, &race.shared.car.probe_point_face),
        n as usize,
        "the saved faces",
    )?;
    Ok(())
}
