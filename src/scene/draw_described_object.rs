//! `scene::draw_described_object` (0x451B61): an object drawn from its description at esi
//! (a course face record of kind 0xFF, part R1 of stream A; also `scene::draw_scene_object`'s
//! modes 1 and 2).
//!
//! In:  esi = the description: +1 the object (`g_objectNumber`), +4 its scene point
//!      (`g_objectPlace`), +0x30 .. +0x38 the offset from it, +8 / +0xC / +0x10 its turns
//!      (<< 8; the first & 0xFFFF), with +0x11 the second turns +9 / +0x12 / +0x13, +0x14 ..
//!      +0x17 its light, +0x18 its size, +0x22 / +0x23 its light's slant, +0x24 the sun's
//!      light, +0x25 bits 1 .. 2, +0x2C its shine
//! Out: the registers as the drawing leaves them (callers return, or `popa`)
//!
//! Nothing for an object above 0x64 or a point past `g_scenePointCount`, nor in a race for
//! object 0x3F. Objects 0x38 and 0x3A .. 0x3F are signs: upright (0x2000), full light, 0x3D /
//! 0x3E half bright when any of +0x19 .. +0x1B, +0x20, +0x28 .. +0x2B is set. Then
//! `dword_6365C8` 2, `race::place_object_matrix`, `g_lightKind` 2 (1 when the object's material
//! record has +0x1F), `scene::draw_scene_object` (esi the material record, ecx the object, eax
//! its +0x1F); after it (and in the cases of nothing) `dword_635058` 0 and `g_objectPlace` -1.
//! Refuses (before anything) what `draw_scene_object` refuses for the object (a race car
//! without its block).

use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::race::prefs_store::STAGE_PREFS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::work_store::RACE_WORK;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;

/// What `scene::draw_scene_object` would refuse for `object`: in a race (`g_inRace`) a car
/// (0x50 ..) without its block.
fn check_object(object: u32) -> Result<(), Refused> {
    let car = object.wrapping_sub(0x50);
    if NUMBERS.in_race.bits() != 0 && car < 0xA0 && RACE_WORK.car_block((car / 0x14) as usize) == 0
    {
        return Err(BadInput("a car without its block").into());
    }
    Ok(())
}

/// The Rust version for the description at `d` (esi at the original's entry). Its calls:
/// `race::place_object_matrix` and `scene::draw_scene_object`. What it leaves in the
/// registers (ebx the object) no caller reads (register liveness, 2026-10-23: only the
/// unreached `sub_49081A`), nor do its two calls read what it had in them: the matrix's
/// placing reads no register, and the drawing only esi, which it is given.
pub fn draw_described_object(d: u32) -> Result<(), Refused> {
    draw_description(&|len| GamePtr::new(d).read_bytes(len, "a described object"))
}

/// The same for course face `face` (kind 0xFF), its own description.
pub fn draw_described_face(face: u32) -> Result<(), Refused> {
    draw_description(&|len| {
        let bytes: [u8; 0x40] =
            crate::race::course_places::face_field(face, 0, "a described face")?;
        Ok(bytes[..len].to_vec())
    })
}

