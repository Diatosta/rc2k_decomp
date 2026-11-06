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
use crate::physics::car_store::CAR;
use crate::race::owner::Race;
use crate::race::vars_store::RACE_VARS;
use crate::session::MACHINE;
use rc2k_core::physics::lighting;

/// The two-way blend of the four corners now in `dword_5CCC90` ..: into `5CCCA4`,
/// `5CCCA0`, returned.
fn corners(race: &Race) -> u32 {
    let c = [0usize, 1, 2, 3].map(|k| race.course.course.face_light_bytes[k].get());
    let ([a, b], v, _) = lighting::corners(
        c,
        race.physics.ground.share_s.get(),
        race.physics.ground.share_t.get(),
    );
    race.course.course.face_light_a.set(a);
    race.course.course.face_light_b.set(b);
    v
}

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::lighting`.
pub fn car_lighting(race: &Race) -> Result<(), Refused> {
    let race_owner = race;
    race_owner
        .physics
        .ground
        .ground_x
        .set(CAR.position[0].get());
    race_owner
        .physics
        .ground
        .ground_z
        .set(CAR.position[2].get());
    race_owner
        .course
        .course
        .light_kept
        .set(CAR.light_level.get());
    crate::scene::owner::scene()
        .lights
        .work
        .car_light_colour
        .set(0);
    crate::scene::owner::scene()
        .lights
        .work
        .lamp_a_colour
        .set(0);
    race_owner
        .course
        .course
        .ground_face_last
        .set(CAR.probe_point_face[4].get());
    { crate::physics::find_ground::find_ground(race) }?;
    let face = race_owner.course.course.ground_slot.get();
    if face == u32::MAX {
        return Ok(());
    }
    if race_owner.course.course.course_faces.get() == 0 {
        return Err(BadInput("no course faces").into());
    }
    if race_owner.course.course.ground_half.get() == 1 {
        let (s, t) = lighting::half_shares(
            1,
            race_owner.physics.ground.share_s.get(),
            race_owner.physics.ground.share_t.get(),
        );
        race_owner.physics.ground.share_s.set(s);
        race_owner.physics.ground.share_t.set(t);
    }
    crate::scene::owner::scene()
        .lights
        .work
        .ground_texture_s
        .set(race_owner.physics.ground.share_s.get());
    crate::scene::owner::scene()
        .lights
        .work
        .ground_texture_t
        .set(race_owner.physics.ground.share_t.get());
    let face_byte = |offset| -> Result<u32, BadInput> {
        Ok(u32::from(crate::race::course_places::face_field::<u8>(
            face,
            offset,
            "a course face's byte",
        )?))
    };
    for k in 0..4u32 {
        race_owner.course.course.face_light_bytes[k as usize].set(face_byte(0x14 + k)?);
    }
    if crate::scene::polygon::texture_info_byte(face_byte(0x21)?, 0x2C)? == 1 {
        for k in 0..4u32 {
            let at = &race_owner.course.course.face_light_bytes[k as usize];
            let (v, _) = lighting::shaded(at.get(), face_byte(0x22 + k)? as u8);
            at.set(v);
        }
    }
    let light = corners(race);
    CAR.light_level.set(light);
    if CAR.light_hold_steps.get() != 0 {
        let held = lighting::hold_change(
            light,
            race_owner.course.course.light_kept.get(),
            RACE_VARS.scene_switch.get() != 0,
        );
        CAR.light_level.set(held.light);
    }
    let light = CAR.light_level.get();
    if light >= lighting::TABLE {
        return Err(BadInput("a light index past 0xFF").into());
    }
    let colour = crate::scene::owner::scene().light_tables.light_table[light as usize].get();
    crate::scene::owner::scene()
        .lights
        .work
        .car_light_colour
        .set(colour);
    CAR.light_colour.set(colour);
    if face_byte(2)? & 4 == 0 {
        return Ok(());
    }
    for (first, table, to) in [
        (
            0x28u32,
            &crate::scene::owner::scene().light_tables.lamp_table_a,
            &crate::scene::owner::scene().lights.work.lamp_a_colour,
        ),
        (
            0x2C,
            &crate::scene::owner::scene().light_tables.lamp_table_b,
            &crate::scene::owner::scene().lights.work.lamp_b_colour,
        ),
    ] {
        for k in 0..4u32 {
            race_owner.course.course.face_light_bytes[k as usize].set(face_byte(first + k)?);
        }
        let i = corners(race);
        if i >= lighting::TABLE {
            return Err(BadInput("a colour index past 0xFF").into());
        }
        let colour = table[i as usize].get();
        to.set(colour);
        CAR.light_colour.set(lighting::add_colour(
            colour,
            CAR.light_colour.get(),
            MACHINE.has_mmx.get() != 0,
        ));
    }
    Ok(())
}
