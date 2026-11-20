//! Puts the car's place into `ground_x`, `ground_z`, then runs `physics::cell_face_contacts`
//! for its cell and the eight about it (+-0x200000 each way; the place is read back from
//! `ground_x` before each).
//!
//! Refuses what `cell_face_contacts` refuses.

use crate::errors::Refused;
use crate::physics::step_families::ContactInputs;

/// The steps, not guarded.
// Core: `rc2k_core::physics::ground_search::cells_around`.
pub(super) fn face_contacts_around(
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
