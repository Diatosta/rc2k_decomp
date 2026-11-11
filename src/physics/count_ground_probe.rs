//! `physics::count_ground_probe` (0x438E22): one ground probe, counted: the face mask
//! `settle_mark` 0x40, `ground_probes` up by one, `physics::find_ground`; when found
//! `ground_found` up by one, and when the face is the last one again (`ground_face_last` =
//! `ground_slot`) `ground_same` too. A reading: the camera's (or a test drive's) ground
//! statistics.
//!
//! Takes: the point (`ground_x`, `ground_z`; the callers set it)
//!
//! Callers: `physics::camera_low_switch`, `physics::camera_over_ground` (three). Refuses what
//! `find_ground` refuses.

use crate::errors::Refused;
use crate::physics::camera_store::CameraWork;
use crate::physics::ground_probe_store::GroundProbe;
use crate::physics::ground_store::GroundWork;
use crate::race::owner_course::Course;
use rc2k_core::physics::walls::ProbeCounts;

// Converted: `rc2k_core::physics::walls::ProbeCounts`.
pub fn count_ground_probe(
    camera: &CameraWork,
    ground_probe: &GroundProbe,
    ground: &GroundWork,
    course: &Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let mut counts = ProbeCounts {
        tries: camera.ground_probes.get(),
        found: camera.ground_found.get(),
        same: camera.ground_same.get(),
    };
    ground_probe.settle_mark.set(0x40);
    counts.tried();
    camera.ground_probes.set(counts.tries);
    { crate::physics::find_ground::find_ground(ground_probe, ground, course, scene) }?;
    if ground.ground_found.get() != 0 {
        counts.searched(
            true,
            course.course.ground_face_last.get(),
            course.course.ground_slot.get(),
        );
        camera.ground_found.set(counts.found);
        camera.ground_same.set(counts.same);
    }
    Ok(())
}
