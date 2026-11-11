//! `physics::checkpoint_offset` (0x4415E7): the work car's place against its checkpoint
//! (`g_car` +0x98). With a checkpoint after it (+0x98 + 1 below `g_checkpointCount`, by
//! the sign of the difference): the checkpoint's face (`g_checkpointFaces`), that face's
//! first point (`g_courseFaces` +4, `g_scenePoints`), the car's x and z less the point's,
//! turned by minus the checkpoint's angle (`dword_9ED808`, `math::rotate_point_about_y`) into
//! +0x620 and +0x624. Else both 0.
//!
//! Caller: `physics::track_checkpoints`. Refuses (before anything) a checkpoint below 0 or past
//! 0x1418, or no faces or points.

use crate::errors::{BadInput, Refused};
use crate::math::rotate_point_about_y::rotate_point_about_y;
use crate::race::owner::Race;
use rc2k_core::physics::progress::checkpoint_ahead;

// Converted: `rc2k_core::physics::progress::checkpoint_ahead` (the turn stays
// `math::rotate_point_about_y`'s).
pub fn checkpoint_offset(race: &Race) -> Result<(), Refused> {
    let course_store = &race.course;
    let c = race.shared.car.checkpoint.get();
    if checkpoint_ahead(c, race.shared.vars.checkpoint_count.get())?.is_none() {
        race.shared.car.checkpoint_offset[0].set(0);
        race.shared.car.checkpoint_offset[1].set(0);
        return Ok(());
    }
    if course_store.course.course_faces.get() == 0 || course_store.course.scene_points.get() == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    let angle = crate::race::checkpoints_store::angle(c as usize).get();
    let face = crate::race::checkpoints_store::face(c as usize).get();
    let point = crate::race::course_places::face_field::<u32>(&race.course, face, 4)?;
    let x = race.shared.car.position[0]
        .get()
        .wrapping_sub(crate::race::scene_points::point_word(
            &race.course,
            point,
            0,
        )?);
    let z = race.shared.car.position[2]
        .get()
        .wrapping_sub(crate::race::scene_points::point_word(
            &race.course,
            point,
            2,
        )?);
    let [x2, _, z2, _] =
        rotate_point_about_y(x as i32, point as i32, z as i32, angle.wrapping_neg());
    race.shared.car.checkpoint_offset[0].set(x2);
    race.shared.car.checkpoint_offset[1].set(z2);
    Ok(())
}
