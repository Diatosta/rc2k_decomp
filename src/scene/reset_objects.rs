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
use crate::framework::{game_addr, game_read, game_write};
use crate::scene::objects::{
    FACE_COUNT_RVA, FACE_CURSOR_RVA, FIRST_POINT_RVA, OBJECT_KINDS_RVA, POINT_COUNT_RVA,
    POINTS_ROOM_RVA, POINTS_RVA, SLOTS,
};

/// The five tables of 0x100 dwords from `g_objectFirstPoint` on.
const TABLES_LEN: usize = SLOTS as usize * 4 * 5;

// ---- Rust version ----

pub unsafe fn reset_objects() -> Result<(), BadInput> {
    unsafe {
        let (memory, room): (u32, u32) = (game_read(POINTS_RVA), game_read(POINTS_ROOM_RVA));
        if memory == 0 && room >= 4 {
            return Err(BadInput("room for the objects' points without memory"));
        }
        std::ptr::write_bytes(memory as *mut u32, 0, (room >> 2) as usize);
        std::ptr::write_bytes(game_addr(FIRST_POINT_RVA) as *mut u8, 0, TABLES_LEN);
        game_write(POINT_COUNT_RVA, 1u32);
        game_write(FACE_COUNT_RVA, 0u32);
        game_write(FACE_CURSOR_RVA, 0u32);
        game_write(OBJECT_KINDS_RVA, [u32::MAX; SLOTS as usize]);
        Ok(())
    }
}
