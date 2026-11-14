//! The course face under the point (`ground_x`, `ground_z`): `physics::try_last_face_at`, else
//! `ground_slot` = -1 and `physics::search_cell` on the point's cell and its eight neighbours
//! (`ground::CELLS`) until one has it. `ground_found` is 1 when found, else 0.
//!
//! See `physics/ground.rs`. Caller: `physics::find_ground`. Refuses (before anything) what
//! `find_course_record` or `search_record_faces_at` would refuse on any of the nine cells (also on
//! cells past the one found).

use crate::errors::Refused;
use crate::physics::ground::*;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_store::GroundWork;
use crate::race::owner_course::Course;

/// Whether the face under the point was found.
pub fn ground_search(
    ground_probe: &GroundProbe,
    ground: &GroundWork,
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    ground_search_at(
        course.reads(),
        &crate::physics::ground_query::GroundQuery::of_physics(
            ground,
            ground_probe,
            course.course.get(),
            &crate::race::scene_points::no_own(),
        ),
        scene.draw_switches.get(),
    )
}
