//! `scene::reset_objects` (0x425CFF): empties the objects' tables before they are
//! loaded.
//!
//! In:  nothing
//! Out: eax = 0xFFFFFFFF, ecx = 0, edi = the end of the kinds' table
//!
//! The points' memory zeroed (its room `dword_5C6FAC`, in dwords); the slots' first points,
//! numbers of points and normals, first faces and numbers of faces zeroed; `g_objectPointCount`
//! = 1 (point 0 is never an object's), the faces' number and the faces' memory used = 0; every
//! slot's kind = -1. One caller, `scene::load_front_end_objects`. Refuses points' room without
//! memory.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::scene::object_slots_store::OBJECT_SLOTS;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;

// ---- Rust version ----

pub fn reset_objects() -> Result<(), BadInput> {
    {
        let (memory, room): (u32, u32) = (
            COURSE_LISTS.object_points.bits(),
            COURSE_LISTS.object_points_room.bits(),
        );
        if memory == 0 && room >= 4 {
            return Err(BadInput("room for the objects' points without memory"));
        }
        if room >> 2 != 0 {
            GamePtr::new(memory).fill(0, (room >> 2) as usize * 4, "the objects' points memory")?;
        }
        for table in [
            &OBJECT_SLOTS.first_point,
            &OBJECT_SLOTS.point_counts,
            &OBJECT_SLOTS.normal_counts,
            &OBJECT_SLOTS.first_face,
            &OBJECT_SLOTS.face_counts,
        ] {
            for cell in table {
                cell.set(0);
            }
        }
        OBJECT_TABLES.point_count.set(1);
        OBJECT_TABLES.face_count.set(0);
        OBJECT_WORK.face_cursor.set_bits(0u32);
        for kind in &OBJECT_TABLES.object_kinds {
            kind.set(u32::MAX);
        }
        Ok(())
    }
}
