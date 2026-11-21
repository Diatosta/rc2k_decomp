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
    {
        let mut search = c.ground.search.borrow_mut();
        search.x = c.car.position.bits_at(0);
        search.z = c.car.position.bits_at(2);
    }
    rc2k_core::physics::ground_search::cells_around(
        || {
            let search = c.ground.search.borrow();
            (search.x, search.z)
        },
        |x, z| crate::physics::cell_face_contacts::steps(c, x, z, scene),
    )
}
