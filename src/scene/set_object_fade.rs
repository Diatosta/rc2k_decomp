//! `scene::set_object_fade` (0x451609): how see-through an object is drawn by how far
//! off it is, and whether another, simpler object is drawn in its place.
//!
//! In:  nothing
//! Out: the original leaves eax, ebx, ecx, edx and esi changed; its caller goes on
//!      without them, and the Rust version keeps them
//!
//! From the record of the object's kind (0x80 bytes at `unk_97DF30`) two words: +0x1D to
//! `qword_6214E0` (another kind to draw instead) and +0x22 to its second half (a distance).
//! `dword_635004` and `dword_63500C` = 0, `dword_63996C` = 0 (no fading), `dword_639974` =
//! 0x100 (not see-through). Nothing more if both words are 0 or the object's place
//! `dword_634FFC` (a point of the scene) is -1. Else that point's record (made if it is not
//! there: `sub_44CF33`, the same as `scene::project_point` makes one) gives `dword_635004` and
//! `dword_63500C`: its x and depth in the view as whole numbers. With the depth beyond the
//! distance times 65536: `dword_639974` and `dword_639978` = (the distance times 65536 less the
//! depth, plus 0x100000, not under 0) over 0x1000, `dword_63996C` = 1, and if the first word is
//! not 0 and an object of that kind exists (`sub_422298`: the first of the objects 1 .. 0xFF
//! with that kind), `g_objectNumber` becomes that object. One caller, `sub_45045D`. Refuses an
//! object's number past the tables and, where a record has to be made, no records or no points.

use crate::errors::BadInput;
use crate::math::x87;
use crate::physics::ground_store::GROUND_WORK;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::polygon::dword;
use crate::scene::project_point::make_scene_record;
use crate::scene::set_object_material::MATERIAL_LEN;

// ---- game data ----
const OBJECTS: u32 = 0x100;

// ---- Rust version ----

unsafe fn record_at() -> Option<u32> {
    {
        let (place, records): (u32, u32) = (
            OBJECT_WORK.object_place.bits(),
            COURSE_LISTS.point_records.bits(),
        );
        (records != 0).then(|| records.wrapping_add((place & 0xFFF) << 5))
    }
}

/// The two words of the object's kind, and whether the object's place is looked at;
/// None for an object's number past the tables.
unsafe fn words() -> Option<(u32, u32, bool)> {
    unsafe {
        let object: u32 = FRAME_SWITCHES.object_number.bits();
        if object >= OBJECTS {
            return None;
        }
        let kind: u32 = OBJECT_TABLES.object_kind(object);
        let material = crate::scene::polygon::materials_addr()
            .wrapping_add((kind as usize).wrapping_mul(MATERIAL_LEN));
        let word = |at: usize| ((material + at) as *const u16).read_unaligned() as u32;
        let (other_kind, distance) = (word(0x1D), word(0x22));
        let looked_at = (other_kind | distance) != 0 && OBJECT_WORK.object_place.bits() != u32::MAX;
        Some((other_kind, distance, looked_at))
    }
}

pub unsafe fn set_object_fade() -> Result<(), BadInput> {
    unsafe {
        let Some((other_kind, distance, needs_record)) = words() else {
            return Err(BadInput("an object's number past the tables"));
        };
        let place: u32 = OBJECT_WORK.object_place.bits();
        // What can refuse is asked before anything is written.
        let record = match (needs_record, record_at()) {
            (false, _) => 0,
            (true, None) => return Err(BadInput("no records for the points")),
            (true, Some(record)) => {
                if dword(record as usize) != place && COURSE.scene_points.bits() == 0 {
                    return Err(BadInput("no points"));
                }
                record
            }
        };
        crate::session::write_bits(&FRAME_SWITCHES.fade_words, [other_kind, distance]);
        OBJECT_WORK.object_position[0].set_bits(0u32);
        OBJECT_WORK.object_position[2].set_bits(0u32);
        GROUND_WORK.object_fading.set_bits(0u32);
        GROUND_WORK.object_fade[0].set_bits(0x100u32);
        if !needs_record {
            return Ok(());
        }
        if dword(record as usize) != place {
            make_scene_record(place, record)?;
        }
        OBJECT_WORK.object_position[0].set_bits(x87::to_int(dword(record as usize + 0x10)) as u32);
        let depth = x87::to_int(dword(record as usize + 0x18));
        OBJECT_WORK.object_position[2].set_bits(depth as u32);
        let near = ((distance << 16) as i32).wrapping_sub(depth);
        if near >= 0 {
            return Ok(());
        }
        let fade = (near.wrapping_add(0x10_0000).max(0) as u32) >> 12;
        crate::session::write_bits(&GROUND_WORK.object_fade, [fade, fade]);
        GROUND_WORK.object_fading.set_bits(1u32);
        if other_kind != 0 {
            let kinds: [u32; 0x100] = crate::session::read_bits(&OBJECT_TABLES.object_kinds);
            if let Some(other) = (1..0x100).find(|&n| kinds[n] == other_kind) {
                FRAME_SWITCHES.object_number.set_bits(other as u32);
            }
        }
        Ok(())
    }
}
