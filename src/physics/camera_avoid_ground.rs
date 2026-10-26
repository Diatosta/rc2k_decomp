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
//! In:  nothing
//! Out: the registers as the steps and calls leave them; no flags read (the caller goes
//!      on)
//!
//! Caller: `physics::update_camera`. Refuses what the ground checks refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::camera_clear_of_ground::camera_clear_of_ground;
use crate::physics::camera_over_ground::camera_over_ground;
use crate::physics::camera_store::CAMERA_WORK;
use crate::physics::camera_view::{rd, record};
use crate::physics::car_store::CAR;
use crate::race::course_store::COURSE;
use crate::race::tables_store::RACE_TABLES;
use crate::scene::draw_store::DRAW;
use crate::session::Dword;
use rc2k_core::physics::camera_ground;

fn place() {
    unsafe { crate::physics::place_camera::place_camera() };
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(back: u32) -> bool {
    let Some(checkpoint) = camera_ground::back_checkpoint(CAR.checkpoint.get(), back) else {
        return false;
    };
    let face = (crate::race::checkpoints_store::face(checkpoint as usize).get() << 6)
        .wrapping_add(COURSE.course_faces.get());
    let point = rd(face + 4)
        .wrapping_mul(12)
        .wrapping_add(COURSE.scene_points.get());
    let first = CAR.position[0].get().wrapping_sub(rd(point));
    let second = CAR.position[2].get().wrapping_sub(rd(point + 8));
    let angle = crate::scene::angle_between::angle_of(first, second);
    CAR.camera_heading.set(angle << 16);
    true
}

/// The five ground faces from 0x71AEC8 the loop below saves.
fn ground_faces() -> [&'static Dword; 5] {
    let f = &CAR.camera_ground_faces;
    [&f[0], &f[1], &f[2], &f[3], &f[4]]
}

/// The turn back toward the kept heading (0x43919D ..).
fn turn_back() -> Result<(), Refused> {
    RACE_TABLES
        .camera_turn_step
        .set(camera_ground::FIRST_TURN_STEP);
    loop {
        DRAW.camera_heading_saved.set(CAR.camera_heading.get());
        for k in 0..5 {
            DRAW.camera_saved[k].set(ground_faces()[k].get());
        }
        let mut stepped_back = false;
        if let Some(step) = camera_ground::turn_step(
            DRAW.camera_heading_kept.get(),
            CAR.camera_heading.get(),
            RACE_TABLES.camera_turn_step.get(),
        ) {
            CAR.camera_heading
                .set(CAR.camera_heading.get().wrapping_add(step));
            place();
            if camera_over_ground()? {
                continue;
            }
            stepped_back = true;
        }
        if stepped_back {
            CAR.camera_heading.set(DRAW.camera_heading_saved.get());
            for k in 0..5 {
                ground_faces()[k].set(DRAW.camera_saved[k].get());
            }
        }
        let (next, last) = camera_ground::next_turn_step(RACE_TABLES.camera_turn_step.get());
        RACE_TABLES.camera_turn_step.set(next);
        if last {
            return Ok(());
        }
    }
}

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::{back_checkpoint, turn_step,
// next_turn_step}`.
pub fn steps() -> Result<(), Refused> {
    crate::physics::ease_view_turn::ease_view_turn();
    RACE_TABLES.camera_ground_flag.set(0);
    RACE_TABLES.camera_turn_step.set(u32::MAX);
    DRAW.camera_heading_kept.set(CAR.camera_heading.get());
    place();
    if rd(record() + 0x4C) == 0 {
        return Ok(());
    }
    if !camera_over_ground()? {
        let mut found = false;
        for back in [1, 2] {
            if !face_checkpoint(back) {
                break;
            }
            place();
            if camera_over_ground()? {
                found = true;
                break;
            }
        }
        if found {
            turn_back()?;
        } else {
            CAR.camera_heading.set(DRAW.camera_heading_kept.get());
        }
    }
    // 0x4392A3: clear of the ground by the pitch.
    RACE_TABLES.camera_ground_flag.set(u32::MAX);
    place();
    if camera_clear_of_ground()? {
        return Ok(());
    }
    CAMERA_WORK.pitch_ceiling.set(camera_ground::PITCH_START);
    let mut up = true;
    loop {
        if up {
            CAMERA_WORK.pitch_floor.set(CAR.camera_pitch.get());
            crate::physics::ease_pitch_up::ease_pitch_up();
        } else {
            CAMERA_WORK.pitch_ceiling.set(CAR.camera_pitch.get());
            crate::physics::ease_pitch_down::ease_pitch_down();
        }
        if CAMERA_WORK.pitch_step.get() == 0 {
            return Ok(());
        }
        place();
        up = !camera_clear_of_ground()?;
    }
}
