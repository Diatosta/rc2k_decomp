//! `physics::find_ground` (0x441E29): `call physics::ground_search; retn`.
//!
//! In:  the point (`ground_x`, `ground_z`), the face mask `settle_mark`, the
//!      last face `ground_face_last`
//!
//! See `physics/ground.rs`. Callers (7): `physics::probe_ground` (the car's ground points),
//! `physics::move_slot`, `drive::animate_lit_faces`, `race::shadow_points`,
//! `physics::car_lighting`, `drive::simple_car_step`, `physics::count_ground_probe`. Refuses what
//! `ground_search` refuses.

use crate::errors::Refused;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_search::ground_search;
use crate::physics::ground_store::GroundWork;
use crate::race::owner_course::Course;

/// The carry (the registers are kept).
pub fn find_ground(
    ground_probe: &GroundProbe,
    ground: &GroundWork,
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    ground_search(ground_probe, ground, course, scene)
}

/// The same on the fields of `q` (a drawing's own: `race::car_shadow`).
pub fn find_ground_with(
    course: &Course,
    q: &crate::physics::ground_query::GroundQuery<'_>,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    crate::physics::ground::ground_search_at(course, q, switches)
}
