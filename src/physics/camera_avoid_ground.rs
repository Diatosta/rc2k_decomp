//! `physics::camera_avoid_ground` (0x43907F): the camera kept out of the ground.
//! `physics::ease_view_turn`; `dword_605268` = 0, `dword_605264` = -1, the heading
//! (`unk_71A688`) kept in `dword_623FD8`; `physics::place_camera`. With the mode's record's
//! +0x4C set and the camera not over ground (`physics::camera_over_ground`): turned to face
//! from the checkpoint before (`unk_71A070` - 1, its face's first scene point,
//! `scene::angle_between` << 16), then the one before that, each placed and tried; none
//! clear: the heading put back. One clear: turned back toward the kept heading in steps of
//! `dword_605264` (0x4000000, then each an eighth of the last while above 0x1000), each
//! step placed and tried and undone (the heading and the five last faces `unk_71AEC8` ..
//! kept in `dword_623FD4` ..) when it leaves the ground. Then `dword_605268` = -1, placed;
//! while not clear of the ground (`physics::camera_clear_of_ground`): the pitch eased up
//! (`physics::ease_pitch_up`, `dword_608C1C` the pitch, `dword_608C20` 0x40000000 first)
//! or, once clear, down (`ease_pitch_down`, `dword_608C20` the pitch), placed and tried,
//! until `dword_608C24` is 0.
//!
//! Caller: `physics::update_camera`. Refuses what the ground checks refuse.
//!
//! Seen in play (owner, 2026-10-27): the camera checks serve the hood camera; when the car
//! flips over, the view changes to chase 1 for a while so the camera does not clip into
//! the ground. (Which mode's record holds +0x4C, and where the switch is made, not read.)
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_clear_of_ground::camera_clear_of_ground;
use crate::physics::camera_over_ground::camera_over_ground;
use crate::physics::camera_view::record;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_shared::Shared;
use crate::session::Dword;
use rc2k_core::physics::camera_ground;

fn place(race: &Race, scene: &crate::scene::owner::Scene) {
    crate::physics::place_camera::place_camera(race, scene);
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(course: &Course, shared: &Shared, back: u32) -> bool {
    let Some(checkpoint) = camera_ground::back_checkpoint(shared.car.checkpoint.get(), back) else {
        return false;
    };
    let face = crate::race::checkpoints_store::face(checkpoint as usize).get();
    let point = crate::race::course_places::face_field::<u32>(face, 4, "a checkpoint face")
        .unwrap_or_else(|e| crate::fatal::raise::end("a checkpoint face", e));
    let coordinate = |k| {
        crate::race::scene_points::point_word(point, k)
            .unwrap_or_else(|e| crate::fatal::raise::end("a scene point", e))
    };
    let first = shared.car.position[0].get().wrapping_sub(coordinate(0));
    let second = shared.car.position[2].get().wrapping_sub(coordinate(2));
    let angle = crate::scene::angle_between::angle_of(course, first, second);
    // `scene::angle_between` left the angle in `g_scratch`.
    crate::scene::owner::scene()
        .drawing
        .scene
        .scratch
        .set_bits(angle);
    shared.car.camera_heading.set(angle << 16);
    true
}

/// The five ground faces from 0x71AEC8 the loop below saves.
fn ground_faces(shared: &Shared) -> [&Dword; 5] {
    let f = &shared.car.camera_ground_faces;
    [&f[0], &f[1], &f[2], &f[3], &f[4]]
}

/// The turn back toward the kept heading (0x43919D ..).
fn turn_back(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    race.shared
        .tables
        .camera_turn_step
        .set(camera_ground::FIRST_TURN_STEP);
    loop {
        drawing
            .scene
            .camera_heading_saved
            .set(race.shared.car.camera_heading.get());
        for k in 0..5 {
            drawing.scene.camera_saved[k].set(ground_faces(&race.shared)[k].get());
        }
        let mut stepped_back = false;
        if let Some(step) = camera_ground::turn_step(
            drawing.scene.camera_heading_kept.get(),
            race.shared.car.camera_heading.get(),
            race.shared.tables.camera_turn_step.get(),
        ) {
            race.shared
                .car
                .camera_heading
                .set(race.shared.car.camera_heading.get().wrapping_add(step));
            place(race, scene);
            if camera_over_ground(race, scene)? {
                continue;
            }
            stepped_back = true;
        }
        if stepped_back {
            race.shared
                .car
                .camera_heading
                .set(drawing.scene.camera_heading_saved.get());
            for k in 0..5 {
                ground_faces(&race.shared)[k].set(drawing.scene.camera_saved[k].get());
            }
        }
        let (next, last) = camera_ground::next_turn_step(race.shared.tables.camera_turn_step.get());
        race.shared.tables.camera_turn_step.set(next);
        if last {
            return Ok(());
        }
    }
}

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::{back_checkpoint, turn_step,
// next_turn_step}`.
pub fn steps(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let drawing = &scene.drawing;
    let physics_store = &race.physics;
    crate::physics::ease_view_turn::ease_view_turn(race, scene);
    race.shared.tables.camera_ground_flag.set(0);
    race.shared.tables.camera_turn_step.set(u32::MAX);
    drawing
        .scene
        .camera_heading_kept
        .set(race.shared.car.camera_heading.get());
    place(race, scene);
    if record(&race.shared)?.halve_tilt.get() == 0 {
        return Ok(());
    }
    if !camera_over_ground(race, scene)? {
        let mut found = false;
        for back in [1, 2] {
            if !face_checkpoint(&race.course, &race.shared, back) {
                break;
            }
            place(race, scene);
            if camera_over_ground(race, scene)? {
                found = true;
                break;
            }
        }
        if found {
            turn_back(race, scene)?;
        } else {
            race.shared
                .car
                .camera_heading
                .set(drawing.scene.camera_heading_kept.get());
        }
    }
    // 0x4392A3: clear of the ground by the pitch.
    race.shared.tables.camera_ground_flag.set(u32::MAX);
    place(race, scene);
    if camera_clear_of_ground(race, scene)? {
        return Ok(());
    }
    physics_store
        .camera
        .pitch_ceiling
        .set(camera_ground::PITCH_START);
    let mut up = true;
    loop {
        if up {
            physics_store
                .camera
                .pitch_floor
                .set(race.shared.car.camera_pitch.get());
            crate::physics::ease_pitch_up::ease_pitch_up(race);
        } else {
            physics_store
                .camera
                .pitch_ceiling
                .set(race.shared.car.camera_pitch.get());
            crate::physics::ease_pitch_down::ease_pitch_down(race);
        }
        if physics_store.camera.pitch_step.get() == 0 {
            return Ok(());
        }
        place(race, scene);
        up = !camera_clear_of_ground(race, scene)?;
    }
}
