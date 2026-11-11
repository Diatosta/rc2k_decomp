//! `physics::face_contacts_around` (0x45E196): the car's place (`position`, `unk_71A114`)
//! into `ground_x`, `ground_z`, then `physics::cell_face_contacts` for its cell
//! and the eight about it (± 0x200000 each way, in the original's order; the place read
//! back from `ground_x` .. before each).
//!
//! Caller: `physics::face_contacts` (0x45E190, its only code). Refuses what
//! `cell_face_contacts` refuses.

use crate::errors::Refused;
use crate::physics::step_families::ContactInputs;

/// The steps, not guarded.
// Converted: `rc2k_core::physics::ground_search::cells_around`.
pub fn face_contacts_around(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    c.ground.ground.ground_x.set(c.car.position[0].get());
    c.ground.ground.ground_z.set(c.car.position[2].get());
    rc2k_core::physics::ground_search::cells_around(
        || {
            (
                c.ground.ground.ground_x.get(),
                c.ground.ground.ground_z.get(),
            )
        },
        |x, z| crate::physics::cell_face_contacts::steps(c, x, z, scene),
    )
}
