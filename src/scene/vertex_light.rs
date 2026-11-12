//! `scene::vertex_light` (0x450F17): the colour the light gives a vertex of an object.
//!
//! Takes: the vertex's fourth dword, the number of its normal
//!
//! By the kind of lighting (`light_kind`):
//! - with neither `scene_set_up` nor the stage's `lit_b` set, or kind 0 (and not kind 4):
//!   plain white, 0xFFFFFF;
//! - 4: the lit colour;
//! - 2 and 5: the lit colour plus `light_added`;
//! - 1: the light table's entry `light_level` plus `light_added`;
//! - 3: `car_light` plus `light_added`, then blended (below);
//! - anything else: the number as it came.
//!
//! "Plus" is byte by byte without overflow: with MMX all four bytes, without it the low
//! three. The lit colour: the normal's place in the view (the point record 0x1000 +
//! (number & 0xFFF), its +0x10 .. +0x18) goes to `light_dot_point`; if the object's
//! `light_strength` is not 0, the light's direction goes to `light_dot_light`, the strength
//! as a float to `light_dot_factor`, and `scene::light_dot` gives a number; that plus
//! `light_level`, held to `light_lowest` .. `light_highest`, picks a colour from the light
//! table (0x100 entries), kept in `light_colour`. The blend: the same number without the
//! table, held to -255 .. 255; each of the colour's three bytes moves that many 256ths of
//! the way to the stage's `lit_colour` (red, green, blue by the byte's place) for a number
//! of 0 or more, and by its size towards `dark_colour` for one below. One caller,
//! `scene::draw_object_face`. Refuses when there are no records and when the table's place
//! is outside it.

use crate::errors::BadInput;
use crate::math::x87;
use crate::scene::light_dot::light_dot;

// ---- game data ----
const TABLE_LEN: u32 = 0x100;

// ---- Rust version ----

/// Byte by byte without overflow: all four with MMX, the low three without.
pub fn plus(a: u32, b: u32, mmx: bool) -> u32 {
    rc2k_core::scene::lamps::brighter(a, b, mmx)
}

/// Holds `value` to `low ..= high`, compared by the sign of the difference as the game
/// does.
fn held(value: i32, low: i32, high: i32) -> i32 {
    let value = if value.wrapping_sub(low) < 0 {
        low
    } else {
        value
    };
    if value.wrapping_sub(high) < 0 {
        value
    } else {
        high
    }
}

/// The normal's place in the view to the light's work variables, the colour set to 0.
fn set_normal(
    lists: &crate::race::course_lists_store::CourseLists,
    part: &crate::workers::PartScratch,
    number: u32,
    light: &crate::scene::object_light::ObjectLight,
) -> Result<(), BadInput> {
    if !crate::scene::point_records::held(lists) {
        return Err(BadInput("no records for the points"));
    }
    let place = crate::scene::reading::PointRecords::new(lists, part)?.place(number)?;
    light.light_colour.set(0u32);
    crate::session::write_bits(&light.light_dot_point, place);
    Ok(())
}

/// The light on the normal as a number (the start of both the lit colour and the blend).
fn light_number(
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> i32 {
    let strength = object.light_strength.get() as i32;
    if strength == 0 {
        return 0;
    }
    crate::session::write_bits(
        &light.light_dot_light,
        crate::session::read_bits::<_, 3>(&light.light_direction),
    );
    light.light_dot_factor.set(x87::from_int(strength));
    light_dot(light)
}

/// Where the lit colour is looked for in the light table, checked.
fn table_place(
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<usize, BadInput> {
    let place = held(
        light_number(light, object).wrapping_add(light.light_level.get() as i32),
        light.light_lowest.get() as i32,
        light.light_highest.get() as i32,
    );
    if !(0..TABLE_LEN as i32).contains(&place) {
        return Err(BadInput("a place outside the light table"));
    }
    Ok(place as usize)
}

/// The lit colour, also left in `light_colour`.
fn lit(
    reads: crate::workers::DrawReads<'_>,
    part: &crate::workers::PartScratch,
    number: u32,
    mmx: bool,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<u32, BadInput> {
    // The checks first, on a copy of what they would write.
    let place = set_normal(reads.course_lists, part, number, light)
        .and_then(|()| table_place(light, object));
    let place = place?;
    let colour = plus(
        reads.light_tables.light_table[place].get(),
        light.light_colour.get(),
        mmx,
    );
    light.light_colour.set(colour);
    Ok(colour)
}

/// The colour in `light_colour` blended by the light.
fn blend(
    course: crate::race::owner_course::CourseReads<'_>,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) {
    let amount = held(light_number(light, object), -255, 255);
    let (amount, towards) = if amount < 0 {
        (amount.wrapping_neg(), &course.stage_prefs.dark_colour)
    } else {
        (amount, &course.stage_prefs.lit_colour)
    };
    let towards = crate::session::read_bits::<_, 3>(towards).map(|bits| bits as i32);
    let mut colour: u32 = light.light_colour.get();
    // The third byte first, towards the first of the three.
    for (n, to) in towards.into_iter().enumerate() {
        let at = (2 - n) * 8;
        let byte = (colour >> at & 0xFF) as i32;
        let moved = (amount.wrapping_mul(to.wrapping_sub(byte)) >> 8).wrapping_add(byte);
        colour = colour & !(0xFF << at) | (moved.clamp(0, 0xFF) as u32) << at;
    }
    light.light_colour.set(colour);
}

#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's stores the part reads, as they are lent"
)]
pub fn vertex_light(
    part: &crate::workers::PartScratch,
    number: u32,
    reads: crate::workers::DrawReads<'_>,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
    mmx: bool,
) -> Result<u32, BadInput> {
    let kind: u32 = light.light_kind.get();
    if kind == 4 {
        // The lit colour added to nothing: it only makes the bytes the right ones.
        return Ok(plus(0, lit(reads, part, number, mmx, light, object)?, mmx));
    }
    let lit_at_all = reads.switches.scene_set_up.get() != 0 || reads.stage_prefs.lit_b.get() != 0;
    if !lit_at_all || kind == 0 {
        return Ok(0x00FF_FFFF);
    }
    let added: u32 = light.light_added.get();
    match kind {
        2 | 5 => Ok(plus(
            plus(0, lit(reads, part, number, mmx, light, object)?, mmx),
            added,
            mmx,
        )),
        1 => {
            let level: u32 = light.light_level.get();
            if level >= TABLE_LEN {
                return Err(BadInput("a place outside the light table"));
            }
            Ok(plus(
                reads.light_tables.light_table[level as usize].get(),
                added,
                mmx,
            ))
        }
        3 => {
            set_normal(reads.course_lists, part, number, light)?;
            // `set_normal` has just set the colour to 0.
            let colour = plus(plus(light.car_light.get(), 0, mmx), added, mmx);
            light.light_colour.set(colour);
            blend(reads.course(), light, object);
            Ok(light.light_colour.get())
        }
        _ => Ok(number),
    }
}
