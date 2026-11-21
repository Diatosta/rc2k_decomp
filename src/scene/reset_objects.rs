//! `scene::reset_objects`: empties the objects' tables before they are loaded.
//!
//! The points' memory zeroed (its room `CourseLists.object_points_room`, in dwords); the slots'
//! first points, numbers of points and normals, first faces and numbers of faces zeroed; the object
//! point count = 1 (point 0 is never an object's), the faces' number and the faces' memory used =
//! 0; every slot's kind = -1. Refuses points' room without memory.

use crate::errors::BadInput;

pub fn reset_objects(
    course: &crate::race::owner_course::Course,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    {
        let (memory, room): (u32, u32) = (
            course.lists.borrow().object_points,
            course.lists.borrow().object_points_room,
        );
        if memory == 0 && room >= 4 {
            return Err(BadInput("room for the objects' points without memory"));
        }
        if room >> 2 != 0 {
            course.stage.object_points.clear((room >> 2) as usize * 4)?;
        }
        {
            let mut slots = scene.objects.slots.borrow_mut();
            slots.first_point.fill(0);
            slots.point_counts.fill(0);
            slots.normal_counts.fill(0);
            slots.first_face.fill(0);
            slots.face_counts.fill(0);
        }
        scene.objects.tables.borrow_mut().point_count = 1;
        scene.texture_levels_mut().face_count = 0;
        scene.object_work_mut().face_cursor = 0u32;
        scene
            .objects
            .tables
            .borrow_mut()
            .object_kinds
            .fill(u32::MAX);
        Ok(())
    }
}
