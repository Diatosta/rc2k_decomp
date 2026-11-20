//! Searches for the ground under a point (`physics::ground_search`; see `physics/ground.rs`).
//!
//! Takes: the point (`ground_x`, `ground_z`), the face mask `settle_mark` and the last face
//! `ground_face_last`.
//!
//! Refuses what `ground_search` refuses.

use crate::errors::Refused;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_query::{GroundEnv, GroundSearch};
use crate::physics::ground_search::ground_search;
use crate::race::owner_course::Course;
use std::cell::RefCell;

/// Whether the face under the point was found.
pub fn find_ground(
    ground_probe: &RefCell<GroundProbe>,
    search: &RefCell<GroundSearch>,
    course: &Course,
) -> Result<bool, Refused> {
    ground_search(ground_probe, search, course)
}

/// The same on the record `q` (a drawing's own: `race::car_shadow`).
pub fn find_ground_with(env: GroundEnv<'_>, q: &mut GroundSearch) -> Result<bool, Refused> {
    crate::physics::ground::ground_search_at(env, q)
}
