//! `scene::vertex_light`: the colour the light gives a vertex of an object. Takes the vertex's
//! fourth dword and the number of its normal.
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
//! "Plus" is byte by byte without overflow. The lit colour: the normal's place in the view (the
//! point record 0x1000 + (number & 0xFFF), its view place) goes to `light_dot_point`; if the
//! object's `light_strength` is not 0, the light's direction goes to `light_dot_light`, the
//! strength as a float to `light_dot_factor`, and `scene::light_dot` gives a number; that plus
//! `light_level`, held to `light_lowest` .. `light_highest`, picks a colour from the light table
//! (0x100 entries), kept in `light_colour`. The blend: the same number without the table, held to
//! -255 .. 255; each of the colour's three bytes moves that many 256ths of the way to the stage's
//! `lit_colour` (red, green, blue by the byte's place) for a number of 0 or more, and by its size
//! towards `dark_colour` for one below. Refuses when there are no records and when the table's
//! place is outside it.

use crate::errors::BadInput;
use crate::math::x87;
use crate::scene::light_dot::light_dot;

// ---- game data ----
const TABLE_LEN: u32 = 0x100;

/// Byte by byte without overflow.
pub fn plus(a: u32, b: u32) -> u32 {
    rc2k_core::scene::lamps::brighter(a, b)
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
    part: &crate::scene::scratch_store::PartScratch,
    number: u32,
    light: &crate::scene::object_light::ObjectLight,
) -> Result<(), BadInput> {
    if !crate::scene::point_records::held(lists) {
        return Err(BadInput("no records for the points"));
    }
    let place = crate::scene::reading::PointRecords::new(lists, part)?.place(number)?;
    light.borrow_mut().light_colour = 0u32;
    light.borrow_mut().light_dot_point = place;
    Ok(())
}

/// The light on the normal as a number (the start of both the lit colour and the blend).
fn light_number(
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> i32 {
    let strength = object.borrow().light_strength as i32;
    if strength == 0 {
        return 0;
    }
    let direction = light.borrow().light_direction;
    {
        let mut light = light.borrow_mut();
        light.light_dot_light = direction;
        light.light_dot_factor = x87::from_int(strength).to_bits();
    }
    light_dot(light)
}

/// Where the lit colour is looked for in the light table, checked.
fn table_place(
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<usize, BadInput> {
    let place = held(
        light_number(light, object).wrapping_add(light.borrow().light_level as i32),
        light.borrow().light_lowest as i32,
        light.borrow().light_highest as i32,
    );
    if !(0..TABLE_LEN as i32).contains(&place) {
        return Err(BadInput("a place outside the light table"));
    }
    Ok(place as usize)
}

/// The lit colour, also left in `light_colour`.
fn lit(
    reads: crate::scene::workers::DrawReads<'_>,
    part: &crate::scene::scratch_store::PartScratch,
    number: u32,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<u32, BadInput> {
    // The checks first, on a copy of what they would write.
    let place = set_normal(reads.course_lists, part, number, light)
        .and_then(|()| table_place(light, object));
    let place = place?;
    let colour = plus(
        reads.light_tables.light_table[place].get(),
        light.borrow().light_colour,
    );
    light.borrow_mut().light_colour = colour;
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
    let towards = crate::memory::session::read_bits::<_, 3>(towards).map(|bits| bits as i32);
    let mut colour: u32 = light.borrow().light_colour;
    // The third byte first, towards the first of the three.
    for (n, to) in towards.into_iter().enumerate() {
        let at = (2 - n) * 8;
        let byte = (colour >> at & 0xFF) as i32;
        let moved = (amount.wrapping_mul(to.wrapping_sub(byte)) >> 8).wrapping_add(byte);
        colour = colour & !(0xFF << at) | (moved.clamp(0, 0xFF) as u32) << at;
    }
    light.borrow_mut().light_colour = colour;
}

#[allow(
    clippy::too_many_arguments,
    reason = "the drawing's stores the part reads, as they are lent"
)]
pub fn vertex_light(
    part: &crate::scene::scratch_store::PartScratch,
    number: u32,
    reads: crate::scene::workers::DrawReads<'_>,
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<u32, BadInput> {
    let kind: u32 = light.borrow().light_kind;
    if kind == 4 {
        // The lit colour added to nothing: it only makes the bytes the right ones.
        return Ok(plus(0, lit(reads, part, number, light, object)?));
    }
    let lit_at_all = reads.switches.scene_set_up.get() != 0 || reads.stage_prefs.lit_b.get() != 0;
    if !lit_at_all || kind == 0 {
        return Ok(0x00FF_FFFF);
    }
    let added: u32 = light.borrow().light_added;
    match kind {
        2 | 5 => Ok(plus(
            plus(0, lit(reads, part, number, light, object)?),
            added,
        )),
        1 => {
            let level: u32 = light.borrow().light_level;
            if level >= TABLE_LEN {
                return Err(BadInput("a place outside the light table"));
            }
            Ok(plus(
                reads.light_tables.light_table[level as usize].get(),
                added,
            ))
        }
        3 => {
            set_normal(reads.course_lists, part, number, light)?;
            // `set_normal` has just set the colour to 0.
            let colour = plus(plus(light.borrow().car_light, 0), added);
            light.borrow_mut().light_colour = colour;
            blend(reads.course(), light, object);
            Ok(light.borrow().light_colour)
        }
        _ => Ok(number),
    }
}
