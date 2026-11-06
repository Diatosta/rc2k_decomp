//! `physics::face_contacts_around` (0x45E196): the car's place (`unk_71A10C`, `unk_71A114`)
//! into `dword_639824`, `dword_63982C`, then `physics::cell_face_contacts` for its cell
//! and the eight about it (± 0x200000 each way, in the original's order; the place read
//! back from `dword_639824` .. before each).
//!
//! In:  nothing
//! Out: the registers as the last `cell_face_contacts` leaves them; no flags read
//!
//! Caller: `physics::face_contacts` (0x45E190, its only code). Refuses what
//! `cell_face_contacts` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::race::owner::race;
use crate::race::owner_physics::Physics;

/// The steps, not guarded. The registers left are read by none of the callers.
// Converted: `rc2k_core::physics::ground_search::cells_around`.
pub fn face_contacts_around(physics: &Physics) -> Result<(), Refused> {
    let shared_store = &race().shared;

    let physics_store = physics;
    physics_store
        .ground
        .ground_x
        .set(shared_store.car.position[0].get());
    physics_store
        .ground
        .ground_z
        .set(shared_store.car.position[2].get());
    rc2k_core::physics::ground_search::cells_around(
        || {
            (
                physics_store.ground.ground_x.get(),
                physics_store.ground.ground_z.get(),
            )
        },
        crate::physics::cell_face_contacts::steps,
    )
}
