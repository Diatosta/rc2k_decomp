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
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::numbers_store::NUMBERS;

// ---- game data ----

use rc2k_core::scene::objects::object_rooms as rooms;

// ---- Rust version ----

/// The points and the faces Rust's (`scene::object_points`, `scene::object_faces`, phase
/// 13), counted on the list as the game's block was.
// Converted: `rc2k_core::scene::objects::object_rooms`
pub fn allocate_object_memory_in_game() -> Result<(), Refused> {
    let (points, faces) = rooms(NUMBERS.in_race.bits() != 0);
    let list = crate::pointers::GamePtr::to(&COURSE_LISTS.object_points_list[0]);
    COURSE_LISTS.course_bytes_total.set_bits(
        COURSE_LISTS
            .course_bytes_total
            .bits()
            .wrapping_add(points + faces),
    );
    crate::scene::object_points::take(list, points)?;
    crate::scene::object_faces::take(list, faces)?;
    COURSE_LISTS.object_points_room.set_bits(points);
    COURSE_LISTS.face_memory_room.set_bits(faces);
    Ok(())
}
