//! `scene::reset_objects` (0x425CFF): empties the objects' tables before they are
//! loaded.
//!
//! The points' memory zeroed (its room `CourseLists.object_points_room`, in dwords); the slots' first points,
//! numbers of points and normals, first faces and numbers of faces zeroed; `g_objectPointCount`
//! = 1 (point 0 is never an object's), the faces' number and the faces' memory used = 0; every
//! slot's kind = -1. One caller, `scene::load_front_end_objects`. Refuses points' room without
//! memory.

use crate::errors::BadInput;

// ---- Rust version ----

pub fn reset_objects(
    course: &crate::race::owner_course::Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    {
        let (memory, room): (u32, u32) = (
            course.lists.get().object_points.get(),
            course.lists.get().object_points_room.get(),
        );
        if memory == 0 && room >= 4 {
            return Err(BadInput("room for the objects' points without memory"));
        }
        if room >> 2 != 0 {
            crate::scene::object_points::clear((room >> 2) as usize * 4)?;
        }
        for table in [
            &scene.objects.slots.get().first_point,
            &scene.objects.slots.get().point_counts,
            &scene.objects.slots.get().normal_counts,
            &scene.objects.slots.get().first_face,
            &scene.objects.slots.get().face_counts,
        ] {
            for cell in table {
                cell.set(0);
            }
        }
        scene.objects.tables.get().point_count.set(1);
        scene.objects.tables.get().face_count.set(0);
        scene.object_work().face_cursor.set(0u32);
        for kind in &scene.objects.tables.get().object_kinds {
            kind.set(u32::MAX);
        }
        Ok(())
    }
}
