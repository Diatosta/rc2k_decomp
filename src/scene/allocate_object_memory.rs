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

/// The points' part from `memory::allocate`; the faces' Rust's (`scene::object_faces`,
/// phase 13), counted on the same list.
// Converted: `rc2k_core::scene::objects::object_rooms`
pub fn allocate_object_memory_in_game() -> Result<(), Refused> {
    let (points, faces) = rooms(NUMBERS.in_race.bits() != 0);
    let list = COURSE_LISTS.object_points_list[0].as_ptr() as *mut _;
    COURSE_LISTS.course_bytes_total.set_bits(
        COURSE_LISTS
            .course_bytes_total
            .bits()
            .wrapping_add(points + faces),
    );
    let block = crate::pointers::game_address(allocate(list, points)?);
    crate::scene::object_faces::take(list, faces)?;
    COURSE_LISTS.object_points.set_bits(block);
    COURSE_LISTS.object_points_room.set_bits(points);
    COURSE_LISTS.face_memory_room.set_bits(faces);
    Ok(())
}
