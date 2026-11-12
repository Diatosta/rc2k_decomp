//! `physics::camera_avoid_ground` (0x43907F): the camera kept out of the ground.
//! `physics::ease_view_turn`; `camera_ground_flag` = 0, `camera_turn_step` = -1, the heading
//! (`camera_heading`) kept in `camera_heading_kept`; `physics::place_camera`. With the mode's record's
//! +0x4C set and the camera not over ground (`physics::camera_over_ground`): turned to face
//! from the checkpoint before (`checkpoint` - 1, its face's first scene point,
//! `scene::angle_between` << 16), then the one before that, each placed and tried; none
//! clear: the heading put back. One clear: turned back toward the kept heading in steps of
//! `camera_turn_step` (0x4000000, then each an eighth of the last while above 0x1000), each
//! step placed and tried and undone (the heading and the five last faces `camera_ground_faces` ..
//! kept in `camera_heading_saved` ..) when it leaves the ground. Then `camera_ground_flag` = -1, placed;
//! while not clear of the ground (`physics::camera_clear_of_ground`): the pitch eased up
//! (`physics::ease_pitch_up`, `pitch_floor` the pitch, `pitch_ceiling` 0x40000000 first)
//! or, once clear, down (`ease_pitch_down`, `pitch_ceiling` the pitch), placed and tried,
//! until `pitch_step` is 0.
//!
//! Caller: `physics::update_camera`. Refuses what the ground checks refuse.
//!
//! Seen in play: the camera checks serve the hood camera; when the car
//! flips over, the view changes to chase 1 for a while so the camera does not clip into
//! the ground. (Which mode's record holds +0x4C, and where the switch is made, not read.)

use crate::errors::Refused;
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
use crate::session::PlainDword;
use rc2k_core::physics::camera_ground;

fn place(
    camera: &CameraWork,
    lights_data: &LightsData,
    view_records: &ViewAndRecords,
    numbers: &Numbers,
    frame_work: &FrameWork,
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
) {
    crate::physics::place_camera::place_camera(
        camera,
        lights_data,
        view_records,
        numbers,
        frame_work,
        shared,
        scene,
    );
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(
    checkpoints: &crate::race::checkpoints_store::Checkpoints,
    course: &Course,
    shared: &RaceShared,
    back: u32,
    scene: &crate::scene::owner::Scene,
) -> bool {
    let Some(checkpoint) = camera_ground::back_checkpoint(shared.car().checkpoint.get(), back)
    else {
        return false;
    };
    let face = crate::race::checkpoints_store::face(checkpoints, checkpoint as usize).get();
    let point = crate::race::course_places::face_field::<u32>(course.reads(), face, 4)
        .unwrap_or_else(|e| crate::fatal::raise::end("a checkpoint face", e));
    let coordinate = |k| {
        crate::race::scene_points::point_word(
            course.reads(),
            &crate::race::scene_points::NO_OWN,
            point,
            k,
        )
        .unwrap_or_else(|e| crate::fatal::raise::end("a scene point", e))
    };
    let first = shared.car().position[0].get().wrapping_sub(coordinate(0));
    let second = shared.car().position[2].get().wrapping_sub(coordinate(2));
    let angle = crate::scene::angle_between::angle_of(course, first, second);
    // `scene::angle_between` left the angle in `g_scratch`.
    scene.drawing.scene.get().scratch.set(angle);
    shared.car().camera_heading.set(angle << 16);
    true
}

/// The five ground faces from 0x71AEC8 the loop below saves.
fn ground_faces(car: &Car) -> [&PlainDword; 5] {
    let f = &car.camera_ground_faces;
    [&f[0], &f[1], &f[2], &f[3], &f[4]]
}

/// The turn back toward the kept heading (0x43919D ..).
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
                scene,
            );
            if camera_over_ground(&c.shared.buffers.engine_tables, &c.ground, scene)? {
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
// Converted: `rc2k_core::physics::camera_ground::{back_checkpoint, turn_step,
// next_turn_step}`.
pub fn steps(c: &CameraInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    crate::physics::ease_view_turn::ease_view_turn(c.camera, c.replay_frames, c.car, scene);
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
        scene,
    );
    if record(c.view_records, c.car)?.halve_tilt.get() == 0 {
        return Ok(());
    }
    if !camera_over_ground(&c.shared.buffers.engine_tables, &c.ground, scene)? {
        let mut found = false;
        for back in [1, 2] {
            if !face_checkpoint(c.checkpoints, c.course, c.shared, back, scene) {
                break;
            }
            place(
                c.camera,
                c.lights_data,
                c.view_records,
                c.numbers,
                c.frame_work,
                c.shared,
                scene,
            );
            if camera_over_ground(&c.shared.buffers.engine_tables, &c.ground, scene)? {
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
    // 0x4392A3: clear of the ground by the pitch.
    c.race_tables.camera_ground_flag.set(u32::MAX);
    place(
        c.camera,
        c.lights_data,
        c.view_records,
        c.numbers,
        c.frame_work,
        c.shared,
        scene,
    );
    if camera_clear_of_ground(&c.ground, scene)? {
        return Ok(());
    }
    c.camera.pitch_ceiling.set(camera_ground::PITCH_START);
    let mut up = true;
    loop {
        if up {
            c.camera.pitch_floor.set(c.car.camera_pitch.get());
            crate::physics::ease_pitch_up::ease_pitch_up(c.car, c.camera);
        } else {
            c.camera.pitch_ceiling.set(c.car.camera_pitch.get());
            crate::physics::ease_pitch_down::ease_pitch_down(c.car, c.camera);
        }
        if c.camera.pitch_step.get() == 0 {
            return Ok(());
        }
        place(
            c.camera,
            c.lights_data,
            c.view_records,
            c.numbers,
            c.frame_work,
            c.shared,
            scene,
        );
        up = !camera_clear_of_ground(&c.ground, scene)?;
    }
}
