//! `scene::allocate_object_memory`: takes the memory the objects' points and faces are loaded
//! into.
//!
//! One block on the memory list, counted: in a race 0x300000 bytes, 0xC0000 of them for the points
//! and 0x240000 for the faces; else 0x580000, 0x180000 and 0x400000. The object points are the
//! block, the faces' memory what follows the points' part, and the two rooms are kept.

use crate::errors::Refused;

// ---- game data ----

use rc2k_core::scene::objects::object_rooms as rooms;

/// The points and the faces are Rust's (`scene::object_points_store`, `scene::object_faces_store`),
/// counted on the list.
// Logic: `rc2k_core::scene::objects::object_rooms`
pub fn allocate_object_memory_in_game(
    course: &crate::race::owner_course::Course,
) -> Result<(), Refused> {
    let (points, faces) = rooms(course.number_vars.get().in_race.is_set());
    let list = &course.lists.get().object_points_list;
    course.lists.get().course_bytes_total.set(
        course
            .lists
            .get()
            .course_bytes_total
            .get()
            .wrapping_add(points + faces),
    );
    course.stage.object_points.take(course, list, points)?;
    course.stage.object_faces.take(course, list, faces)?;
    course.lists.get().object_points_room.set(points);
    course.lists.get().face_memory_room.set(faces);
    Ok(())
}
