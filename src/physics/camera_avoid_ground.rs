//! Keeps the camera out of the ground. After `physics::ease_view_turn` and `place_camera`: when
//! the mode's record flag is set and the camera is not over ground
//! (`physics::camera_over_ground`), the camera is turned to face from the checkpoint before, then
//! the one before that, each placed and tried; with none clear the heading is put back. With one
//! clear it is turned back toward the kept heading in steps of `camera_turn_step` (0x4000000,
//! then an eighth of the last while above 0x1000), each step placed, tried and undone when it
//! leaves the ground. Then, while not clear of the ground (`physics::camera_clear_of_ground`),
//! the pitch is eased up (`ease_pitch_up`) or, once clear, down (`ease_pitch_down`), placed and
//! tried, until `pitch_step` is 0.
//!
//! Refuses what the ground checks refuse.
//!
//! Seen in play: the camera checks serve the hood camera; when the car flips over the view
//! changes to chase 1 for a while so the camera does not clip into the ground.

use crate::errors::Refused;
use crate::memory::plain_floats::FloatRow;
use crate::memory::session::DrawArc;
use crate::physics::camera_clear_of_ground::camera_clear_of_ground;
use crate::physics::camera_over_ground::camera_over_ground;
use crate::physics::camera_store::CameraWork;
use crate::physics::camera_view::record;
use crate::physics::car_store::Car;
use crate::physics::step_families::CameraInputs;
use crate::race::frame_store::FrameWorkStore;
use crate::race::groups_store::ViewAndRecords;
use crate::race::lights_store::LightsData;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::race::owner_shared::RaceShared;
use rc2k_core::physics::camera_ground;
use std::cell::RefCell;

fn place(
    camera: &RefCell<CameraWork>,
    lights_data: &LightsData,
    view_records: &DrawArc<ViewAndRecords>,
    numbers: &Numbers,
    frame_work: &FrameWorkStore,
    shared: &RaceShared,
    car: &mut Car,
) {
    crate::physics::place_camera::place_camera(
        &camera.borrow(),
        lights_data,
        view_records,
        numbers,
        frame_work,
        shared,
        car,
    );
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(
    checkpoints: crate::race::checkpoints_store::CheckpointsRef,
    course: &Course,
    car: &mut Car,
    back: u32,
) -> bool {
    let Some(checkpoint) = camera_ground::back_checkpoint(car.checkpoint, back) else {
        return false;
    };
    let face = crate::race::checkpoints_store::face_run_on(checkpoints, checkpoint as usize);
    let point = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)
        .unwrap_or_else(|e| crate::fatal::raise::end("a checkpoint face", e));
    let coordinate = |k| {
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            k,
        )
        .unwrap_or_else(|e| crate::fatal::raise::end("a scene point", e))
    };
    let first = car.position.bits_at(0).wrapping_sub(coordinate(0));
    let second = car.position.bits_at(2).wrapping_sub(coordinate(2));
    let angle = crate::scene::angle_between::angle_of(course, first, second);
    car.camera_heading = angle << 16;
    true
}

/// The turn back toward the kept heading.
fn turn_back(
    c: &CameraInputs<'_>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<(), Refused> {
    c.race_tables
        .camera_turn_step
        .set(camera_ground::FIRST_TURN_STEP);
    loop {
        scene.draw_work_mut().camera_heading_saved = car.camera_heading;
        scene.draw_work_mut().camera_saved = car.camera_ground_faces;
        let mut stepped_back = false;
        if let Some(step) = camera_ground::turn_step(
            scene.draw_work().camera_heading_kept,
            car.camera_heading,
            c.race_tables.camera_turn_step.get(),
        ) {
            car.camera_heading = car.camera_heading.wrapping_add(step);
            place(
                c.camera,
                c.lights_data,
                c.view_records,
                c.numbers,
                c.frame_work,
                c.shared,
                car,
            );
            if camera_over_ground(c.shared.buffers.engine_tables(), &c.ground, car)? {
                continue;
            }
            stepped_back = true;
        }
        if stepped_back {
            car.camera_heading = scene.draw_work().camera_heading_saved;
            car.camera_ground_faces = scene.draw_work().camera_saved;
        }
        let (next, last) = camera_ground::next_turn_step(c.race_tables.camera_turn_step.get());
        c.race_tables.camera_turn_step.set(next);
        if last {
            return Ok(());
        }
    }
}

/// The steps.
// Core: `rc2k_core::physics::camera_ground::{back_checkpoint, turn_step,
// next_turn_step}`.
pub fn steps(
    c: &CameraInputs<'_>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<(), Refused> {
    crate::physics::ease_view_turn::ease_view_turn(&mut c.camera.borrow_mut(), c.step_first, car);
    c.race_tables.camera_ground_flag.set(0);
    c.race_tables.camera_turn_step.set(u32::MAX);
    scene.draw_work_mut().camera_heading_kept = car.camera_heading;
    place(
        c.camera,
        c.lights_data,
        c.view_records,
        c.numbers,
        c.frame_work,
        c.shared,
        car,
    );
    if record(c.view_records, car)?.halve_tilt == 0 {
        return Ok(());
    }
    if !camera_over_ground(c.shared.buffers.engine_tables(), &c.ground, car)? {
        let mut found = false;
        for back in [1, 2] {
            if !face_checkpoint(c.checkpoints, c.course, car, back) {
                break;
            }
            place(
                c.camera,
                c.lights_data,
                c.view_records,
                c.numbers,
                c.frame_work,
                c.shared,
                car,
            );
            if camera_over_ground(c.shared.buffers.engine_tables(), &c.ground, car)? {
                found = true;
                break;
            }
        }
        if found {
            turn_back(c, scene, car)?;
        } else {
            car.camera_heading = scene.draw_work().camera_heading_kept;
        }
    }
    // Clear of the ground by the pitch.
    c.race_tables.camera_ground_flag.set(u32::MAX);
    place(
        c.camera,
        c.lights_data,
        c.view_records,
        c.numbers,
        c.frame_work,
        c.shared,
        car,
    );
    if camera_clear_of_ground(&c.ground, car)? {
        return Ok(());
    }
    c.camera.borrow_mut().pitch_ceiling = camera_ground::PITCH_START;
    let mut up = true;
    loop {
        if up {
            c.camera.borrow_mut().pitch_floor = car.camera_pitch;
            crate::physics::ease_pitch_up::ease_pitch_up(car, &mut c.camera.borrow_mut());
        } else {
            c.camera.borrow_mut().pitch_ceiling = car.camera_pitch;
            crate::physics::ease_pitch_down::ease_pitch_down(car, &mut c.camera.borrow_mut());
        }
        if c.camera.borrow().pitch_step == 0 {
            return Ok(());
        }
        place(
            c.camera,
            c.lights_data,
            c.view_records,
            c.numbers,
            c.frame_work,
            c.shared,
            car,
        );
        up = !camera_clear_of_ground(&c.ground, car)?;
    }
}
