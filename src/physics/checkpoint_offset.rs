//! The work car's place against its checkpoint. With a checkpoint after the car's (its number + 1
//! below the checkpoint count): the checkpoint's face's first point, the car's x and z less the
//! point's, turned by minus the checkpoint's angle (`math::rotate_point_about_y`) into the two
//! checkpoint offset fields. Otherwise both are 0.
//!
//! Refuses (before anything) a checkpoint below 0 or past 0x1418, or no faces or points.

use crate::errors::{BadInput, Refused};
use crate::math::rotate_point_about_y::rotate_point_about_y;
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::race::owner_course::Course;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::progress::checkpoint_ahead;

// Core: `rc2k_core::physics::progress::checkpoint_ahead` (the turn stays
// `math::rotate_point_about_y`'s).
pub fn checkpoint_offset(
    engine_tables: crate::math::engine_tables::EngineTablesRef,
    course: &Course,
    vars: &RaceVars,
    checkpoints: crate::race::checkpoints_store::CheckpointsRef,
    car: &mut Car,
) -> Result<(), Refused> {
    let c = car.checkpoint;
    if checkpoint_ahead(c, vars.checkpoint_count.get())?.is_none() {
        car.checkpoint_offset[0] = 0;
        car.checkpoint_offset[1] = 0;
        return Ok(());
    }
    if !crate::race::course_places::held(&course.stage.course)
        || !crate::race::scene_points::held(&course.stage.course)
    {
        return Err(BadInput("no course faces or points").into());
    }
    let angle = crate::race::checkpoints_store::angle_run_on(checkpoints, c as usize);
    let face = crate::race::checkpoints_store::face_run_on(checkpoints, c as usize);
    let point = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)?;
    let x = car
        .position
        .bits_at(0)
        .wrapping_sub(crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            0,
        )?);
    let z = car
        .position
        .bits_at(2)
        .wrapping_sub(crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::no_own(),
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
    car.checkpoint_offset[0] = x2;
    car.checkpoint_offset[1] = z2;
    Ok(())
}
