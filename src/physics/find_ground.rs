//! Searches for the ground under a point (`physics::ground_search`; see `physics/ground.rs`).
//!
//! Takes: the point (`ground_x`, `ground_z`), the face mask `settle_mark` and the last face
//! `ground_face_last`.
//!
//! Callers: `physics::probe_ground` (the car's ground points), `physics::move_slot`,
//! `drive::animate_lit_faces`, `race::shadow_points`, `physics::car_lighting`,
//! `drive::simple_car_step`, `physics::count_ground_probe`. Refuses what `ground_search`
//! refuses.

use crate::errors::Refused;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_search::ground_search;
use crate::physics::ground_store::GroundWork;
use crate::race::owner_course::Course;

/// Whether the face under the point was found.
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
    course: crate::race::owner_course::CourseReads<'_>,
    q: &crate::physics::ground_query::GroundQuery<'_>,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    crate::physics::ground::ground_search_at(course, q, switches)
}
