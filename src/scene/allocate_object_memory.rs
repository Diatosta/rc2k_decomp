//! `scene::allocate_object_memory` (0x41F814): takes the memory the objects' points
//! and faces are loaded into.
//!
//! In:  nothing
//! Out: eax = the memory's end, ebx = the list it is taken on; everything else
//!      preserved
//!
//! One block on the list `stru_5C6DD0`, counted in `dword_5C6F90`: in a race 0x300000 bytes,
//! 0xC0000 of them for the points and 0x240000 for the faces; else 0x580000, 0x180000 and
//! 0x400000. `g_objectPoints` = the block, `dword_5C6FA8` (the faces' memory) = what follows
//! the points' part, `dword_5C6FAC` and `dword_5C6FB0` the two rooms. One caller,
//! `scene::set_up_object_memory`.

use crate::errors::Refused;
use crate::memory::allocate::allocate;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::numbers_store::NUMBERS;

// ---- game data ----

use rc2k_core::scene::objects::object_rooms as rooms;

// ---- Rust version ----

/// `take`: what stands for `memory::allocate`, given the bytes and the list.
// Converted: `rc2k_core::scene::objects::object_rooms`
pub fn allocate_object_memory(
    take: &mut dyn FnMut(u32, u32) -> Result<u32, Refused>,
) -> Result<(), Refused> {
    let (points, faces) = rooms(NUMBERS.in_race.bits() != 0);
    let list = COURSE_LISTS.object_points_list[0].addr() as u32;
    COURSE_LISTS.course_bytes_total.set_bits(
        COURSE_LISTS
            .course_bytes_total
            .bits()
            .wrapping_add(points + faces),
    );
    let block = take(points + faces, list)?;
    COURSE_LISTS.object_points.set_bits(block);
    COURSE_LISTS
        .face_memory
        .set_bits(block.wrapping_add(points));
    COURSE_LISTS.object_points_room.set_bits(points);
    COURSE_LISTS.face_memory_room.set_bits(faces);
    Ok(())
}

pub fn allocate_object_memory_in_game() -> Result<(), Refused> {
    allocate_object_memory(&mut |bytes, _| {
        Ok(allocate(COURSE_LISTS.object_points_list[0].addr() as *mut _, bytes)? as u32)
    })
}
