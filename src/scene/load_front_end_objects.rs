//! `scene::load_front_end_objects` (0x425CA9): loads the shapes of all 0x100 object
//! slots.
//!
//! `scene::reset_objects`; `ObjectTables.texture_count` and `ObjectTables.no_normals` = 0; for each slot its kind from
//! `ObjectTables.slot_kinds[0]` to `ObjectWork.loading_kind`, the slot to `ObjectWork.loading_slot`, and `scene::load_object`; then
//! `ObjectWork.point_memory_total` = the points used, in bytes, and `ObjectWork.face_memory_total` = the faces' memory used. Two
//! callers (the front end's start and a stage's leaving).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::scene::load_object::load_object;
use crate::scene::objects::SLOTS;
use crate::scene::reset_objects::reset_objects;

// ---- game data ----

// ---- Rust version ----

pub fn load_front_end_objects(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    reset_objects(course, scene)?;
    scene.objects.slots.texture_count.set(0);
    scene.objects.slots.no_normals.set(0);
    for slot in 0..SLOTS {
        let kind: u32 = scene.objects.tables.slot_kinds[slot as usize].get();
        scene.drawing.object_work.loading_kind.set(kind);
        scene.drawing.object_work.loading_slot.set(slot);
        load_object(scene, race)?;
    }
    let cursor: u32 = scene.drawing.object_work.face_cursor.get();
    scene
        .drawing
        .object_work
        .point_memory_total
        .set(scene.objects.tables.point_count.get().wrapping_mul(12));
    scene.drawing.object_work.face_memory_total.set(cursor);
    Ok(())
}
