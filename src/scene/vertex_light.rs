//! `scene::vertex_light` (0x450F17): the colour the light gives a vertex of an object.
//!
//! In:  eax = the vertex's fourth dword, the number of its normal
//! Out: eax = the colour; ebp = the number given. On the one way that blends
//!      (kind 3) the original also leaves ebx, ecx and edx changed; its one caller
//!      keeps all three, and the Rust version does not change them
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
//! without it the low three. The lit colour (`sub_45112D` and its twin `sub_451170`, with
//! `sub_4511F1`): the normal's place in the view (the record 0x1000 + (number & 0xFFF) of
//! `g_pointRecords`, its +0x10 .. +0x18) goes to `dword_62DAA8` ..; if the light's strength
//! `dword_635090` is not 0, the light's direction `dword_6240A8` .. goes to `dword_62DAB4` ..,
//! the strength as a float to `flt_62DAC0`, and `scene::light_dot` gives a number; that plus
//! `dword_63508C`, held to `dword_635084` .. `dword_635088`, picks a colour from the table
//! `dword_635118` (0x100 entries), kept in `dword_62DAC4`. The blend (`sub_4511B3` for the
//! normal, `sub_451296`): the same number without the table, held to -255 .. 255; each of the
//! colour's three bytes moves that many 256ths of the way to `dword_6092C0`, `C4`, `C8` (red,
//! green, blue by the byte's place) for a number of 0 or more, and by its size towards
//! `dword_6092B4`, `B8`, `BC` for one below. One caller, `sub_450E37`. Refuses when there are
//! no records and when the table's place is outside it.

use crate::errors::BadInput;
use crate::math::x87;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::draw_store::DRAW;
use crate::scene::light_dot::light_dot;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::session::MACHINE;

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
unsafe fn set_normal(number: u32) -> Result<(), BadInput> {
    unsafe {
        let records: u32 = COURSE_LISTS.point_records.bits();
        if records == 0 {
            return Err(BadInput("no records for the points"));
        }
        let record = records.wrapping_add(((number & 0xFFF) + 0x1000) << 5) as usize;
        LIGHT_WORK.light_colour.set_bits(0u32);
        let place: [u32; 3] = ((record + 0x10) as *const [u32; 3]).read_unaligned();
        crate::session::write_bits(&LIGHT_WORK.light_dot_point, place);
        Ok(())
    }
}

/// The start of `sub_4511F1` and of `sub_451296`: the light on the normal as a number.
unsafe fn light() -> i32 {
    unsafe {
        let strength = OBJECT_WORK.light_strength.bits() as i32;
        if strength == 0 {
            return 0;
        }
        crate::session::write_bits(
            &LIGHT_WORK.light_dot_light,
            crate::session::read_bits::<_, 3>(&DRAW.light_direction),
        );
        LIGHT_WORK
            .light_dot_factor
            .set_bits(x87::from_int(strength));
        light_dot()
    }
}

/// Where `sub_4511F1` would look in the table, checked.
unsafe fn table_place() -> Result<usize, BadInput> {
    unsafe {
        let place = held(
            light().wrapping_add(OBJECT_WORK.light_level.bits() as i32),
            OBJECT_WORK.light_lowest.bits() as i32,
            OBJECT_WORK.light_highest.bits() as i32,
        );
        if !(0..TABLE_LEN as i32).contains(&place) {
            return Err(BadInput("a place outside the light table"));
        }
        Ok(place as usize)
    }
}

/// `sub_45112D` / `sub_451170` with `sub_4511F1`: the lit colour, left in `dword_62DAC4`.
unsafe fn lit(number: u32, mmx: bool) -> Result<u32, BadInput> {
    unsafe {
        // The checks first, on a copy of what they would write.
        let place = set_normal(number).and_then(|()| table_place());
        let place = place?;
        let colour = plus(
            crate::scene::light_tables_store::LIGHT_TABLES.light_table[place].get(),
            LIGHT_WORK.light_colour.bits(),
            mmx,
        );
        LIGHT_WORK.light_colour.set_bits(colour);
        Ok(colour)
    }
}

/// `sub_451296`: the colour in `dword_62DAC4` blended by the light.
unsafe fn blend() {
    unsafe {
        let amount = held(light(), -255, 255);
        let (amount, towards) = if amount < 0 {
            (amount.wrapping_neg(), &STAGE_PREFS.dark_colour)
        } else {
            (amount, &STAGE_PREFS.lit_colour)
        };
        let towards = crate::session::read_bits::<_, 3>(towards).map(|bits| bits as i32);
        let mut colour: u32 = LIGHT_WORK.light_colour.bits();
        // The third byte first, towards the first of the three.
        for (n, to) in towards.into_iter().enumerate() {
            let at = (2 - n) * 8;
            let byte = (colour >> at & 0xFF) as i32;
            let moved = (amount.wrapping_mul(to.wrapping_sub(byte)) >> 8).wrapping_add(byte);
            colour = colour & !(0xFF << at) | (moved.clamp(0, 0xFF) as u32) << at;
        }
        LIGHT_WORK.light_colour.set_bits(colour);
    }
}

pub unsafe fn vertex_light(number: u32) -> Result<u32, BadInput> {
    unsafe {
        let kind: u32 = LIGHT_WORK.light_kind.bits();
        let mmx = MACHINE.has_mmx.get() != 0;
        if kind == 4 {
            // 0 plus the lit colour.
            return Ok(plus(0, lit(number, mmx)?, mmx));
        }
        let lit_at_all = FRAME_SWITCHES.other_way.bits() != 0 || STAGE_PREFS.lit_b.bits() != 0;
        if !lit_at_all || kind == 0 {
            return Ok(0x00FF_FFFF);
        }
        let added: u32 = OBJECT_WORK.light_added.bits();
        match kind {
            2 | 5 => Ok(plus(plus(0, lit(number, mmx)?, mmx), added, mmx)),
            1 => {
                let level: u32 = OBJECT_WORK.light_level.bits();
                if level >= TABLE_LEN {
                    return Err(BadInput("a place outside the light table"));
                }
                Ok(plus(
                    crate::scene::light_tables_store::LIGHT_TABLES.light_table[level as usize]
                        .get(),
                    added,
                    mmx,
                ))
            }
            3 => {
                set_normal(number)?;
                // The colour was just set to 0.
                let colour = plus(plus(LIGHT_WORK.view_light_base.bits(), 0, mmx), added, mmx);
                LIGHT_WORK.light_colour.set_bits(colour);
                blend();
                Ok(LIGHT_WORK.light_colour.bits())
            }
            _ => Ok(number),
        }
    }
}
