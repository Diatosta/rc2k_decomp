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
use crate::physics::camera_view::{rd, record};
use crate::physics::car_store::CAR;
use crate::physics::flow::{g, p};
use crate::physics::touch_face::FACES;
use crate::scene::draw_store::DRAW;
use rc2k_core::physics::camera_ground;

const STEP: usize = 0x605264;

fn place() {
    unsafe { crate::physics::place_camera::place_camera() };
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(back: u32) -> bool {
    let Some(checkpoint) = camera_ground::back_checkpoint(CAR.checkpoint.get(), back) else {
        return false;
    };
    let face = (g(0x9E3738 + 4 * checkpoint as usize) << 6).wrapping_add(g(FACES));
    let point = rd(face + 4)
        .wrapping_mul(12)
        .wrapping_add(g(crate::physics::scene_point_hits_car::SCENE_POINTS));
    let first = CAR.position[0].get().wrapping_sub(rd(point));
    let second = CAR.position[2].get().wrapping_sub(rd(point + 8));
    let angle = unsafe { crate::scene::angle_between::angle_of(first, second) };
    CAR.camera_heading.set(angle << 16);
    true
}

/// The turn back toward the kept heading (0x43919D ..).
fn turn_back() -> Result<(), Refused> {
    p(STEP, camera_ground::FIRST_TURN_STEP);
    loop {
        DRAW.camera_heading_saved.set(CAR.camera_heading.get());
        for k in 0..5 {
            DRAW.camera_saved[k].set(g(0x71AEC8 + 4 * k));
        }
        let mut stepped_back = false;
        if let Some(step) = camera_ground::turn_step(
            DRAW.camera_heading_kept.get(),
            CAR.camera_heading.get(),
            g(STEP),
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
                p(0x71AEC8 + 4 * k, DRAW.camera_saved[k].get());
            }
        }
        let (next, last) = camera_ground::next_turn_step(g(STEP));
        p(STEP, next);
        if last {
            return Ok(());
        }
    }
}

/// The steps. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::camera_ground::{back_checkpoint, turn_step,
// next_turn_step}`.
pub fn steps() -> Result<(), Refused> {
    unsafe { crate::physics::ease_view_turn::ease_view_turn() };
    p(0x605268, 0);
    p(STEP, u32::MAX);
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
    p(0x605268, u32::MAX);
    place();
    if camera_clear_of_ground()? {
        return Ok(());
    }
    p(0x608C20, camera_ground::PITCH_START);
    let mut up = true;
    loop {
        if up {
            p(0x608C1C, CAR.camera_pitch.get());
            unsafe { crate::physics::ease_pitch_up::ease_pitch_up() };
        } else {
            p(0x608C20, CAR.camera_pitch.get());
            unsafe { crate::physics::ease_pitch_down::ease_pitch_down() };
        }
        if g(0x608C24) == 0 {
            return Ok(());
        }
        place();
        up = !camera_clear_of_ground()?;
    }
}