/// The drawing from a description's first `len` bytes as `read` gives them.
fn draw_description(read: &dyn Fn(usize) -> Result<Vec<u8>, BadInput>) -> Result<(), Refused> {
    let head = read(8)?;
    let object = head[1] as u32;
    let place = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
    let shown = object <= 0x64
        && place < COURSE.scene_point_count.get()
        && !(FRAME_SWITCHES.other_way.get() != 0 && object == 0x3F);
    if shown {
        check_object(object)?;
    }
    FRAME_SWITCHES.object_number.set_bits(object);
    let end = || {
        OBJECT_WORK.only_object.set(0u32);
        OBJECT_WORK.object_place.set_bits(u32::MAX);
    };
    if object > 0x64 {
        end();
        return Ok(());
    }
    if place >= COURSE.scene_point_count.get() {
        end();
        return Ok(());
    }
    OBJECT_WORK.object_place.set_bits(place);
    if !shown {
        end();
        return Ok(());
    }
    // Everything the description gives from here on lies in its first 0x3C bytes.
    let desc = read(0x3C)?;
    let byte = |o: u32| desc[o as usize] as u32;
    let get = |o: usize| u32::from_le_bytes(desc[o..o + 4].try_into().unwrap());
    let at: [u32; 3] = crate::race::scene_points::point_number(place)?;
    OBJECT_WORK.place[0].set(at[0].wrapping_add(get(0x30)));
    OBJECT_WORK.place[1].set(at[1].wrapping_add(get(0x34)));
    OBJECT_WORK.place[2].set(at[2].wrapping_add(get(0x38)));
    OBJECT_WORK.angles[0].set((byte(8) << 8) & 0xFFFF);
    OBJECT_WORK.angles[1].set(byte(0xC) << 8);
    OBJECT_WORK.angles[2].set(byte(0x10) << 8);
    OBJECT_WORK.described_cleared[1].set(0u32);
    OBJECT_WORK.described_cleared[0].set(0u32);
    OBJECT_WORK.described_cleared[2].set(0u32);
    if byte(0x11) != 0 {
        OBJECT_WORK.described_cleared[2].set(byte(9) << 8);
        OBJECT_WORK.described_cleared[0].set(byte(0x12) << 8);
        OBJECT_WORK.described_cleared[1].set(byte(0x13) << 8);
    }
    OBJECT_WORK.other_light.set(get(0x2C));
    OBJECT_WORK.only_object.set((byte(0x25) >> 1) & 3);
    let light = |lowest: u32, highest: u32, level: u32, strength: u32| {
        OBJECT_WORK.light_lowest.set(lowest);
        OBJECT_WORK.light_highest.set(highest);
        OBJECT_WORK.light_level.set(level);
        OBJECT_WORK.light_strength.set(strength);
    };
    if object == 0x38 || (0x3A..0x40).contains(&object) {
        OBJECT_WORK.angles[0].set(0x2000u32);
        OBJECT_WORK.angles[1].set(0u32);
        OBJECT_WORK.angles[2].set(0u32);
        OBJECT_WORK.size.set(byte(0x18));
        light(0xFF, 0xFF, 0xFF, 0);
        OBJECT_WORK.light_turns[0].set(0u32);
        OBJECT_WORK.light_turns[1].set(0u32);
        if (object == 0x3E || object == 0x3D)
            && [0x28u32, 0x29, 0x2A, 0x2B, 0x19, 0x1A, 0x1B, 0x20]
                .iter()
                .any(|&o| byte(o) != 0)
        {
            OBJECT_WORK.light_highest.set(0x80u32);
        }
    } else {
        OBJECT_WORK.size.set(byte(0x18));
        light(byte(0x14), byte(0x15), byte(0x16), byte(0x17));
        OBJECT_WORK.light_turns[0].set(byte(0x22));
        OBJECT_WORK.light_turns[1].set(byte(0x23));
        if byte(0x24) != 0 {
            OBJECT_WORK.light_turns[0].set(STAGE_PREFS.sun_turn.get().wrapping_neg());
            OBJECT_WORK.light_turns[1].set(STAGE_PREFS.sun_height.get());
            OBJECT_WORK.light_level.set(STAGE_PREFS.light_level.get());
            OBJECT_WORK
                .light_strength
                .set(STAGE_PREFS.light_strength_kept.get());
            OBJECT_WORK.light_lowest.set(0u32);
            OBJECT_WORK.light_highest.set(0xFFu32);
        }
    }
    LIGHT_STATE.view_kind.set(2u32);
    crate::race::frame_draw::place_object_matrix()?;
    LIGHT_WORK.light_kind.set(2u32);
    let object: u32 = FRAME_SWITCHES.object_number.bits();
    let kind: u32 = OBJECT_TABLES.object_kind(object);
    let record = crate::scene::polygon::materials_ptr().offset(kind.wrapping_mul(0x80));
    if record.offset(0x1F).read::<u8>("an object's material")? != 0 {
        LIGHT_WORK.light_kind.set(1u32);
    }
    crate::scene::draw_scene_object::draw_scene_object_in_game(record.addr())?;
    end();
    Ok(())
}
