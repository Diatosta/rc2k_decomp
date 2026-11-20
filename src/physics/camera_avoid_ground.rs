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
use crate::memory::session::PlainDword;
use crate::physics::camera_clear_of_ground::camera_clear_of_ground;
use crate::physics::camera_over_ground::camera_over_ground;
use crate::physics::camera_store::CameraWork;
use crate::physics::camera_view::record;
use crate::physics::car_store::Car;
use crate::physics::step_families::CameraInputs;
use crate::race::frame_store::FrameWork;
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
    view_records: &ViewAndRecords,
    numbers: &Numbers,
    frame_work: &FrameWork,
    shared: &RaceShared,
) {
    crate::physics::place_camera::place_camera(
        &camera.borrow(),
        lights_data,
        view_records,
        numbers,
        frame_work,
        shared,
    );
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    course: &Course,
    shared: &RaceShared,
    back: u32,
) -> bool {
    let Some(checkpoint) = camera_ground::back_checkpoint(shared.car().checkpoint.get(), back)
    else {
        return false;
    };
    let face = crate::race::checkpoints_store::face_run_on(checkpoints, checkpoint as usize).get();
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
    let first = shared.car().position[0].get().wrapping_sub(coordinate(0));
    let second = shared.car().position[2].get().wrapping_sub(coordinate(2));
    let angle = crate::scene::angle_between::angle_of(course, first, second);
    shared.car().camera_heading.set(angle << 16);
    true
}

/// The five ground faces the loop below saves.
fn ground_faces(car: &Car) -> [&PlainDword; 5] {
    let f = &car.camera_ground_faces;
    [&f[0], &f[1], &f[2], &f[3], &f[4]]
}

/// The turn back toward the kept heading.
fn turn_back(c: &CameraInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    c.race_tables
        .camera_turn_step
        .set(camera_ground::FIRST_TURN_STEP);
    loop {
        drawing
            .scene
            .get()
            .camera_heading_saved
            .set(c.car.camera_heading.get());
        for k in 0..5 {
            drawing.scene.get().camera_saved[k].set(ground_faces(c.car)[k].get());
        }
        let mut stepped_back = false;
        if let Some(step) = camera_ground::turn_step(
            drawing.scene.get().camera_heading_kept.get(),
            c.car.camera_heading.get(),
            c.race_tables.camera_turn_step.get(),
        ) {
            c.car
                .camera_heading
                .set(c.car.camera_heading.get().wrapping_add(step));
            place(
                c.camera,
                c.lights_data,
                c.view_records,
                c.numbers,
                c.frame_work,
                c.shared,
            );
            if camera_over_ground(&c.shared.buffers.reads.get().engine_tables, &c.ground)? {
                continue;
            }
            stepped_back = true;
        }
        if stepped_back {
            c.car
                .camera_heading
                .set(drawing.scene.get().camera_heading_saved.get());
            for k in 0..5 {
                ground_faces(c.car)[k].set(drawing.scene.get().camera_saved[k].get());
            }
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
pub fn steps(c: &CameraInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    crate::physics::ease_view_turn::ease_view_turn(&mut c.camera.borrow_mut(), c.step_first, c.car);
    c.race_tables.camera_ground_flag.set(0);
    c.race_tables.camera_turn_step.set(u32::MAX);
    drawing
        .scene
        .get()
        .camera_heading_kept
        .set(c.car.camera_heading.get());
    place(
        c.camera,
        c.lights_data,
        c.view_records,
        c.numbers,
        c.frame_work,
        c.shared,
    );
    if record(c.view_records, c.car)?.halve_tilt.get() == 0 {
        return Ok(());
    }
    if !camera_over_ground(&c.shared.buffers.reads.get().engine_tables, &c.ground)? {
        let mut found = false;
        for back in [1, 2] {
            if !face_checkpoint(c.checkpoints, c.course, c.shared, back) {
                break;
            }
            place(
                c.camera,
                c.lights_data,
                c.view_records,
                c.numbers,
                c.frame_work,
                c.shared,
            );
            if camera_over_ground(&c.shared.buffers.reads.get().engine_tables, &c.ground)? {
                found = true;
                break;
            }
        }
        if found {
            turn_back(c, scene)?;
        } else {
            c.car
                .camera_heading
                .set(drawing.scene.get().camera_heading_kept.get());
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
    );
    if camera_clear_of_ground(&c.ground)? {
        return Ok(());
    }
    c.camera.borrow_mut().pitch_ceiling = camera_ground::PITCH_START;
    let mut up = true;
    loop {
        if up {
            c.camera.borrow_mut().pitch_floor = c.car.camera_pitch.get();
            crate::physics::ease_pitch_up::ease_pitch_up(c.car, &mut c.camera.borrow_mut());
        } else {
            c.camera.borrow_mut().pitch_ceiling = c.car.camera_pitch.get();
            crate::physics::ease_pitch_down::ease_pitch_down(c.car, &mut c.camera.borrow_mut());
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
        );
        up = !camera_clear_of_ground(&c.ground)?;
    }
}
