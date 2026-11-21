//! The course face under the point (`ground_x`, `ground_z`): `physics::try_last_face_at`, else
//! `ground_slot` = -1 and `physics::search_cell` on the point's cell and its eight neighbours
//! (`ground::CELLS`) until one has it. `ground_found` is 1 when found, else 0.
//!
//! See `physics/ground.rs`. Refuses (before anything) what `find_course_record` or
//! `search_record_faces_at` would refuse on any of the nine cells (also on cells past the one
//! found).

use crate::errors::Refused;
use crate::physics::ground::*;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_query::{GroundEnv, GroundSearch};
use crate::race::owner_course::Course;
use std::cell::RefCell;

/// Whether the face under the point was found.
pub fn ground_search(
    ground_probe: &RefCell<GroundProbe>,
    search: &RefCell<GroundSearch>,
    course: &Course,
) -> Result<bool, Refused> {
    ground_search_at(
        GroundEnv {
            course: course.reads(),
            points: &crate::race::scene_points::no_own(),
            mark: ground_probe.borrow().settle_mark,
        },
        &mut search.borrow_mut(),
    )
}
