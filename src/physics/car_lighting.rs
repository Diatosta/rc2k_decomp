//! `physics::car_lighting` (0x44ECE4): the light on the work car from the face under it.
//! The car's x / z into the probe point, the old light (+0x244) into `light_kept`,
//! `car_light_colour` / `62DB00` 0, the last face (+0xE00) into `ground_face_last`;
//! `physics::find_ground`; no face: done. With `ground_half` 1 the shares
//! (`share_s`, `804`) taken from 0x10000; kept in `ground_texture_s` / `0C`. The face's
//! four corner lights (+0x14 ..; with its texture's +0x2C 1 each scaled by
//! ((byte +0x22 + k) >> 1 ^ 0x7F) + 0x80, `mul`, >> 8) blended by the shares (`imul`,
//! `shrd` 16) into +0x244; with +0xFB4 set the change held to 10 a step (or none with
//! `scene_switch`); `g_lightTable[it]` into `car_light_colour` and +0x240. With bit 2 of the
//! face's +2 also two colour blends (+0x28 .., +0x2C ..) through `lamp_table_a`[]` /
//! `lamp_table_b`[]` into `lamp_a_colour` / `62DB04`, each added to +0x240 byte by byte
//! with saturation (four bytes with `g_hasMmx`, else the low three).
//!
//! Caller: `physics::car_physics_step` (0x43B990), every physics step. Refuses a light index past
//! 0xFF (the tables are 256) and no faces; what was written is put back.

use crate::errors::{BadInput, Refused};
use crate::physics::ground_store::GroundWork;
use crate::physics::step_families::LightingInputs;
use crate::race::course_store::CourseWork;
use rc2k_core::physics::lighting;

/// The two-way blend of the four corners now in `face_light_bytes` ..: into `5CCCA4`,
/// `5CCCA0`, returned.
fn corners(ground: &GroundWork, course_work: &CourseWork) -> u32 {
    let c = [0usize, 1, 2, 3].map(|k| course_work.face_light_bytes[k].get());
    let ([a, b], v, _) = lighting::corners(c, ground.share_s.get(), ground.share_t.get());
    course_work.face_light_a.set(a);
    course_work.face_light_b.set(b);
    v
}

// Converted: `rc2k_core::physics::lighting`.
pub fn car_lighting(
    c: &LightingInputs<'_>,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    c.ground.ground_x.set(c.car.position[0].get());
    c.ground.ground_z.set(c.car.position[2].get());
    c.course.course.light_kept.set(c.car.light_level.get());
    scene.lights.work.car_light_colour.set(0);
    scene.lights.work.lamp_a_colour.set(0);
    c.course
        .course
        .ground_face_last
        .set(c.car.probe_point_face[4].get());
    { crate::physics::find_ground::find_ground(c.probe, c.ground, c.course, scene) }?;
    let face = c.course.course.ground_slot.get();
    if face == u32::MAX {
        return Ok(());
    }
    if c.course.course.course_faces.get() == 0 {
        return Err(BadInput("no course faces").into());
    }
    if c.course.course.ground_half.get() == 1 {
        let (s, t) = lighting::half_shares(1, c.ground.share_s.get(), c.ground.share_t.get());
        c.ground.share_s.set(s);
        c.ground.share_t.set(t);
    }
    scene
        .lights
        .work
        .ground_texture_s
        .set(c.ground.share_s.get());
    scene
        .lights
        .work
        .ground_texture_t
        .set(c.ground.share_t.get());
    let face_byte = |offset| -> Result<u32, BadInput> {
        Ok(u32::from(crate::race::course_places::face_field::<u8>(
            c.course, face, offset,
        )?))
    };
    for k in 0..4u32 {
        c.course.course.face_light_bytes[k as usize].set(face_byte(0x14 + k)?);
    }
    if crate::scene::polygon::texture_info_byte(c.polygon_tables, face_byte(0x21)?, 0x2C)? == 1 {
        for k in 0..4u32 {
            let at = &c.course.course.face_light_bytes[k as usize];
            let (v, _) = lighting::shaded(at.get(), face_byte(0x22 + k)? as u8);
            at.set(v);
        }
    }
    let light = corners(c.ground, &c.course.course);
    c.car.light_level.set(light);
    if c.car.light_hold_steps.get() != 0 {
        let held = lighting::hold_change(
            light,
            c.course.course.light_kept.get(),
            c.vars.scene_switch.get() != 0,
        );
        c.car.light_level.set(held.light);
    }
    let light = c.car.light_level.get();
    if light >= lighting::TABLE {
        return Err(BadInput("a light index past 0xFF").into());
    }
    let colour = scene.light_tables.get().light_table[light as usize].get();
    scene.lights.work.car_light_colour.set(colour);
    c.car.light_colour.set(colour);
    if face_byte(2)? & 4 == 0 {
        return Ok(());
    }
    for (first, table, to) in [
        (
            0x28u32,
            &scene.light_tables.get().lamp_table_a,
            &scene.lights.work.lamp_a_colour,
        ),
        (
            0x2C,
            &scene.light_tables.get().lamp_table_b,
            &scene.lights.work.lamp_b_colour,
        ),
    ] {
        for k in 0..4u32 {
            c.course.course.face_light_bytes[k as usize].set(face_byte(first + k)?);
        }
        let i = corners(c.ground, &c.course.course);
        if i >= lighting::TABLE {
            return Err(BadInput("a colour index past 0xFF").into());
        }
        let colour = table[i as usize].get();
        to.set(colour);
        c.car.light_colour.set(lighting::add_colour(
            colour,
            c.car.light_colour.get(),
            system.session.machine.has_mmx.get() != 0,
        ));
    }
    Ok(())
}
