//! `scene::allocate_object_memory` (0x41F814): takes the memory the objects' points
//! and faces are loaded into.
//!
//! Out: eax = the memory's end, ebx = the list it is taken on; everything else
//!      preserved
//!
//! One block on the list `stru_5C6DD0`, counted in `dword_5C6F90`: in a race 0x300000 bytes,
//! 0xC0000 of them for the points and 0x240000 for the faces; else 0x580000, 0x180000 and
//! 0x400000. `g_objectPoints` = the block, `dword_5C6FA8` (the faces' memory) = what follows
//! the points' part, `dword_5C6FAC` and `dword_5C6FB0` the two rooms. One caller,
//! `scene::set_up_object_memory`.

use crate::errors::Refused;

// ---- game data ----

use rc2k_core::scene::objects::object_rooms as rooms;

// ---- Rust version ----

/// The points and the faces Rust's (`scene::object_points`, `scene::object_faces`, phase
/// 13), counted on the list as the game's block was.
// Converted: `rc2k_core::scene::objects::object_rooms`
pub fn allocate_object_memory_in_game(
    course: &crate::race::owner_course::Course,
) -> Result<(), Refused> {
    let (points, faces) = rooms(course.numbers.in_race.bits() != 0);
    let list = &course.lists.object_points_list;
    course.lists.course_bytes_total.set_bits(
        course
            .lists
            .course_bytes_total
            .bits()
            .wrapping_add(points + faces),
    );
    crate::scene::object_points::take(course, list, points)?;
    crate::scene::object_faces::take(course, list, faces)?;
    course.lists.object_points_room.set_bits(points);
    course.lists.face_memory_room.set_bits(faces);
    Ok(())
}
