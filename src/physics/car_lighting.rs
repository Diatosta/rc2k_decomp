//! `physics::car_lighting` (0x44ECE4): the light on the work car from the face under it.
//! The car's x / z into the probe point, the old light (+0x244) into `dword_5CCCA8`,
//! `dword_62DAFC` / `62DB00` 0, the last face (+0xE00) into `dword_5CCC78`;
//! `physics::find_ground`; no face: done. With `dword_5CCC6C` 1 the shares
//! (`dword_639800`, `804`) taken from 0x10000; kept in `dword_62DB08` / `0C`. The face's
//! four corner lights (+0x14 ..; with its texture's +0x2C 1 each scaled by
//! ((byte +0x22 + k) >> 1 ^ 0x7F) + 0x80, `mul`, >> 8) blended by the shares (`imul`,
//! `shrd` 16) into +0x244; with +0xFB4 set the change held to 10 a step (or none with
//! `dword_6235E8`); `g_lightTable[it]` into `dword_62DAFC` and +0x240. With bit 2 of the
//! face's +2 also two colour blends (+0x28 .., +0x2C ..) through `dword_635518[]` /
//! `dword_635918[]` into `dword_62DB00` / `62DB04`, each added to +0x240 byte by byte
//! with saturation (four bytes with `g_hasMmx`, else the low three).
//!
//! In:  nothing
//! Out: eax, ebx (the texture's record), ecx, edx, esi (the face) as left; no flags read
//!
//! Caller: `sub_43B635` (0x43B990), every physics step. Refuses a light index past 0xFF
//! (the tables are 256) and no faces; what was written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::physics::car_store::CAR;
use crate::race::course_store::COURSE;
use crate::race::vars_store::RACE_VARS;
use crate::session::MACHINE;
use rc2k_core::physics::lighting;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn byte(a: u32) -> u32 {
    unsafe { (a as *const u8).read() as u32 }
}

/// The two-way blend of the four corners now in `dword_5CCC90` ..: into `5CCCA4`,
/// `5CCCA0`, returned.
fn corners() -> u32 {
    let c = [0, 1, 2, 3].map(|k| g(0x5CCC90 + 4 * k));
    let ([a, b], v, _) = lighting::corners(c, g(0x639800), g(0x639804));
    COURSE.face_light_a.set(a);
    COURSE.face_light_b.set(b);
    v
}

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::lighting`.
pub fn car_lighting() -> Result<(), Refused> {
    p(0x639824, CAR.position[0].get());
    p(0x63982C, CAR.position[2].get());
    COURSE.light_kept.set(CAR.light_level.get());
    p(0x62DAFC, 0);
    p(0x62DB00, 0);
    COURSE.ground_face_last.set(CAR.probe_point_face[4].get());
    unsafe { crate::physics::find_ground::find_ground() }?;
    let face = COURSE.ground_slot.get();
    if face == u32::MAX {
        return Ok(());
    }
    if COURSE.course_faces.get() == 0 {
        return Err(BadInput("no course faces").into());
    }
    if COURSE.ground_half.get() == 1 {
        let (s, t) = lighting::half_shares(1, g(0x639800), g(0x639804));
        p(0x639800, s);
        p(0x639804, t);
    }
    p(0x62DB08, g(0x639800));
    p(0x62DB0C, g(0x639804));
    let f = (face << 6).wrapping_add(COURSE.course_faces.get());
    unsafe {
        for k in 0..4u32 {
            p(0x5CCC90 + 4 * k as usize, byte(f + 0x14 + k));
        }
        let material = (game_addr(rva(0x979F30)) as u32).wrapping_add(byte(f + 0x21) << 6);
        if byte(material + 0x2C) == 1 {
            for k in 0..4u32 {
                let at = 0x5CCC90 + 4 * k as usize;
                let (v, _) = lighting::shaded(g(at), byte(f + 0x22 + k) as u8);
                p(at, v);
            }
        }
        let light = corners();
        CAR.light_level.set(light);
        if CAR.light_hold_steps.get() != 0 {
            let held = lighting::hold_change(
                light,
                COURSE.light_kept.get(),
                RACE_VARS.scene_switch.get() != 0,
            );
            CAR.light_level.set(held.light);
        }
        let light = CAR.light_level.get();
        if light >= lighting::TABLE {
            return Err(BadInput("a light index past 0xFF").into());
        }
        let colour = g(0x635118 + 4 * light as usize);
        p(0x62DAFC, colour);
        CAR.light_colour.set(colour);
        if byte(f + 2) & 4 == 0 {
            return Ok(());
        }
        for (first, table, to) in [
            (0x28u32, 0x635518usize, 0x62DB00usize),
            (0x2C, 0x635918, 0x62DB04),
        ] {
            for k in 0..4u32 {
                p(0x5CCC90 + 4 * k as usize, byte(f + first + k));
            }
            let i = corners();
            if i >= lighting::TABLE {
                return Err(BadInput("a colour index past 0xFF").into());
            }
            let colour = g(table + 4 * i as usize);
            p(to, colour);
            p(
                0x71A218,
                lighting::add_colour(colour, CAR.light_colour.get(), MACHINE.has_mmx.get() != 0),
            );
        }
    }
    Ok(())
}
