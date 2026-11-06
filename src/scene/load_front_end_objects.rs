//! `scene::load_front_end_objects` (0x425CA9): loads the shapes of all 0x100 object
//! slots.
//!
//! In:  nothing
//! Out: eax = the faces' memory used, ecx = 0x100, edi = the end of the kinds' table,
//!      ebx = the last slot that had a kind (unchanged if none had)
//!
//! `scene::reset_objects`; `dword_631B68` and `dword_631F6C` = 0; for each slot its kind from
//! `dword_5CCE7C` to `dword_634FE8`, the slot to `dword_634FEC`, and `scene::load_object`; then
//! `dword_634FA4` = the points used, in bytes, and `dword_634FA8` = the faces' memory used. Two
//! callers (the front end's start and a stage's leaving).

use crate::errors::Refused;
use crate::race::owner::race;
use crate::scene::load_object::load_object;
use crate::scene::objects::SLOTS;
use crate::scene::reset_objects::reset_objects;

// ---- game data ----

// ---- Rust version ----

/// The Rust version. The registers the original leaves (eax the faces' memory used, ecx
/// 0x100, edi, ebx) are read by no caller (taint at its sites: `race::set_object_kinds`' and
/// `load_front_end_objects_logged`'s).
pub fn load_front_end_objects(scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let race = race();
    let course = &race.course;
    reset_objects(course)?;
    scene.objects.slots.texture_count.set(0);
    scene.objects.slots.no_normals.set(0);
    for slot in 0..SLOTS {
        let kind: u32 = scene.objects.tables.slot_kinds[slot as usize].get();
        scene.drawing.object_work.loading_kind.set_bits(kind);
        scene.drawing.object_work.loading_slot.set_bits(slot);
        load_object(scene, race)?;
    }
    let cursor: u32 = scene.drawing.object_work.face_cursor.bits();
    scene
        .drawing
        .object_work
        .point_memory_total
        .set_bits(scene.objects.tables.point_count.get().wrapping_mul(12));
    scene.drawing.object_work.face_memory_total.set_bits(cursor);
    Ok(())
}
