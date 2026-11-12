//! `physics::checkpoint_offset` (0x4415E7): the work car's place against its checkpoint
//! (`g_car` +0x98). With a checkpoint after it (+0x98 + 1 below `g_checkpointCount`, by
//! the sign of the difference): the checkpoint's face (`g_checkpointFaces`), that face's
//! first point (`g_courseFaces` +4, `g_scenePoints`), the car's x and z less the point's,
//! turned by minus the checkpoint's angle (`Buffers.checkpoints.angles[2]`, `math::rotate_point_about_y`) into
//! +0x620 and +0x624. Else both 0.
//!
//! Caller: `physics::track_checkpoints`. Refuses (before anything) a checkpoint below 0 or past
//! 0x1418, or no faces or points.

use crate::errors::{BadInput, Refused};
use crate::math::rotate_point_about_y::rotate_point_about_y;
use crate::physics::car_store::Car;
use crate::race::owner_course::Course;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::progress::checkpoint_ahead;

// Converted: `rc2k_core::physics::progress::checkpoint_ahead` (the turn stays
// `math::rotate_point_about_y`'s).
pub fn checkpoint_offset(
    engine_tables: &crate::scene::engine_tables_store::EngineTables,
    course: &Course,
    vars: &RaceVars,
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    car: &Car,
) -> Result<(), Refused> {
    let course_store = course;
    let c = car.checkpoint.get();
    if checkpoint_ahead(c, vars.checkpoint_count.get())?.is_none() {
        car.checkpoint_offset[0].set(0);
        car.checkpoint_offset[1].set(0);
        return Ok(());
    }
    if course_store.course.get().course_faces.get() == 0
        || course_store.course.get().scene_points.get() == 0
    {
        return Err(BadInput("no course faces or points").into());
    }
    let angle = crate::race::checkpoints_store::angle_run_on(checkpoints, c as usize).get();
    let face = crate::race::checkpoints_store::face_run_on(checkpoints, c as usize).get();
    let point = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)?;
    let x = car.position[0]
        .get()
        .wrapping_sub(crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::NO_OWN,
            point,
            0,
        )?);
    let z = car.position[2]
        .get()
        .wrapping_sub(crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::NO_OWN,
            point,
            2,
        )?);
    let [x2, _, z2, _] = rotate_point_about_y(
        engine_tables,
        x as i32,
        point as i32,
        z as i32,
        angle.wrapping_neg(),
    );
    car.checkpoint_offset[0].set(x2);
    car.checkpoint_offset[1].set(z2);
    Ok(())
}
