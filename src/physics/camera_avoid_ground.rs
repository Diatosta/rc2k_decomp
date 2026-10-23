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
//! Caller: `physics::update_camera`. It begins with a call: hooked by a private copy of
//! the whole function (0x291 bytes, one chunk) with its 19 calls relocated and a
//! hand-made patch.
//! Refuses what the ground checks refuse. Shadow mode runs the original alone (as the
//! checks: `find_ground`'s probes).
//! Self-test (in a race; skipped otherwise): the original and the Rust version on the
//! stage (`physics/update_camera.rs`, `camera_test`), the camera where it is, moved below
//! the car and into the ground, every mode.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::camera_clear_of_ground::camera_clear_of_ground;
use crate::physics::camera_over_ground::camera_over_ground;
use crate::physics::camera_view::{rd, record};
use crate::physics::flow::{g, p};
use crate::physics::touch_face::FACES;
use crate::race::car_starts::Part;
use rc2k_core::physics::camera_ground;

const HEADING: usize = 0x71A688;
const STEP: usize = 0x605264;
const KEPT: usize = 0x623FD8;

fn place(r: &mut Regs) {
    unsafe { crate::physics::place_camera::place_camera(r) }.apply(r);
}

/// Placed, and whether over ground (`camera_over_ground`'s CF clear).
fn over(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    *r = unsafe { camera_over_ground(r, part) }?;
    Ok(!r.carry())
}

fn clear(r: &mut Regs, part: Part) -> Result<bool, Refused> {
    *r = unsafe { camera_clear_of_ground(r, part) }?;
    Ok(!r.carry())
}

/// Turned to face from checkpoint `back` before the car's; false when there is none.
fn face_checkpoint(r: &mut Regs, back: u32) -> bool {
    r.ebx = g(0x71A070).wrapping_sub(back);
    if camera_ground::back_checkpoint(g(0x71A070), back).is_none() {
        return false;
    }
    r.ebx = g(0x9E3738 + 4 * r.ebx as usize);
    r.esi = (r.ebx << 6).wrapping_add(g(FACES));
    r.ebx = rd(r.esi + 4);
    r.esi = r
        .ebx
        .wrapping_mul(12)
        .wrapping_add(g(crate::physics::scene_point_hits_car::SCENE_POINTS));
    r.eax = rd(r.esi);
    r.ebx = rd(r.esi + 8);
    r.ecx = g(0x71A10C);
    r.edx = g(0x71A114);
    let (first, second) = (r.ecx.wrapping_sub(r.eax), r.edx.wrapping_sub(r.ebx));
    r.eax = unsafe { crate::scene::angle_between::angle_of(first, second) };
    r.ecx = first;
    r.edx = second;
    p(HEADING, r.eax << 16);
    true
}

/// The turn back toward the kept heading (0x43919D ..).
fn turn_back(r: &mut Regs, part: Part) -> Result<(), Refused> {
    p(STEP, camera_ground::FIRST_TURN_STEP);
    loop {
        p(0x623FD4, g(HEADING));
        for k in 0..5 {
            p(0x623FDC + 4 * k, g(0x71AEC8 + 4 * k));
        }
        r.eax = camera_ground::turn_gap(g(KEPT), g(HEADING));
        let mut stepped_back = false;
        if let Some(step) = camera_ground::turn_step(g(KEPT), g(HEADING), g(STEP)) {
            r.ebx = step;
            r.eax = g(KEPT).wrapping_sub(g(HEADING));
            p(HEADING, g(HEADING).wrapping_add(r.ebx));
            place(r);
            if over(r, part)? {
                continue;
            }
            stepped_back = true;
        }
        if stepped_back {
            p(HEADING, g(0x623FD4));
            for k in 0..5 {
                p(0x71AEC8 + 4 * k, g(0x623FDC + 4 * k));
            }
        }
        let (next, last) = camera_ground::next_turn_step(g(STEP));
        p(STEP, next);
        if last {
            return Ok(());
        }
    }
}

/// The steps; `r` as the original leaves it.
// Converted: `rc2k_core::physics::camera_ground::{back_checkpoint, turn_gap, turn_step,
// next_turn_step}`.
pub fn steps(r: &mut Regs, part: Part) -> Result<(), Refused> {
    unsafe { crate::physics::ease_view_turn::ease_view_turn(r) }.apply(r);
    p(0x605268, 0);
    p(STEP, u32::MAX);
    p(KEPT, g(HEADING));
    place(r);
    r.esi = record();
    if rd(r.esi + 0x4C) == 0 {
        return Ok(());
    }
    if !over(r, part)? {
        let mut found = false;
        for back in [1, 2] {
            if !face_checkpoint(r, back) {
                break;
            }
            place(r);
            if over(r, part)? {
                found = true;
                break;
            }
        }
        if found {
            turn_back(r, part)?;
        } else {
            p(HEADING, g(KEPT));
        }
    }
    // 0x4392A3: clear of the ground by the pitch.
    p(0x605268, u32::MAX);
    place(r);
    if clear(r, part)? {
        return Ok(());
    }
    p(0x608C20, camera_ground::PITCH_START);
    let mut up = true;
    loop {
        r.eax = g(0x71A690);
        if up {
            p(0x608C1C, r.eax);
            unsafe { crate::physics::ease_pitch_up::ease_pitch_up() }.apply(r);
        } else {
            p(0x608C20, r.eax);
            unsafe { crate::physics::ease_pitch_down::ease_pitch_down() }.apply(r);
        }
        if g(0x608C24) == 0 {
            return Ok(());
        }
        place(r);
        up = !clear(r, part)?;
    }
}
