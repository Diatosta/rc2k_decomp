//! The light on the work car from the face under it. The car's x and z become the probe point, the
//! old light is kept in `light_kept`, `car_light_colour` is 0 and the last face goes into
//! `ground_face_last`; then `physics::find_ground`; no face: done. With `ground_half` 1 the shares
//! are taken from 0x10000 and kept in `ground_texture_s`. The face's four corner lights (each
//! scaled by a texture byte when its texture asks for it) are blended by the shares into the car's
//! light; with the car's hold set the change is held to 10 a step (or none with `scene_switch`);
//! the light table's entry goes into `car_light_colour`. With bit 2 of the face's flags also two
//! colour blends (`lamp_table_a`, `lamp_table_b`) go into `lamp_a_colour`, each added to the car's
//! lamp colour byte by byte with saturation.
//!
//! Refuses a light index past 0xFF (the tables hold 256) and no faces; what was written is put
//! back.

use crate::errors::{BadInput, Refused};
use crate::physics::ground_query::GroundSearch;
use crate::physics::step_families::LightingInputs;
use crate::race::course_store::CourseWork;
use rc2k_core::physics::lighting;

/// The two-way blend of the four corners now in `face_light_bytes` ..: into `5CCCA4`,
/// `5CCCA0`, returned.
fn corners(ground: &GroundSearch, course_work: &CourseWork) -> u32 {
    let c = [0usize, 1, 2, 3].map(|k| course_work.face_light_bytes[k].get());
    let ([a, b], v, _) = lighting::corners(c, ground.share_s, ground.share_t);
    course_work.face_light_a.set(a);
    course_work.face_light_b.set(b);
    v
}

// Core: `rc2k_core::physics::lighting`.
pub fn car_lighting(
    c: &LightingInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    {
        let mut search = c.search.borrow_mut();
        search.x = c.car.position[0].get();
        search.z = c.car.position[2].get();
        search.last = c.car.probe_point_face[4].get();
    }
    c.course
        .course
        .get()
        .light_kept
        .set(c.car.light_level.get());
    { crate::physics::find_ground::find_ground(c.probe, c.search, c.course) }?;
    let face = c.search.borrow().slot;
    if face == u32::MAX {
        return Ok(());
    }
    if !crate::race::course_places::held(&c.course.stage.course) {
        return Err(BadInput("no course faces").into());
    }
    if c.search.borrow().half == 1 {
        let mut search = c.search.borrow_mut();
        let (s, t) = lighting::half_shares(1, search.share_s, search.share_t);
        search.share_s = s;
        search.share_t = t;
    }
    let face_byte = |offset| -> Result<u32, BadInput> {
        Ok(u32::from(crate::race::course_places::face_field::<u8>(
            c.course.reads(),
            face,
            offset,
        )?))
    };
    for k in 0..4u32 {
        c.course.course.get().face_light_bytes[k as usize].set(face_byte(0x14 + k)?);
    }
    if crate::scene::polygon::texture_info_byte(c.polygon_tables, face_byte(0x21)?, 0x2C)? == 1 {
        for k in 0..4u32 {
            let at = &c.course.course.get().face_light_bytes[k as usize];
            let (v, _) = lighting::shaded(at.get(), face_byte(0x22 + k)? as u8);
            at.set(v);
        }
    }
    let light = corners(&c.search.borrow(), c.course.course.get());
    c.car.light_level.set(light);
    if c.car.light_hold_steps.get() != 0 {
        let held = lighting::hold_change(
            light,
            c.course.course.get().light_kept.get(),
            c.vars.scene_switch.get() != 0,
        );
        c.car.light_level.set(held.light);
    }
    let light = c.car.light_level.get();
    if light >= lighting::TABLE {
        return Err(BadInput("a light index past 0xFF").into());
    }
    let colour = scene.light_tables.get().light_table[light as usize].get();
    c.car.light_colour.set(colour);
    if face_byte(2)? & 4 == 0 {
        return Ok(());
    }
    for (first, table) in [
        (0x28u32, &scene.light_tables.get().lamp_table_a),
        (0x2C, &scene.light_tables.get().lamp_table_b),
    ] {
        for k in 0..4u32 {
            c.course.course.get().face_light_bytes[k as usize].set(face_byte(first + k)?);
        }
        let i = corners(&c.search.borrow(), c.course.course.get());
        if i >= lighting::TABLE {
            return Err(BadInput("a colour index past 0xFF").into());
        }
        let colour = table[i as usize].get();
        c.car
            .light_colour
            .set(lighting::add_colour(colour, c.car.light_colour.get()));
    }
    Ok(())
}
