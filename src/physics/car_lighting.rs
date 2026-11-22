//! The light on the work car from the face under it. The car's x and z become the probe point, the
//! old light is kept, `car_light_colour` is 0 and the last face goes into
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
use crate::memory::plain_floats::FloatRow;
use crate::physics::car_store::Car;
use crate::physics::ground_query::GroundSearch;
use crate::physics::step_families::LightingInputs;
use rc2k_core::physics::lighting;

/// The two-way blend of the four corner lights by the ground search's shares.
fn corners(ground: &GroundSearch, corner_lights: [u32; 4]) -> u32 {
    let (_, v, _) = lighting::corners(corner_lights, ground.share_s, ground.share_t);
    v
}

// Core: `rc2k_core::physics::lighting`.
pub fn car_lighting(
    c: &LightingInputs<'_>,
    scene: &crate::scene::owner::Scene,
    car: &mut Car,
) -> Result<(), Refused> {
    {
        let mut search = c.search.borrow_mut();
        search.x = car.position.bits_at(0);
        search.z = car.position.bits_at(2);
        search.last = car.probe_point_face[4];
    }
    let light_kept = car.light_level;
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
    let mut corner_lights = [0u32; 4];
    for k in 0..4u32 {
        corner_lights[k as usize] = face_byte(0x14 + k)?;
    }
    if crate::scene::polygon::texture_info_byte(c.polygon_tables, face_byte(0x21)?, 0x2C)? == 1 {
        for k in 0..4u32 {
            let (v, _) = lighting::shaded(corner_lights[k as usize], face_byte(0x22 + k)? as u8);
            corner_lights[k as usize] = v;
        }
    }
    let light = corners(&c.search.borrow(), corner_lights);
    car.light_level = light;
    if car.light_hold_steps != 0 {
        let held = lighting::hold_change(light, light_kept, c.vars.scene_switch.get() != 0);
        car.light_level = held.light;
    }
    let light = car.light_level;
    if light >= lighting::TABLE {
        return Err(BadInput("a light index past 0xFF").into());
    }
    let colour = scene.light_tables.borrow().light_table[light as usize];
    car.light_colour = colour;
    if face_byte(2)? & 4 == 0 {
        return Ok(());
    }
    let lamp_tables = {
        let tables = scene.light_tables.borrow();
        [(0x28u32, tables.lamp_table_a), (0x2C, tables.lamp_table_b)]
    };
    for (first, table) in lamp_tables {
        let mut corner_lights = [0u32; 4];
        for k in 0..4u32 {
            corner_lights[k as usize] = face_byte(first + k)?;
        }
        let i = corners(&c.search.borrow(), corner_lights);
        if i >= lighting::TABLE {
            return Err(BadInput("a colour index past 0xFF").into());
        }
        let colour = table[i as usize];
        car.light_colour = lighting::add_colour(colour, car.light_colour);
    }
    Ok(())
}
