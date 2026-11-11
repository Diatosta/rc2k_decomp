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
use crate::race::owner::Race;
use rc2k_core::physics::walls::ProbeCounts;

// Converted: `rc2k_core::physics::walls::ProbeCounts`.
pub fn count_ground_probe(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let mut counts = ProbeCounts {
        tries: race.physics.camera.ground_probes.get(),
        found: race.physics.camera.ground_found.get(),
        same: race.physics.camera.ground_same.get(),
    };
    race.physics.ground_probe.settle_mark.set(0x40);
    counts.tried();
    race.physics.camera.ground_probes.set(counts.tries);
    { crate::physics::find_ground::find_ground(race, scene) }?;
    if race.physics.ground.ground_found.get() != 0 {
        counts.searched(
            true,
            race.course.course.ground_face_last.get(),
            race.course.course.ground_slot.get(),
        );
        race.physics.camera.ground_found.set(counts.found);
        race.physics.camera.ground_same.set(counts.same);
    }
    Ok(())
}
