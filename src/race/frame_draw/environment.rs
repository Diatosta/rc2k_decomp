//! The EAX sound environment: the weight of the course faces round the first car.

use super::*;
use crate::audio::store::ENVIRONMENT;
use crate::errors::BadInput;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner::race;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use crate::scene::draw_store::DRAW;

/// The game's table root of a float (`sqrt` by `g_rootTable` x `g_exponentRootTable`),
/// times 1/65536 (`race().course.numbers.one_over_65536`), as stored.
pub(super) fn table_root(course: &Course, v: u32) -> u32 {
    // g_rootTable: 256 floats by mantissa bits; g_exponentRootTable: by exponent
    let root = course.lists.root_table[((v >> 15) & 0xFF) as usize].bits();
    let scale = course.lists.exponent_root_table[((v & 0x7F80_0000) >> 23) as usize].bits();
    crate::math::x87_load::product3(root, scale, course.numbers.one_over_65536.bits())
}

/// A point's x and z less car 0's (`g_carBlockStore` +0x134, +0x13C), and its y.
pub(super) fn point_from_car(physics: &Physics, point: u32) -> Result<(u32, u32, u32), BadInput> {
    let [px, py, pz] = crate::race::scene_points::point_number(point)?;
    let x = px.wrapping_sub(CAR_BLOCKS.cars[0].position[0].bits());
    let z = pz.wrapping_sub(CAR_BLOCKS.cars[0].position[2].bits());
    physics.contact.eax_point_x.set(x);
    physics.contact.eax_point_z.set(z);
    Ok((x, py, z))
}

/// The weight's end: `DRAW.scratch` = `v`, held to 16.0 (`race().course.numbers.sixteen`; kept when below or
/// not a number), added to `g_eaxEnvironment`; `race().frame.sound_spots.eax_face_count` + 1.
pub(super) fn add_eax_weight(race: &Race, v: u32) {
    let sixteen: u32 = race.course.numbers.sixteen.bits();
    let w = if crate::math::x87_load::below_or_unordered(v, sixteen) {
        v
    } else {
        sixteen
    };
    DRAW.scratch.set_bits(w);
    ENVIRONMENT.set(crate::math::x87::add(w, ENVIRONMENT.get()));
    race.frame
        .sound_spots
        .eax_face_count
        .set_bits(race.frame.sound_spots.eax_face_count.bits().wrapping_add(1));
}

/// `race::eax_face_weight` (0x45C881): the weight of `face` in the EAX sound environment
/// (`g_eaxEnvironment`), by its distance from car 0 (x and z): kind 0xFF with an object
/// (+1) below 0x38: its +0x18 byte over the distance to its first point; kinds below 0x10
/// with |+0x30| below 0.25: 3.0 over the distance to the middle of its second and third
/// points (`CONTACT_WORK.eax_mid_x`, `CONTACT_WORK.eax_mid_z`); each held to 16 (see `add_eax_weight`). Distances
/// by the game's root tables into `CONTACT_WORK.eax_distance`.
pub fn eax_face_weight(race: &Race, face: u32) -> Result<(), BadInput> {
    use crate::math::x87::{div, from_int};
    use crate::math::x87_load::{
        abs_below_or_unordered, float_square_sum, int_over_float, int_plus_times, int_square_sum,
    };
    {
        // The face read once for its kind, objects, switches and first point.
        let read = crate::race::course_places::face_number(face, "a course face")?;
        let kind = read.kind();
        if kind == 0xFF {
            if face_field::<u8>(face, 1)? >= 0x38 {
                return Ok(());
            }
            let (x, _, z) = point_from_car(&race.physics, face_field(face, 4)?)?;
            let distance = table_root(&race.course, int_square_sum(x as i32, z as i32));
            race.physics.contact.eax_distance.set_bits(distance);
            let strength = face_field::<u8>(face, 0x18)? as u32;
            DRAW.scratch.set_bits(strength);
            add_eax_weight(race, int_over_float(strength as i32, distance));
            return Ok(());
        }
        if kind >= 0x10 {
            return Ok(());
        }
        if !abs_below_or_unordered(face_field(face, 0x30)?, race.course.numbers.quarter.bits()) {
            return Ok(());
        }
        let (x1, _, z1) = point_from_car(&race.physics, face_field(face, 8)?)?;
        let (mx, mz) = (from_int(x1 as i32), from_int(z1 as i32));
        race.physics.contact.eax_mid_x.set_bits(mx);
        race.physics.contact.eax_mid_z.set_bits(mz);
        let (x2, _, z2) = point_from_car(&race.physics, face_field(face, 0xC)?)?;
        let half: u32 = race.course.numbers.half.bits();
        let mx = int_plus_times(x2 as i32, mx, half);
        race.physics.contact.eax_mid_x.set_bits(mx);
        let mz = int_plus_times(z2 as i32, mz, half);
        race.physics.contact.eax_mid_z.set_bits(mz);
        let distance = table_root(&race.course, float_square_sum(mx, mz));
        race.physics.contact.eax_distance.set_bits(distance);
        add_eax_weight(race, div(race.course.numbers.three.bits(), distance));
    }
    Ok(())
}

/// `race::eax_environment` (0x45C7EE): `g_eaxEnvironment` 0.0 and
/// `race().frame.sound_spots.eax_face_count` 0, then the 3 x 3 grid cells around the middle (from
/// `hold_to_limit(race().course.stage_prefs.grid_reach)` - 1 on each axis; `sub_45C84C` each): every face of
/// each cell's course record (numbers below 0x0FFFFFFF) weighed by `eax_face_weight`.
pub fn eax_environment() -> Result<(), BadInput> {
    let race = race();
    {
        ENVIRONMENT.set(0);
        race.frame.sound_spots.eax_face_count.set_bits(0u32);
        // Converted: `rc2k_core::race::view::{cells_from, record_faces}`.
        use rc2k_core::race::view::{NO_RECORD, cells_from, record_faces};
        for cell in cells_from(
            hold_to_limit(race.course.stage_prefs.grid_reach.bits()).wrapping_sub(1),
            3,
        ) {
            // `sub_45C84C`.
            let record = course_grid_cell(cell)?.get();
            if record >= NO_RECORD {
                continue;
            }
            let mut face = record_field(record, 8)?;
            if record_faces(face, record_field(record, 0xC)?).is_none() {
                continue;
            }
            loop {
                eax_face_weight(race, face)?;
                face = face.wrapping_add(1);
                if face > record_field(record, 0xC)? {
                    break;
                }
            }
        }
    }
    Ok(())
}
