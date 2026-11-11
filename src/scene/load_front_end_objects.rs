//! `scene::load_front_end_objects` (0x425CA9): loads the shapes of all 0x100 object
//! slots.
//!
//! `scene::reset_objects`; `dword_631B68` and `dword_631F6C` = 0; for each slot its kind from
//! `dword_5CCE7C` to `dword_634FE8`, the slot to `dword_634FEC`, and `scene::load_object`; then
//! `dword_634FA4` = the points used, in bytes, and `dword_634FA8` = the faces' memory used. Two
//! callers (the front end's start and a stage's leaving).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::scene::load_object::load_object;
use crate::scene::objects::SLOTS;
use crate::scene::reset_objects::reset_objects;

// ---- game data ----

// ---- Rust version ----

/// The Rust version. The registers the original leaves (eax the faces' memory used, ecx
/// 0x100, edi, ebx) are read by no caller (taint at its sites: `race::set_object_kinds`' and
/// `load_front_end_objects_logged`'s).
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
