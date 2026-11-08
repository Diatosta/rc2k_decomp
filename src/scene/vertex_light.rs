//! `scene::vertex_light` (0x450F17): the colour the light gives a vertex of an object.
//!
//! Takes: the vertex's fourth dword, the number of its normal
//!
//! By the kind of lighting `dword_62DAF4`:
//! - with neither `dword_621480` nor `dword_60931C` set, or kind 0 (and not kind 4):
//!   plain white, 0xFFFFFF;
//! - 4: the lit colour;
//! - 2 and 5: the lit colour plus `dword_635068`;
//! - 1: the light table's entry `dword_63508C` plus `dword_635068`;
//! - 3: `dword_62DAD4` plus `dword_635068`, then blended (below);
//! - anything else: eax as it came.
//!
//! "Plus" is byte by byte without overflow: with MMX (`g_hasMmx`) all four bytes (`paddusb`),
//! without it the low three. The lit colour (0x45112D and its twin 0x451170, with
//! 0x4511F1): the normal's place in the view (the record 0x1000 + (number & 0xFFF) of
//! `g_pointRecords`, its +0x10 .. +0x18) goes to `dword_62DAA8` ..; if the light's strength
//! `dword_635090` is not 0, the light's direction `dword_6240A8` .. goes to `dword_62DAB4` ..,
//! the strength as a float to `flt_62DAC0`, and `scene::light_dot` gives a number; that plus
//! `dword_63508C`, held to `dword_635084` .. `dword_635088`, picks a colour from the table
//! `dword_635118` (0x100 entries), kept in `dword_62DAC4`. The blend (0x4511B3 for the
//! normal, 0x451296): the same number without the table, held to -255 .. 255; each of the
//! colour's three bytes moves that many 256ths of the way to `dword_6092C0`, `C4`, `C8` (red,
//! green, blue by the byte's place) for a number of 0 or more, and by its size towards
//! `dword_6092B4`, `B8`, `BC` for one below. One caller, 0x450E37. Refuses when there are
//! no records and when the table's place is outside it.

use crate::errors::BadInput;
use crate::math::x87;
use crate::scene::light_dot::light_dot;
use crate::scene::owner::scene;

// ---- game data ----
const TABLE_LEN: u32 = 0x100;

// ---- Rust version ----

/// Byte by byte without overflow: all four with MMX, the low three without.
pub fn plus(a: u32, b: u32, mmx: bool) -> u32 {
    rc2k_core::scene::lamps::brighter(a, b, mmx)
}

/// `cmp eax, low; jns; eax = low` and `cmp eax, high; js; eax = high`, by the sign
/// of the difference as the original takes it.
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
fn set_normal(number: u32, scene: &crate::scene::owner::Scene) -> Result<(), BadInput> {
    let lights = &scene.lights;
    if !crate::scene::point_records::held() {
        return Err(BadInput("no records for the points"));
    }
    let place = crate::scene::reading::PointRecords::new()?.place(number)?;
    lights.work.light_colour.set_bits(0u32);
    crate::session::write_bits(&lights.work.light_dot_point, place);
    Ok(())
}

/// The start of 0x4511F1 and of 0x451296: the light on the normal as a number.
fn light(scene: &crate::scene::owner::Scene) -> i32 {
    let strength = scene.drawing.object_work.light_strength.bits() as i32;
    if strength == 0 {
        return 0;
    }
    crate::session::write_bits(
        &scene.lights.work.light_dot_light,
        crate::session::read_bits::<_, 3>(&scene.drawing.scene.light_direction),
    );
    scene
        .lights
        .work
        .light_dot_factor
        .set_bits(x87::from_int(strength));
    light_dot(scene)
}

/// Where 0x4511F1 would look in the table, checked.
fn table_place(scene: &crate::scene::owner::Scene) -> Result<usize, BadInput> {
    let drawing = &scene.drawing;
    let place = held(
        light(scene).wrapping_add(drawing.object_work.light_level.bits() as i32),
        drawing.object_work.light_lowest.bits() as i32,
        drawing.object_work.light_highest.bits() as i32,
    );
    if !(0..TABLE_LEN as i32).contains(&place) {
        return Err(BadInput("a place outside the light table"));
    }
    Ok(place as usize)
}

/// 0x45112D / 0x451170 with 0x4511F1: the lit colour, left in `dword_62DAC4`.
fn lit(number: u32, mmx: bool) -> Result<u32, BadInput> {
    let scene = scene();
    // The checks first, on a copy of what they would write.
    let place = set_normal(number, scene).and_then(|()| table_place(scene));
    let place = place?;
    let colour = plus(
        scene.light_tables.light_table[place].get(),
        scene.lights.work.light_colour.bits(),
        mmx,
    );
    scene.lights.work.light_colour.set_bits(colour);
    Ok(colour)
}

/// 0x451296: the colour in `dword_62DAC4` blended by the light.
fn blend(course: &crate::race::owner_course::Course) {
    let scene = crate::scene::owner::scene();
    let lights = &scene.lights;
    let amount = held(light(scene), -255, 255);
    let (amount, towards) = if amount < 0 {
        (amount.wrapping_neg(), &course.stage_prefs.dark_colour)
    } else {
        (amount, &course.stage_prefs.lit_colour)
    };
    let towards = crate::session::read_bits::<_, 3>(towards).map(|bits| bits as i32);
    let mut colour: u32 = lights.work.light_colour.bits();
    // The third byte first, towards the first of the three.
    for (n, to) in towards.into_iter().enumerate() {
        let at = (2 - n) * 8;
        let byte = (colour >> at & 0xFF) as i32;
        let moved = (amount.wrapping_mul(to.wrapping_sub(byte)) >> 8).wrapping_add(byte);
        colour = colour & !(0xFF << at) | (moved.clamp(0, 0xFF) as u32) << at;
    }
    lights.work.light_colour.set_bits(colour);
}

pub fn vertex_light(
    race: &crate::race::owner::Race,
    number: u32,
    scene: &crate::scene::owner::Scene,
    system: &crate::system::owner::System,
) -> Result<u32, BadInput> {
    let course = &race.course;
    let kind: u32 = scene.lights.work.light_kind.bits();
    let mmx = system.session.machine.has_mmx.get() != 0;
    if kind == 4 {
        // 0 plus the lit colour.
        return Ok(plus(0, lit(number, mmx)?, mmx));
    }
    let lit_at_all =
        race.shared.switches.other_way.bits() != 0 || course.stage_prefs.lit_b.bits() != 0;
    if !lit_at_all || kind == 0 {
        return Ok(0x00FF_FFFF);
    }
    let added: u32 = scene.drawing.object_work.light_added.bits();
    match kind {
        2 | 5 => Ok(plus(plus(0, lit(number, mmx)?, mmx), added, mmx)),
        1 => {
            let level: u32 = scene.drawing.object_work.light_level.bits();
            if level >= TABLE_LEN {
                return Err(BadInput("a place outside the light table"));
            }
            Ok(plus(
                scene.light_tables.light_table[level as usize].get(),
                added,
                mmx,
            ))
        }
        3 => {
            set_normal(number, scene)?;
            // The colour was just set to 0.
            let colour = plus(
                plus(scene.lights.work.view_light_base.bits(), 0, mmx),
                added,
                mmx,
            );
            scene.lights.work.light_colour.set_bits(colour);
            blend(course);
            Ok(scene.lights.work.light_colour.bits())
        }
        _ => Ok(number),
    }
}
