//! Counts one ground probe: the face mask `settle_mark` is 0x40, `ground_probes` goes up by one
//! and `physics::find_ground` runs; when found `ground_found` goes up by one, and when the face is
//! the last one again (`ground_face_last` = `ground_slot`) `ground_same` too. A reading: the
//! camera's (or a test drive's) ground statistics.
//!
//! Takes: the point (`ground_x`, `ground_z`; the callers set it).
//!
//! Refuses what `find_ground` refuses.

use crate::errors::Refused;
use crate::physics::camera_store::CameraWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_query::GroundSearch;
use crate::race::owner_course::Course;
use rc2k_core::physics::walls::ProbeCounts;
use std::cell::RefCell;

// Core: `rc2k_core::physics::walls::ProbeCounts`.
pub fn count_ground_probe(
    camera: &RefCell<CameraWork>,
    ground_probe: &RefCell<GroundProbe>,
    search: &RefCell<GroundSearch>,
    course: &Course,
) -> Result<(), Refused> {
    let mut counts = ProbeCounts {
        tries: camera.borrow().ground_probes,
        found: camera.borrow().ground_found,
        same: camera.borrow().ground_same,
    };
    ground_probe.borrow_mut().settle_mark = 0x40;
    counts.tried();
    camera.borrow_mut().ground_probes = counts.tries;
    { crate::physics::find_ground::find_ground(ground_probe, search, course) }?;
    let (found, last, slot) = {
        let search = search.borrow();
        (search.found, search.last, search.slot)
    };
    if found != 0 {
        counts.searched(true, last, slot);
        let mut work = camera.borrow_mut();
        work.ground_found = counts.found;
        work.ground_same = counts.same;
    }
    Ok(())
}
